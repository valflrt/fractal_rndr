use image::{Pixel, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

use crate::error::{ErrorKind, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sampling {
    pub level: SamplingLevel,
    pub random_offsets: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SamplingLevel {
    Exploration,
    Low,
    Medium,
    High,
    Ultra,
    Extreme,
    Extreme1,
    Extreme2,
    Extreme3,
}

pub fn generate_sampling_points(sampling_level: SamplingLevel) -> Vec<(f64, f64)> {
    let n = match sampling_level {
        SamplingLevel::Exploration => 5,
        SamplingLevel::Low => 21,
        SamplingLevel::Medium => 34,
        SamplingLevel::High => 55,
        SamplingLevel::Ultra => 89,
        SamplingLevel::Extreme => 144,
        SamplingLevel::Extreme1 => 233,
        SamplingLevel::Extreme2 => 377,
        SamplingLevel::Extreme3 => 610,
    };

    const PHI: f64 = 1.618033988749895;
    const EPS: f64 = 0.5;

    (0..n)
        .map(|i| {
            (
                i as f64 / PHI % 1.,
                (i as f64 + EPS) / ((n - 1) as f64 + 2. * EPS),
            )
        })
        .collect::<Vec<_>>()
}

pub fn map_points_with_offsets(x: f64, y: f64, offset_x: f64, offset_y: f64) -> Option<(f64, f64)> {
    #[inline]
    fn tent(x: f64) -> f64 {
        let x = 2. * x - 1.;
        if x != 0. {
            x / x.abs().powf(0.7) - x.signum()
        } else {
            0.
        }
    }

    let (x, y) = ((x + offset_x) % 1., (y + offset_y) % 1.);

    const R: f64 = 1.5;
    let (x, y) = (R * tent(x), R * tent(y));

    (f64::max(x.abs(), y.abs()) < 0.5).then_some((x, y))
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

            let (offset_x, offset_y) = (fastrand::f64(), fastrand::f64());
            for &(x, y) in sampling_points {
                if let Some((x, y)) = map_points_with_offsets(x, y, offset_x, offset_y) {
                    preview.put_pixel(
                        (center as f64 + 2. * px as f64 * (x + i as f64)) as u32,
                        (center as f64 + 2. * px as f64 * (y + j as f64)) as u32,
                        color,
                    );
                }
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
