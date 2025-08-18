use serde::{Deserialize, Serialize};
use wide::CmpLe;

use crate::{complexx::Complexx, F, FX};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Fractal<T>
where
    T: Clone + Serialize,
{
    Mandelbrot {
        max_iter: u32,
    },
    MandelbrotCustomExp {
        max_iter: u32,
        exp: T,
    },
    /// Second Degree Recursive sequence with Growing Exponent
    Sdrge {
        max_iter: u32,
    },
    /// Second Degree Recursive sequence with Growing custom Integer Exponent
    SdrgeCustomIntExp {
        max_iter: u32,
        exp: usize,
    },
    /// Second Degree Recursive sequence with Growing custom Exponent
    SdrgeCustomExp {
        max_iter: u32,
        exp: T,
    },
    SdrgeParam {
        max_iter: u32,
        a_re: T,
        a_im: T,
    },
    /// Second degree recursive alternating sequence with growing exponent
    Sdrage {
        max_iter: u32,
    },
    /// Third Degree Recursive sequence with Growing Exponent
    Tdrge {
        max_iter: u32,
    },
    /// Nth Degree Recursive sequence with Growing Exponent
    NthDrge {
        max_iter: u32,
        n: usize,
    },
    ThirdDegreeRecPairs {
        max_iter: u32,
    },
    SecondDegreeThirtySevenBlend {
        max_iter: u32,
    },
    ComplexLogisticMapLike {
        max_iter: u32,
        a_re: T,
        a_im: T,
    },

    // This is where I started lacking inspiration for names...
    Vshqwj {
        max_iter: u32,
    },
    Wmriho {
        max_iter: u32,
        a_re: T,
        a_im: T,
    },
    Iigdzh {
        max_iter: u32,
        a_re: T,
        a_im: T,
    },
    Fxdicq {
        max_iter: u32,
    },
    Mjygzr {
        max_iter: u32,
    },
    Sfwypc {
        max_iter: u32,
        alpha: (T, T),
        beta: (T, T),
        gamma: (T, T),
    },

    MoireTest,
}

#[cfg(feature = "force_f32")]
type Out = [F; 8];
#[cfg(not(feature = "force_f32"))]
type Out = [F; 4];

impl Fractal<F> {
    pub fn sample(self, c: Complexx) -> Out {
        let one = FX::splat(1.0);
        let zero = FX::splat(0.0);

        let (iter, _last_z) = match self {
            Fractal::Mandelbrot { max_iter } => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z = Complexx::zeros();

                let mut iter = FX::splat(0.);
                for _ in 0..max_iter {
                    let undiverged_mask = z.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    z = z * z + c;

                    iter += undiverged_mask.blend(one, zero);
                }

                (iter, z)
            }
            Fractal::MandelbrotCustomExp { max_iter, exp } => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z = Complexx::zeros();

                let mut iter = FX::splat(0.);
                for _ in 0..max_iter {
                    let undiverged_mask = z.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    z = z.powf(exp) + c;

                    iter += undiverged_mask.blend(one, zero);
                }

                (iter, z)
            }
            Fractal::Sdrge { max_iter } => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

                let mut iter = FX::splat(0.);
                for _ in 0..max_iter {
                    let undiverged_mask = z1.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    let new_z1 = z1 * z1 + z0 + c;
                    z0 = z1;
                    z1 = new_z1;

                    iter += undiverged_mask.blend(one, zero);
                }

                (iter, z1)
            }
            Fractal::SdrgeCustomExp { max_iter, exp } => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

                let mut iter = FX::splat(0.);
                for _ in 0..max_iter {
                    let undiverged_mask = z1.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    let new_z1 = z1.powf(exp) + z0 + c;
                    z0 = z1;
                    z1 = new_z1;

                    iter += undiverged_mask.blend(one, zero);
                }

                (iter, z1)
            }
            Fractal::SdrgeCustomIntExp { max_iter, exp } => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

                let mut iter = FX::splat(0.);
                for _ in 0..max_iter {
                    let undiverged_mask = z1.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    let new_z1 = z1.powu(exp) + z0 + c;
                    z0 = z1;
                    z1 = new_z1;

                    iter += undiverged_mask.blend(one, zero);
                }

                (iter, z1)
            }
            Fractal::SdrgeParam {
                max_iter,
                a_re,
                a_im,
            } => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let a = Complexx::splat(a_re, a_im);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

                let mut iter = FX::splat(0.);
                for _ in 0..max_iter {
                    let undiverged_mask = z1.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    let new_z1 = z1 * z1 + a * z0 + c;
                    z0 = z1;
                    z1 = new_z1;

                    iter += undiverged_mask.blend(one, zero);
                }

                (iter, z1)
            }
            Fractal::Sdrage { max_iter } => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

                let mut iter = FX::splat(0.);
                for _ in 0..max_iter {
                    let undiverged_mask = z1.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    let new_z1 = z1 * z1 - z0 + c;
                    z0 = z1;
                    z1 = new_z1;

                    iter += undiverged_mask.blend(one, zero);
                }

                (iter, z1)
            }
            Fractal::Tdrge { max_iter } => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();
                let mut z2 = Complexx::zeros();

                let mut iter = FX::splat(0.);
                for _ in 0..max_iter {
                    let undiverged_mask = z2.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }
                    let new_z2 = z2 * z2 * z2 + z1 * z1 + z0 + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;

                    iter += undiverged_mask.blend(one, zero);
                }

                (iter, z2)
            }
            Fractal::NthDrge { max_iter, n } => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z = vec![Complexx::zeros(); n];

                let mut iter = FX::splat(0.);
                for _ in 0..max_iter {
                    let undiverged_mask = z[n - 1].norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    let mut new_z = c;
                    for (k, z_k) in z.iter().enumerate() {
                        new_z += z_k.powu(k + 1);
                    }
                    for k in 0..n - 1 {
                        z[k] = z[k + 1];
                    }
                    z[n - 1] = new_z;

                    iter += undiverged_mask.blend(one, zero);
                }

                (iter, z[n - 1])
            }
            Fractal::ThirdDegreeRecPairs { max_iter } => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();
                let mut z2 = Complexx::zeros();

                let mut iter = FX::splat(0.);
                for _ in 0..max_iter {
                    let undiverged_mask = z2.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    let new_z2 = z0 * z1 + z0 * z2 + z1 * z2 + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;

                    iter += undiverged_mask.blend(one, zero);
                }

                (iter, z2)
            }
            Fractal::SecondDegreeThirtySevenBlend { max_iter } => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

                let mut iter = FX::splat(0.);
                for i in 0..max_iter {
                    let undiverged_mask = z1.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    if i % 37 == 0 {
                        let new_z1 = z1 * z1 - z0 + c;
                        z0 = z1;
                        z1 = new_z1;
                    } else {
                        let new_z1 = z1 * z1 + z0;
                        z0 = z1;
                        z1 = new_z1;
                    }

                    iter += undiverged_mask.blend(one, zero);
                }

                (iter, z1)
            }
            Fractal::ComplexLogisticMapLike {
                max_iter,
                a_re,
                a_im,
            } => {
                const BAILOUT: F = 50.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

                let mut iter = FX::splat(0.);
                for _ in 0..max_iter {
                    let undiverged_mask = z1.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    let new_z1 = z1 * (Complexx::splat(a_re, a_im) - z0) + c;
                    z0 = z1;
                    z1 = new_z1;

                    iter += undiverged_mask.blend(one, zero);
                }

                (iter, z1)
            }

            Fractal::Vshqwj { max_iter } => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();
                let mut z2 = Complexx::zeros();

                let mut iter = FX::splat(0.);
                for _ in 0..max_iter {
                    let undiverged_mask = z2.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }
                    let new_z2 = (z2 + z1) * (z1 + z0) * (z2 - z0) + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;

                    iter += undiverged_mask.blend(one, zero);
                }

                (iter, z2)
            }
            Fractal::Wmriho {
                max_iter,
                a_re,
                a_im,
            } => {
                const BAILOUT: F = 10.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();
                let mut z2 = Complexx::splat(a_re, a_im);

                let mut iter = FX::splat(0.);
                for _ in 0..max_iter {
                    let undiverged_mask = z2.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }
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

                    iter += undiverged_mask.blend(one, zero);
                }

                (iter, z2)
            }
            Fractal::Iigdzh {
                max_iter,
                a_re,
                a_im,
            } => {
                const BAILOUT: F = 10.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();
                let mut z2 = Complexx::splat(a_re, a_im);

                let mut iter = FX::splat(0.);
                for _ in 0..max_iter {
                    let undiverged_mask = z2.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }
                    let new_z2 = z2 * z2
                        + Complexx {
                            re: z0.im + z1.re,
                            im: z2.re,
                        }
                        + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;

                    iter += undiverged_mask.blend(one, zero);
                }

                (iter, z2)
            }
            Fractal::Fxdicq { max_iter } => {
                const BAILOUT: F = 10.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();
                let mut z2 = Complexx::zeros();

                let mut iter = FX::splat(0.);
                for _ in 0..max_iter {
                    let undiverged_mask = z2.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }
                    let new_z2 = z2 * z2
                        + Complexx {
                            re: z0.im * z1.re,
                            im: z2.re,
                        }
                        + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;

                    iter += undiverged_mask.blend(one, zero);
                }

                (iter, z2)
            }
            Fractal::Mjygzr { max_iter } => {
                const BAILOUT: F = 5.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

                let mut iter = FX::splat(0.);
                for _ in 0..max_iter {
                    let undiverged_mask = z1.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    let new_z = z1 * z1 * c + z0 + c;
                    z0 = z1;
                    z1 = new_z;

                    iter += undiverged_mask.blend(one, zero);
                }

                (iter, z1)
            }
            Fractal::Sfwypc {
                max_iter,
                alpha,
                beta,
                gamma,
            } => {
                const BAILOUT: F = 100.;
                let bailout_mask = FX::splat(BAILOUT);

                let alpha = Complexx::splat(alpha.0, alpha.1);
                let beta = Complexx::splat(beta.0, beta.1);
                let gamma = Complexx::splat(gamma.0, gamma.1);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();
                let mut z2 = Complexx::zeros();

                let mut iter = FX::splat(0.);
                for _ in 0..max_iter {
                    let undiverged_mask = z1.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    let new_z = (z0 - alpha) * (z1 - beta) * (z2 - gamma) + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z;

                    iter += undiverged_mask.blend(one, zero);
                }

                (iter, z1)
            }

            Fractal::MoireTest => {
                let Complexx { re: x, im: y } = c * 100.;
                ((x * x + y * y).sin().abs(), Complexx::splat(1., 0.))
            }
        };

        // let s = _last_z.norm_sqr().ln().ln();
        // (iter + one - s.min(20. * one)).to_array()
        // (iter + one - s).to_array()

        iter.to_array()
    }
}

impl<T> Fractal<T>
where
    T: Clone + Serialize,
{
    pub fn max_iter(&self) -> Option<u32> {
        macro_rules! extract_field {
            // Syntax: extract_field!(enum_value, field_name, field_type, [VariantA, VariantB, ...])
            ($e:ident, $v:expr, $field:ident, [$($variant:ident),+]) => {{
                match $v {
                    $($e::$variant { $field, .. } => Some(*$field)),+,
                    _ => None,
                }
            }};
        }

        extract_field!(
            Fractal,
            self,
            max_iter,
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
                Sfwypc
            ]
        )
    }

    pub fn max_iter_mut(&mut self) -> Option<&mut u32> {
        macro_rules! extract_field {
            // Syntax: extract_field!(enum_value, field_name, field_type, [VariantA, VariantB, ...])
            ($e:ident, $v:expr, $field:ident, [$($variant:ident),+]) => {{
                match $v {
                    $($e::$variant { $field, .. } => Some($field)),+,
                    _ => None,
                }
            }};
        }

        extract_field!(
            Fractal,
            self,
            max_iter,
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
                Sfwypc
            ]
        )
    }
}
