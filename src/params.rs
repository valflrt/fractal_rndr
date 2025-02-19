use animation::RenderStep;
use serde::{Deserialize, Serialize};

use crate::{coloring::ColoringMode, fractal::Fractal, sampling::Sampling, F};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamsKind {
    Frame {
        img_width: u32,
        img_height: u32,

        zoom: F,
        center_x: F,
        center_y: F,
        fractal: Fractal,

        max_iter: u32,
        coloring_mode: ColoringMode,
        sampling: Sampling,

        custom_gradient: Option<Vec<(f32, [u8; 3])>>,

        dev_options: Option<DevOptions>,
    },
    Animation {
        img_width: u32,
        img_height: u32,

        zoom: Vec<RenderStep>,
        center_x: Vec<RenderStep>,
        center_y: Vec<RenderStep>,
        fractal: animation::Fractal,
        duration: f32,
        fps: f32,

        max_iter: u32,
        coloring_mode: ColoringMode,
        sampling: Sampling,

        custom_gradient: Option<Vec<(f32, [u8; 3])>>,

        dev_options: Option<DevOptions>,
    },
}

impl ParamsKind {
    pub fn get_frame_params(&self, t: f32) -> FrameParams {
        match self.to_owned() {
            ParamsKind::Frame {
                img_width,
                img_height,
                zoom,
                center_x,
                center_y,
                fractal,
                max_iter,
                coloring_mode,
                sampling,
                custom_gradient,
                dev_options,
            } => FrameParams {
                img_width,
                img_height,
                zoom,
                center_x,
                center_y,
                fractal,
                max_iter,
                coloring_mode,
                sampling,
                custom_gradient,
                dev_options,
            },
            ParamsKind::Animation {
                img_width,
                img_height,
                zoom,
                center_x,
                center_y,
                fractal,
                max_iter,
                coloring_mode,
                sampling,
                custom_gradient,
                dev_options,
                ..
            } => FrameParams {
                img_width: img_width,
                img_height: img_height,
                zoom: zoom[RenderStep::get_current_step_index(&zoom, t)].get_value(t),
                center_x: center_x[RenderStep::get_current_step_index(&center_x, t)].get_value(t),
                center_y: center_y[RenderStep::get_current_step_index(&center_y, t)].get_value(t),
                fractal: fractal.get_fractal(t),
                max_iter: max_iter,
                coloring_mode: coloring_mode,
                sampling: sampling,
                custom_gradient: custom_gradient.to_owned(),
                dev_options: dev_options,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameParams {
    pub img_width: u32,
    pub img_height: u32,

    pub zoom: F,
    pub center_x: F,
    pub center_y: F,
    pub fractal: Fractal,

    pub max_iter: u32,
    pub coloring_mode: ColoringMode,
    pub sampling: Sampling,

    pub custom_gradient: Option<Vec<(f32, [u8; 3])>>,

    pub dev_options: Option<DevOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationParams {
    pub img_width: u32,
    pub img_height: u32,

    pub zoom: Vec<RenderStep>,
    pub center_x: Vec<RenderStep>,
    pub center_y: Vec<RenderStep>,
    pub fractal: animation::Fractal,
    pub duration: f32,
    pub fps: f32,

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

pub mod animation {
    use serde::{Deserialize, Serialize};

    use crate::F;

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub enum RenderStep {
        /// (start_time, end_time, value)
        Const(f32, f32, F),
        /// (start_time, end_time, start_value, end_value)
        Linear(f32, f32, F, F),
        /// (start_time, end_time, start_value, end_value)
        Smooth(f32, f32, F, F),
    }

    impl RenderStep {
        pub fn get_current_step_index(steps: &[RenderStep], t: f32) -> usize {
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

        pub fn get_value(&self, t: f32) -> F {
            // see https://www.desmos.com/calculator/a1ddmg7pxk
            match *self {
                RenderStep::Const(_, _, value) => value,
                RenderStep::Linear(start_time, end_time, start_value, end_value) => {
                    let w = ((t - start_time) / (end_time - start_time)) as F;
                    start_value * (1. - w) + end_value * w
                }
                RenderStep::Smooth(start_time, end_time, start_value, end_value) => {
                    let w = ((t - start_time) / (end_time - start_time)) as F;
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
        SecondDegreeRecWithGrowingExponent,
        SecondDegreeRecWithGrowingExponentParam {
            a_re: Vec<RenderStep>,
            a_im: Vec<RenderStep>,
        },
        SecondDegreeRecAlternating1WithGrowingExponent,
        ThirdDegreeRecWithGrowingExponent,
        NthDegreeRecWithGrowingExponent(usize),
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
            re: Vec<RenderStep>,
            im: Vec<RenderStep>,
        },
    }

    impl Fractal {
        pub fn get_fractal(&self, t: f32) -> crate::fractal::Fractal {
            match self {
                Self::Mandelbrot => crate::fractal::Fractal::Mandelbrot,
                Self::MandelbrotCustomExp { exp } => crate::fractal::Fractal::MandelbrotCustomExp {
                    exp: exp[RenderStep::get_current_step_index(exp, t)].get_value(t),
                },
                Self::SecondDegreeRecWithGrowingExponent => {
                    crate::fractal::Fractal::SecondDegreeRecWithGrowingExponent
                }
                Self::SecondDegreeRecWithGrowingExponentParam { a_re, a_im } => {
                    crate::fractal::Fractal::SecondDegreeRecWithGrowingExponentParam {
                        a_re: a_re[RenderStep::get_current_step_index(a_re, t)].get_value(t),
                        a_im: a_im[RenderStep::get_current_step_index(a_im, t)].get_value(t),
                    }
                }
                Self::SecondDegreeRecAlternating1WithGrowingExponent => {
                    crate::fractal::Fractal::SecondDegreeRecAlternating1WithGrowingExponent
                }
                Self::ThirdDegreeRecWithGrowingExponent => {
                    crate::fractal::Fractal::ThirdDegreeRecWithGrowingExponent
                }
                &Self::NthDegreeRecWithGrowingExponent(n) => {
                    crate::fractal::Fractal::NthDegreeRecWithGrowingExponent(n)
                }
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

                Self::ComplexLogisticMapLike { re, im } => {
                    crate::fractal::Fractal::ComplexLogisticMapLike {
                        re: re[RenderStep::get_current_step_index(re, t)].get_value(t),
                        im: im[RenderStep::get_current_step_index(im, t)].get_value(t),
                    }
                }
            }
        }
    }
}
