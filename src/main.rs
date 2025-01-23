mod coloring;
mod complex;
mod error;
mod fractal;
mod mat;
mod params;
mod sampling;

use std::{
    array, env, fs,
    io::Write,
    sync::{
        atomic::{self, AtomicUsize},
        mpsc,
    },
    time::Instant,
};

use fractal::Fractal;
use image::{Rgb, RgbImage};
use rayon::iter::{ParallelBridge, ParallelIterator};
use uni_path::PathBuf;
use wide::f64x4;

use crate::{
    coloring::{
        color_mapping,
        cumulative_histogram::{compute_histogram, cumulate_histogram, get_histogram_value},
        ColoringMode,
    },
    complex::Complex4,
    error::{ErrorKind, Result},
    mat::{Mat2D, Mat3D},
    params::{DevOptions, FractalParams, RenderStep},
    sampling::{generate_sampling_points, map_points_with_offsets, preview_sampling_points},
};

const CHUNK_SIZE: usize = 512;
const RDR_KERNEL_SIZE: usize = 1;

struct ViewParams {
    width: f64,
    height: f64,
    x_min: f64,
    y_min: f64,
}

#[derive(Debug, Clone, Copy)]
struct ChunkDimensions {
    v_chunks: usize,
    h_chunks: usize,
    last_v_chunk: usize,
    last_h_chunk: usize,
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        3 => {
            let (param_file_path, output_image_path) =
                (PathBuf::from(&args[1]), PathBuf::from(&args[2]));

            let params = ron::from_str::<FractalParams>(
                &fs::read_to_string(param_file_path.as_str())
                    .map_err(ErrorKind::ReadParameterFile)?,
            )
            .map_err(ErrorKind::DecodeParameterFile)?;

            // println!(
            //     "{}",
            //     ron::ser::to_string_pretty(&params, PrettyConfig::default()).unwrap()
            // );

            let FractalParams {
                img_width,
                img_height,
                render,
                max_iter,
                sampling,
                coloring_mode,
                custom_gradient,
                diverging_areas,
                dev_options,
            } = params;

            // Parse diverging areas

            let diverging_areas = diverging_areas.map(|areas| {
                areas
                    .iter()
                    .map(|&[min_x, max_x, min_y, max_y]| (min_x..max_x, (-max_y)..(-min_y)))
                    .collect::<Vec<_>>()
            });

            // sampling

            let sampling_points = generate_sampling_points(sampling.level);
            if let Some(DevOptions {
                save_sampling_pattern: true,
                ..
            }) = dev_options
            {
                preview_sampling_points(&sampling_points)?;
            }

            // Get chunks

            let v_chunks = (img_height as usize).div_euclid(CHUNK_SIZE);
            let h_chunks = (img_width as usize).div_euclid(CHUNK_SIZE);
            let last_v_chunk = (img_height as usize).rem_euclid(CHUNK_SIZE);
            let last_h_chunk = (img_width as usize).rem_euclid(CHUNK_SIZE);
            let chunks_dims = ChunkDimensions {
                v_chunks,
                h_chunks,
                last_v_chunk,
                last_h_chunk,
            };

            // Render

            let start = Instant::now();
            let stdout = std::io::stdout();

            // Compute escape time (number of iterations) for each pixel

            let render_raw_image = |fractal: Fractal,
                                    view_params: ViewParams,
                                    chunk_dims: ChunkDimensions,
                                    render_progress: AtomicUsize,
                                    render_total_progress: usize|
             -> Mat2D<f64> {
                let ViewParams {
                    width,
                    height,
                    x_min,
                    y_min,
                } = view_params;
                let ChunkDimensions {
                    v_chunks,
                    h_chunks,
                    last_v_chunk,
                    last_h_chunk,
                } = chunk_dims;

                let mut raw_image = Mat2D::filled_with(
                    0.,
                    img_width as usize + 2 * RDR_KERNEL_SIZE,
                    img_height as usize + 2 * RDR_KERNEL_SIZE,
                );

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

                        let rng = fastrand::Rng::new();
                        let (tx, rx) = mpsc::channel();
                        (0..chunk_height + 2 * RDR_KERNEL_SIZE)
                            .flat_map(|j| {
                                (0..chunk_width + 2 * RDR_KERNEL_SIZE).map(move |i| (i, j))
                            })
                            .par_bridge()
                            .for_each_with((tx, rng), |(s, rng), (i, j)| {
                                let x = (pi + i - RDR_KERNEL_SIZE) as f64;
                                let y = (pj + j - RDR_KERNEL_SIZE) as f64;

                                let should_render = diverging_areas
                                    .as_ref()
                                    .map(|areas| {
                                        !areas.iter().any(|(rx, ry)| {
                                            rx.contains(&(x_min + width * x / img_width as f64))
                                                && ry.contains(
                                                    &(y_min + height * y / img_height as f64),
                                                )
                                        })
                                    })
                                    .unwrap_or(true);

                                s.send((
                                    (i, j),
                                    should_render.then(|| {
                                        let (offset_x, offset_y) = if sampling.random_offsets {
                                            (rng.f64(), rng.f64())
                                        } else {
                                            (0., 0.)
                                        };
                                        let sampling_points = sampling_points
                                            .iter()
                                            .filter_map(|&(dx, dy)| {
                                                map_points_with_offsets(dx, dy, offset_x, offset_y)
                                            })
                                            .collect::<Vec<_>>();

                                        sampling_points
                                            .chunks(4)
                                            .flat_map(|d| {
                                                let l = d.len();
                                                let re = f64x4::from(array::from_fn(|i| {
                                                    // Here we use `i % l` to avoid out of bounds error (when i < 4).
                                                    // When `i < 4`, the modulo operation will repeat the sample
                                                    // but as we use simd this is acceptable (the cost is the
                                                    // same whether it is computed along with the others or not).
                                                    let (dx, _) = d[i % l];
                                                    x_min
                                                        + width * (x + 0.5 + dx) / img_width as f64
                                                }));
                                                let im = f64x4::from(array::from_fn(|i| {
                                                    let (_, dy) = d[i % l];
                                                    y_min
                                                        + height * (y + 0.5 + dy)
                                                            / img_height as f64
                                                }));

                                                let iter = fractal
                                                    .get_pixel(Complex4 { re, im }, max_iter);

                                                (0..l).map(move |i| (d[i], iter[i]))
                                            })
                                            .collect::<Vec<_>>()
                                    }),
                                ))
                                .unwrap();

                                // Using atomic::Ordering::Relaxed because we don't really
                                // care about the order `progress` is updated. As long as it
                                // is updated it should be fine :>
                                render_progress.fetch_add(1, atomic::Ordering::Relaxed);
                                let progress = render_progress.load(atomic::Ordering::Relaxed);

                                if progress % (render_total_progress / 100000 + 1) == 0 {
                                    stdout
                                        .lock()
                                        .write_all(
                                            format!(
                                                "\r {:.1}% - {:.1}s elapsed",
                                                100. * progress as f32
                                                    / render_total_progress as f32,
                                                start.elapsed().as_secs_f32(),
                                            )
                                            .as_bytes(),
                                        )
                                        .unwrap();
                                }
                            });

                        let mut chunk_samples = Mat3D::filled_with(
                            None,
                            chunk_width + 2 * RDR_KERNEL_SIZE,
                            chunk_height + 2 * RDR_KERNEL_SIZE,
                            sampling_points.len(),
                        );
                        for ((i, j), pixel_samples) in rx {
                            if let Some(pixel_samples) = pixel_samples {
                                for (k, &v) in pixel_samples.iter().enumerate() {
                                    chunk_samples.set((i, j, k), Some(v)).unwrap();
                                }
                            }
                        }

                        let (tx, rx) = mpsc::channel();
                        (0..chunk_height)
                            .flat_map(|j| (0..chunk_width).map(move |i| (i, j)))
                            .par_bridge()
                            .for_each_with(tx, |s, (i, j)| {
                                let mut weighted_sum = 0.;
                                let mut weight_total = 0.;

                                let mut is_empty = true;
                                const RDR_KERNEL_SIZE_I: isize = RDR_KERNEL_SIZE as isize;
                                for dj in -RDR_KERNEL_SIZE_I..=RDR_KERNEL_SIZE_I {
                                    for di in -RDR_KERNEL_SIZE_I..=RDR_KERNEL_SIZE_I {
                                        let ii = (i + RDR_KERNEL_SIZE)
                                            .checked_add_signed(di)
                                            .expect("should never overflow");
                                        let jj = (j + RDR_KERNEL_SIZE)
                                            .checked_add_signed(dj)
                                            .expect("should never overflow");

                                        for k in 0..sampling_points.len() {
                                            if let &Some(((dx, dy), v)) =
                                                chunk_samples.get((ii, jj, k)).unwrap()
                                            {
                                                let dx = dx + di as f64;
                                                let dy = dy + dj as f64;

                                                // This only includes samples from a round-cornered square.
                                                // see https://www.desmos.com/3d/kgdwgwp4dk

                                                if dx.abs() <= 0.5 && dy.abs() <= 0.5 {
                                                    let w = 1.;
                                                    weighted_sum += w * v;
                                                    weight_total += w;
                                                } else if pi + i != 0 && pj + j != 0 {
                                                    // `pi + i != 0 && pj + j != 0`` is an ugly fix for a small issue
                                                    // I wasn't able to find the origin: the first column and the
                                                    // first row of pixels of the image are colored weirdly without
                                                    // this condition...

                                                    // Maximum distance for picking samples (out of the pixel)
                                                    const D: f64 = 0.4;
                                                    const D_SQR: f64 = D * D;
                                                    // This is the value of the weight at the border.
                                                    const T: f64 = 0.5;
                                                    // The "radius" of the square (half its side length).
                                                    // This should not be changed.
                                                    const R: f64 = 0.5;

                                                    let smooth_distance_sqr =
                                                        (dx.abs() - R).max(0.).powi(2)
                                                            + (dy.abs() - R).max(0.).powi(2);
                                                    if smooth_distance_sqr < D_SQR {
                                                        let w =
                                                            1. - T * smooth_distance_sqr / D_SQR;
                                                        weighted_sum += w * v;
                                                        weight_total += w;
                                                    }
                                                }

                                                is_empty = false;
                                            };
                                        }
                                    }
                                }

                                s.send((
                                    (pi + i, pj + j),
                                    if is_empty {
                                        max_iter as f64
                                    } else {
                                        weighted_sum / weight_total
                                    },
                                ))
                                .unwrap();
                            });

                        for (index, v) in rx {
                            raw_image.set(index, v).unwrap();
                        }
                    }
                }

                raw_image
            };

            match render {
                params::Render::Frame {
                    zoom,
                    center_x,
                    center_y,
                    fractal,
                } => {
                    let view_params = setup_view(img_width, img_height, zoom, center_x, center_y);

                    let (progress, total_progress) = init_progress(chunks_dims);

                    let raw_image = render_raw_image(
                        fractal,
                        view_params,
                        chunks_dims,
                        progress,
                        total_progress,
                    );

                    println!();

                    let output_image = color_raw_image(
                        img_width,
                        img_height,
                        max_iter,
                        raw_image,
                        coloring_mode,
                        custom_gradient.as_ref(),
                        dev_options,
                    );

                    output_image
                        .save(output_image_path.as_str())
                        .map_err(ErrorKind::SaveImage)?;

                    let image_size = fs::metadata(output_image_path.as_str()).unwrap().len();
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
                        if let Some(ext) = output_image_path.extension() {
                            format!("- {} ", ext)
                        } else {
                            "".to_string()
                        }
                    );
                }
                params::Render::Animation {
                    zoom,
                    center_x,
                    center_y,
                    fractal,
                    duration,
                    fps,
                } => {
                    let frame_count = (duration * fps) as usize;

                    println!("frame count: {}", frame_count);
                    println!();

                    for frame_i in 0..frame_count {
                        let t = frame_i as f64 / fps;

                        let zoom = zoom[RenderStep::get_current_step_index(&zoom, t)].get_value(t);
                        let center_x =
                            center_x[RenderStep::get_current_step_index(&center_x, t)].get_value(t);
                        let center_y =
                            center_y[RenderStep::get_current_step_index(&center_y, t)].get_value(t);

                        let view_params =
                            setup_view(img_width, img_height, zoom, center_x, center_y);

                        let (progress, total_progress) = init_progress(chunks_dims);

                        let raw_image = render_raw_image(
                            fractal.get_fractal(t),
                            view_params,
                            chunks_dims,
                            progress,
                            total_progress,
                        );

                        println!();

                        let output_image = color_raw_image(
                            img_width,
                            img_height,
                            max_iter,
                            raw_image,
                            coloring_mode,
                            custom_gradient.as_ref(),
                            dev_options,
                        );

                        let output_image_path = PathBuf::from(
                            output_image_path.parent().unwrap().to_string()
                                + "/"
                                + output_image_path.file_stem().unwrap()
                                + "_"
                                + &format!("{:06}", frame_i)
                                + "."
                                + output_image_path.extension().unwrap(),
                        );

                        output_image
                            .save(output_image_path.as_str())
                            .map_err(ErrorKind::SaveImage)?;

                        let image_size = fs::metadata(output_image_path.as_str()).unwrap().len();
                        println!(
                            " frame {}: {}x{} - {} {}",
                            frame_i + 1,
                            img_width,
                            img_height,
                            if image_size / 1_000_000 != 0 {
                                format!("{:.1}mb", image_size as f32 / 1_000_000.)
                            } else if image_size / 1_000 != 0 {
                                format!("{:.1}kb", image_size as f32 / 1_000.)
                            } else {
                                format!("{}b", image_size)
                            },
                            if let Some(ext) = output_image_path.extension() {
                                format!("- {} ", ext)
                            } else {
                                "".to_string()
                            }
                        );
                        println!();
                    }

                    println!("{} frames - {:?} elapsed", frame_count, start.elapsed())
                }
            }
        }
        _ => {
            println!("This is a fractal renderer.");
            println!("Usage: fractal_renderer <param file path>.json <output image path>.png");
            println!("More information: https://gh.valflrt.dev/fractal_renderer");
        }
    }

    Ok(())
}

fn setup_view(
    img_width: u32,
    img_height: u32,
    zoom: f64,
    center_x: f64,
    center_y: f64,
) -> ViewParams {
    let aspect_ratio = img_width as f64 / img_height as f64;

    let width = zoom;
    let height = width / aspect_ratio;
    let x_min = center_x - width / 2.;
    // make center_y negative to match complex number representation
    // (in which the imaginary axis is pointing upward)
    let y_min = -center_y - height / 2.;

    ViewParams {
        width,
        height,
        x_min,
        y_min,
    }
}

fn init_progress(
    ChunkDimensions {
        v_chunks,
        h_chunks,
        last_v_chunk,
        last_h_chunk,
    }: ChunkDimensions,
) -> (AtomicUsize, usize) {
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

                (chunk_width + 2 * RDR_KERNEL_SIZE) * (chunk_height + 2 * RDR_KERNEL_SIZE)
            })
        })
        .sum::<usize>();

    (progress, total)
}

fn color_raw_image(
    img_width: u32,
    img_height: u32,
    max_iter: u32,
    mut raw_image: Mat2D<f64>,
    coloring_mode: ColoringMode,
    custom_gradient: Option<&Vec<(f64, [u8; 3])>>,
    dev_options: Option<DevOptions>,
) -> RgbImage {
    let mut output_image = RgbImage::new(img_width, img_height);

    let max = raw_image.vec.iter().copied().fold(0., f64::max);
    let min = raw_image.vec.iter().copied().fold(max, f64::min);

    match coloring_mode {
        ColoringMode::CumulativeHistogram => {
            raw_image.vec.iter_mut().for_each(|v| *v /= max);
            let cumulative_histogram = cumulate_histogram(compute_histogram(&raw_image.vec));
            for j in 0..img_height as usize {
                for i in 0..img_width as usize {
                    let &value = raw_image.get((i, j)).unwrap();
                    output_image.put_pixel(
                        i as u32,
                        j as u32,
                        color_mapping(
                            get_histogram_value(value, &cumulative_histogram).powi(12),
                            custom_gradient,
                        ),
                    );
                }
            }
        }
        ColoringMode::MaxIterNorm { map_value } => {
            for j in 0..img_height as usize {
                for i in 0..img_width as usize {
                    let &value = raw_image.get((i, j)).unwrap();

                    let t = map_value.map_value(value / max_iter as f64);

                    output_image.put_pixel(i as u32, j as u32, color_mapping(t, custom_gradient));
                }
            }
        }
        ColoringMode::MaxNorm { map_value } => {
            for j in 0..img_height as usize {
                for i in 0..img_width as usize {
                    let &value = raw_image.get((i, j)).unwrap();

                    let t = map_value.map_value(value / max);

                    output_image.put_pixel(i as u32, j as u32, color_mapping(t, custom_gradient));
                }
            }
        }
        ColoringMode::MinMaxNorm { map_value } => {
            for j in 0..img_height as usize {
                for i in 0..img_width as usize {
                    let &value = raw_image.get((i, j)).unwrap();

                    let t = map_value.map_value((value - min) / (max - min));

                    output_image.put_pixel(i as u32, j as u32, color_mapping(t, custom_gradient));
                }
            }
        }
        ColoringMode::CustomMaxNorm { max, map_value } => {
            for j in 0..img_height as usize {
                for i in 0..img_width as usize {
                    let &value = raw_image.get((i, j)).unwrap();

                    let t = map_value.map_value(value / max);

                    output_image.put_pixel(i as u32, j as u32, color_mapping(t, custom_gradient));
                }
            }
        }
        ColoringMode::CustomMinMaxNorm {
            min,
            max,
            map_value,
        } => {
            for j in 0..img_height as usize {
                for i in 0..img_width as usize {
                    let &value = raw_image.get((i, j)).unwrap();

                    let t = map_value.map_value((value - min) / (max - min));

                    output_image.put_pixel(i as u32, j as u32, color_mapping(t, custom_gradient));
                }
            }
        }
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
    };

    if let Some(DevOptions {
        display_gradient: true,
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
                    color_mapping(i as f64 / GRADIENT_WIDTH as f64, custom_gradient),
                );
            }
        }
    }

    output_image
}
