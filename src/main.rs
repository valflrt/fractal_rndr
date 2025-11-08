mod array2;
mod coloring;
mod complexx;
mod dithering;
mod error;
mod fractal;
mod gui;
mod params;
#[allow(dead_code)]
mod presets;
mod progress;
mod rendering;
mod sampling;

use std::{
    fs,
    io::Write,
    path::{self, PathBuf},
    thread,
    time::{Duration, Instant},
};

use eframe::egui::ViewportBuilder;
use gui::WINDOW_SIZE;

use crate::{
    coloring::{color_mapping, color_raw_image},
    dithering::blue_noise,
    error::{ErrorKind, Result},
    gui::Gui,
    params::{
        animation::{AnimationCfg, AnimationSteps},
        read_parameter_file, DevOptions, Params, ParamsKind,
    },
    progress::Progress,
    rendering::render_raw_image,
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
use wide::f64x8;
#[cfg(not(feature = "force_f32"))]
type FX = f64x8;

const USAGE: &str = "This is a fractal renderer.
Usage: fractal_rndr <param file path> <output image path>
Use --no-gui for cli mode.

More information: https://gitlab.com/valflrt/fractal_rndr";

fn main() -> Result<()> {
    let args = valargs::parse();

    let (param_file_path, output_image_path) = (
        args.nth(1)
            .map(PathBuf::from)
            .map(path::absolute)
            .transpose()
            .map_err(ErrorKind::MakePathAbsolute)?,
        args.nth(2)
            .map(PathBuf::from)
            .map(path::absolute)
            .transpose()
            .map_err(ErrorKind::MakePathAbsolute)?,
    );

    let params = param_file_path
        .as_ref()
        .map(read_parameter_file)
        .transpose()?
        .unwrap_or_default();

    if args.has_option("help") || args.has_option("h") {
        println!("{}", USAGE);
        Ok(())
    } else if args.has_option("no-gui") {
        if let (Some(_), Some(output_image_path)) = (param_file_path, output_image_path) {
            match params {
                ParamsKind::Frame(params) => render_frame(params, output_image_path),
                ParamsKind::Animation(animation_params) => {
                    if animation_params.animation_cfg.is_some() {
                        render_animation(animation_params, output_image_path)
                    } else {
                        Err(ErrorKind::MissingAnimationCfg)
                    }
                }
            }
        } else {
            Err(ErrorKind::MissingCliArg)
        }
    } else {
        start_gui(params, param_file_path, output_image_path)
    }
}

fn start_gui(
    params: ParamsKind,
    param_file_path: Option<PathBuf>,
    output_image_path: Option<PathBuf>,
) -> Result<()> {
    if let ParamsKind::Frame(frame_params) = params {
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
                    frame_params,
                    param_file_path,
                    output_image_path,
                )))
            }),
        )
        .map_err(|_| ErrorKind::StartGui)
    } else {
        Err(ErrorKind::StartGui)
    }
}

fn render_frame(params: Params<F>, output_image_path: PathBuf) -> Result<()> {
    let Params {
        img_width,
        img_height,
        ..
    } = params;

    let progress = Progress::new(img_width * img_height);

    let start = Instant::now();

    let params_clone = params.clone();
    let progress_clone = progress.clone();
    let handle = thread::spawn(move || render_raw_image(&params_clone, Some(progress_clone)));

    while !handle.is_finished() {
        print!(
            "\r {:.1}% - {:.1}s elapsed",
            100. * progress.get_progress(),
            start.elapsed().as_secs_f32(),
        );
        std::io::stdout().flush().unwrap();

        thread::sleep(Duration::from_millis(50));
    }

    let raw_image = handle.join().unwrap();

    println!();

    let output_image = color_raw_image(&params, raw_image);

    output_image
        .save(&output_image_path)
        .map_err(ErrorKind::SaveImage)?;

    let image_size = fs::metadata(&output_image_path).unwrap().len();
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
        if let Some(ext) = output_image_path.extension().and_then(|s| s.to_str()) {
            format!("- {} ", ext)
        } else {
            "".to_string()
        }
    );

    Ok(())
}

fn render_animation(params: Params<AnimationSteps>, output_image_path: PathBuf) -> Result<()> {
    let Params { animation_cfg, .. } = params;
    let AnimationCfg { duration, fps } = animation_cfg.unwrap();

    let frame_count = (duration * fps) as usize;

    println!("frame count: {}", frame_count);
    println!();

    let global_start = Instant::now();

    for frame_i in 0..frame_count {
        let t = frame_i as F / fps;

        let params = params.get_frame_params(t);
        let Params {
            img_width,
            img_height,
            ..
        } = params;

        let progress = Progress::new(img_width * img_height);

        let start = Instant::now();

        let params_clone = params.clone();
        let progress_clone = progress.clone();
        let handle = thread::spawn(move || render_raw_image(&params_clone, Some(progress_clone)));

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

        let mut output_image = color_raw_image(&params, raw_image);

        if let Some(DevOptions {
            display_gradient: Some(true),
            ..
        }) = params.dev_options
        {
            const GRADIENT_HEIGHT: usize = 8;
            const GRADIENT_WIDTH: usize = 64;
            const OFFSET: usize = 8;

            for j in 0..GRADIENT_HEIGHT {
                for i in 0..GRADIENT_WIDTH {
                    output_image.put_pixel(
                        (img_width - GRADIENT_WIDTH - OFFSET + i) as u32,
                        (img_height - GRADIENT_HEIGHT - OFFSET + j) as u32,
                        color_mapping(
                            i as F / GRADIENT_WIDTH as F,
                            &params.gradient,
                            blue_noise(i, j),
                        ),
                    );
                }
            }
        }

        let output_image_path = PathBuf::from(
            output_image_path
                .parent()
                .and_then(|p| p.to_str())
                .unwrap()
                .to_string()
                + "/"
                + output_image_path
                    .file_stem()
                    .and_then(|e| e.to_str())
                    .unwrap()
                + "_"
                + &format!("{:06}", frame_i)
                + "."
                + output_image_path
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap(),
        );

        output_image
            .save(&output_image_path)
            .map_err(ErrorKind::SaveImage)?;

        let image_size = fs::metadata(&output_image_path).unwrap().len();
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
            if let Some(ext) = output_image_path.extension().and_then(|s| s.to_str()) {
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
