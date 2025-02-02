use serde::{Deserialize, Serialize};
use wide::{f64x4, CmpLe};

use crate::complex4::Complex4;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Fractal {
    Mandelbrot,
    MandelbrotCustomExp { exp: f64 },
    SecondDegreeRecWithGrowingExponent,
    SecondDegreeRecWithGrowingExponentParam { a_re: f64, a_im: f64 },
    SecondDegreeRecAlternating1WithGrowingExponent,
    ThirdDegreeRecWithGrowingExponent,
    NthDegreeRecWithGrowingExponent(usize),
    ThirdDegreeRecPairs,
    SecondDegreeThirtySevenBlend,
    ComplexLogisticMapLike { re: f64, im: f64 },

    // This is where I started lacking inspiration for names...
    Vshqwj,
    Wmriho { a_re: f64, a_im: f64 },
    Iigdzh,

    MoireTest,
}

impl Fractal {
    pub fn get_pixel(&self, c: Complex4, max_iter: u32) -> [f64; 4] {
        let one = f64x4::splat(1.0);
        let zero = f64x4::splat(0.0);

        match self {
            Fractal::Mandelbrot => {
                const BAILOUT: f64 = 4.;
                let bailout_mask = f64x4::splat(BAILOUT);

                let mut z = Complex4::zeros();

                let mut iter = f64x4::splat(0.);
                for _ in 0..max_iter {
                    let undiverged_mask = z.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    z = z * z + c;

                    iter += undiverged_mask.blend(one, zero);
                }

                iter.to_array()
            }
            &Fractal::MandelbrotCustomExp { exp } => {
                const BAILOUT: f64 = 4.;
                let bailout_mask = f64x4::splat(BAILOUT);

                let mut z = Complex4::zeros();

                let mut iter = f64x4::splat(0.);
                for _ in 0..max_iter {
                    let undiverged_mask = z.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    z = z.powf(exp) + c;

                    iter += undiverged_mask.blend(one, zero);
                }

                iter.to_array()
            }
            Fractal::SecondDegreeRecWithGrowingExponent => {
                const BAILOUT: f64 = 4.;
                let bailout_mask = f64x4::splat(BAILOUT);

                let mut z0 = Complex4::zeros();
                let mut z1 = Complex4::zeros();

                let mut iter = f64x4::splat(0.);
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

                iter.to_array()
            }
            &Fractal::SecondDegreeRecWithGrowingExponentParam { a_re, a_im } => {
                const BAILOUT: f64 = 4.;
                let bailout_mask = f64x4::splat(BAILOUT);

                let a = Complex4::splat(a_re, a_im);

                let mut z0 = Complex4::zeros();
                let mut z1 = Complex4::zeros();

                let mut iter = f64x4::splat(0.);
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

                iter.to_array()
            }
            Fractal::SecondDegreeRecAlternating1WithGrowingExponent => {
                const BAILOUT: f64 = 4.;
                let bailout_mask = f64x4::splat(BAILOUT);

                let mut z0 = Complex4::zeros();
                let mut z1 = Complex4::zeros();

                let mut iter = f64x4::splat(0.);
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

                iter.to_array()
            }
            Fractal::ThirdDegreeRecWithGrowingExponent => {
                const BAILOUT: f64 = 4.;
                let bailout_mask = f64x4::splat(BAILOUT);

                let mut z0 = Complex4::zeros();
                let mut z1 = Complex4::zeros();
                let mut z2 = Complex4::zeros();

                let mut iter = f64x4::splat(0.);
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

                iter.to_array()
            }
            Fractal::NthDegreeRecWithGrowingExponent(n) => {
                const BAILOUT: f64 = 4.;
                let bailout_mask = f64x4::splat(BAILOUT);

                let n = *n;
                let mut z = vec![Complex4::zeros(); n];

                let mut iter = f64x4::splat(0.);
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

                iter.to_array()
            }
            Fractal::ThirdDegreeRecPairs => {
                const BAILOUT: f64 = 4.;
                let bailout_mask = f64x4::splat(BAILOUT);

                let mut z0 = Complex4::zeros();
                let mut z1 = Complex4::zeros();
                let mut z2 = Complex4::zeros();

                let mut iter = f64x4::splat(0.);
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

                iter.to_array()
            }
            Fractal::SecondDegreeThirtySevenBlend => {
                const BAILOUT: f64 = 4.;
                let bailout_mask = f64x4::splat(BAILOUT);

                let mut z0 = Complex4::zeros();
                let mut z1 = Complex4::zeros();

                let mut iter = f64x4::splat(0.);
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

                iter.to_array()
            }
            &Fractal::ComplexLogisticMapLike { re, im } => {
                const BAILOUT: f64 = 50.;
                let bailout_mask = f64x4::splat(BAILOUT);

                let mut z0 = Complex4::zeros();
                let mut z1 = Complex4::zeros();

                let mut iter = f64x4::splat(0.);
                for _ in 0..max_iter {
                    let undiverged_mask = z1.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }

                    let new_z1 = z1 * (Complex4::splat(re, im) - z0) + c;
                    z0 = z1;
                    z1 = new_z1;

                    iter += undiverged_mask.blend(one, zero);
                }

                iter.to_array()
            }

            Fractal::Vshqwj => {
                const BAILOUT: f64 = 4.;
                let bailout_mask = f64x4::splat(BAILOUT);

                let mut z0 = Complex4::zeros();
                let mut z1 = Complex4::zeros();
                let mut z2 = Complex4::zeros();

                let mut iter = f64x4::splat(0.);
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

                iter.to_array()
            }
            &Fractal::Wmriho { a_re, a_im } => {
                const BAILOUT: f64 = 10.;
                let bailout_mask = f64x4::splat(BAILOUT);

                let mut z0 = Complex4::zeros();
                let mut z1 = Complex4::zeros();
                let mut z2 = Complex4::splat(a_re, a_im);

                let mut iter = f64x4::splat(0.);
                for _ in 0..max_iter {
                    let undiverged_mask = z2.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }
                    let new_z2 = z2 * z2
                        + z1 * z0
                        + Complex4 {
                            re: z0.im,
                            im: z0.re,
                        }
                        + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;

                    iter += undiverged_mask.blend(one, zero);
                }

                iter.to_array()
            }
            &Fractal::Iigdzh => {
                const BAILOUT: f64 = 10.;
                let bailout_mask = f64x4::splat(BAILOUT);

                let mut z0 = Complex4::zeros();
                let mut z1 = Complex4::zeros();
                let mut z2 = Complex4::zeros();

                let mut iter = f64x4::splat(0.);
                for _ in 0..max_iter {
                    let undiverged_mask = z2.norm_sqr().cmp_le(bailout_mask);
                    if !undiverged_mask.any() {
                        break;
                    }
                    let new_z2 = z2 * z2
                        + Complex4 {
                            re: z0.im + z1.re,
                            im: z2.re,
                        }
                        + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;

                    iter += undiverged_mask.blend(one, zero);
                }

                iter.to_array()
            }

            Fractal::MoireTest => {
                let Complex4 { re: x, im: y } = c * 100.;
                (x * x + y * y).sin().abs().to_array()
            }
        }
    }
}
