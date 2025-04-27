use image::{Pixel, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

use crate::{
    error::{ErrorKind, Result},
    F,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sampling {
    pub level: SamplingLevel,
    pub random_offsets: bool,
}

impl Sampling {
    pub fn generate_sampling_points(&self) -> Vec<(F, F)> {
        const PHI: F = 1.618033988749895;
        const EPS: F = 0.5;

        let n = self.sample_count();

        (0..n)
            .map(|i| {
                (
                    i as F / PHI % 1.,
                    (i as F + EPS) / ((n - 1) as F + 2. * EPS),
                )
            })
            .collect::<Vec<_>>()
    }

    pub fn sample_count(&self) -> usize {
        match self.level {
            SamplingLevel::Exploration => 8,
            SamplingLevel::Low => 21,
            SamplingLevel::Medium => 34,
            SamplingLevel::High => 55,
            SamplingLevel::Ultra => 89,
            SamplingLevel::Extreme => 144,
            SamplingLevel::Extreme1 => 233,
            SamplingLevel::Extreme2 => 377,
            SamplingLevel::Extreme3 => 610,
        }
    }
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

pub fn map_points_with_offsets(x: F, y: F, offset_x: F, offset_y: F) -> (F, F) {
    #[inline]
    fn tent(x: F) -> F {
        let x = 2. * x - 1.;
        if x != 0. {
            x / x.abs().powf(0.7) - x.signum()
        } else {
            0.
        }
    }

    let (x, y) = ((x + offset_x) % 1., (y + offset_y) % 1.);

    const R: F = 1.8;
    let (x, y) = (R * tent(x), R * tent(y));

    (x, y)
}

pub fn preview_sampling_points(sampling_points: &Vec<(F, F)>) -> Result<()> {
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

            if i == 0 && j == 0 {
                #[cfg(feature = "force_f32")]
                let (offset_x, offset_y) = (fastrand::f32(), fastrand::f32());
                #[cfg(not(feature = "force_f32"))]
                let (offset_x, offset_y) = (fastrand::f64(), fastrand::f64());
                for &(x, y) in sampling_points {
                    let (x, y) = map_points_with_offsets(x, y, offset_x, offset_y);
                    preview.put_pixel(
                        (center as F + 2. * px as F * (x + i as F)) as u32,
                        (center as F + 2. * px as F * (y + j as F)) as u32,
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
