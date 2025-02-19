mod cli;
mod coloring;
mod complexx;
mod error;
mod fractal;
mod gui;
mod mat;
mod params;
mod progress;
mod rendering;
mod sampling;

use std::{fs, time::Instant};

use gui::Gui;
use params::FrameParams;
use uni_path::PathBuf;

use crate::{
    cli::get_args_and_options,
    coloring::{color_mapping, color_raw_image},
    error::{ErrorKind, Result},
    params::{DevOptions, ParamsKind},
    progress::Progress,
    rendering::render_raw_image,
    sampling::preview_sampling_points,
    sampling::{generate_sampling_points, Sampling},
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

#[derive(Debug, Clone)]
pub struct RenderCtx {
    pub img_width: u32,
    pub img_height: u32,

    pub max_iter: u32,

    pub sampling: Sampling,
    pub sampling_points: Vec<(F, F)>,

    pub start: Instant,
}

fn main() -> Result<()> {
    let (args, options) = get_args_and_options();

    match args.len() {
        3 => {
            let (param_file_path, output_image_path) =
                (PathBuf::from(&args[1]), PathBuf::from(&args[2]));

            let params = ron::from_str::<ParamsKind>(
                &fs::read_to_string(param_file_path.as_str())
                    .map_err(ErrorKind::ReadParameterFile)?,
            )
            .map_err(ErrorKind::DecodeParameterFile)?;

            // println!(
            //     "{}",
            //     ron::ser::to_string_pretty(&params, PrettyConfig::default()).unwrap()
            // );

            match params {
                ParamsKind::Frame { .. } => {
                    let params = params.get_frame_params(0.);
                    let FrameParams {
                        img_width,
                        img_height,
                        zoom,
                        center_x,
                        center_y,
                        fractal,
                        ..
                    } = params;
                    let render_ctx = RenderCtx::new(&params)?;

                    if let Some(DevOptions {
                        save_sampling_pattern: Some(true),
                        ..
                    }) = params.dev_options
                    {
                        preview_sampling_points(&render_ctx.sampling_points)?;
                    }

                    let view = View::new(img_width, img_height, zoom, center_x, center_y);

                    if options.contains_key("gui") {
                        let options = eframe::NativeOptions::default();
                        eframe::run_native(
                            "app",
                            options,
                            Box::new(|cc| {
                                Ok(Box::new(Gui::new(
                                    cc,
                                    params,
                                    render_ctx,
                                    view,
                                    param_file_path,
                                )))
                            }),
                        )
                        .unwrap();
                    } else {
                        let progress = Progress::new((img_width * img_height) as usize);

                        let raw_image =
                            render_raw_image(fractal, &view, &render_ctx, Some(progress));

                        println!();

                        let output_image = color_raw_image(
                            &render_ctx,
                            params.coloring_mode,
                            params.custom_gradient.as_ref(),
                            raw_image,
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
                }
                ParamsKind::Animation { duration, fps, .. } => {
                    let frame_count = (duration * fps) as usize;

                    println!("frame count: {}", frame_count);
                    println!();

                    let global_start = Instant::now();

                    for frame_i in 0..frame_count {
                        let t = frame_i as f32 / fps;

                        let params = params.get_frame_params(t);
                        let FrameParams {
                            img_width,
                            img_height,
                            zoom,
                            center_x,
                            center_y,
                            fractal,
                            ..
                        } = params;

                        let render_ctx = RenderCtx::new(&params)?;

                        let view = View::new(img_width, img_height, zoom, center_x, center_y);

                        let progress = Progress::new((img_width * img_height) as usize);

                        let raw_image =
                            render_raw_image(fractal, &view, &render_ctx, Some(progress));

                        println!();

                        let mut output_image = color_raw_image(
                            &render_ctx,
                            params.coloring_mode,
                            params.custom_gradient.as_ref(),
                            raw_image,
                        );

                        if let Some(DevOptions {
                            display_gradient: Some(true),
                            ..
                        }) = params.dev_options
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
                                            i as F / GRADIENT_WIDTH as F,
                                            params.custom_gradient.as_ref(),
                                        ),
                                    );
                                }
                            }
                        }

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
                        global_start.elapsed().as_secs_f32()
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

#[derive(Debug, Clone, Copy)]
struct View {
    width: F,
    height: F,
    x_min: F,
    y_min: F,
}

impl View {
    pub fn new(img_width: u32, img_height: u32, zoom: F, center_x: F, center_y: F) -> View {
        let aspect_ratio = img_width as F / img_height as F;

        let width = zoom;
        let height = width / aspect_ratio;
        let x_min = center_x - width / 2.;
        // make center_y negative to match complex number representation
        // (in which the imaginary axis is pointing upward)
        let y_min = -center_y - height / 2.;

        View {
            width,
            height,
            x_min,
            y_min,
        }
    }
}

impl RenderCtx {
    pub fn new(params: &FrameParams) -> Result<RenderCtx> {
        let &FrameParams {
            img_width,
            img_height,
            max_iter,
            sampling,
            ..
        } = params;

        // sampling

        let sampling_points = generate_sampling_points(sampling.level);

        // start timestamp

        let start = Instant::now();

        // Compute escape time (number of iterations) for each pixel

        Ok(RenderCtx {
            img_width,
            img_height,
            max_iter,
            sampling,
            sampling_points,
            start,
        })
    }
}
