use std::{array, sync::mpsc};

use rayon::prelude::*;

use crate::{
    complexx::{self, Complexx},
    mat::Mat2D,
    params::FrameParams,
    progress::Progress,
    View, F, FX,
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
        ..
    } = params;

    let &View {
        width,
        height,
        cx,
        cy,
        rotate,
        ..
    } = view;

    let mut raw_image = Mat2D::filled_with(0., img_width as usize, img_height as usize);

    for chunk in sampling_points.chunks(1024) {
        let (tx, rx) = mpsc::channel();
        chunk
            .chunks(complexx::SIZE)
            .par_bridge()
            .for_each_with(tx, |s, d| {
                let l = d.len();

                // Here we use `i % l` to avoid out of bounds error (when i < 4).
                // When `i < 4`, the modulo operation will repeat the sample
                // but as we use simd this is acceptable (the cost is the
                // same whether it is computed along with the others or not).

                const SCALE: F = 2.;
                const R_OFFSET: F = 0.0001;
                const THETA_OFFSET: F = 0.0001;
                let re = FX::from(array::from_fn(|i| {
                    let (r, theta) = d[i % l];
                    let (r, theta) = (
                        r + R_OFFSET * (2. * fastrand::f64() - 1.),
                        theta + THETA_OFFSET * (2. * fastrand::f64() - 1.),
                    );
                    r * SCALE * theta.cos()
                }));
                let im = FX::from(array::from_fn(|i| {
                    let (r, theta) = d[i % l];
                    let (r, theta) = (
                        r + R_OFFSET * (2. * fastrand::f64() - 1.),
                        theta + THETA_OFFSET * (2. * fastrand::f64() - 1.),
                    );
                    r * SCALE * theta.sin()
                }));

                let values = {
                    let c = Complexx::splat(cx, cy);
                    fractal.sample(
                        (Complexx { re, im } - c) * Complexx::from_polar_splat(1., rotate) + c,
                        max_iter,
                    )
                };

                for v in values {
                    for i in 0..l {
                        let (re, im) = v[i];

                        let (re, im) = (-im, re);

                        let i = (2. * (re - cx) / width + 0.5) * img_width as F;
                        let j = (2. * (im - cy) / height + 0.5) * img_height as F;

                        if (0. ..img_width as F).contains(&i) && (0. ..img_height as F).contains(&j)
                        {
                            s.send((i as usize, j as usize)).unwrap();
                        }
                    }
                }

                if let Some(progress) = &progress {
                    progress.add(4);
                }
            });

        for (i, j) in rx {
            raw_image[(i as usize, j as usize)] += 1.;
        }
    }

    raw_image
}
