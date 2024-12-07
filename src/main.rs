mod coloring;
mod fractal_kind;

use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    sync::atomic,
    time::Instant,
};

use image::{ImageBuffer, Rgb};
use num::complex::Complex;
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};

use coloring::{color_mapping, compute_histogram, cumulate_histogram, ColoringMode};
use fractal_kind::FractalKind;

#[derive(Debug, Serialize, Deserialize)]
struct FractalParams {
    img_width: u32,
    img_height: u32,
    zoom: f64,
    center_x: f64,
    center_y: f64,
    max_iter: u32,
    supersampling: Option<u32>,
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
                    supersampling,
                    fractal_kind,
                    coloring_mode,
                }) => {
                    if supersampling.is_some_and(|s| s == 0 || s > 256) {
                        panic!("supersampling should be between 1 and 256");
                    };

                    let aspect_ratio = img_width as f64 / img_height as f64;

                    let width = zoom;
                    let height = width / aspect_ratio;
                    let x_min = center_x - width / 2.;
                    let x_max = center_x + width / 2.;
                    let y_min = center_y - height / 2.;
                    let y_max = center_y + height / 2.;

                    let mut img = ImageBuffer::new(img_width, img_height);

                    let start = Instant::now();

                    let progress = atomic::AtomicU32::new(0);
                    let total = img_height * img_width;

                    let stdout = std::io::stdout();

                    let pixel_values = (0..img_height)
                        .flat_map(|y| (0..img_width).map(move |x| (x, y)))
                        .par_bridge()
                        .map(|(x, y)| {
                            let real = x_min + (x as f64 / img_width as f64) * (x_max - x_min);
                            let imag = y_min + (y as f64 / img_height as f64) * (y_max - y_min);
                            let c = Complex::new(real, imag);

                            let (mut iterations, _) = fractal_kind.get_pixel(c, max_iter);

                            if let Some(s) = supersampling {
                                if s > 0 {
                                    iterations += (0..s).fold(0, |acc, _| {
                                        let dx = fastrand::f64() - 0.5;
                                        let dy = fastrand::f64() - 0.5;

                                        let real = x_min
                                            + ((x as f64 + dx) / img_width as f64)
                                                * (x_max - x_min);
                                        let imag = y_min
                                            + ((y as f64 + dy) / img_height as f64)
                                                * (y_max - y_min);
                                        let c = Complex::new(real, imag);

                                        let (iter, _) = fractal_kind.get_pixel(c, max_iter);

                                        acc + iter
                                    });
                                    iterations /= s + 1;
                                }
                            };

                            // using atomic::Ordering::Relaxed
                            progress.fetch_add(1, atomic::Ordering::Relaxed);
                            let progress = progress.load(atomic::Ordering::Relaxed);
                            if progress % (total / 100000 + 1) == 0 {
                                stdout
                                    .lock()
                                    .write_all(
                                        format!(
                                            "\r {:.1}% - {:3.1}s elapsed ",
                                            100. * progress as f32 / total as f32,
                                            start.elapsed().as_secs_f32()
                                        )
                                        .as_bytes(),
                                    )
                                    .unwrap();
                            }

                            (x, y, iterations)
                        })
                        .collect::<Vec<_>>();

                    println!();

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
                        ColoringMode::LinearMinMax => {
                            let (min, max) = pixel_values.iter().fold(
                                (max_iter, 0),
                                |(acc_min, acc_max), &(_, _, v)| {
                                    (
                                        if acc_min > v { v } else { acc_min },
                                        if acc_max < v { v } else { acc_max },
                                    )
                                },
                            );

                            for (x, y, iterations) in pixel_values {
                                let t = (iterations - min) as f64 / (max - min) as f64;
                                img.put_pixel(x, y, color_mapping(t));
                            }
                        }
                        ColoringMode::CumulativeHistogram => {
                            let cumulative_histogram = cumulate_histogram(
                                compute_histogram(&pixel_values, max_iter),
                                max_iter,
                            );
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

                    let path = PathBuf::from(&args[2]);
                    img.save(&path).expect("failed to save fractal image");
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
