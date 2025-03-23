use serde::{Deserialize, Serialize};
use wide::CmpLe;

use crate::{
    complexx::{self, Complexx},
    F, FX,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Fractal {
    Mandelbrot,
    MandelbrotCustomExp { exp: F },
    SecondDegreeRecWithGrowingExponent,
    SecondDegreeRecWithGrowingExponentParam { a_re: F, a_im: F },
    SecondDegreeRecAlternating1WithGrowingExponent,
    ThirdDegreeRecWithGrowingExponent,
    NthDegreeRecWithGrowingExponent(usize),
    ThirdDegreeRecPairs,
    SecondDegreeThirtySevenBlend,
    ComplexLogisticMapLike { a_re: F, a_im: F },

    // This is where I started lacking inspiration for names...
    Vshqwj,
    Wmriho { a_re: F, a_im: F },
    Iigdzh { a_re: F, a_im: F },
    Fxdicq,
    Mjygzr,
    Zqcqvm,
}

impl Fractal {
    pub fn sample(&self, c: Complexx, max_iter: u32) -> Vec<[(F, F); complexx::SIZE]> {
        let mut values = Vec::with_capacity(max_iter as usize);

        match self {
            Fractal::Mandelbrot => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z = Complexx::zeros();

                for _ in 0..max_iter {
                    let undiverged_mask = z.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    z = z * z + c;

                    values.push(z);
                }
            }
            &Fractal::MandelbrotCustomExp { exp } => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z = Complexx::zeros();

                for _ in 0..max_iter {
                    let undiverged_mask = z.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    z = z.powf(exp) + c;

                    values.push(z);
                }
            }
            Fractal::SecondDegreeRecWithGrowingExponent => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

                for _ in 0..max_iter {
                    let undiverged_mask = z1.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    let new_z1 = z1 * z1 + z0 + c;
                    z0 = z1;
                    z1 = new_z1;

                    values.push(z1);
                }
            }
            &Fractal::SecondDegreeRecWithGrowingExponentParam { a_re, a_im } => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let a = Complexx::splat(a_re, a_im);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

                for _ in 0..max_iter {
                    let undiverged_mask = z1.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    let new_z1 = z1 * z1 + a * z0 + c;
                    z0 = z1;
                    z1 = new_z1;

                    values.push(z1);
                }
            }
            Fractal::SecondDegreeRecAlternating1WithGrowingExponent => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

                for _ in 0..max_iter {
                    let undiverged_mask = z1.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    let new_z1 = z1 * z1 - z0 + c;
                    z0 = z1;
                    z1 = new_z1;

                    values.push(z1);
                }
            }
            Fractal::ThirdDegreeRecWithGrowingExponent => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();
                let mut z2 = Complexx::zeros();

                for _ in 0..max_iter {
                    let undiverged_mask = z2.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }
                    let new_z2 = z2 * z2 * z2 + z1 * z1 + z0 + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;

                    values.push(z2);
                }
            }
            Fractal::NthDegreeRecWithGrowingExponent(n) => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let n = *n;
                let mut z = vec![Complexx::zeros(); n];

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

                    values.push(z[n - 1]);
                }
            }
            Fractal::ThirdDegreeRecPairs => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();
                let mut z2 = Complexx::zeros();

                for _ in 0..max_iter {
                    let undiverged_mask = z2.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    let new_z2 = z0 * z1 + z0 * z2 + z1 * z2 + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;

                    values.push(z2);
                }
            }
            Fractal::SecondDegreeThirtySevenBlend => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

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

                    values.push(z1);
                }
            }
            &Fractal::ComplexLogisticMapLike { a_re: re, a_im: im } => {
                const BAILOUT: F = 50.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

                for _ in 0..max_iter {
                    let undiverged_mask = z1.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    let new_z1 = z1 * (Complexx::splat(re, im) - z0) + c;
                    z0 = z1;
                    z1 = new_z1;

                    values.push(z1);
                }
            }

            Fractal::Vshqwj => {
                const BAILOUT: F = 4.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();
                let mut z2 = Complexx::zeros();

                for _ in 0..max_iter {
                    let undiverged_mask = z2.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }
                    let new_z2 = (z2 + z1) * (z1 + z0) * (z2 - z0) + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;

                    values.push(z2);
                }
            }
            &Fractal::Wmriho { a_re, a_im } => {
                const BAILOUT: F = 10.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();
                let mut z2 = Complexx::splat(a_re, a_im);

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

                    values.push(z2);
                }
            }
            &Fractal::Iigdzh { a_re, a_im } => {
                const BAILOUT: F = 10.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();
                let mut z2 = Complexx::splat(a_re, a_im);

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

                    values.push(z2);
                }
            }
            Fractal::Fxdicq => {
                const BAILOUT: F = 10.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();
                let mut z2 = Complexx::zeros();

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

                    values.push(z2);
                }
            }
            Fractal::Mjygzr => {
                const BAILOUT: F = 5.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

                for _ in 0..max_iter {
                    let undiverged_mask = z1.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    let new_z = z1 * z1 * c + z0 + c;
                    z0 = z1;
                    z1 = new_z;

                    values.push(z1);
                }
            }
            Fractal::Zqcqvm => {
                const BAILOUT: F = 5.;
                let bailout_mask = FX::splat(BAILOUT);

                let mut z0 = Complexx::zeros();
                let mut z1 = Complexx::zeros();

                for _ in 0..max_iter {
                    let undiverged_mask = z1.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    let new_z = z1 + z0 + c;
                    z0 = z1;
                    z1 = new_z;

                    values.push(z1);
                }
            }
        };

        // let s = _last_z.norm_sqr().ln().ln();
        // (iter + one - s.min(20. * one)).to_array()
        // (iter + one - s).to_array()

        values.iter().map(|c| c.to_array()).collect::<Vec<_>>()
    }
}
