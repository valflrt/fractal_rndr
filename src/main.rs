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
    sync::{atomic, mpsc},
    time::Instant,
};

use image::{Rgb, RgbImage};
use mat::{Mat2D, Mat3D};
use num_complex::Complex;
use rayon::iter::{ParallelBridge, ParallelIterator};
use sampling::{
    generate_sampling_points, map_points_with_offsets, preview_sampling_points, Sampling,
};
use serde::{Deserialize, Serialize};

use coloring::{
    color_mapping, compute_histogram, cumulate_histogram, get_histogram_value, ColoringMode,
};
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
    sampling: Option<Sampling>,

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
                sampling,
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

            let sampling_points =
                generate_sampling_points(Some(sampling.unwrap_or_default().level));
            if let Some(DevOptions {
                save_sampling_pattern: Some(true),
                ..
            }) = dev_options
            {
                preview_sampling_points(&sampling_points)?;
            }

            // Get chunks

            const CHUNK_SIZE: usize = 128;
            const KERNEL_SIZE: usize = 1;
            const KERNEL_SIZE_I: isize = KERNEL_SIZE as isize;

            let mut raw_image = Mat2D::filled_with(
                0.,
                img_width as usize + 2 * KERNEL_SIZE,
                img_height as usize + 2 * KERNEL_SIZE,
            );

            let (v_chunks, last_v_chunk) = (
                (img_height as usize).div_euclid(CHUNK_SIZE),
                (img_height as usize).rem_euclid(CHUNK_SIZE),
            );
            let (h_chunks, last_h_chunk) = (
                (img_width as usize).div_euclid(CHUNK_SIZE),
                (img_width as usize).rem_euclid(CHUNK_SIZE),
            );

            // Progress related init

            let start = Instant::now();

            let progress = atomic::AtomicUsize::new(0);
            let total = (0..v_chunks + 1)
                .flat_map(|cj| {
                    (0..h_chunks + 1).map(move |ci| {
                        let chunk_width = if ci == h_chunks {
                            last_h_chunk
                        } else {
                            CHUNK_SIZE
                        };
                        let chunk_height = if cj == v_chunks {
                            last_v_chunk
                        } else {
                            CHUNK_SIZE
                        };

                        (chunk_width + 2 * KERNEL_SIZE) * (chunk_height + 2 * KERNEL_SIZE)
                    })
                })
                .sum::<usize>();

            let stdout = std::io::stdout();

            // Compute escape time (number of iterations) for each pixel

            for cj in 0..v_chunks + 1 {
                for ci in 0..h_chunks + 1 {
                    let chunk_width = if ci == h_chunks {
                        last_h_chunk
                    } else {
                        CHUNK_SIZE
                    };
                    let chunk_height = if cj == v_chunks {
                        last_v_chunk
                    } else {
                        CHUNK_SIZE
                    };

                    // pi and pj are the coordinates of the first pixel of the
                    // chunk (top-left corner pixel)
                    let pi = ci * CHUNK_SIZE;
                    let pj = cj * CHUNK_SIZE;

                    let (tx, rx) = mpsc::channel();
                    (0..chunk_height + 2 * KERNEL_SIZE)
                        .flat_map(|j| (0..chunk_width + 2 * KERNEL_SIZE).map(move |i| (i, j)))
                        .par_bridge()
                        .for_each_with(tx, |s, (i, j)| {
                            let x = (pi + i - KERNEL_SIZE) as f64;
                            let y = (pj + j - KERNEL_SIZE) as f64;

                            let (offset_x, offset_y) = if let Some(Sampling {
                                random_offsets: Some(true),
                                ..
                            }) = sampling
                            {
                                let mut rng = fastrand::Rng::new();
                                (rng.f64(), rng.f64())
                            } else {
                                (0., 0.)
                            };
                            let samples = sampling_points
                                .iter()
                                .filter_map(|&(dx, dy)| {
                                    map_points_with_offsets(dx, dy, offset_x, offset_y)
                                })
                                .map(|(dx, dy)| {
                                    let re = x_min + width * (x + 0.5 + dx) / img_width as f64;
                                    let im = y_min + height * (y + 0.5 + dy) / img_height as f64;

                                    let (iter, _) =
                                        fractal.get_pixel(Complex::new(re, im), max_iter);

                                    ((dx, dy), iter as f64 / max_iter as f64)
                                })
                                .collect::<Vec<_>>();

                            // Using atomic::Ordering::Relaxed because we don't really
                            // care about the order `progress` is updated. As long as it
                            // is updated it should be fine :>
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

                            s.send(((i, j), samples)).unwrap();
                        });

                    let mut chunk_samples = Mat3D::filled_with(
                        ((0., 0.), 0.),
                        chunk_width as usize + 2 * KERNEL_SIZE,
                        chunk_height as usize + 2 * KERNEL_SIZE,
                        sampling_points.len(),
                    );
                    for ((i, j), pixel_samples) in rx {
                        for (k, &v) in pixel_samples.iter().enumerate() {
                            chunk_samples.set((i as usize, j as usize, k), v).unwrap();
                        }
                    }

                    for j in 0..chunk_height as usize {
                        for i in 0..chunk_width as usize {
                            let i = i + KERNEL_SIZE;
                            let j = j + KERNEL_SIZE;

                            let mut weighted_sum = 0.;
                            let mut weight_total = 0.;

                            for dj in -KERNEL_SIZE_I..=KERNEL_SIZE_I {
                                for di in -KERNEL_SIZE_I..=KERNEL_SIZE_I {
                                    let ii =
                                        i.saturating_add_signed(di).min(img_width as usize - 1);
                                    let jj =
                                        j.saturating_add_signed(dj).min(img_height as usize - 1);

                                    for k in 0..sampling_points.len() {
                                        let &((dx, dy), v) =
                                            chunk_samples.get((ii, jj, k)).unwrap();
                                        let dx = di as f64 + dx;
                                        let dy = dj as f64 + dy;

                                        // This only includes samples from a round-cornered square
                                        // (it works kind of like a distance function)
                                        if f64::max(dx.abs(), dy.abs()) < 0.5 {
                                            let w = 1.;
                                            weighted_sum += w * v;
                                            weight_total += w;
                                        } else {
                                            const R: f64 = 0.4;
                                            const R_SQR: f64 = R * R;
                                            let distance_sqr = (dx.abs() - 0.5).max(0.).powi(2)
                                                + (dy.abs() - 0.5).max(0.).powi(2);
                                            if distance_sqr < R_SQR {
                                                let w = 1. - 0.25 * distance_sqr / R_SQR;
                                                weighted_sum += w * v;
                                                weight_total += w;
                                            }
                                        }
                                    }
                                }
                            }

                            raw_image
                                .set(
                                    (pi as usize + i, pj as usize + j),
                                    weighted_sum / weight_total,
                                )
                                .unwrap();
                        }
                    }
                }
            }

            println!();

            let mut output_image = RgbImage::new(img_width, img_height);

            let max = raw_image.vec.iter().copied().fold(0., f64::max);
            // let min = raw_image.vec.iter().copied().fold(max, f64::min);

            match coloring_mode.unwrap_or_default() {
                ColoringMode::BlackAndWhite => {
                    for j in 0..img_height as usize {
                        for i in 0..img_width as usize {
                            let &value = raw_image.get((i, j)).unwrap();
                            output_image.put_pixel(
                                i as u32,
                                j as u32,
                                if value >= 0.95 {
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
                            let &value = raw_image.get((i, j)).unwrap();
                            output_image.put_pixel(
                                i as u32,
                                j as u32,
                                color_mapping(value / max, custom_gradient.as_ref()),
                            );
                        }
                    }
                }
                ColoringMode::Squared => {
                    for j in 0..img_height as usize {
                        for i in 0..img_width as usize {
                            let &value = raw_image.get((i, j)).unwrap();
                            output_image.put_pixel(
                                i as u32,
                                j as u32,
                                color_mapping(value.powi(2), custom_gradient.as_ref()),
                            );
                        }
                    }
                }
                ColoringMode::CumulativeHistogram => {
                    let cumulative_histogram =
                        cumulate_histogram(compute_histogram(&raw_image.vec));
                    for j in 0..img_height as usize {
                        for i in 0..img_width as usize {
                            let &value = raw_image.get((i, j)).unwrap();
                            output_image.put_pixel(
                                i as u32,
                                j as u32,
                                color_mapping(
                                    get_histogram_value(value, &cumulative_histogram).powi(12),
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
