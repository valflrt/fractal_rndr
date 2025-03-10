use std::{array, sync::mpsc};

use rayon::prelude::*;

use crate::{
    complexx::Complexx, fractal::Fractal, mat::Mat2D, params::FrameParams, progress::Progress,
    sampling::map_points_with_offsets, View, F, FX,
};

pub fn render_raw_image(
    params: &FrameParams,
    view: &View,
    sampling_points: &[(F, F)],
    progress: Option<Progress>,
) -> Mat2D<F> {
    let &FrameParams {
        img_width,
        img_height,

        fractal,

        max_iter,

        sampling,
        ..
    } = params;

    let &View {
        width,
        height,
        mut cx,
        mut cy,
        rotate,
        ..
    } = view;

    if matches!(fractal, Fractal::MoireTest) {
        cx = 0.;
        cy = 0.;
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
                        cx + 0.5 * width * ((x + 0.5 + dx) / img_width as F - 0.5)
                    }));
                    let im = FX::from(array::from_fn(|i| {
                        let (_, dy) = d[i % l];
                        cy + 0.5 * height * ((y + 0.5 + dy) / img_height as F - 0.5)
                    }));

                    let iter = {
                        let c = Complexx::splat(cx, cy);
                        fractal.sample(
                            (Complexx { re, im } - c)
                                * Complexx::from_polar_splat(1., rotate.unwrap_or(0.))
                                + c,
                            max_iter,
                        )
                    };

                    (0..l).map(move |i| iter[i])
                })
                .sum::<F>()
                / sampling_points.len() as F;

            s.send(((i, j), value)).unwrap();

            if let Some(progress) = &progress {
                progress.incr();
            }
        });

    for ((i, j), sample) in rx {
        raw_image[(i as usize, j as usize)] = sample;
    }

    raw_image
}
