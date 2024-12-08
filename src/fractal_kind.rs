use num_complex::Complex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FractalKind {
    Mandelbrot,
    SecondDegreeWithGrowingExponent,
    ThirdDegreeWithGrowingExponent,
    NthDegreeWithGrowingExponent(usize),
}

impl FractalKind {
    /// Outputs (iteration_count, escape_z)
    pub fn get_pixel(&self, c: Complex<f64>, max_iter: u32) -> (u32, Complex<f64>) {
        match self {
            FractalKind::Mandelbrot => {
                let mut z = Complex::new(0., 0.);

                let mut i = 0;
                while i < max_iter && z.norm_sqr() < 4. {
                    z = z * z + c;
                    i += 1;
                }

                (i, z)
            }
            FractalKind::SecondDegreeWithGrowingExponent => {
                let mut z0 = Complex::new(0., 0.);
                let mut z1 = Complex::new(0., 0.);

                let mut i = 0;
                while i < max_iter && z1.norm_sqr() < 4. {
                    let new_z1 = z1 * z1 + z0 + c;
                    z0 = z1;
                    z1 = new_z1;

                    i += 1;
                }

                (i, z1)
            }
            FractalKind::ThirdDegreeWithGrowingExponent => {
                let mut z0 = Complex::new(0., 0.);
                let mut z1 = Complex::new(0., 0.);
                let mut z2 = Complex::new(0., 0.);

                let mut i = 0;
                while i < max_iter && z2.norm_sqr() < 4. {
                    let new_z2 = z2 * z2 * z2 + z1 * z1 + z0 + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;

                    i += 1;
                }

                (i, z2)
            }
            FractalKind::NthDegreeWithGrowingExponent(n) => {
                let n = *n;
                let mut z = vec![Complex::new(0., 0.); n];

                let mut i = 0;
                while i < max_iter && z[n - 1].norm_sqr() < 4. {
                    let mut new_z = c;
                    for (k, z_k) in z.iter().enumerate() {
                        new_z += z_k.powi(k as i32 + 1);
                    }
                    for k in 0..n - 1 {
                        z[k] = z[k + 1];
                    }
                    z[n - 1] = new_z;

                    i += 1;
                }

                (i, z[n - 1])
            }
        }
    }
}
