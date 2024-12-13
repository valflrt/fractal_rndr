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
use mat::Mat;
use num_complex::Complex;
use rayon::iter::{ParallelBridge, ParallelIterator};
use sampling::{preview_sampling_points, spiral_sampling_points, SamplingLevel};
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

#[inline]
fn gaussian(x: f64) -> f64 {
    (-x * x).exp()
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

            let samples = (0..img_height)
                .flat_map(|j| (0..img_width).map(move |i| (i, j)))
                .par_bridge()
                .map(|(i, j)| {
                    let x = i as f64 + 0.5;
                    let y = j as f64 + 0.5;

                    let samples = sampling_points
                        .iter()
                        .map(|&(dx, dy)| {
                            let re = x_min + width * (x + dx) / img_width as f64;
                            let im = y_min + height * (y + dy) / img_height as f64;

                            let (iter, _) = fractal.get_pixel(Complex::new(re, im), max_iter);

                            iter
                        })
                        .collect::<Vec<_>>();

                    ((i, j), samples)
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

            // Create samples image

            let mut samples_image =
                Mat::filled_with(vec![], img_width as usize, img_height as usize);

            // Get max/min iteration counts

            // const KEPT_PERCENTILE: usize = 98;
            // sorted_samples.truncate(KEPT_PERCENTILE * sorted_samples.len() / 100);

            let mut max_iter = 0;
            // let mut min_iter = 0;
            for (_, pixel_samples) in &samples {
                for &sample in pixel_samples {
                    max_iter = sample.max(max_iter);
                    // min_iter = sample.min(max_iter);
                }
            }

            // Fill `samples_image`

            for ((i, j), pixel_samples) in &samples {
                let filtered_samples = pixel_samples
                    .iter()
                    .copied()
                    .filter(|&v| v < 99 * max_iter / 100)
                    .collect::<Vec<_>>();

                samples_image
                    .set((*i as usize, *j as usize), filtered_samples)
                    .unwrap();
            }

            // -> Render image from samples and using bilateral filtering.

            let mut processed_image = Mat::filled_with(0., img_width as usize, img_height as usize);

            // How far the filter "reaches" in terms of spatial extent.
            const SPATIAL_SIGMA: f64 = 1.;
            // How tolerant the filter is to differences in values.
            const RANGE_SIGMA: f64 = 100.;

            // Normalize iteration count from range (min_iter, max_iter)
            // to (0, 1).
            #[inline]
            fn normalize_sample(iter: u32, max_iter: u32) -> f64 {
                let _min_iter = 0;
                (iter - _min_iter) as f64 / (max_iter - _min_iter) as f64
            }

            for j in 0..img_height as usize {
                for i in 0..img_width as usize {
                    // Note: the way sample vectors are constructed makes the
                    // first element always the one in the center of the pixel.
                    let center_sample =
                        normalize_sample(samples_image.get((i, j)).unwrap()[0], max_iter);

                    // see https://en.wikipedia.org/wiki/Bilateral_filter#Definition
                    let mut numerator = 0.;
                    let mut denominator = 0.;

                    // The kernel has to be only a 3 by 3 grid because otherwise,
                    // it picks samples from too far away in the picture. The
                    // samples from neighboring pixels are enough.
                    for dj in -1..1 {
                        for di in -1..1 {
                            let ii = i.wrapping_add_signed(di).min(img_width as usize - 1);
                            let jj = j.wrapping_add_signed(dj).min(img_height as usize - 1);

                            let other_samples = samples_image.get((ii, jj)).unwrap();

                            for (k, &other_sample) in other_samples.iter().enumerate() {
                                let other_sample = normalize_sample(other_sample, max_iter);

                                // Skip center_sample
                                if di != 0 && dj != 0 && k != 0 {
                                    let w = gaussian(
                                        (center_sample - other_sample).abs() / RANGE_SIGMA
                                            + ((di * di + dj * dj) as f64).sqrt() / SPATIAL_SIGMA,
                                    );

                                    numerator += w * other_sample;
                                    denominator += w;
                                }
                            }
                        }
                    }

                    processed_image
                        .set((i, j), (numerator / denominator).min(1.))
                        .unwrap();
                }
            }

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
