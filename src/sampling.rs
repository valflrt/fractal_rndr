use std::f64::consts::TAU;

use crate::F;

pub fn generate_sampling_points(n: usize) -> Vec<(F, F)> {
    const PHI: F = 1.618033988749895;
    const EPS: F = 0.5;

    (0..n)
        .map(|i| {
            (
                i as F / PHI % 1.,
                (i as F + EPS) / ((n - 1) as F + 2. * EPS),
            )
        })
        .map(|(x, y)| {
            let r = y.sqrt();
            let theta = (TAU as F) * x;
            (r, theta)
        })
        .collect::<Vec<_>>()
}
