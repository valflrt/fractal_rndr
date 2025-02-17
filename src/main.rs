mod coloring;
mod complexx;
mod error;
mod fractal;
mod mat;
mod params;
mod progress;
mod rendering;
mod sampling;

use std::{env, fs, time::Instant};

use image::{Rgb, RgbImage};
use uni_path::PathBuf;

use crate::{
    coloring::{
        color_mapping,
        cumulative_histogram::{compute_histogram, cumulate_histogram, get_histogram_value},
        ColoringMode,
    },
    error::{ErrorKind, Result},
    mat::Mat2D,
    params::{DevOptions, FractalParams, RenderStep},
    progress::Progress,
    rendering::{render_raw_image, RenderingCtx},
    sampling::{generate_sampling_points, preview_sampling_points},
};

#[cfg(feature = "force_f32")]
type F = f32;
#[cfg(feature = "force_f32")]
use wide::f32x8;
#[cfg(feature = "force_f32")]
type FX = f32x8;

#[cfg(not(feature = "force_f32"))]
type F = f64;
#[cfg(not(feature = "force_f32"))]
use wide::f64x4;
#[cfg(not(feature = "force_f32"))]
type FX = f64x4;

struct ViewParams {
    width: F,
    height: F,
    x_min: F,
    y_min: F,
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
                dev_options,
            } = params;

            // sampling

            let sampling_points = generate_sampling_points(sampling.level);
            if let Some(DevOptions {
                save_sampling_pattern: Some(true),
                ..
            }) = dev_options
            {
                preview_sampling_points(&sampling_points)?;
            }

            // Render

            let start = Instant::now();
            let stdout = std::io::stdout();

            // Compute escape time (number of iterations) for each pixel

            let rendering_ctx = RenderingCtx {
                img_width,
                img_height,
                max_iter,
                sampling,
                sampling_points: &sampling_points,
                start,
                stdout: &stdout,
            };

            match render {
                params::RenderKind::Frame {
                    zoom,
                    center_x,
                    center_y,
                    fractal,
                } => {
                    let view_params = setup_view(img_width, img_height, zoom, center_x, center_y);

                    let progress = Progress::new((img_width * img_height) as usize);

                    let raw_image = render_raw_image(fractal, view_params, rendering_ctx, progress);

                    println!();

                    let output_image = color_raw_image(
                        img_width,
                        img_height,
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
                params::RenderKind::Animation {
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
                        let t = frame_i as f32 / fps;

                        let zoom = zoom[RenderStep::get_current_step_index(&zoom, t)].get_value(t);
                        let center_x =
                            center_x[RenderStep::get_current_step_index(&center_x, t)].get_value(t);
                        let center_y =
                            center_y[RenderStep::get_current_step_index(&center_y, t)].get_value(t);

                        let view_params =
                            setup_view(img_width, img_height, zoom, center_x, center_y);

                        let progress = Progress::new((img_width * img_height) as usize);

                        let raw_image = render_raw_image(
                            fractal.get_fractal(t),
                            view_params,
                            rendering_ctx,
                            progress,
                        );

                        println!();

                        let output_image = color_raw_image(
                            img_width,
                            img_height,
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

                    println!(
                        "{} frames - {:.1}s elapsed",
                        frame_count,
                        start.elapsed().as_secs_f32()
                    )
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

fn setup_view(img_width: u32, img_height: u32, zoom: F, center_x: F, center_y: F) -> ViewParams {
    let aspect_ratio = img_width as F / img_height as F;

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

fn color_raw_image(
    img_width: u32,
    img_height: u32,
    mut raw_image: Mat2D<F>,
    coloring_mode: ColoringMode,
    custom_gradient: Option<&Vec<(f32, [u8; 3])>>,
    dev_options: Option<DevOptions>,
) -> RgbImage {
    let mut output_image = RgbImage::new(img_width, img_height);

    let max_v = raw_image.vec.iter().copied().fold(0., F::max);
    let min_v = raw_image.vec.iter().copied().fold(max_v, F::min);

    match coloring_mode {
        ColoringMode::CumulativeHistogram { map } => {
            raw_image.vec.iter_mut().for_each(|v| *v /= max_v);
            let cumulative_histogram = cumulate_histogram(compute_histogram(&raw_image.vec));
            for j in 0..img_height as usize {
                for i in 0..img_width as usize {
                    let &value = raw_image.get((i, j)).unwrap();

                    let t = map.apply(get_histogram_value(value, &cumulative_histogram));

                    output_image.put_pixel(i as u32, j as u32, color_mapping(t, custom_gradient));
                }
            }
        }
        ColoringMode::MaxNorm { max, map } => {
            let max = max.unwrap_or(max_v);

            for j in 0..img_height as usize {
                for i in 0..img_width as usize {
                    let &value = raw_image.get((i, j)).unwrap();

                    let t = map.apply(value / max);

                    output_image.put_pixel(i as u32, j as u32, color_mapping(t, custom_gradient));
                }
            }
        }
        ColoringMode::MinMaxNorm { min, max, map } => {
            let min = min.unwrap_or(min_v);
            let max = max.unwrap_or(max_v);

            for j in 0..img_height as usize {
                for i in 0..img_width as usize {
                    let &value = raw_image.get((i, j)).unwrap();

                    let t = map.apply((value - min) / (max - min));

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
                    color_mapping(i as F / GRADIENT_WIDTH as F, custom_gradient),
                );
            }
        }
    }

    output_image
}
