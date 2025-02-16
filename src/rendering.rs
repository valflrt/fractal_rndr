use std::{
    array,
    io::{Stdout, Write},
    sync::mpsc,
    time::Instant,
};

use rayon::prelude::*;

use crate::{
    complexx::Complexx,
    fractal::Fractal,
    mat::Mat2D,
    progress::Progress,
    sampling::{map_points_with_offsets, Sampling},
    ViewParams, F, FX,
};

#[derive(Debug, Clone, Copy)]
pub struct RenderingCtx<'a> {
    pub img_width: u32,
    pub img_height: u32,

    pub max_iter: u32,
    pub sampling: Sampling,
    pub sampling_points: &'a [(F, F)],

    pub start: Instant,
    pub stdout: &'a Stdout,
}

pub fn render_raw_image(
    fractal: Fractal,
    view_params: ViewParams,
    rendering_ctx: RenderingCtx,
    progress: Progress,
) -> Mat2D<F> {
    let RenderingCtx {
        img_width,
        img_height,
        max_iter,
        sampling,
        sampling_points,
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
            let x = i as F;
            let y = j as F;

            let (offset_x, offset_y) = if sampling.random_offsets {
                #[cfg(feature = "force_f32")]
                let v = (rng.f32(), rng.f32());
                #[cfg(not(feature = "force_f32"))]
                let v = (rng.f64(), rng.f64());

                v
            } else {
                (0., 0.)
            };
            let sampling_points = sampling_points
                .iter()
                .map(|&(dx, dy)| map_points_with_offsets(dx, dy, offset_x, offset_y))
                .collect::<Vec<_>>();

            #[cfg(feature = "force_f32")]
            const CHUNK_SIZE: usize = 8;
            #[cfg(not(feature = "force_f32"))]
            const CHUNK_SIZE: usize = 4;
            let value = sampling_points
                .chunks(CHUNK_SIZE)
                .flat_map(|d| {
                    let l = d.len();
                    let re = FX::from(array::from_fn(|i| {
                        // Here we use `i % l` to avoid out of bounds error (when i < 4).
                        // When `i < 4`, the modulo operation will repeat the sample
                        // but as we use simd this is acceptable (the cost is the
                        // same whether it is computed along with the others or not).
                        let (dx, _) = d[i % l];
                        x_min + width * (x + 0.5 + dx) / img_width as F
                    }));
                    let im = FX::from(array::from_fn(|i| {
                        let (_, dy) = d[i % l];
                        y_min + height * (y + 0.5 + dy) / img_height as F
                    }));

                    let iter = fractal.sample(Complexx { re, im }, max_iter);

                    (0..l).map(move |i| iter[i])
                })
                .sum::<F>()
                / sampling_points.len() as F;

            s.send(((i, j), value)).unwrap();

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
