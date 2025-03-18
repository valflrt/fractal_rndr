mod cli;
mod coloring;
mod complexx;
mod error;
mod fractal;
mod gui;
mod mat;
mod params;
#[allow(dead_code)]
mod presets;
mod progress;
mod rendering;
mod sampling;

use std::{
    fs,
    io::Write,
    thread,
    time::{Duration, Instant},
};

use eframe::egui::vec2;
use gui::Gui;
use params::{AnimationParams, FrameParams};
use ron::ser::PrettyConfig;
use uni_path::PathBuf;

use crate::{
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
}

fn main() -> Result<()> {
    let (args, options) = cli::parse();

    match args.len() {
        3 => {
            let (param_file_path, output_image_path) =
                (PathBuf::from(&args[1]), PathBuf::from(&args[2]));

            let params = if fs::exists(param_file_path.as_str()).unwrap() {
                ron::from_str::<ParamsKind>(
                    &fs::read_to_string(param_file_path.as_str())
                        .map_err(ErrorKind::ReadParameterFile)?,
                )
                .map_err(ErrorKind::DecodeParameterFile)?
            } else {
                let params = ParamsKind::default();
                fs::write(
                    param_file_path.as_str(),
                    ron::ser::to_string_pretty(&params, PrettyConfig::default())
                        .map_err(ErrorKind::EncodeParameterFile)?,
                )
                .map_err(ErrorKind::WriteParameterFile)?;
                params
            };

            // println!(
            //     "{}",
            //     ron::ser::to_string_pretty(&params, PrettyConfig::default()).unwrap()
            // );

            match params {
                ParamsKind::Frame(params) => {
                    if options.contains_key("gui") {
                        start_gui(params, param_file_path, output_image_path)?;
                    } else {
                        render_frame(params, output_image_path)?;
                    }
                }
                ParamsKind::Animation(animation_params) => {
                    if options.contains_key("gui") {
                        println!("gui is not supported for animations. exiting...");
                        return Ok(());
                    }

                    render_animation(animation_params, output_image_path)?;
                }
            }
        }
        _ => {
            println!("This is a fractal renderer.");
            println!("Usage: fractal_rndr <param file path>.json <output image path>.png");
            println!("More information: https://gh.valflrt.dev/fractal_rndr");
        }
    }

    Ok(())
}

fn render_frame(params: FrameParams, output_image_path: PathBuf) -> Result<()> {
    let FrameParams {
        img_width,
        img_height,

        zoom,
        center_x,
        center_y,
        rotate,

        sampling,
        ..
    } = params;

    let view = View::new(img_width, img_height, zoom, center_x, center_y, rotate);

    let sampling_points = generate_sampling_points(sampling.level);

    if let Some(DevOptions {
        save_sampling_pattern: Some(true),
        ..
    }) = params.dev_options
    {
        preview_sampling_points(&sampling_points)?;
    }

    let progress = Progress::new((img_width * img_height) as usize);

    let start = Instant::now();

    let params_clone = params.clone();
    let progress_clone = progress.clone();
    let sampling_points_clone = sampling_points.clone();
    let handle = thread::spawn(move || {
        render_raw_image(
            &params_clone,
            &view,
            &sampling_points_clone,
            Some(progress_clone),
        )
    });

    while !handle.is_finished() {
        print!(
            "\r {:.1}% - {:.1}s elapsed",
            100. * progress.get_progress(),
            start.elapsed().as_secs_f32(),
        );
        std::io::stdout().flush().unwrap();

        thread::sleep(Duration::from_millis(50));
    }

    let raw_image = handle.join().unwrap(); // TODO replace unwrap

    println!();

    let output_image = color_raw_image(
        &params,
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

    Ok(())
}

fn render_animation(params: AnimationParams, output_image_path: PathBuf) -> Result<()> {
    let AnimationParams {
        sampling,

        duration,
        fps,
        ..
    } = params;

    let frame_count = (duration * fps) as usize;

    println!("frame count: {}", frame_count);
    println!();

    let sampling_points = generate_sampling_points(sampling.level);

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
            rotate,
            ..
        } = params;

        let view = View::new(img_width, img_height, zoom, center_x, center_y, rotate);

        let progress = Progress::new((img_width * img_height) as usize);

        let start = Instant::now();

        let params_clone = params.clone();
        let progress_clone = progress.clone();
        let sampling_points_clone = sampling_points.clone();
        let handle = thread::spawn(move || {
            render_raw_image(
                &params_clone,
                &view,
                &sampling_points_clone,
                Some(progress_clone),
            )
        });

        while !handle.is_finished() {
            print!(
                "\r {:.1}% - {:.1}s elapsed",
                100. * progress.get_progress(),
                start.elapsed().as_secs_f32(),
            );
            std::io::stdout().flush().unwrap();

            thread::sleep(Duration::from_millis(50));
        }

        let raw_image = handle.join().unwrap(); // TODO replace unwrap

        println!();

        let mut output_image = color_raw_image(
            &params,
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
    );

    Ok(())
}

fn start_gui(
    params: FrameParams,
    param_file_path: PathBuf,
    output_image_path: PathBuf,
) -> Result<()> {
    let FrameParams {
        img_width,
        img_height,

        zoom,
        center_x,
        center_y,
        rotate,
        ..
    } = params;

    let mut options = eframe::NativeOptions::default();
    let size = Some(vec2(900., 440.));
    options.viewport.inner_size = size;
    options.viewport.min_inner_size = size;

    eframe::run_native(
        "fractal renderer",
        options,
        Box::new(|cc| {
            Ok(Box::new(Gui::new(
                cc,
                params,
                View::new(img_width, img_height, zoom, center_x, center_y, rotate),
                output_image_path,
                param_file_path,
            )))
        }),
    )
    .map_err(|_| ErrorKind::StartGui)?;

    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct View {
    width: F,
    height: F,
    cx: F,
    cy: F,
    rotate: F,
}

impl View {
    pub fn new(
        img_width: u32,
        img_height: u32,
        zoom: F,
        center_x: F,
        center_y: F,
        rotate: Option<F>,
    ) -> View {
        let aspect_ratio = img_width as F / img_height as F;

        View {
            width: zoom,
            height: zoom / aspect_ratio,
            cx: center_x,
            cy: -center_y,
            rotate: rotate.unwrap_or(0.),
        }
    }
}
