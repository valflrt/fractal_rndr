use std::f64::consts::TAU;

pub fn get_sampling_points_spiral(s: u32) -> Vec<(f64, f64)> {
    const R: f64 = 1.5;
    const MEMBERS: u32 = 8;
    const BASE_ANGLE: f64 = TAU / MEMBERS as f64;
    const CURL: f64 = 10.;

    (1..=s)
        .flat_map(|i| {
            (0..MEMBERS).map(move |j| {
                let r = R * i as f64 / s as f64;
                let theta = j as f64 * BASE_ANGLE + CURL * i as f64 / MEMBERS as f64;

                let point = (r * theta.cos(), r * theta.sin());

                point
            })
        })
        .collect()
}
