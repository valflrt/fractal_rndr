use animation::RenderStep;
use serde::{Deserialize, Serialize};

use crate::{
    coloring::{ColoringMode, Extremum, MapValue},
    fractal::Fractal,
    sampling::{Sampling, SamplingLevel},
    F,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamsKind {
    Frame(FrameParams),
    Animation(AnimationParams),
}

impl Default for ParamsKind {
    fn default() -> Self {
        ParamsKind::Frame(FrameParams {
            img_width: 1920,
            img_height: 1080,
            zoom: 10.,
            center_x: -0.5,
            center_y: 0.,
            rotate: None,
            fractal: Fractal::Mandelbrot,
            max_iter: 100,
            coloring_mode: ColoringMode::MinMaxNorm {
                min: Extremum::Custom(0.),
                max: Extremum::Custom(100.),
                map: MapValue::Linear,
            },
            sampling: Sampling {
                level: SamplingLevel::Exploration,
                random_offsets: true,
            },
            custom_gradient: None,
            dev_options: None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameParams {
    pub img_width: u32,
    pub img_height: u32,

    pub zoom: F,
    pub center_x: F,
    pub center_y: F,
    pub rotate: Option<F>,
    pub fractal: Fractal,

    pub max_iter: u32,

    pub coloring_mode: ColoringMode,
    pub sampling: Sampling,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_gradient: Option<Vec<(F, [u8; 3])>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_options: Option<DevOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationParams {
    pub img_width: u32,
    pub img_height: u32,

    pub zoom: Vec<RenderStep>,
    pub center_x: Vec<RenderStep>,
    pub center_y: Vec<RenderStep>,
    pub rotate: Option<Vec<RenderStep>>,
    pub fractal: animation::Fractal,

    pub max_iter: u32,

    pub duration: F,
    pub fps: F,

    pub coloring_mode: ColoringMode,
    pub sampling: Sampling,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_gradient: Option<Vec<(F, [u8; 3])>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_options: Option<DevOptions>,
}

impl AnimationParams {
    pub fn get_frame_params(&self, t: F) -> FrameParams {
        FrameParams {
            img_width: self.img_width,
            img_height: self.img_height,
            zoom: self.zoom[RenderStep::get_current_step_index(&self.zoom, t)].get_value(t),
            center_x: self.center_x[RenderStep::get_current_step_index(&self.center_x, t)]
                .get_value(t),
            center_y: self.center_y[RenderStep::get_current_step_index(&self.center_y, t)]
                .get_value(t),
            rotate: self
                .rotate
                .clone()
                .map(|v| v[RenderStep::get_current_step_index(&v, t)].get_value(t)),
            fractal: self.fractal.get_fractal(t),
            max_iter: self.max_iter,
            coloring_mode: self.coloring_mode,
            sampling: self.sampling,
            custom_gradient: self.custom_gradient.to_owned(),
            dev_options: self.dev_options,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DevOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_sampling_pattern: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_gradient: Option<bool>,
}

pub mod animation {
    use serde::{Deserialize, Serialize};

    use crate::F;

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub enum RenderStep {
        /// (start_time, end_time, value)
        Const(F, F, F),
        /// (start_time, end_time, start_value, end_value)
        Linear(F, F, F, F),
        /// (start_time, end_time, start_value, end_value)
        Smooth(F, F, F, F),
    }

    impl RenderStep {
        pub fn get_current_step_index(steps: &[RenderStep], t: F) -> usize {
            steps
                .iter()
                .enumerate()
                .find_map(|(i, &step)| match step {
                    RenderStep::Const(start_time, end_time, _)
                    | RenderStep::Linear(start_time, end_time, _, _)
                    | RenderStep::Smooth(start_time, end_time, _, _) => {
                        (start_time <= t && t <= end_time).then_some(i)
                    }
                })
                .unwrap()
        }

        pub fn get_value(&self, t: F) -> F {
            // see https://www.desmos.com/calculator/a1ddmg7pxk
            match *self {
                RenderStep::Const(_, _, value) => value,
                RenderStep::Linear(start_time, end_time, start_value, end_value) => {
                    let w = (t - start_time) / (end_time - start_time);
                    start_value * (1. - w) + end_value * w
                }
                RenderStep::Smooth(start_time, end_time, start_value, end_value) => {
                    let w = (t - start_time) / (end_time - start_time);
                    let smooth_w = w * w * (3. - 2. * w);
                    start_value * (1. - smooth_w) + end_value * smooth_w
                }
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Fractal {
        Mandelbrot,
        MandelbrotCustomExp {
            exp: Vec<RenderStep>,
        },
        Sdrge,
        SdrgeParam {
            a_re: Vec<RenderStep>,
            a_im: Vec<RenderStep>,
        },
        Sdrage,
        Tdrge,
        NthDrge(usize),
        ThirdDegreeRecPairs,
        SecondDegreeThirtySevenBlend,

        Vshqwj,
        Wmriho {
            a_re: Vec<RenderStep>,
            a_im: Vec<RenderStep>,
        },
        Iigdzh {
            a_re: Vec<RenderStep>,
            a_im: Vec<RenderStep>,
        },
        Mjygzr,

        ComplexLogisticMapLike {
            a_re: Vec<RenderStep>,
            a_im: Vec<RenderStep>,
        },
    }

    impl Fractal {
        pub fn get_fractal(&self, t: F) -> crate::fractal::Fractal {
            match self {
                Self::Mandelbrot => crate::fractal::Fractal::Mandelbrot,
                Self::MandelbrotCustomExp { exp } => crate::fractal::Fractal::MandelbrotCustomExp {
                    exp: exp[RenderStep::get_current_step_index(exp, t)].get_value(t),
                },
                Self::Sdrge => crate::fractal::Fractal::Sdrge,
                Self::SdrgeParam { a_re, a_im } => crate::fractal::Fractal::SdrgeParam {
                    a_re: a_re[RenderStep::get_current_step_index(a_re, t)].get_value(t),
                    a_im: a_im[RenderStep::get_current_step_index(a_im, t)].get_value(t),
                },
                Self::Sdrage => crate::fractal::Fractal::Sdrage,
                Self::Tdrge => crate::fractal::Fractal::Tdrge,
                &Self::NthDrge(n) => crate::fractal::Fractal::NthDrge(n),
                Self::ThirdDegreeRecPairs => crate::fractal::Fractal::ThirdDegreeRecPairs,
                Self::SecondDegreeThirtySevenBlend => {
                    crate::fractal::Fractal::SecondDegreeThirtySevenBlend
                }

                Self::Vshqwj => crate::fractal::Fractal::Vshqwj,
                Self::Wmriho { a_re, a_im } => crate::fractal::Fractal::Wmriho {
                    a_re: a_re[RenderStep::get_current_step_index(a_re, t)].get_value(t),
                    a_im: a_im[RenderStep::get_current_step_index(a_im, t)].get_value(t),
                },
                Self::Iigdzh { a_re, a_im } => crate::fractal::Fractal::Iigdzh {
                    a_re: a_re[RenderStep::get_current_step_index(a_re, t)].get_value(t),
                    a_im: a_im[RenderStep::get_current_step_index(a_im, t)].get_value(t),
                },
                Self::Mjygzr => crate::fractal::Fractal::Mjygzr,

                Self::ComplexLogisticMapLike { a_re, a_im } => {
                    crate::fractal::Fractal::ComplexLogisticMapLike {
                        a_re: a_re[RenderStep::get_current_step_index(a_re, t)].get_value(t),
                        a_im: a_im[RenderStep::get_current_step_index(a_im, t)].get_value(t),
                    }
                }
            }
        }
    }
}
