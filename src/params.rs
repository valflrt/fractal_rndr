mod render;

use serde::{Deserialize, Serialize};

use crate::{coloring::ColoringMode, sampling::Sampling};
pub use {render::Render, render::RenderStep};

#[derive(Debug, Serialize, Deserialize)]
pub struct FractalParams {
    pub img_width: u32,
    pub img_height: u32,

    pub render: Render,

    pub max_iter: u32,
    pub coloring_mode: ColoringMode,
    pub sampling: Sampling,

    pub custom_gradient: Option<Vec<(f64, [u8; 3])>>,

    pub diverging_areas: Option<Vec<[f64; 4]>>,

    pub dev_options: Option<DevOptions>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DevOptions {
    pub save_sampling_pattern: bool,
    pub display_gradient: bool,
}
