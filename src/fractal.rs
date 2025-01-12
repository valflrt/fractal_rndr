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
}

impl Fractal {
    /// Outputs (iteration_count, escape_z)
    pub fn get_pixel(&self, c: Complex, max_iter: u32) -> (u32, Complex) {
        match self {
            Fractal::Mandelbrot => {
                let mut z = Complex::ZERO;

                let mut i = 0;
                while i < max_iter && z.norm_sqr() < 4. {
                    z = z * z + c;
                    i += 1;
                }

                (i, z)
            }
            Fractal::SecondDegreeRecWithGrowingExponent => {
                let mut z0 = Complex::ZERO;
                let mut z1 = Complex::ZERO;

                let mut i = 0;
                while i < max_iter && z1.norm_sqr() < 4. {
                    let new_z1 = z1 * z1 + z0 + c;
                    z0 = z1;
                    z1 = new_z1;

                    i += 1;
                }

                (i, z1)
            }
            Fractal::SecondDegreeRecAlternating1WithGrowingExponent => {
                let mut z0 = Complex::ZERO;
                let mut z1 = Complex::ZERO;

                let mut i = 0;
                while i < max_iter && z1.norm_sqr() < 4. {
                    let new_z1 = z1 * z1 - z0 + c;
                    z0 = z1;
                    z1 = new_z1;

                    i += 1;
                }

                (i, z1)
            }
            Fractal::ThirdDegreeRecWithGrowingExponent => {
                let mut z0 = Complex::ZERO;
                let mut z1 = Complex::ZERO;
                let mut z2 = Complex::ZERO;

                let mut i = 0;
                while i < max_iter {
                    if z2.re * z2.re + z2.im * z2.im >= 4. {
                        break;
                    }

                    let new_z2 = z2 * z2 * z2 + z1 * z1 + z0 + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;

                    i += 1;
                }

                (i, z2)
            }
            Fractal::NthDegreeRecWithGrowingExponent(n) => {
                let n = *n;
                let mut z = vec![Complex::ZERO; n];

                let mut i = 0;
                while i < max_iter && z[n - 1].norm_sqr() < 4. {
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

                (i, z[n - 1])
            }
            Fractal::ThirdDegreeRecPairs => {
                let mut z0 = Complex::ZERO;
                let mut z1 = Complex::ZERO;
                let mut z2 = Complex::ZERO;

                let mut i = 0;
                while i < max_iter && z2.norm_sqr() < 4. {
                    let new_z2 = z0 * z1 + z0 * z2 + z1 * z2 + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;

                    i += 1;
                }

                (i, z2)
            }
        }
    }
}
