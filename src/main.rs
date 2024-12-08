mod coloring;
mod fractal_kind;
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
use num_complex::Complex;
use rayon::iter::{ParallelBridge, ParallelIterator};
use sampling::spiral_sampling_points;
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
    custom_gradient: Option<Vec<(f64, [u8; 3])>>,
    display_gradient: Option<bool>,
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
                    custom_gradient,
                    display_gradient,
                }) => {
                    let aspect_ratio = img_width as f64 / img_height as f64;

                    let width = zoom;
                    let height = width / aspect_ratio;
                    let x_min = center_x - width / 2.;
                    // -center_y to match complex number representation (vertical
                    // axis flipped)
                    let y_min = -center_y - height / 2.;

                    // init image

                    let mut img = RgbImage::new(img_width, img_height);

                    // sampling

                    let sampling_points = supersampling.map(|s| spiral_sampling_points(s));

                    // // preview sampling points
                    // if let Some(points) = &sampling_points {
                    //     let size = 400;
                    //     let mut sampling_points = RgbImage::new(size, size);

                    //     for &(x, y) in points {
                    //         sampling_points.put_pixel(
                    //             (size as f64 / 2. + 50. * x) as u32,
                    //             (size as f64 / 2. + 50. * y) as u32,
                    //             Rgb([255, 255, 255]),
                    //         );
                    //     }

                    //     sampling_points.save("pattern.png").unwrap();
                    // };

                    // progress

                    let start = Instant::now();

                    let progress = atomic::AtomicU32::new(0);
                    let total = img_height * img_width;

                    let stdout = std::io::stdout();

                    let pixel_values = (0..img_height)
                        .flat_map(|y| (0..img_width).map(move |x| (x, y)))
                        .par_bridge()
                        .map(|(x, y)| {
                            let re = x_min + (x as f64 / img_width as f64) * width;
                            let im = y_min + (y as f64 / img_height as f64) * height;
                            let c = Complex::new(re, im);

                            let (mut iterations, _) = fractal_kind.get_pixel(c, max_iter);

                            if let Some(points) = &sampling_points {
                                let (weighted_iteration_sum, weights_sum) =
                                    points.iter().fold((0., 0.), |acc, &(dx, dy)| {
                                        let re =
                                            x_min + ((x as f64 + dx) / img_width as f64) * width;
                                        let im =
                                            y_min + ((y as f64 + dy) / img_height as f64) * height;

                                        let (iter, _) =
                                            fractal_kind.get_pixel(Complex::new(re, im), max_iter);

                                        let weight = 1.;
                                        (acc.0 + iter as f64, acc.1 + weight)
                                    });

                                iterations += (weighted_iteration_sum / weights_sum) as u32;
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
                                    color_mapping(
                                        iterations as f64 / max_iter as f64,
                                        &custom_gradient,
                                    ),
                                );
                            }
                        }
                        ColoringMode::Squared => {
                            for (x, y, iterations) in pixel_values {
                                img.put_pixel(
                                    x,
                                    y,
                                    color_mapping(
                                        (iterations as f64 / max_iter as f64).powi(2),
                                        &custom_gradient,
                                    ),
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
                                img.put_pixel(x, y, color_mapping(t, &custom_gradient));
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
                                        &custom_gradient,
                                    ),
                                );
                            }
                        }
                    };

                    if let Some(true) = display_gradient {
                        const GRADIENT_HEIGHT: u32 = 8;
                        const GRADIENT_WIDTH: u32 = 64;
                        const OFFSET: u32 = 8;

                        for j in 0..GRADIENT_HEIGHT {
                            for i in 0..GRADIENT_WIDTH {
                                img.put_pixel(
                                    img_width - GRADIENT_WIDTH - OFFSET + i,
                                    img_height - GRADIENT_HEIGHT - OFFSET + j,
                                    color_mapping(
                                        i as f64 / GRADIENT_WIDTH as f64,
                                        &custom_gradient,
                                    ),
                                );
                            }
                        }
                    }

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
