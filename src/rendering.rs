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
    mat::Mat2D,
    progress::Progress,
    sampling::{map_points_with_offsets, Sampling},
    ViewParams,
};

#[derive(Debug, Clone, Copy)]
pub struct RenderingCtx<'a> {
    pub img_width: u32,
    pub img_height: u32,

    pub max_iter: u32,
    pub sampling: Sampling,
    pub sampling_points: &'a [(f64, f64)],

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
        diverging_areas,
        start,
        stdout,
        ..
    } = rendering_ctx;

    let ViewParams {
        width,
        height,
        mut x_min,
        mut y_min,
    } = view_params;

    if matches!(fractal, Fractal::MoireTest) {
        x_min = 0.;
        y_min = 0.;
    }

    let mut raw_image = Mat2D::filled_with(0., img_width as usize, img_height as usize);

    let rng = fastrand::Rng::new();
    let (tx, rx) = mpsc::channel();
    (0..img_height)
        .flat_map(|j| (0..img_width).map(move |i| (i, j)))
        .par_bridge()
        .for_each_with((tx, rng), |(s, rng), (i, j)| {
            let x = i as f64;
            let y = j as f64;

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

            if should_render {
                let (offset_x, offset_y) = if sampling.random_offsets {
                    (rng.f64(), rng.f64())
                } else {
                    (0., 0.)
                };
                let sampling_points = sampling_points
                    .iter()
                    .map(|&(dx, dy)| map_points_with_offsets(dx, dy, offset_x, offset_y))
                    .collect::<Vec<_>>();

                let value = sampling_points
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

                        let iter = fractal.sample(Complex4 { re, im }, max_iter);

                        (0..l).map(move |i| iter[i])
                    })
                    .sum::<f64>()
                    / sampling_points.len() as f64;

                s.send(((i, j), value)).unwrap();
            }

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

    for ((i, j), sample) in rx {
        let _ = raw_image.set((i as usize, j as usize), sample);
    }

    raw_image
}
