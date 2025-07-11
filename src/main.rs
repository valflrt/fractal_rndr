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

use eframe::egui::ViewportBuilder;
use gui::WINDOW_SIZE;
use uni_path::PathBuf;

use crate::{
    coloring::{color_mapping, color_raw_image},
    error::{ErrorKind, Result},
    gui::Gui,
    params::{AnimationParams, DevOptions, FrameParams, ParamsKind},
    progress::Progress,
    rendering::render_raw_image,
    sampling::preview_sampling_points,
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

const USAGE: &str = "This is a fractal renderer.
Usage: fractal_rndr <param file path> <output image path>
Use --gui to start the gui for image rendering (not animation).

More information: https://gh.valflrt.dev/fractal_rndr";

fn main() -> Result<()> {
    let args = valargs::parse();

    let (param_file_path, output_image_path) = (
        args.nth(1).map(PathBuf::from),
        args.nth(2).map(PathBuf::from),
    );

    if let (Some(param_file_path), Some(output_image_path)) = (param_file_path, output_image_path) {
        let params = ron::from_str::<ParamsKind>(
            &fs::read_to_string(param_file_path.as_str()).map_err(ErrorKind::ReadParameterFile)?,
        )
        .map_err(ErrorKind::DecodeParameterFile)?;

        match params {
            ParamsKind::Frame(params) => {
                if args.has_option("gui") {
                    start_gui(params, param_file_path, output_image_path)?;
                } else {
                    render_frame(params, output_image_path)?;
                }
            }
            ParamsKind::Animation(animation_params) => {
                if args.has_option("gui") {
                    println!("There is no gui implementation for animations. Exiting...");
                } else {
                    render_animation(animation_params, output_image_path)?;
                }
            }
        }
    } else {
        println!("{}", USAGE);
    }

    Ok(())
}

fn start_gui(
    params: FrameParams,
    param_file_path: PathBuf,
    output_image_path: PathBuf,
) -> Result<()> {
    eframe::run_native(
        "fractal renderer",
        eframe::NativeOptions {
            viewport: ViewportBuilder::default()
                .with_inner_size(WINDOW_SIZE)
                .with_min_inner_size(WINDOW_SIZE),
            ..Default::default()
        },
        Box::new(|cc| {
            Ok(Box::new(Gui::new(
                cc,
                params,
                param_file_path,
                output_image_path,
            )))
        }),
    )
    .map_err(|_| ErrorKind::StartGui)
}

fn render_frame(params: FrameParams, output_image_path: PathBuf) -> Result<()> {
    let FrameParams {
        img_width,
        img_height,

        sampling,
        ..
    } = params;

    let sampling_points = sampling.generate_sampling_points();

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
        render_raw_image(&params_clone, &sampling_points_clone, Some(progress_clone))
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

    let sampling_points = sampling.generate_sampling_points();

    let global_start = Instant::now();

    for frame_i in 0..frame_count {
        let t = frame_i as F / fps;

        let params = params.get_frame_params(t);
        let FrameParams {
            img_width,
            img_height,
            ..
        } = params;

        let progress = Progress::new((img_width * img_height) as usize);

        let start = Instant::now();

        let params_clone = params.clone();
        let progress_clone = progress.clone();
        let sampling_points_clone = sampling_points.clone();
        let handle = thread::spawn(move || {
            render_raw_image(&params_clone, &sampling_points_clone, Some(progress_clone))
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
