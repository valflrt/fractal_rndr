use image::Rgb;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum ColoringMode {
    BlackAndWhite,
    Linear,
    Squared,
    LinearMinMax,
    #[default]
    CumulativeHistogram,
}

pub fn compute_histogram(iter_values: &[f64], max_iter: u32) -> Vec<u32> {
    let mut histogram = vec![0; max_iter as usize + 1];

    for &iteration_count in iter_values.iter() {
        histogram[iteration_count as usize] += 1;
    }

    histogram
}

pub fn cumulate_histogram(histogram: Vec<u32>, max_iter: u32) -> Vec<f64> {
    let total = histogram.iter().sum::<u32>();
    let mut cumulative = vec![0.; max_iter as usize + 1];
    let mut cumulative_sum = 0.;
    for (i, &count) in histogram.iter().enumerate() {
        cumulative_sum += count as f64 / total as f64;
        cumulative[i] = cumulative_sum;
    }

    cumulative
}

const DEFAULT_GRADIENT: [(f64, [u8; 3]); 8] = [
    (0., [10, 2, 20]),
    (0.1, [200, 40, 230]),
    (0.25, [20, 160, 230]),
    (0.4, [60, 230, 80]),
    (0.55, [255, 230, 20]),
    (0.7, [255, 120, 20]),
    (0.85, [255, 40, 60]),
    (0.95, [2, 0, 4]),
];

pub fn color_mapping(t: f64, custom_gradient: Option<&Vec<(f64, [u8; 3])>>) -> Rgb<u8> {
    fn map(t: f64, gradient: &[(f64, [u8; 3])]) -> Rgb<u8> {
        let first = gradient[0];
        let last = gradient.last().unwrap();
        if t <= first.0 {
            Rgb(first.1)
        } else if t >= last.0 {
            Rgb(last.1)
        } else {
            for i in 0..gradient.len() {
                if gradient[i].0 <= t && t <= gradient[i + 1].0 {
                    let ratio = (t - gradient[i].0) / (gradient[i + 1].0 - gradient[i].0);
                    let [r1, g1, b1] = gradient[i].1;
                    let [r2, g2, b2] = gradient[i + 1].1;
                    let r = (r1 as f64 * (1. - ratio) + r2 as f64 * ratio).clamp(0., 255.) as u8;
                    let g = (g1 as f64 * (1. - ratio) + g2 as f64 * ratio).clamp(0., 255.) as u8;
                    let b = (b1 as f64 * (1. - ratio) + b2 as f64 * ratio).clamp(0., 255.) as u8;
                    return Rgb([r, g, b]);
                }
            }
            Rgb(last.1)
        }
    }

    if let Some(g) = custom_gradient {
        map(t, g)
    } else {
        map(t, DEFAULT_GRADIENT.as_ref())
    }
}
