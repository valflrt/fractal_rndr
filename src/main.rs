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
use mat::Mat2D;
use num_complex::Complex;
use rayon::iter::{ParallelBridge, ParallelIterator};
use sampling::{generate_sampling_points, preview_sampling_points, SamplingLevel};
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

            let sampling_points = generate_sampling_points(sampling_mode);
            if let Some(DevOptions {
                save_sampling_pattern: Some(true),
                ..
            }) = dev_options
            {
                preview_sampling_points(&sampling_points)?;
            }

            // Get chunks

            const CHUNK_SIZE: usize = 256;
            const KERNEL_SIZE: isize = 1;

            let mut processed_image = Mat2D::filled_with(
                0.,
                img_width as usize + 2 * KERNEL_SIZE as usize,
                img_height as usize + 2 * KERNEL_SIZE as usize,
            );

            let (v_chunks, last_v_chunk) = (
                img_height.div_euclid(CHUNK_SIZE as u32),
                img_height.rem_euclid(CHUNK_SIZE as u32),
            );
            let (h_chunks, last_h_chunk) = (
                img_width.div_euclid(CHUNK_SIZE as u32),
                img_width.rem_euclid(CHUNK_SIZE as u32),
            );

            // Progress related init

            let start = Instant::now();

            let progress = atomic::AtomicU32::new(0);
            let total = (0..v_chunks + 1)
                .flat_map(|cj| {
                    (0..h_chunks + 1).map(move |ci| {
                        let chunk_width = if ci == h_chunks {
                            last_h_chunk
                        } else {
                            CHUNK_SIZE as u32
                        };
                        let chunk_height = if cj == v_chunks {
                            last_v_chunk
                        } else {
                            CHUNK_SIZE as u32
                        };

                        (chunk_width + 2 * KERNEL_SIZE as u32)
                            * (chunk_height + 2 * KERNEL_SIZE as u32)
                    })
                })
                .sum::<u32>();

            let stdout = std::io::stdout();

            // Compute escape time (number of iterations) for each pixel

            for cj in 0..v_chunks + 1 {
                for ci in 0..h_chunks + 1 {
                    let chunk_width = if ci == h_chunks {
                        last_h_chunk
                    } else {
                        CHUNK_SIZE as u32
                    };
                    let chunk_height = if cj == v_chunks {
                        last_v_chunk
                    } else {
                        CHUNK_SIZE as u32
                    };

                    // pi and pj are the coordinates of the first pixel of the
                    // chunk (top-left corner pixel)
                    let pi = ci * CHUNK_SIZE as u32;
                    let pj = cj * CHUNK_SIZE as u32;

                    let (tx, rx) = mpsc::channel();

                    (0..chunk_height + 2 * KERNEL_SIZE as u32)
                        .flat_map(|j| {
                            (0..chunk_width + 2 * KERNEL_SIZE as u32).map(move |i| (i, j))
                        })
                        .par_bridge()
                        .for_each_with(tx, |s, (i, j)| {
                            let x = (pi + i - KERNEL_SIZE as u32) as f64 + 0.5;
                            let y = (pj + j - KERNEL_SIZE as u32) as f64 + 0.5;

                            let samples = sampling_points
                                .iter()
                                .map(|&(dx, dy)| {
                                    let re = x_min + width * (x + dx) / img_width as f64;
                                    let im = y_min + height * (y + dy) / img_height as f64;

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

                    let mut chunk_samples =
                        Mat2D::filled_with(vec![], img_width as usize, img_height as usize);
                    for ((i, j), pixel_samples) in rx {
                        chunk_samples
                            .set((i as usize, j as usize), pixel_samples.to_owned())
                            .unwrap();
                    }

                    for j in 0..chunk_height as usize {
                        for i in 0..chunk_width as usize {
                            let i = i + KERNEL_SIZE as usize;
                            let j = j + KERNEL_SIZE as usize;

                            let mut weighted_sum = 0.;
                            let mut weight_total = 0.;

                            for dj in -KERNEL_SIZE..=KERNEL_SIZE {
                                for di in -KERNEL_SIZE..=KERNEL_SIZE {
                                    let ii =
                                        i.saturating_add_signed(di).min(img_width as usize - 1);
                                    let jj =
                                        j.saturating_add_signed(dj).min(img_height as usize - 1);

                                    for &((dx, dy), v) in chunk_samples.get((ii, jj)).unwrap() {
                                        let dx = di as f64 + dx;
                                        let dy = dj as f64 + dy;

                                        const R: f64 = 1.5;
                                        let d = dx * dx + dy * dy;
                                        if d < R {
                                            let w = 1. / (1. + d / R);
                                            weighted_sum += w * v;
                                            weight_total += w;
                                        }
                                    }
                                }
                            }

                            processed_image
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

            match coloring_mode.unwrap_or_default() {
                ColoringMode::BlackAndWhite => {
                    for j in 0..img_height as usize {
                        for i in 0..img_width as usize {
                            let &value = processed_image.get((i, j)).unwrap();
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
                            let &value = processed_image.get((i, j)).unwrap();
                            output_image.put_pixel(
                                i as u32,
                                j as u32,
                                color_mapping(value, custom_gradient.as_ref()),
                            );
                        }
                    }
                }
                ColoringMode::Squared => {
                    for j in 0..img_height as usize {
                        for i in 0..img_width as usize {
                            let &value = processed_image.get((i, j)).unwrap();
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
                        cumulate_histogram(compute_histogram(&processed_image.vec));
                    for j in 0..img_height as usize {
                        for i in 0..img_width as usize {
                            let &value = processed_image.get((i, j)).unwrap();
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
