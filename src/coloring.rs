use image::{Rgb, RgbImage};
use serde::{Deserialize, Serialize};

use crate::{array2::Array2, params::Params, F};

pub fn color_raw_image(params: &Params<F>, raw_image: Array2<F>) -> RgbImage {
    let &Params {
        img_width,
        img_height,
        ..
    } = params;

    let mut output_image = RgbImage::new(img_width, img_height);

    let max_v = raw_image.vec.iter().copied().fold(0., F::max);
    let min_v = raw_image.vec.iter().copied().fold(max_v, F::min);

    match params.coloring_mode {
        ColoringMode::MinMaxNorm { min, max, map } => {
            let min = min.unwrap_custom_or(min_v);
            let max = max.unwrap_custom_or(max_v);

            for j in 0..img_height as usize {
                for i in 0..img_width as usize {
                    let value = raw_image[(i, j)];

                    let t = map.apply((value - min) / (max - min));

                    output_image.put_pixel(i as u32, j as u32, color_mapping(t, &params.gradient));
                }
            }
        }
    };

    output_image
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ColoringMode {
    MinMaxNorm {
        #[serde(default)]
        min: Extremum,
        #[serde(default)]
        max: Extremum,
        map: MapValue,
    },
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum Extremum {
    #[default]
    Auto,
    Custom(F),
}

impl Extremum {
    pub fn is_auto(&self) -> bool {
        matches!(self, Extremum::Auto)
    }

    pub fn unwrap_custom_or(self, default: F) -> F {
        if let Extremum::Custom(x) = self {
            x
        } else {
            default
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MapValue {
    Linear,
    Powf(F),
}

impl MapValue {
    #[inline]
    pub fn apply(&self, t: F) -> F {
        match self {
            MapValue::Linear => t,
            MapValue::Powf(p) => {
                let t = t.powf(*p);
                if t.is_normal() {
                    t
                } else {
                    0.
                }
            }
        }
    }
}

pub const DEFAULT_GRADIENT: &[(F, [u8; 3])] = &[
    (0.0, [230, 230, 240]),
    (0.3, [230, 180, 180]),
    (0.5, [60, 60, 90]),
    (1.0, [220, 210, 220]),
];
#[allow(dead_code)]
pub const OLD_DEFAULT_GRADIENT: &[(F, [u8; 3])] = &[
    (0., [20, 8, 30]),
    (0.1, [160, 30, 200]),
    (0.25, [20, 160, 230]),
    (0.4, [60, 230, 80]),
    (0.55, [255, 230, 20]),
    (0.7, [255, 120, 20]),
    (0.85, [255, 40, 60]),
    (1., [20, 2, 10]),
];

pub fn color_mapping(t: F, gradient: &[(F, [u8; 3])]) -> Rgb<u8> {
    let first = gradient[0];
    let last = gradient.last().unwrap();

    if t < first.0 {
        return Rgb(first.1);
    }

    if t > last.0 {
        return Rgb(last.1);
    }

    let i = gradient.partition_point(|&(v, _)| v < t).saturating_sub(1);

    let ratio = (t - gradient[i].0) / (gradient[i + 1].0 - gradient[i].0);
    let [r1, g1, b1] = gradient[i].1;
    let [r2, g2, b2] = gradient[i + 1].1;
    let r = (r1 as F * (1. - ratio) + r2 as F * ratio).clamp(0., 255.) as u8;
    let g = (g1 as F * (1. - ratio) + g2 as F * ratio).clamp(0., 255.) as u8;
    let b = (b1 as F * (1. - ratio) + b2 as F * ratio).clamp(0., 255.) as u8;

    Rgb([r, g, b])
}
