mod render_kind;

use serde::{Deserialize, Serialize};

use crate::{coloring::ColoringMode, sampling::Sampling};
pub use {render_kind::RenderKind, render_kind::RenderStep};

#[derive(Debug, Serialize, Deserialize)]
pub struct FractalParams {
    pub img_width: u32,
    pub img_height: u32,

    pub render: RenderKind,

    pub max_iter: u32,
    pub coloring_mode: ColoringMode,
    pub sampling: Sampling,

    pub custom_gradient: Option<Vec<(f32, [u8; 3])>>,

    pub dev_options: Option<DevOptions>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DevOptions {
    pub save_sampling_pattern: Option<bool>,
    pub display_gradient: Option<bool>,
}
