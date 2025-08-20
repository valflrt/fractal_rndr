use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    coloring::{ColoringMode, Extremum, MapValue, DEFAULT_GRADIENT},
    error::{ErrorKind, Result},
    fractal::Fractal,
    params::animation::{AnimationCfg, AnimationSteps},
    sampling::{Sampling, SamplingLevel},
    F,
};

fn default_gradient() -> Vec<(F, [u8; 3])> {
    DEFAULT_GRADIENT.to_vec()
}

pub fn read_parameter_file<P>(path: P) -> Result<ParamsKind>
where
    P: AsRef<Path>,
{
    let param_file_str = fs::read_to_string(path).map_err(ErrorKind::ReadParameterFile)?;
    let params =
        ron::from_str::<ParamsKind>(&param_file_str).map_err(ErrorKind::DecodeParameterFile)?;
    Ok(params)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamsKind {
    Frame(Params<F>),
    Animation(Params<AnimationSteps>),
}

impl Default for ParamsKind {
    fn default() -> Self {
        ParamsKind::Frame(Params::<F> {
            img_width: 1920,
            img_height: 1080,

            zoom: 10.,
            center_x: -0.5,
            center_y: 0.,
            rotate: None,
            fractal: Fractal::Mandelbrot { max_iter: 500 },

            coloring_mode: ColoringMode::MinMaxNorm {
                min: Extremum::Custom(0.),
                max: Extremum::Custom(500.),
                map: MapValue::Linear,
            },
            gradient: DEFAULT_GRADIENT.to_vec(),

            sampling: Sampling {
                level: SamplingLevel::Exploration,
                random_offsets: true,
            },

            animation_cfg: None,

            dev_options: None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Params<T>
where
    T: Clone + Serialize,
{
    pub img_width: u32,
    pub img_height: u32,

    pub zoom: T,
    pub center_x: T,
    pub center_y: T,
    pub rotate: Option<T>,
    pub fractal: Fractal<T>,

    pub coloring_mode: ColoringMode,
    #[serde(default = "default_gradient")]
    pub gradient: Vec<(F, [u8; 3])>,

    pub sampling: Sampling,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub animation_cfg: Option<AnimationCfg>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_options: Option<DevOptions>,
}

impl Params<AnimationSteps> {
    pub fn get_frame_params(&self, t: F) -> Params<F> {
        Params::<F> {
            img_width: self.img_width,
            img_height: self.img_height,

            zoom: self.zoom.get(t),
            center_x: self.center_x.get(t),
            center_y: self.center_y.get(t),
            rotate: self.rotate.as_ref().map(|v| v.get(t)),
            fractal: self.fractal.get(t),

            coloring_mode: self.coloring_mode,
            gradient: self.gradient.to_owned(),

            sampling: self.sampling,

            animation_cfg: None,

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

    use crate::{fractal::Fractal, F};

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct AnimationCfg {
        pub duration: F,
        pub fps: F,
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub enum Transition {
        /// (start_time, end_time, value)
        Const(F, F, F),
        /// (start_time, end_time, start_value, end_value)
        Linear(F, F, F, F),
        /// (start_time, end_time, start_value, end_value)
        Smooth(F, F, F, F),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AnimationSteps(Vec<Transition>);

    impl AnimationSteps {
        pub fn get(&self, t: F) -> F {
            self.0
                .iter()
                .find_map(|&transition| match transition {
                    Transition::Const(start_time, end_time, _)
                    | Transition::Linear(start_time, end_time, _, _)
                    | Transition::Smooth(start_time, end_time, _, _) => {
                        (start_time <= t && t <= end_time).then_some(transition)
                    }
                })
                .map(|transition| {
                    // see https://www.desmos.com/calculator/a1ddmg7pxk
                    match transition {
                        Transition::Const(_, _, value) => value,
                        Transition::Linear(start_time, end_time, start_value, end_value) => {
                            let w = (t - start_time) / (end_time - start_time);
                            start_value * (1. - w) + end_value * w
                        }
                        Transition::Smooth(start_time, end_time, start_value, end_value) => {
                            let w = (t - start_time) / (end_time - start_time);
                            let smooth_w = w * w * (3. - 2. * w);
                            start_value * (1. - smooth_w) + end_value * smooth_w
                        }
                    }
                })
                .unwrap()
        }
    }

    impl Fractal<AnimationSteps> {
        pub fn get(&self, t: F) -> Fractal<F> {
            match self {
                &Self::Mandelbrot { max_iter } => Fractal::<F>::Mandelbrot { max_iter },
                &Self::MandelbrotCustomExp { max_iter, ref exp } => {
                    Fractal::<F>::MandelbrotCustomExp {
                        max_iter,
                        exp: exp.get(t),
                    }
                }
                &Self::Sdrge { max_iter } => Fractal::<F>::Sdrge { max_iter },
                &Self::SdrgeParam {
                    max_iter,
                    ref a_re,
                    ref a_im,
                } => Fractal::<F>::SdrgeParam {
                    max_iter,
                    a_re: a_re.get(t),
                    a_im: a_im.get(t),
                },
                &Self::SdrgeCustomExp { max_iter, ref exp } => Fractal::<F>::SdrgeCustomExp {
                    max_iter,
                    exp: exp.get(t),
                },
                &Self::SdrgeCustomIntExp { max_iter, exp } => {
                    Fractal::<F>::SdrgeCustomIntExp { max_iter, exp }
                }
                &Self::Sdrage { max_iter } => Fractal::<F>::Sdrage { max_iter },
                &Self::Tdrge { max_iter } => Fractal::<F>::Tdrge { max_iter },
                &Self::NthDrge { max_iter, n } => Fractal::<F>::NthDrge { max_iter, n },
                &Self::ThirdDegreeRecPairs { max_iter } => {
                    Fractal::<F>::ThirdDegreeRecPairs { max_iter }
                }
                &Self::SecondDegreeThirtySevenBlend { max_iter } => {
                    Fractal::<F>::SecondDegreeThirtySevenBlend { max_iter }
                }
                &Self::Vshqwj { max_iter } => Fractal::<F>::Vshqwj { max_iter },
                &Self::Wmriho {
                    max_iter,
                    ref a_re,
                    ref a_im,
                } => Fractal::<F>::Wmriho {
                    max_iter,
                    a_re: a_re.get(t),
                    a_im: a_im.get(t),
                },
                &Self::Iigdzh {
                    max_iter,
                    ref a_re,
                    ref a_im,
                } => Fractal::<F>::Iigdzh {
                    max_iter,
                    a_re: a_re.get(t),
                    a_im: a_im.get(t),
                },
                &Self::Mjygzr { max_iter } => Fractal::<F>::Mjygzr { max_iter },
                &Self::Fxdicq { max_iter } => Fractal::<F>::Fxdicq { max_iter },
                &Self::Sfwypc {
                    max_iter,
                    alpha: (ref alpha_re, ref alpha_im),
                    beta: (ref beta_re, ref beta_im),
                    gamma: (ref gamma_re, ref gamma_im),
                } => Fractal::<F>::Sfwypc {
                    max_iter,
                    alpha: (alpha_re.get(t), alpha_im.get(t)),
                    beta: (beta_re.get(t), beta_im.get(t)),
                    gamma: (gamma_re.get(t), gamma_im.get(t)),
                },

                &Self::ComplexLogisticMapLike {
                    max_iter,
                    ref a_re,
                    ref a_im,
                } => Fractal::<F>::ComplexLogisticMapLike {
                    max_iter,
                    a_re: a_re.get(t),
                    a_im: a_im.get(t),
                },

                Fractal::MoireTest => Fractal::MoireTest,
            }
        }
    }
}
