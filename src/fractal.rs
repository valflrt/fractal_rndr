use serde::{Deserialize, Serialize};
use wide::CmpLe;

use crate::{complexx::Complexx, F, FX};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Fractal {
    Mandelbrot,
    MandelbrotCustomExp {
        exp: F,
    },
    /// Second Degree Recursive sequence with Growing Exponent
    SDRGE,
    SDRGECustomExp {
        exp: F,
    },
    SDRGEParam {
        a_re: F,
        a_im: F,
    },
    /// Second degree recursive alternating sequence with growing exponent
    SDRAGE,
    /// Third Degree Recursive sequence with Growing Exponent
    TDRGE,
    /// Nth Degree Recursive sequence with Growing Exponent
    NthDRGE(usize),
    ThirdDegreeRecPairs,
    SecondDegreeThirtySevenBlend,
    ComplexLogisticMapLike {
        a_re: F,
        a_im: F,
    },

    // This is where I started lacking inspiration for names...
    Vshqwj,
    Wmriho {
        a_re: F,
        a_im: F,
    },
    Iigdzh {
        a_re: F,
        a_im: F,
    },
    Fxdicq,
    Mjygzr,

    MoireTest,
}

#[cfg(feature = "force_f32")]
type Out = [F; 8];
#[cfg(not(feature = "force_f32"))]
type Out = [F; 4];

impl Fractal {
    pub fn sample(&self, c: Complexx, max_iter: u32) -> Out {
        let one = FX::splat(1.0);
        let zero = FX::splat(0.0);

        let (iter, _last_z) = match self {
            Fractal::Mandelbrot => {
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
            &Fractal::MandelbrotCustomExp { exp } => {
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
            Fractal::SDRGE => {
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
            &Fractal::SDRGECustomExp { exp } => {
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
            &Fractal::SDRGEParam { a_re, a_im } => {
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
            Fractal::SDRAGE => {
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
            Fractal::TDRGE => {
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
            Fractal::NthDRGE(n) => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let n = *n;
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
            Fractal::ThirdDegreeRecPairs => {
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
            Fractal::SecondDegreeThirtySevenBlend => {
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
            &Fractal::ComplexLogisticMapLike { a_re: re, a_im: im } => {
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

                    let new_z1 = z1 * (Complexx::splat(re, im) - z0) + c;
                    z0 = z1;
                    z1 = new_z1;

                    iter += undiverged_mask.blend(one, zero);
                }

                (iter, z1)
            }

            Fractal::Vshqwj => {
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
            &Fractal::Wmriho { a_re, a_im } => {
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
            &Fractal::Iigdzh { a_re, a_im } => {
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
            Fractal::Fxdicq => {
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
            Fractal::Mjygzr => {
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
