use serde::{Deserialize, Serialize};

use crate::params::RenderStep;

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

            Self::ComplexLogisticMapLike { re, im } => {
                crate::fractal::Fractal::ComplexLogisticMapLike {
                    re: re[RenderStep::get_current_step_index(re, t)].get_value(t),
                    im: im[RenderStep::get_current_step_index(im, t)].get_value(t),
                }
            }
        }
    }
}
