use serde::{Deserialize, Serialize};
use wide::CmpLe;

use crate::{complexx::Complexx, F, FX};

const fn default_bailout() -> F {
    4.
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Fractal<T>
where
    T: Clone + Serialize,
{
    Mandelbrot {
        max_iter: u32,
        #[serde(default = "default_bailout")]
        bailout: F,
    },
    MandelbrotCustomExp {
        max_iter: u32,
        #[serde(default = "default_bailout")]
        bailout: F,
        exp: T,
    },
    /// Second Degree Recursive sequence with Growing Exponent
    Sdrge {
        max_iter: u32,
        #[serde(default = "default_bailout")]
        bailout: F,
    },
    /// Second Degree Recursive sequence with Growing custom Integer Exponent
    SdrgeCustomIntExp {
        max_iter: u32,
        #[serde(default = "default_bailout")]
        bailout: F,
        exp: usize,
    },
    /// Second Degree Recursive sequence with Growing custom Exponent
    SdrgeCustomExp {
        max_iter: u32,
        #[serde(default = "default_bailout")]
        bailout: F,
        exp: T,
    },
    SdrgeParam {
        max_iter: u32,
        #[serde(default = "default_bailout")]
        bailout: F,
        a_re: T,
        a_im: T,
    },
    /// Second degree recursive alternating sequence with growing exponent
    Sdrage {
        max_iter: u32,
        #[serde(default = "default_bailout")]
        bailout: F,
    },
    /// Third Degree Recursive sequence with Growing Exponent
    Tdrge {
        max_iter: u32,
        #[serde(default = "default_bailout")]
        bailout: F,
    },
    /// Nth Degree Recursive sequence with Growing Exponent
    NthDrge {
        max_iter: u32,
        #[serde(default = "default_bailout")]
        bailout: F,
        n: usize,
    },
    ThirdDegreeRecPairs {
        max_iter: u32,
        #[serde(default = "default_bailout")]
        bailout: F,
    },
    SecondDegreeThirtySevenBlend {
        max_iter: u32,
        #[serde(default = "default_bailout")]
        bailout: F,
    },
    ComplexLogisticMapLike {
        max_iter: u32,
        #[serde(default = "default_bailout")]
        bailout: F,
        a_re: T,
        a_im: T,
    },

    // This is where I started lacking inspiration for names...
    Vshqwj {
        max_iter: u32,
        #[serde(default = "default_bailout")]
        bailout: F,
    },
    Wmriho {
        max_iter: u32,
        #[serde(default = "default_bailout")]
        bailout: F,
        a_re: T,
        a_im: T,
    },
    Iigdzh {
        max_iter: u32,
        #[serde(default = "default_bailout")]
        bailout: F,
        a_re: T,
        a_im: T,
    },
    Fxdicq {
        max_iter: u32,
        #[serde(default = "default_bailout")]
        bailout: F,
    },
    Mjygzr {
        max_iter: u32,
        #[serde(default = "default_bailout")]
        bailout: F,
    },
    Sfwypc {
        max_iter: u32,
        #[serde(default = "default_bailout")]
        bailout: F,
        alpha: (T, T),
        beta: (T, T),
        gamma: (T, T),
    },
    // Test {
    //     max_iter: u32,
    //     #[serde(default = "default_bailout")]
    //     bailout: F,
    // },
    MoireTest,
}

impl Fractal<F> {
    pub fn sample(self, c: Complexx) -> [F; 8] {
        // #[inline]
        // fn zeroed_inf_nan(x: FX) -> FX {
        //     let finite_mask = x.is_finite();
        //     finite_mask.blend(x, FX::ZERO)
        // }

        macro_rules! iterate {
            ($max_iter:expr, $bailout:expr, $update:expr) => {{
                let bailout_sqr = $bailout * $bailout;

                let mut z = Complexx::zeros();

                let mut iter = FX::ZERO;
                for i in 0..$max_iter {
                    let norm_sqr = z.norm_sqr();

                    let not_diverged_mask = norm_sqr.simd_le(bailout_sqr);

                    if not_diverged_mask.none() {
                        break;
                    }

                    let finite_mask = z.is_finite();
                    let u = finite_mask.blend(FX::ONE, FX::ZERO);
                    let nu = finite_mask.blend(FX::ZERO, FX::ONE);

                    z = $update(i) * u + z * nu;

                    iter += not_diverged_mask.blend(FX::ONE, FX::ZERO);
                }

                // (iter, zeroed_inf_nan(1. - (z.norm_sqr().ln() / 2.).ln()))
                (iter, FX::ZERO)
            }};
        }

        let (iter, _frac_iter) = match self {
            Fractal::Mandelbrot { max_iter, bailout } => {
                let mut z = Complexx::zeros();

                iterate!(max_iter, bailout, |_| {
                    z = z * z + c;
                    z
                })
            }
            Fractal::MandelbrotCustomExp {
                max_iter,
                bailout,
                exp,
            } => {
                let mut z = Complexx::zeros();

                iterate!(max_iter, bailout, |_| {
                    z = z.powf(exp) + c;
                    z
                })
            }
            Fractal::Sdrge { max_iter, bailout } => {
                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

                iterate!(max_iter, bailout, |_| {
                    let new_z1 = z1 * z1 + z0 + c;
                    z0 = z1;
                    z1 = new_z1;
                    z1
                })
            }
            Fractal::SdrgeCustomExp {
                max_iter,
                bailout,
                exp,
            } => {
                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

                iterate!(max_iter, bailout, |_| {
                    let new_z1 = z1.powf(exp) + z0 + c;
                    z0 = z1;
                    z1 = new_z1;
                    z1
                })
            }
            Fractal::SdrgeCustomIntExp {
                max_iter,
                bailout,
                exp,
            } => {
                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

                iterate!(max_iter, bailout, |_| {
                    let new_z1 = z1.powu(exp) + z0 + c;
                    z0 = z1;
                    z1 = new_z1;
                    z1
                })
            }
            Fractal::SdrgeParam {
                max_iter,
                bailout,
                a_re,
                a_im,
            } => {
                let a = Complexx::splat(a_re, a_im);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

                iterate!(max_iter, bailout, |_| {
                    let new_z1 = z1 * z1 + a * z0 + c;
                    z0 = z1;
                    z1 = new_z1;
                    z1
                })
            }
            Fractal::Sdrage { max_iter, bailout } => {
                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

                iterate!(max_iter, bailout, |_| {
                    let new_z1 = z1 * z1 - z0 + c;
                    z0 = z1;
                    z1 = new_z1;
                    z1
                })
            }
            Fractal::Tdrge { max_iter, bailout } => {
                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();
                let mut z2 = Complexx::zeros();

                iterate!(max_iter, bailout, |_| {
                    let new_z2 = z2 * z2 * z2 + z1 * z1 + z0 + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;
                    z2
                })
            }
            Fractal::NthDrge {
                max_iter,
                bailout,
                n,
            } => {
                let mut z = vec![Complexx::zeros(); n];

                iterate!(max_iter, bailout, |_| {
                    let mut new_z = c;
                    for (k, z_k) in z.iter().enumerate() {
                        new_z += z_k.powu(k + 1);
                    }
                    for k in 0..n - 1 {
                        z[k] = z[k + 1];
                    }
                    z[n - 1] = new_z;
                    z[n - 1]
                })
            }
            Fractal::ThirdDegreeRecPairs { max_iter, bailout } => {
                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();
                let mut z2 = Complexx::zeros();

                iterate!(max_iter, bailout, |_| {
                    let new_z2 = z0 * z1 + z0 * z2 + z1 * z2 + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;
                    z2
                })
            }
            Fractal::SecondDegreeThirtySevenBlend { max_iter, bailout } => {
                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

                iterate!(max_iter, bailout, |i| {
                    if i % 37 == 0 {
                        let new_z1 = z1 * z1 - z0 + c;
                        z0 = z1;
                        z1 = new_z1;
                    } else {
                        let new_z1 = z1 * z1 + z0;
                        z0 = z1;
                        z1 = new_z1;
                    }
                    z1
                })
            }
            Fractal::ComplexLogisticMapLike {
                max_iter,
                bailout,
                a_re,
                a_im,
            } => {
                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

                iterate!(max_iter, bailout, |_| {
                    let new_z1 = z1 * (Complexx::splat(a_re, a_im) - z0) + c;
                    z0 = z1;
                    z1 = new_z1;
                    z1
                })
            }

            Fractal::Vshqwj { max_iter, bailout } => {
                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();
                let mut z2 = Complexx::zeros();

                iterate!(max_iter, bailout, |_| {
                    let new_z2 = (z2 + z1) * (z1 + z0) * (z2 - z0) + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;
                    z2
                })
            }
            Fractal::Wmriho {
                max_iter,
                bailout,
                a_re,
                a_im,
            } => {
                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();
                let mut z2 = Complexx::splat(a_re, a_im);

                iterate!(max_iter, bailout, |_| {
                    let new_z2 = z2 * z2
                        + z1 * z0
                        + Complexx {
                            re: z0.im,
                            im: z0.re,
                        }
                        + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;
                    z2
                })
            }
            Fractal::Iigdzh {
                max_iter,
                bailout,
                a_re,
                a_im,
            } => {
                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();
                let mut z2 = Complexx::splat(a_re, a_im);

                iterate!(max_iter, bailout, |_| {
                    let new_z2 = z2 * z2
                        + Complexx {
                            re: z0.im + z1.re,
                            im: z2.re,
                        }
                        + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;
                    z2
                })
            }
            Fractal::Fxdicq { max_iter, bailout } => {
                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();
                let mut z2 = Complexx::zeros();

                iterate!(max_iter, bailout, |_| {
                    let new_z2 = z2 * z2
                        + Complexx {
                            re: z0.im * z1.re,
                            im: z2.re,
                        }
                        + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;
                    z2
                })
            }
            Fractal::Mjygzr { max_iter, bailout } => {
                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

                iterate!(max_iter, bailout, |_| {
                    let new_z = z1 * z1 * c + z0 + c;
                    z0 = z1;
                    z1 = new_z;
                    z1
                })
            }
            Fractal::Sfwypc {
                max_iter,
                bailout,
                alpha,
                beta,
                gamma,
            } => {
                let alpha = Complexx::splat(alpha.0, alpha.1);
                let beta = Complexx::splat(beta.0, beta.1);
                let gamma = Complexx::splat(gamma.0, gamma.1);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();
                let mut z2 = Complexx::zeros();

                iterate!(max_iter, bailout, |_| {
                    let new_z = (z0 - alpha) * (z1 - beta) * (z2 - gamma) + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z;
                    z2
                })
            }
            // Fractal::Test { max_iter, bailout } => {}
            Fractal::MoireTest => {
                let Complexx { re: x, im: y } = c * 100.;
                ((x * x + y * y).sin().abs(), FX::ONE)
            }
        };

        iter.to_array()
    }
}

macro_rules! impl_extract_field {
    ($field:ident, $field_mut:ident, $type:ident, [$($variant:ident),+]) => {
        #[allow(dead_code)]
        impl<T> Fractal<T>
        where
            T: Clone + Serialize,
        {
            pub fn $field(&self) -> Option<$type> {
                match self {
                    $(Fractal::$variant { $field, .. } => Some(*$field)),+,
                    _ => None,
                }
            }

            pub fn $field_mut(&mut self) -> Option<&mut $type> {
                match self {
                    $(Fractal::$variant { $field, .. } => Some($field)),+,
                    _ => None,
                }
            }
        }
    };
}

impl_extract_field!(
    max_iter,
    max_iter_mut,
    u32,
    [
        Mandelbrot,
        MandelbrotCustomExp,
        Sdrge,
        SdrgeCustomIntExp,
        SdrgeCustomExp,
        SdrgeParam,
        Sdrage,
        Tdrge,
        NthDrge,
        ThirdDegreeRecPairs,
        SecondDegreeThirtySevenBlend,
        ComplexLogisticMapLike,
        Vshqwj,
        Wmriho,
        Iigdzh,
        Fxdicq,
        Mjygzr,
        Sfwypc // , Test
    ]
);

impl_extract_field!(
    bailout,
    bailout_mut,
    F,
    [
        Mandelbrot,
        MandelbrotCustomExp,
        Sdrge,
        SdrgeCustomIntExp,
        SdrgeCustomExp,
        SdrgeParam,
        Sdrage,
        Tdrge,
        NthDrge,
        ThirdDegreeRecPairs,
        SecondDegreeThirtySevenBlend,
        ComplexLogisticMapLike,
        Vshqwj,
        Wmriho,
        Iigdzh,
        Fxdicq,
        Mjygzr,
        Sfwypc // , Test
    ]
);
