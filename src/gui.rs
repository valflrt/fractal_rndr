mod bottom_bar;
mod preview;
mod sections;
mod top_bar;

use std::{
    fs,
    path::{self, PathBuf},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use eframe::{
    egui::{
        CentralPanel, CollapsingHeader, Color32, ColorImage, Context, MenuBar, ScrollArea,
        TextureHandle, TopBottomPanel, Vec2, Vec2b,
    },
    App, CreationContext, Frame as EFrame,
};
use rfd::FileDialog;
use ron::ser::PrettyConfig;
use serde::Serialize;

use crate::{
    array2::Array2,
    coloring::color_raw_image,
    error::ErrorKind,
    params::{read_parameter_file, Params, ParamsKind},
    progress::Progress,
    rendering::render_raw_image,
    sampling::Sampling,
    F,
};

pub const WINDOW_SIZE: Vec2 = Vec2 { x: 1000., y: 540. };
const DEFAULT_ZOOM: F = 5.;

type RenderInfo = Option<(JoinHandle<(Array2<F>, Duration)>, Progress)>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParamsChanges {
    None,
    /// Makes the samples taken until then no longer valid.
    BreakingChanges,
    /// Samples taken until then are still valid.
    NonBreakingChanges,
}

impl ParamsChanges {
    fn set_non_breaking(&mut self) {
        if *self != ParamsChanges::BreakingChanges {
            *self = ParamsChanges::NonBreakingChanges;
        }
    }
    fn set_breaking(&mut self) {
        *self = ParamsChanges::BreakingChanges;
    }
    fn set_none(&mut self) {
        *self = ParamsChanges::None;
    }
}

#[derive(Debug, Clone, Copy)]
enum FileDialogKind {
    SaveFile,
    PickFile,
}

#[derive(Debug, Clone)]
enum FileDialogAction {
    OpenParameterFile(PathBuf),
    SaveParameterFileAs(PathBuf),
    SaveOutputImage(PathBuf),
    LoadGradientFromParameterFile(PathBuf),
    None,
}

pub struct Gui {
    params: Params<F>,
    last_saved_params: Params<F>,

    params_changes: ParamsChanges,

    fractal_param_precision: F,

    param_file_path: Option<PathBuf>,
    output_image_path: Option<PathBuf>,
    file_dialog_handle: Option<JoinHandle<FileDialogAction>>,

    preview_texture: TextureHandle,

    raw_image: Option<Array2<F>>,
    samples_per_pixel: usize,
    should_save_image: bool,

    render_info: RenderInfo,

    message: Option<(String, Instant)>,
}

impl Gui {
    pub const PREVIEW_SIZE: u32 = 256;
    const SLIDER_END_POS: f32 = 350.;

    pub fn new(
        cc: &CreationContext,
        frame_params: Params<F>,
        param_file_path: Option<PathBuf>,
        output_image_path: Option<PathBuf>,
    ) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        Gui {
            last_saved_params: frame_params.clone(),
            params: frame_params,

            params_changes: ParamsChanges::NonBreakingChanges,

            fractal_param_precision: 1e-4,

            param_file_path,
            output_image_path,
            file_dialog_handle: None,

            preview_texture: cc.egui_ctx.load_texture(
                "preview_image",
                ColorImage::filled([0, 0], Color32::TRANSPARENT),
                Default::default(),
            ),

            raw_image: None,
            samples_per_pixel: 0,
            should_save_image: false,

            render_info: None,

            message: None,
        }
    }
}

impl App for Gui {
    fn update(&mut self, ctx: &Context, _frame: &mut EFrame) {
        TopBottomPanel::top("top_bar")
            .show(ctx, |ui| MenuBar::new().ui(ui, |ui| self.show_top_bar(ui)));

        TopBottomPanel::bottom("bottom_bar").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| self.show_bottom_bar(ui));
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.spacing_mut().slider_width = 150.;

            ui.columns_const(|[c1, c2]| {
                // First column

                ScrollArea::vertical()
                    .auto_shrink(Vec2b::new(false, true))
                    .show(c1, |ui| {
                        CollapsingHeader::new("Fractal")
                            .default_open(true)
                            .show(ui, |ui| self.show_section_fractal(ui));

                        CollapsingHeader::new("Controls")
                            .default_open(true)
                            .show(ui, |ui| self.show_section_controls(ui));

                        CollapsingHeader::new("Coloring")
                            .default_open(true)
                            .show(ui, |ui| self.show_section_coloring(ui));

                        CollapsingHeader::new("Render")
                            .default_open(true)
                            .show(ui, |ui| self.show_section_render(ui));

                        ui.add_space(16.);
                    });

                // Second column

                self.show_preview(c2);
            });
        });

        self.handle_update(ctx);
    }
}

impl Gui {
    fn handle_update(&mut self, ctx: &Context) {
        if self.render_info.is_some() {
            // Request repaint for the progress bar to update correctly
            ctx.request_repaint();
        }

        if self.params_changes == ParamsChanges::BreakingChanges {
            // Params relative to fractal and position have
            // changed: stored raw_image is no longer valid.
            self.raw_image = None;
            self.samples_per_pixel = 0;
        }

        if self.params_changes != ParamsChanges::None {
            self.update_preview();
            self.params_changes.set_none();
        }

        if self
            .render_info
            .as_ref()
            .is_some_and(|(h, _)| h.is_finished())
        {
            let (handle, _) = self.render_info.take().unwrap();

            let (new_raw_image, start) = handle.join().unwrap();

            let added_sample_count = self.params.sampling.sample_count();
            if let Some(raw_image) = self.raw_image.as_mut() {
                let w1 = self.samples_per_pixel as F;
                let w2 = added_sample_count as F;
                for (x, y) in raw_image.enumerate() {
                    raw_image[(x, y)] =
                        (w1 * raw_image[(x, y)] + w2 * new_raw_image[(x, y)]) / (w1 + w2);
                }
            } else {
                self.raw_image = Some(new_raw_image);
            }
            self.samples_per_pixel += added_sample_count;

            self.notify(format!("{:.1}s elapsed", start.as_secs_f32()));
        }

        if self.should_save_image {
            self.save_raw_image();

            self.should_save_image = false;
        }

        if self
            .file_dialog_handle
            .as_ref()
            .is_some_and(|h| h.is_finished())
        {
            match self.file_dialog_handle.take().unwrap().join().unwrap() {
                FileDialogAction::OpenParameterFile(path) => match read_parameter_file(&path) {
                    Ok(ParamsKind::Frame(params)) => {
                        self.param_file_path = Some(path);

                        self.last_saved_params = params.clone();
                        self.params = params;

                        self.params_changes.set_breaking();
                        self.notify("loaded new parameter file");
                    }
                    _ => self.notify("failed to load parameter file"),
                },
                FileDialogAction::SaveParameterFileAs(path) => {
                    self.param_file_path = Some(path);
                    self.save_parameter_file();
                }
                FileDialogAction::SaveOutputImage(path) => {
                    self.output_image_path = Some(path);
                    self.save_raw_image();
                }
                FileDialogAction::LoadGradientFromParameterFile(path) => {
                    match read_parameter_file(&path) {
                        Ok(ParamsKind::Frame(params)) => {
                            self.params.gradient = params.gradient;
                            self.notify("new gradient loaded");
                        }
                        _ => self.notify("failed to load gradient"),
                    }
                }
                _ => (),
            }
        }
    }

    fn render_and_save(&mut self) {
        let progress = Progress::new((self.params.img_width * self.params.img_height) as usize);

        let params_clone = self.params.clone();
        let sampling_points_clone = self.params.sampling.generate_sampling_points();
        let progress_clone = progress.clone();
        self.render_info = Some((
            thread::spawn(move || {
                let start = Instant::now();
                let raw_image =
                    render_raw_image(&params_clone, &sampling_points_clone, Some(progress_clone));
                (raw_image, start.elapsed())
            }),
            progress,
        ));
    }

    fn update_preview(&mut self) {
        let (preview_width, preview_height) = if self.params.img_width > self.params.img_height {
            (
                Gui::PREVIEW_SIZE,
                (self.params.img_height * Gui::PREVIEW_SIZE) / self.params.img_width,
            )
        } else {
            (
                (self.params.img_width * Gui::PREVIEW_SIZE) / self.params.img_height,
                Gui::PREVIEW_SIZE,
            )
        };

        let preview_params = Params {
            img_width: preview_width,
            img_height: preview_height,
            sampling: Sampling {
                level: crate::sampling::SamplingLevel::Exploration,
                random_offsets: true,
            },
            ..self.params.clone()
        };

        let sampling_points = preview_params.sampling.generate_sampling_points();

        let raw_image = render_raw_image(&preview_params, &sampling_points, None);

        let output_image = color_raw_image(&preview_params, raw_image);
        let egui_image = ColorImage::from_rgb(
            [output_image.width() as _, output_image.height() as _],
            output_image.as_raw(),
        );
        self.preview_texture.set(egui_image, Default::default());
    }

    fn save_parameter_file(&mut self) {
        if let Some(path) = self.param_file_path.as_ref() {
            match ron::ser::to_string_pretty(
                &ParamsKind::Frame(self.params.clone()),
                PrettyConfig::default(),
            )
            .map_err(ErrorKind::EncodeParameterFile)
            .and_then(|s| fs::write(path, s).map_err(ErrorKind::WriteParameterFile))
            {
                Ok(_) => {
                    self.last_saved_params = self.params.clone();
                    self.notify("parameter file saved");
                }
                Err(_) => {
                    self.notify("failed to save parameter file");
                }
            }
        }
    }

    fn save_raw_image(&mut self) {
        if let Some(output_image_path) = self.output_image_path.as_ref() {
            if let Some(raw_image) = &self.raw_image {
                let output_image = color_raw_image(&self.params, raw_image.to_owned());

                match output_image.save(&output_image_path) {
                    Ok(_) => self.notify("image saved"),
                    Err(_) => self.notify("failed to save image"),
                }
            }
        }
    }

    fn open_file_dialog<F>(&mut self, parent: Option<PathBuf>, kind: FileDialogKind, action: F)
    where
        F: FnOnce(PathBuf) -> FileDialogAction + Send + 'static,
    {
        let mut dialog = FileDialog::new();
        if let Some(parent) = parent {
            dialog = dialog.set_directory(parent);
        }
        self.file_dialog_handle = Some(thread::spawn(move || {
            let res = match kind {
                FileDialogKind::SaveFile => dialog.save_file(),
                FileDialogKind::PickFile => dialog.pick_file(),
            };
            if let Some(path) = res.and_then(|p| path::absolute(p).ok()) {
                action(path)
            } else {
                FileDialogAction::None
            }
        }));
    }

    fn notify<S: ToString>(&mut self, msg: S) {
        self.message = Some((msg.to_string(), Instant::now()));
    }

    fn format_label_ron(value: impl Serialize) -> String {
        ron::to_string(&value)
            .unwrap_or_default()
            .replace(":", ": ")
            .replace(",", ", ")
    }
}
