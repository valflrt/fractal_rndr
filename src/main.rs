use std::{env, fs::File, time::Instant};

use image::{ImageBuffer, Rgb};
use num::complex::Complex;
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct FractalParams {
    img_width: u32,
    img_height: u32,
    zoom: f64,
    center_x: f64,
    center_y: f64,
    max_iter: u32,
    oversampling: Option<bool>,
    fractal_kind: FractalKind,
    coloring_mode: ColoringMode,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        3 => {
            match serde_json::from_reader::<_, FractalParams>(
                File::open(&args[1]).expect("failed to read input param file"),
            ) {
                Ok(FractalParams {
                    img_width,
                    img_height,
                    zoom,
                    center_x,
                    center_y,
                    max_iter,
                    oversampling,
                    fractal_kind,
                    coloring_mode,
                }) => {
                    let aspect_ratio = img_width as f64 / img_height as f64;

                    let width = zoom;
                    let height = width / aspect_ratio;
                    let x_min = center_x - width / 2.;
                    let x_max = center_x + width / 2.;
                    let y_min = center_y - height / 2.;
                    let y_max = center_y + height / 2.;

                    let mut img = ImageBuffer::new(img_width, img_height);

                    let start = Instant::now();

                    let pixel_values = (0..img_height)
                        .flat_map(|y| (0..img_width).map(move |x| (x, y)))
                        .par_bridge()
                        .map(|(x, y)| {
                            if let Some(true) = oversampling {
                                let real1 = x_min + (x as f64 / img_width as f64) * (x_max - x_min);
                                let imag1 =
                                    y_min + (y as f64 / img_height as f64) * (y_max - y_min);
                                let c1 = Complex::new(real1, imag1);

                                let real2 =
                                    x_min + ((x as f64 - 0.5) / img_width as f64) * (x_max - x_min);
                                let imag2 = y_min
                                    + ((y as f64 + 0.866025) / img_height as f64) * (y_max - y_min);
                                let c2 = Complex::new(real2, imag2);

                                let imag3 = y_min
                                    + ((y as f64 - 0.866025) / img_height as f64) * (y_max - y_min);
                                let c3 = Complex::new(real2, imag3);

                                let (iter1, _) = fractal_kind.get_pixel(c1, max_iter);
                                let (iter2, _) = fractal_kind.get_pixel(c2, max_iter);
                                let (iter3, _) = fractal_kind.get_pixel(c3, max_iter);

                                (x, y, ((iter1 + iter2 + iter3) as f64 / 3.) as u32)
                            } else {
                                let real = x_min + (x as f64 / img_width as f64) * (x_max - x_min);
                                let imag = y_min + (y as f64 / img_height as f64) * (y_max - y_min);
                                let c = Complex::new(real, imag);

                                let (iterations, _) = fractal_kind.get_pixel(c, max_iter);

                                (x, y, iterations)
                            }
                        })
                        .collect::<Vec<_>>();

                    match coloring_mode {
                        ColoringMode::BlackAndWhite => {
                            for (x, y, iterations) in pixel_values {
                                img.put_pixel(
                                    x,
                                    y,
                                    if iterations == max_iter {
                                        Rgb([0, 0, 0])
                                    } else {
                                        Rgb([255, 255, 255])
                                    },
                                );
                            }
                        }
                        ColoringMode::Linear => {
                            for (x, y, iterations) in pixel_values {
                                img.put_pixel(
                                    x,
                                    y,
                                    color_mapping(iterations as f64 / max_iter as f64),
                                );
                            }
                        }
                        ColoringMode::Squared => {
                            for (x, y, iterations) in pixel_values {
                                img.put_pixel(
                                    x,
                                    y,
                                    color_mapping((iterations as f64 / max_iter as f64).powi(2)),
                                );
                            }
                        }
                        ColoringMode::CumulativeHistogram => {
                            let cumulative_histogram = compute_histogram(&pixel_values, max_iter);
                            for (x, y, iterations) in pixel_values {
                                img.put_pixel(
                                    x,
                                    y,
                                    color_mapping(
                                        cumulative_histogram[iterations as usize].powi(12),
                                    ),
                                );
                            }
                        }
                    };

                    println!("{:?} elapsed", start.elapsed());

                    img.save(&args[2]).expect("failed to save fractal image");
                    // fs::write("out.json", serde_json::to_string_pretty(&params).unwrap()).unwrap();
                }
                Err(err) => {
                    println!("error reading parameter file: {}", err);
                }
            }
        }
        _ => {
            println!("This is a fractal renderer.\nUsage: fractal_renderer <param file path>.json <output image path>.png")
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum FractalKind {
    Mandelbrot,
    SecondOrderGrowingExponent,
    ThirdOrderGrowingExponent,
    NthOrderGrowingExponent(usize),
}

impl FractalKind {
    /// Outputs (iteration_count, escape_z)
    fn get_pixel(&self, c: Complex<f64>, max_iter: u32) -> (u32, Complex<f64>) {
        match self {
            FractalKind::Mandelbrot => {
                let mut z = Complex::new(0., 0.);

                for i in 0..max_iter {
                    if z.norm_sqr() > 4. {
                        return (i, z);
                    }
                    z = z * z + c;
                }
                (max_iter, z)
            }
            FractalKind::SecondOrderGrowingExponent => {
                let mut z0 = Complex::new(0., 0.);
                let mut z1 = Complex::new(0., 0.);

                for i in 0..max_iter {
                    if z1.norm_sqr() > 4. {
                        return (i, z1);
                    }
                    let new_z1 = z1 * z1 + z0 + c;
                    z0 = z1;
                    z1 = new_z1;
                }
                (max_iter, z1)
            }
            FractalKind::ThirdOrderGrowingExponent => {
                let mut z0 = Complex::new(0., 0.);
                let mut z1 = Complex::new(0., 0.);
                let mut z2 = Complex::new(0., 0.);

                for i in 0..max_iter {
                    if z1.norm_sqr() > 4. {
                        return (i, z2);
                    }
                    let new_z2 = z2 * z2 * z2 + z1 * z1 + z0 + c;
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;
                }
                (max_iter, z2)
            }
            FractalKind::NthOrderGrowingExponent(n) => {
                let n = *n;
                let mut z = vec![Complex::new(0., 0.); n];

                for i in 0..max_iter {
                    if z[n - 1].norm_sqr() > 4. {
                        return (i, z[n - 1]);
                    }
                    let mut new_z = c;
                    for k in 0..n {
                        new_z += z[k].powi(k as i32 + 1);
                    }
                    for k in 0..n - 1 {
                        z[k] = z[k + 1];
                    }
                    z[n - 1] = new_z;
                }
                (max_iter, z[n - 1])
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum ColoringMode {
    BlackAndWhite,
    Linear,
    Squared,
    CumulativeHistogram,
}

fn compute_histogram(pixel_values: &[(u32, u32, u32)], max_iter: u32) -> Vec<f64> {
    let mut histogram = vec![0; max_iter as usize + 1];

    for &(_, _, iteration_count) in pixel_values.iter() {
        histogram[iteration_count as usize] += 1;
    }

    let total = histogram.iter().sum::<u32>();
    let mut cumulative = vec![0.; max_iter as usize + 1];
    let mut cumulative_sum = 0.;
    for (i, &count) in histogram.iter().enumerate() {
        cumulative_sum += count as f64 / total as f64;
        cumulative[i] = cumulative_sum;
    }

    cumulative
}
const GRADIENT_LENGTH: usize = 8;
const GRADIENT_VALUES: [f64; GRADIENT_LENGTH] = [0., 0.10, 0.25, 0.4, 0.55, 0.7, 0.85, 0.95];
const GRADIENT_COLORS: [Rgb<u8>; GRADIENT_LENGTH] = [
    Rgb([10, 2, 20]),
    Rgb([200, 40, 230]),
    Rgb([20, 160, 230]),
    Rgb([60, 230, 80]),
    Rgb([255, 230, 20]),
    Rgb([255, 120, 20]),
    Rgb([255, 40, 60]),
    Rgb([2, 0, 4]),
];

fn color_mapping(t: f64) -> Rgb<u8> {
    if t <= GRADIENT_VALUES[0] {
        GRADIENT_COLORS[0]
    } else if t >= GRADIENT_VALUES[GRADIENT_LENGTH - 1] {
        GRADIENT_COLORS[GRADIENT_LENGTH - 1]
    } else {
        for i in 0..GRADIENT_LENGTH {
            if GRADIENT_VALUES[i] <= t && t <= GRADIENT_VALUES[i + 1] {
                let ratio =
                    (t - GRADIENT_VALUES[i]) / (GRADIENT_VALUES[i + 1] - GRADIENT_VALUES[i]);
                let Rgb([r1, g1, b1]) = GRADIENT_COLORS[i];
                let Rgb([r2, g2, b2]) = GRADIENT_COLORS[i + 1];
                let r = (r1 as f64 * (1. - ratio) + r2 as f64 * ratio)
                    .min(255.)
                    .max(0.) as u8;
                let g = (g1 as f64 * (1. - ratio) + g2 as f64 * ratio)
                    .min(255.)
                    .max(0.) as u8;
                let b = (b1 as f64 * (1. - ratio) + b2 as f64 * ratio)
                    .min(255.)
                    .max(0.) as u8;
                return Rgb([r, g, b]);
            }
        }
        GRADIENT_COLORS[GRADIENT_LENGTH - 1]
    }
}
