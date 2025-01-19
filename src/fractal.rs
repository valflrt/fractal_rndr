use serde::{Deserialize, Serialize};
use wide::{f64x4, CmpLe};

use crate::complex::Complexs;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Fractal {
    Mandelbrot,
    SecondDegreeRecWithGrowingExponent,
    SecondDegreeRecAlternating1WithGrowingExponent,
    ThirdDegreeRecWithGrowingExponent,
    NthDegreeRecWithGrowingExponent(usize),
    ThirdDegreeRecPairs,
    SecondDegreeThirtySevenBlend,
}

impl Fractal {
    pub fn get_pixel(&self, c: Complexs, max_iter: u32) -> [f64; 4] {
        let one = f64x4::splat(1.0);
        let zero = f64x4::splat(0.0);

        match self {
            Fractal::Mandelbrot => {
                const BAILOUT: f64 = 4.;
                let bailout_mask = f64x4::splat(BAILOUT);

                let mut z = Complexs::zeros();

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
            Fractal::SecondDegreeRecWithGrowingExponent => {
                const BAILOUT: f64 = 4.;
                let bailout_mask = f64x4::splat(BAILOUT);

                let mut z0 = Complexs::zeros();
                let mut z1 = Complexs::zeros();

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
            Fractal::SecondDegreeRecAlternating1WithGrowingExponent => {
                const BAILOUT: f64 = 4.;
                let bailout_mask = f64x4::splat(BAILOUT);

                let mut z0 = Complexs::zeros();
                let mut z1 = Complexs::zeros();

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

                let mut z0 = Complexs::zeros();
                let mut z1 = Complexs::zeros();
                let mut z2 = Complexs::zeros();

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
                let mut z = vec![Complexs::zeros(); n];

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

                let mut z0 = Complexs::zeros();
                let mut z1 = Complexs::zeros();
                let mut z2 = Complexs::zeros();

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

                let mut z0 = Complexs::zeros();
                let mut z1 = Complexs::zeros();

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
        }
    }
}
