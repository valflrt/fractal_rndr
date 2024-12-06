mod calc;

use std::{env, fs::File, time::Instant};

use astro_float::{BigFloat, Consts, RoundingMode};
use calc::{add, norm_sqr, pow};
use image::{ImageBuffer, Rgb};
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct FractalParams {
    img_width: u32,
    img_height: u32,
    zoom: BigFloat,
    center_x: BigFloat,
    center_y: BigFloat,
    max_iter: u32,
    oversampling: Option<bool>,
    fractal_kind: FractalKind,
    coloring_mode: ColoringMode,
}

/// Floating point precision (1 is 64 bits, 2 is 128 bits, ...)
const P: usize = 1;
const RM: RoundingMode = RoundingMode::ToEven;

#[derive(Debug, Clone)]
struct LocalConsts {
    one: BigFloat,
    two: BigFloat,
    four: BigFloat,
    half: BigFloat,
    sin_pi_over_3: BigFloat,
    zero_c: (BigFloat, BigFloat),
}

impl Default for LocalConsts {
    fn default() -> Self {
        let mut cc = Consts::new().unwrap();

        let zero = BigFloat::from_f32(0., P);
        let one = BigFloat::from_f32(1., P);
        let two = BigFloat::from_f32(2., P);
        let three = BigFloat::from_f32(3., P);
        let four = BigFloat::from_f32(4., P);
        let half = BigFloat::from_f32(1., P).div(&two, P, RM);
        let sin_pi_over_3 = cc.pi(P, RM).div(&three, P, RM).sin(P, RM, &mut cc);

        LocalConsts {
            one,
            two,
            four,
            half,
            sin_pi_over_3,
            zero_c: (zero.clone(), zero),
        }
    }
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
                    let consts = LocalConsts::default();

                    let img_width_f = BigFloat::from_f32(img_width as f32, P);
                    let img_height_f = BigFloat::from_f32(img_height as f32, P);

                    let aspect_ratio = img_width_f.div(&img_height_f, P, RM);

                    let width = zoom;
                    let height = width.div(&aspect_ratio, P, RM);

                    let x_min = center_x.sub(&width.div(&consts.two, P, RM), P, RM);
                    let x_max = center_x.add(&width.div(&consts.two, P, RM), P, RM);
                    let y_min = center_y.sub(&height.div(&consts.two, P, RM), P, RM);
                    let y_max = center_y.add(&height.div(&consts.two, P, RM), P, RM);

                    let mut img = ImageBuffer::new(img_width, img_height);

                    let start = Instant::now();

                    let pixel_values = (0..img_height)
                        .flat_map(|y| (0..img_width).map(move |x| (x, y)))
                        .par_bridge()
                        .map(|(x, y)| {
                            let x_f = BigFloat::from_f32(x as f32, P);
                            let y_f = BigFloat::from_f32(y as f32, P);

                            let dx = x_max.sub(&x_min, P, RM);
                            let dy = y_max.sub(&y_min, P, RM);

                            if let Some(true) = oversampling {
                                let re1 = x_min.add(
                                    &x_f.add(&consts.one, P, RM)
                                        .div(&img_width_f, P, RM)
                                        .mul(&dx, P, RM),
                                    P,
                                    RM,
                                );
                                let im1 = y_min.add(
                                    &y_f.div(&img_height_f, P, RM).mul(&dy, P, RM),
                                    P,
                                    RM,
                                );
                                let c1 = (re1, im1);

                                let re2 = x_min.add(
                                    &x_f.sub(&consts.half, P, RM)
                                        .div(&img_width_f, P, RM)
                                        .mul(&dx, P, RM),
                                    P,
                                    RM,
                                );
                                let im2 = y_min.add(
                                    &y_f.add(&consts.sin_pi_over_3, P, RM)
                                        .div(&img_height_f, P, RM)
                                        .mul(&dy, P, RM),
                                    P,
                                    RM,
                                );
                                let c2 = (re2.clone(), im2);

                                let im3 = y_min.add(
                                    &y_f.sub(&consts.sin_pi_over_3, P, RM)
                                        .div(&img_height_f, P, RM)
                                        .mul(&dy, P, RM),
                                    P,
                                    RM,
                                );
                                let c3 = (re2, im3);

                                let (iter1, _) = fractal_kind.get_pixel(c1, max_iter, &consts);
                                let (iter2, _) = fractal_kind.get_pixel(c2, max_iter, &consts);
                                let (iter3, _) = fractal_kind.get_pixel(c3, max_iter, &consts);

                                (x, y, ((iter1 + iter2 + iter3) as f64 / 3.) as u32)
                            } else {
                                let re =
                                    x_min.add(&x_f.div(&img_width_f, P, RM).mul(&dx, P, RM), P, RM);
                                let im = y_min.add(
                                    &y_f.div(&img_height_f, P, RM).mul(&dy, P, RM),
                                    P,
                                    RM,
                                );
                                let c = (re, im);

                                let (iterations, _) = fractal_kind.get_pixel(c, max_iter, &consts);

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
    SecondDegreeWithGrowingExponent,
    ThirdDegreeWithGrowingExponent,
    // NthDegreeWithGrowingExponent(usize),
}

impl FractalKind {
    /// Outputs (iteration_count, escape_z)
    fn get_pixel(
        &self,
        c: (BigFloat, BigFloat),
        max_iter: u32,
        consts: &LocalConsts,
    ) -> (u32, (BigFloat, BigFloat)) {
        match self {
            FractalKind::Mandelbrot => {
                let mut z = consts.zero_c.clone();

                let mut i = 0;
                while i < max_iter {
                    if norm_sqr(&z) > consts.four {
                        break;
                    }
                    z = add(&pow(&z, 2), &c);

                    i += 1;
                }

                (i, z)
            }
            FractalKind::SecondDegreeWithGrowingExponent => {
                let mut z0 = consts.zero_c.clone();
                let mut z1 = consts.zero_c.clone();

                let mut i = 0;
                while i < max_iter {
                    if norm_sqr(&z1) > consts.four {
                        break;
                    }
                    let new_z1 = add(&add(&pow(&z1, 2), &z0), &c);
                    z0 = z1;
                    z1 = new_z1;

                    i += 1;
                }

                (i, z1)
            }
            FractalKind::ThirdDegreeWithGrowingExponent => {
                let mut z0 = consts.zero_c.clone();
                let mut z1 = consts.zero_c.clone();
                let mut z2 = consts.zero_c.clone();

                let mut i = 0;
                while i < max_iter {
                    if norm_sqr(&z2) > consts.four {
                        break;
                    }
                    let new_z2 = add(&add(&add(&pow(&z2, 3), &pow(&z1, 2)), &z0), &c);
                    z0 = z1;
                    z1 = z2;
                    z2 = new_z2;

                    i += 1;
                }

                (i, z2)
            } // FractalKind::NthDegreeWithGrowingExponent(n) => {
              //     let n = *n;
              //     let mut z = vec![Complex::new(0., 0.); n];

              //     let mut i = 0;
              //     while i < max_iter {
              //         if z[n - 1].norm_sqr() > 4.into() {
              //             break;
              //         }
              //         let mut new_z = c;
              //         for k in 0..n {
              //             new_z += z[k].powu(k as u32 + 1);
              //         }
              //         for k in 0..n - 1 {
              //             z[k] = z[k + 1];
              //         }
              //         z[n - 1] = new_z;

              //         i += 1;
              //     }

              //     (i, z[n - 1])
              // }
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
