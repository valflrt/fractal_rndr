mod coloring;
mod error;
mod fractal;
mod mat;
mod sampling;

use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    sync::atomic,
    time::Instant,
};

use image::{Rgb, RgbImage};
use mat::Mat2D;
use num_complex::Complex;
use rayon::iter::{ParallelBridge, ParallelIterator};
use sampling::{preview_sampling_points, spiral_sampling_points, SamplingLevel};
use serde::{Deserialize, Serialize};

use coloring::{color_mapping, compute_histogram, cumulate_histogram, ColoringMode};
use error::{ErrorKind, Result};
use fractal::Fractal;

#[derive(Debug, Serialize, Deserialize)]
struct FractalParams {
    img_width: u32,
    img_height: u32,

    zoom: f64,
    center_x: f64,
    center_y: f64,

    max_iter: u32,

    fractal: Fractal,

    coloring_mode: Option<ColoringMode>,
    sampling: Option<SamplingLevel>,

    custom_gradient: Option<Vec<(f64, [u8; 3])>>,

    dev_options: Option<DevOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
struct DevOptions {
    save_sampling_pattern: Option<bool>,
    display_gradient: Option<bool>,
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // In case I want to try out how serde_json serializes the
    // param file
    // fs::write("out.json", serde_json::to_string_pretty(&params).unwrap()).unwrap();

    match args.len() {
        3 => {
            let FractalParams {
                img_width,
                img_height,
                zoom,
                center_x,
                center_y,
                max_iter,
                sampling: sampling_mode,
                fractal,
                coloring_mode,
                custom_gradient,
                dev_options,
            } = serde_json::from_reader::<_, FractalParams>(
                File::open(&args[1]).map_err(ErrorKind::ReadParameterFile)?,
            )
            .map_err(ErrorKind::DecodeParameterFile)?;

            let aspect_ratio = img_width as f64 / img_height as f64;

            let width = zoom;
            let height = width / aspect_ratio;
            let x_min = center_x - width / 2.;
            // make center_y negative to match complex number representation
            // (in which the imaginary axis is pointing upward)
            let y_min = -center_y - height / 2.;

            // sampling

            let sampling_points = spiral_sampling_points(sampling_mode);
            if let Some(DevOptions {
                save_sampling_pattern: Some(true),
                ..
            }) = dev_options
            {
                preview_sampling_points(&sampling_points)?;
            }

            // Progress related init

            let start = Instant::now();

            let progress = atomic::AtomicU32::new(0);
            let total = img_height * img_width;

            let stdout = std::io::stdout();

            // Compute escape time (number of iterations) for each pixel

            let pixel_values = (0..img_height)
                .flat_map(|j| (0..img_width).map(move |i| (i, j)))
                .par_bridge()
                .map(|(i, j)| {
                    let x = i as f64 + 0.5;
                    let y = j as f64 + 0.5;

                    let center_re = x_min + width * x / img_width as f64;
                    let center_im = y_min + height * y / img_height as f64;

                    let (center_iter, _) =
                        fractal.get_pixel(Complex::new(center_re, center_im), max_iter);
                    let center_iter = center_iter as f64;

                    // Performs a weighted average of the iterations
                    let (iter, grad, delta_iter_sum) = sampling_points.iter().fold(
                        (0., (0., 0.), 1.),
                        |acc, &((dx, dy), weight)| {
                            let re = x_min + width * (x + dx) / img_width as f64;
                            let im = y_min + height * (y + dy) / img_height as f64;

                            let (iter, _) = fractal.get_pixel(Complex::new(re, im), max_iter);

                            let length = (dx * dx + dy * dy).sqrt();
                            let (ndx, ndy) = (dx / length, dy / length);

                            let weighted_iter = weight * iter as f64;
                            let delta_iter = (iter as f64 - center_iter).abs();

                            let (iter_acc, grad_acc, delta_iter_acc) = acc;

                            (
                                iter_acc + weighted_iter,
                                (grad_acc.0 + delta_iter * ndx, grad_acc.1 + delta_iter * ndy),
                                delta_iter_acc + delta_iter,
                            )
                        },
                    );

                    let grad = (grad.0 / delta_iter_sum, grad.1 / delta_iter_sum);

                    (i, j, iter, grad)
                })
                .map(|v| {
                    // Using atomic::Ordering::Relaxed because we don't really
                    // care about the order `progress` is updated. As long as it
                    // is updated it should be fine :)
                    progress.fetch_add(1, atomic::Ordering::Relaxed);
                    let progress = progress.load(atomic::Ordering::Relaxed);

                    if progress % (total / 100000 + 1) == 0 {
                        stdout
                            .lock()
                            .write_all(
                                format!(
                                    "\r {:.1}% - {:.1}s elapsed",
                                    100. * progress as f32 / total as f32,
                                    start.elapsed().as_secs_f32(),
                                )
                                .as_bytes(),
                            )
                            .unwrap();
                    }

                    v
                })
                .collect::<Vec<_>>();

            println!();

            let mut iter_image = Mat2D::filled_with(0., img_width as usize, img_height as usize);
            let mut grad_image =
                Mat2D::filled_with((0., 0.), img_width as usize, img_height as usize);

            // fill iter_image and grad_image
            for (x, y, iterations, grad) in pixel_values {
                iter_image.set((x as usize, y as usize), iterations);
                grad_image.set((x as usize, y as usize), grad);
            }

            for j in 0..img_height as usize {
                for i in 0..img_width as usize {
                    let (gx, gy) = grad_image.get((i, j));

                    let mut iter_sum = 0.;
                    let mut weight_sum = 1.;

                    iter_sum += iter_image.get((i, j));
                    weight_sum += 1.;

                    for dj in -1..=1 {
                        for di in -1..=1 {
                            if di != 0 && dj != 0 {
                                let ii = i.checked_add_signed(di);
                                let jj = j.checked_add_signed(dj);

                                match (ii, jj) {
                                    (Some(ii), Some(jj))
                                        if ii < img_width as usize && jj < img_height as usize =>
                                    {
                                        let other_iter = iter_image.get((ii, jj));

                                        let (dx, dy) = (di as f64, dj as f64);
                                        let length = (dx * dx + dy * dy).sqrt();
                                        let (dx, dy) = (dx / length, dy / length);

                                        // Sorry I didn't know how to name this one...
                                        let sideways_dot = 1. - gx * dx - gy * dy;
                                        let weight = 0.01 * sideways_dot;

                                        weight_sum += weight;

                                        iter_sum += weight * other_iter;
                                    }
                                    _ => (),
                                }
                            }
                        }
                    }

                    let new_iter = iter_sum / weight_sum;

                    iter_image.set((i, j), new_iter);
                }
            }

            let mut output_image = RgbImage::new(img_width, img_height);

            match coloring_mode.unwrap_or_default() {
                ColoringMode::BlackAndWhite => {
                    for j in 0..img_height as usize {
                        for i in 0..img_width as usize {
                            let iterations = iter_image.get((i, j));
                            output_image.put_pixel(
                                i as u32,
                                j as u32,
                                if iterations as u32 == max_iter {
                                    Rgb([0, 0, 0])
                                } else {
                                    Rgb([255, 255, 255])
                                },
                            );
                        }
                    }
                }
                ColoringMode::Linear => {
                    for j in 0..img_height as usize {
                        for i in 0..img_width as usize {
                            let iterations = iter_image.get((i, j));
                            output_image.put_pixel(
                                i as u32,
                                j as u32,
                                color_mapping(
                                    iterations as f64 / max_iter as f64,
                                    custom_gradient.as_ref(),
                                ),
                            );
                        }
                    }
                }
                ColoringMode::Squared => {
                    for j in 0..img_height as usize {
                        for i in 0..img_width as usize {
                            let iterations = iter_image.get((i, j));
                            output_image.put_pixel(
                                i as u32,
                                j as u32,
                                color_mapping(
                                    (iterations as f64 / max_iter as f64).powi(2),
                                    custom_gradient.as_ref(),
                                ),
                            );
                        }
                    }
                }
                ColoringMode::LinearMinMax => {
                    let (min, max) = iter_image.vec.iter().fold(
                        (max_iter as f64, 0.),
                        |(acc_min, acc_max), &v| {
                            (
                                if v < acc_min as f64 {
                                    v
                                } else {
                                    acc_min as f64
                                },
                                if v > acc_max as f64 {
                                    v
                                } else {
                                    acc_max as f64
                                },
                            )
                        },
                    );

                    for j in 0..img_height as usize {
                        for i in 0..img_width as usize {
                            let iterations = iter_image.get((i, j));

                            let t = (iterations - min) as f64 / (max - min) as f64;
                            output_image.put_pixel(
                                i as u32,
                                j as u32,
                                color_mapping(t, custom_gradient.as_ref()),
                            );
                        }
                    }
                }
                ColoringMode::CumulativeHistogram => {
                    let cumulative_histogram =
                        cumulate_histogram(compute_histogram(&iter_image.vec, max_iter), max_iter);
                    for j in 0..img_height as usize {
                        for i in 0..img_width as usize {
                            let iterations = iter_image.get((i, j));
                            output_image.put_pixel(
                                i as u32,
                                j as u32,
                                color_mapping(
                                    cumulative_histogram[iterations as usize].powi(12),
                                    custom_gradient.as_ref(),
                                ),
                            );
                        }
                    }
                }
            };

            if let Some(DevOptions {
                display_gradient: Some(true),
                ..
            }) = dev_options
            {
                const GRADIENT_HEIGHT: u32 = 8;
                const GRADIENT_WIDTH: u32 = 64;
                const OFFSET: u32 = 8;

                for j in 0..GRADIENT_HEIGHT {
                    for i in 0..GRADIENT_WIDTH {
                        output_image.put_pixel(
                            img_width - GRADIENT_WIDTH - OFFSET + i,
                            img_height - GRADIENT_HEIGHT - OFFSET + j,
                            color_mapping(
                                i as f64 / GRADIENT_WIDTH as f64,
                                custom_gradient.as_ref(),
                            ),
                        );
                    }
                }
            }

            let path = PathBuf::from(&args[2]);

            output_image.save(&path).map_err(ErrorKind::SaveImage)?;

            let image_size = fs::metadata(&path).unwrap().len();
            println!(
                " output image: {}x{} - {} {}",
                img_width,
                img_height,
                if image_size / 1_000_000 != 0 {
                    format!("{:.1}mb", image_size as f32 / 1_000_000.)
                } else if image_size / 1_000 != 0 {
                    format!("{:.1}kb", image_size as f32 / 1_000.)
                } else {
                    format!("{}b", image_size)
                },
                if let Some(ext) = path.extension() {
                    format!("- {} ", ext.to_str().unwrap())
                } else {
                    "".to_string()
                }
            );
        }
        _ => {
            println!("This is a fractal renderer.");
            println!("Usage: fractal_renderer <param file path>.json <output image path>.png");
            println!("More information: https://gh.valflrt.dev/fractal_renderer");
        }
    }

    Ok(())
}
