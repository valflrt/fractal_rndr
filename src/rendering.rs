use std::{
    array,
    io::{Stdout, Write},
    sync::mpsc,
    time::Instant,
};

use rayon::prelude::*;
use wide::f64x4;

use crate::{
    complex4::Complex4,
    fractal::Fractal,
    mat::{Mat2D, Mat3D},
    progress::Progress,
    sampling::{map_points_with_offsets, Sampling},
    ChunkDimensions, ViewParams, CHUNK_SIZE,
};

pub const RDR_KERNEL_SIZE: usize = 1;

#[derive(Debug, Clone, Copy)]
pub struct RenderingCtx<'a> {
    pub img_width: u32,
    pub img_height: u32,

    pub max_iter: u32,
    pub sampling: Sampling,
    pub sampling_points: &'a [(f64, f64)],

    pub chunk_dims: ChunkDimensions,

    pub diverging_areas: &'a Option<Vec<[f64; 4]>>,

    pub start: Instant,
    pub stdout: &'a Stdout,
}

pub fn render_raw_image(
    fractal: Fractal,
    view_params: ViewParams,
    rendering_ctx: RenderingCtx,
    progress: Progress,
) -> Mat2D<f64> {
    let RenderingCtx {
        img_width,
        img_height,
        max_iter,
        sampling,
        sampling_points,
        chunk_dims,
        diverging_areas,
        start,
        stdout,
    } = rendering_ctx;

    let ViewParams {
        width,
        height,
        mut x_min,
        mut y_min,
    } = view_params;
    let ChunkDimensions {
        v_chunks,
        h_chunks,
        last_v_chunk,
        last_h_chunk,
    } = chunk_dims;

    if matches!(fractal, Fractal::MoireTest) {
        x_min = 0.;
        y_min = 0.;
    }

    let mut raw_image = Mat2D::filled_with(
        0.,
        img_width as usize + 2 * RDR_KERNEL_SIZE,
        img_height as usize + 2 * RDR_KERNEL_SIZE,
    );

    for cj in 0..v_chunks + 1 {
        for ci in 0..h_chunks + 1 {
            let chunk_width = if ci == h_chunks {
                last_h_chunk
            } else {
                CHUNK_SIZE
            };
            let chunk_height = if cj == v_chunks {
                last_v_chunk
            } else {
                CHUNK_SIZE
            };

            // pi and pj are the coordinates of the first pixel of the
            // chunk (top-left corner pixel)
            let pi = ci * CHUNK_SIZE;
            let pj = cj * CHUNK_SIZE;

            let rng = fastrand::Rng::new();
            let (tx, rx) = mpsc::channel();
            (0..chunk_height + 2 * RDR_KERNEL_SIZE)
                .flat_map(|j| (0..chunk_width + 2 * RDR_KERNEL_SIZE).map(move |i| (i, j)))
                .par_bridge()
                .for_each_with((tx, rng), |(s, rng), (i, j)| {
                    let x = (pi + i - RDR_KERNEL_SIZE) as f64;
                    let y = (pj + j - RDR_KERNEL_SIZE) as f64;

                    let should_render = {
                        let x = x_min + width * x / img_width as f64;
                        let y = y_min + height * y / img_height as f64;

                        diverging_areas
                            .as_ref()
                            .map(|areas| {
                                !areas.iter().any(|&[min_x, max_x, min_y, max_y]| {
                                    min_x <= x && x <= max_x && -max_y <= y && y <= -min_y
                                })
                            })
                            .unwrap_or(true)
                    };

                    s.send((
                        (i, j),
                        should_render.then(|| {
                            let (offset_x, offset_y) = if sampling.random_offsets {
                                (rng.f64(), rng.f64())
                            } else {
                                (0., 0.)
                            };
                            let sampling_points = sampling_points
                                .iter()
                                .filter_map(|&(dx, dy)| {
                                    map_points_with_offsets(dx, dy, offset_x, offset_y)
                                })
                                .collect::<Vec<_>>();

                            sampling_points
                                .chunks(4)
                                .flat_map(|d| {
                                    let l = d.len();
                                    let re = f64x4::from(array::from_fn(|i| {
                                        // Here we use `i % l` to avoid out of bounds error (when i < 4).
                                        // When `i < 4`, the modulo operation will repeat the sample
                                        // but as we use simd this is acceptable (the cost is the
                                        // same whether it is computed along with the others or not).
                                        let (dx, _) = d[i % l];
                                        x_min + width * (x + 0.5 + dx) / img_width as f64
                                    }));
                                    let im = f64x4::from(array::from_fn(|i| {
                                        let (_, dy) = d[i % l];
                                        y_min + height * (y + 0.5 + dy) / img_height as f64
                                    }));

                                    let iter = fractal.get_pixel(Complex4 { re, im }, max_iter);

                                    (0..l).map(move |i| (d[i], iter[i]))
                                })
                                .collect::<Vec<_>>()
                        }),
                    ))
                    .unwrap();

                    progress.incr();

                    if progress.get() % (progress.total / 100000 + 1) == 0 {
                        stdout
                            .lock()
                            .write_all(
                                format!(
                                    "\r {:.1}% - {:.1}s elapsed",
                                    progress.get_percent(),
                                    start.elapsed().as_secs_f32(),
                                )
                                .as_bytes(),
                            )
                            .unwrap();
                    }
                });

            let mut chunk_samples = Mat3D::filled_with(
                None,
                chunk_width + 2 * RDR_KERNEL_SIZE,
                chunk_height + 2 * RDR_KERNEL_SIZE,
                sampling_points.len(),
            );
            for ((i, j), pixel_samples) in rx {
                if let Some(pixel_samples) = pixel_samples {
                    for (k, &v) in pixel_samples.iter().enumerate() {
                        chunk_samples.set((i, j, k), Some(v)).unwrap();
                    }
                }
            }

            let (tx, rx) = mpsc::channel();
            (0..chunk_height)
                .flat_map(|j| (0..chunk_width).map(move |i| (i, j)))
                .par_bridge()
                .for_each_with(tx, |s, (i, j)| {
                    let mut weighted_sum = 0.;
                    let mut weight_total = 0.;

                    let mut is_empty = true;
                    const RDR_KERNEL_SIZE_I: isize = RDR_KERNEL_SIZE as isize;
                    for dj in -RDR_KERNEL_SIZE_I..=RDR_KERNEL_SIZE_I {
                        for di in -RDR_KERNEL_SIZE_I..=RDR_KERNEL_SIZE_I {
                            let ii = (i + RDR_KERNEL_SIZE)
                                .checked_add_signed(di)
                                .expect("should never overflow");
                            let jj = (j + RDR_KERNEL_SIZE)
                                .checked_add_signed(dj)
                                .expect("should never overflow");

                            for k in 0..sampling_points.len() {
                                if let &Some(((dx, dy), v)) =
                                    chunk_samples.get((ii, jj, k)).unwrap()
                                {
                                    let dx = dx + di as f64;
                                    let dy = dy + dj as f64;

                                    // This only includes samples from a round-cornered square.
                                    // see https://www.desmos.com/3d/kgdwgwp4dk

                                    if dx.abs() <= 0.5 && dy.abs() <= 0.5 {
                                        let w = 1.;
                                        weighted_sum += w * v;
                                        weight_total += w;
                                    } else if pi + i != 0 && pj + j != 0 {
                                        // `pi + i != 0 && pj + j != 0`` is an ugly fix for a small issue
                                        // I wasn't able to find the origin: the first column and the
                                        // first row of pixels of the image are colored weirdly without
                                        // this condition...

                                        // Maximum distance for picking samples (out of the pixel)
                                        const D: f64 = 0.5;
                                        const D_SQR: f64 = D * D;
                                        // This is the value of the weight at the border.
                                        const T: f64 = 0.5;
                                        // The "radius" of the square (half its side length).
                                        // This should not be changed.
                                        const R: f64 = 0.5;

                                        let smooth_distance_sqr = (dx.abs() - R).max(0.).powi(2)
                                            + (dy.abs() - R).max(0.).powi(2);
                                        if smooth_distance_sqr < D_SQR {
                                            let w = 1. - T * smooth_distance_sqr / D_SQR;
                                            weighted_sum += w * v;
                                            weight_total += w;
                                        }
                                    }

                                    is_empty = false;
                                };
                            }
                        }
                    }

                    s.send((
                        (pi + i, pj + j),
                        if is_empty {
                            max_iter as f64
                        } else {
                            weighted_sum / weight_total
                        },
                    ))
                    .unwrap();
                });

            for (index, v) in rx {
                raw_image.set(index, v).unwrap();
            }
        }
    }

    raw_image
}
