use image::{Pixel, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

use crate::error::{ErrorKind, Result};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SamplingLevel {
    Low,
    #[default]
    Medium,
    High,
    Ultra,
    Extreme,
}

pub fn generate_sampling_points(sampling_level: Option<SamplingLevel>) -> Vec<(f64, f64)> {
    // Maybe too precise but yes.
    // const PHI: f64 = 1.618033988749895;

    let n: i32 = match sampling_level.unwrap_or_default() {
        SamplingLevel::Low => 1,
        SamplingLevel::Medium => 2,
        SamplingLevel::High => 3,
        SamplingLevel::Ultra => 4,
        SamplingLevel::Extreme => 5,
    };

    let samples = (0..n)
        .flat_map(|j| (0..n).map(move |i| (i, j)))
        .flat_map(|(i, j)| {
            let (x, y) = (2. * i as f64 / n as f64 - 1., 2. * j as f64 / n as f64 - 1.);
            [(x, y), (x + 1. / n as f64, y + 1. / n as f64)]
        })
        .collect::<Vec<_>>();

    // (0..n)
    //     .map(|i| (i as f64 / PHI % 1., i as f64 / (n - 1) as f64))
    //     .filter_map(|(x, y)| {
    //         let (x, y) = (2. * x - 1., 2. * y - 1.);
    //         (x.max(y) < 1.).then_some((x, y));
    //         Some((x, y))
    //     })
    //     // .filter_map(|(x, y)| {
    //     //     let r = R * y.sqrt();
    //     //     let theta = f64::consts::TAU * x;
    //     //     let x = r * theta.cos();
    //     //     let y = r * theta.sin();
    //     //     (x.abs().max(y.abs()) < 1.).then_some((r, theta))
    //     // })
    //     .collect::<Vec<_>>();

    samples
}

pub fn preview_sampling_points(sampling_points: &Vec<(f64, f64)>) -> Result<()> {
    let size = 350;
    let center = size / 2;
    let px = 50;
    let mut preview = RgbaImage::from_pixel(size, size, Rgba([0, 0, 0, 255]));

    for i in -1..=1 {
        for j in -1..=1 {
            let color = if i == 0 && j == 0 {
                Rgba([255, 255, 255, 255])
            } else {
                Rgba([120, 120, 120, 255])
            };
            for &(x, y) in sampling_points {
                preview.put_pixel(
                    (center as f64 + px as f64 * (x + 2. * i as f64)) as u32,
                    (center as f64 + px as f64 * (y + 2. * j as f64)) as u32,
                    color,
                );
            }
        }
    }

    let color = Rgba([255, 0, 0, 220]);
    preview
        .get_pixel_mut(center - px, center - px)
        .blend(&color);
    preview.get_pixel_mut(center - px, center).blend(&color);
    preview
        .get_pixel_mut(center - px, center + px)
        .blend(&color);
    preview.get_pixel_mut(center, center - px).blend(&color);
    preview.get_pixel_mut(center, center + px).blend(&color);
    preview
        .get_pixel_mut(center + px, center - px)
        .blend(&color);
    preview.get_pixel_mut(center + px, center).blend(&color);
    preview
        .get_pixel_mut(center + px, center + px)
        .blend(&color);

    preview
        .save("_sampling_pattern.png")
        .map_err(ErrorKind::SaveImage)?;

    Ok(())
}
