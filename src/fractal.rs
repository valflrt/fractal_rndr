use serde::{Deserialize, Serialize};

use crate::complex::Complex;

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
    pub fn get_pixel(&self, c: Complex, max_iter: u32) -> u32 {
        match self {
            Fractal::Mandelbrot => {
                const BAILOUT: f64 = 4.;

                let mut z = Complex::ZERO;

                let mut i = 0;
                while i < max_iter && z.norm_sqr() < BAILOUT {
                    z = z * z + c;
                    i += 1;
                }

                i
            }
            Fractal::SecondDegreeRecWithGrowingExponent => {
                const BAILOUT: f64 = 4.;

                let mut z0 = Complex::ZERO;
                let mut z1 = Complex::ZERO;

                let mut i = 0;
                while i < max_iter && z1.norm_sqr() < BAILOUT {
                    let new_z1 = z1 * z1 + z0 + c;
                    z0 = z1;
                    z1 = new_z1;

                    i += 1;
                }

                i
            }
            Fractal::SecondDegreeRecAlternating1WithGrowingExponent => {
                const BAILOUT: f64 = 4.;

                let mut z0 = Complex::ZERO;
                let mut z1 = Complex::ZERO;

                let mut i = 0;
                while i < max_iter && z1.norm_sqr() < BAILOUT {
                    let new_z1 = z1 * z1 - z0 + c;
                    z0 = z1;
                    z1 = new_z1;

                    i += 1;
                }

                i
            }
            Fractal::ThirdDegreeRecWithGrowingExponent => {
                const BAILOUT: f64 = 4.;

                let mut z0 = Complex::ZERO;
                let mut z1 = Complex::ZERO;
                let mut z2 = Complex::ZERO;

                let mut i = 0;
                while i < max_iter && z2.norm_sqr() < BAILOUT {
                    let new_z2 = z2 * z2 * z2 + z1 * z1 + z0 + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;

                    i += 1;
                }

                i
            }
            Fractal::NthDegreeRecWithGrowingExponent(n) => {
                const BAILOUT: f64 = 4.;

                let n = *n;
                let mut z = vec![Complex::ZERO; n];

                let mut i = 0;
                while i < max_iter && z[n - 1].norm_sqr() < BAILOUT {
                    let mut new_z = c;
                    for (k, z_k) in z.iter().enumerate() {
                        new_z += z_k.powu(k + 1);
                    }
                    for k in 0..n - 1 {
                        z[k] = z[k + 1];
                    }
                    z[n - 1] = new_z;

                    i += 1;
                }

                i
            }
            Fractal::ThirdDegreeRecPairs => {
                const BAILOUT: f64 = 4.;

                let mut z0 = Complex::ZERO;
                let mut z1 = Complex::ZERO;
                let mut z2 = Complex::ZERO;

                let mut i = 0;
                while i < max_iter && z2.norm_sqr() < BAILOUT {
                    let new_z2 = z0 * z1 + z0 * z2 + z1 * z2 + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;

                    i += 1;
                }

                i
            }
            Fractal::SecondDegreeThirtySevenBlend => {
                const BAILOUT: f64 = 4.;

                let mut z0 = Complex::ZERO;
                let mut z1 = Complex::ZERO;

                let mut i = 0;
                while i < max_iter && z1.norm_sqr() < BAILOUT {
                    if i % 37 == 0 {
                        let new_z1 = z1 * z1 - z0 + c;
                        z0 = z1;
                        z1 = new_z1;
                    } else {
                        let new_z1 = z1 * z1 + z0;
                        z0 = z1;
                        z1 = new_z1;
                    }

                    i += 1;
                }

                i
            }
        }
    }
}
