use std::{
    f64::consts::{PI, TAU},
    fs,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use eframe::{
    egui::{
        self, Color32, ComboBox, DragValue, Grid, Image, ProgressBar, ScrollArea, Slider, Vec2,
    },
    App, CreationContext, Frame as EFrame,
};
use image::codecs::png::PngEncoder;
use ron::ser::PrettyConfig;
use serde::Serialize;
use uni_path::PathBuf;

use crate::{
    coloring::{color_raw_image, ColoringMode, Extremum, MapValue},
    error::{ErrorKind, Result},
    fractal::Fractal,
    mat::Mat2D,
    params::{FrameParams, ParamsKind},
    presets::PRESETS,
    progress::Progress,
    rendering::render_raw_image,
    sampling::{Sampling, SamplingLevel},
    F,
};

pub const WINDOW_SIZE: Vec2 = Vec2 { x: 1000., y: 500. };
const DEFAULT_ZOOM: F = 5.;

type RenderInfo = Option<(JoinHandle<(Mat2D<F>, Duration)>, Progress)>;

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
        if self != &ParamsChanges::BreakingChanges {
            *self = ParamsChanges::NonBreakingChanges;
        }
    }
    fn set_breaking(&mut self) {
        *self = ParamsChanges::BreakingChanges;
    }
    fn set_none(&mut self) {
        *self = ParamsChanges::None;
    }

    fn changed(&self) -> bool {
        self != &ParamsChanges::None
    }
    fn breaking(&self) -> bool {
        self == &ParamsChanges::BreakingChanges
    }
}

pub struct Gui {
    params: FrameParams,
    init_params: FrameParams,

    params_changes: ParamsChanges,

    param_file_path: PathBuf,
    output_image_path: PathBuf,

    preview_bytes: Option<Vec<u8>>,
    preview_size: Option<Vec2>,
    preview_id: u128,

    raw_image: Option<Mat2D<F>>,
    samples_per_pixel: usize,
    should_save_image: bool,

    render_info: RenderInfo,

    message: Option<(String, Instant)>,
}

impl Gui {
    pub const PREVIEW_SIZE: u32 = 256;

    pub fn new(
        cc: &CreationContext,
        params: FrameParams,
        param_file_path: PathBuf,
        output_image_path: PathBuf,
    ) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        Gui {
            init_params: params.clone(),
            params,

            params_changes: ParamsChanges::NonBreakingChanges,

            param_file_path,
            output_image_path,

            preview_bytes: None,
            preview_size: None,
            preview_id: 0,

            raw_image: None,
            samples_per_pixel: 0,
            should_save_image: false,

            render_info: None,

            message: None,
        }
    }
}

impl App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut EFrame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            const SPACE_SIZE: f32 = 8.;
            const SLIDER_END_POS: f32 = 350.;
            ui.spacing_mut().slider_width = 150.;

            ui.add_enabled_ui(self.render_info.is_none(), |ui| {
                ui.columns_const(|[c1, c2]| {
                    // First column

                    c1.heading("Fractal");
                    c1.separator();

                    c1.horizontal(|ui| {
                        ui.label("fractal:");

                        let inner_res = ComboBox::from_id_salt("fractal")
                            .selected_text(Self::format_label_ron(self.params.fractal))
                            .show_ui(ui, |ui| self.show_combobox_fractal(ui));

                        inner_res
                            .response
                            .on_hover_text("select the fractal to render");

                        if inner_res.inner.unwrap_or(false) {
                            // Reset view
                            self.params.center_x = 0.;
                            self.params.center_y = 0.;
                            self.params.zoom = DEFAULT_ZOOM;

                            self.params_changes.set_breaking();
                        }
                    });

                    if self.show_fractal_parameters(c1) {
                        self.params_changes.set_breaking();
                    }

                    c1.horizontal(|ui| {
                        let label_width = ui.label("max_iter:").rect.width();
                        ui.spacing_mut().slider_width = SLIDER_END_POS - label_width;
                        let prev_max_iter = self.params.max_iter;
                        let res = ui.add(
                            Slider::new(&mut self.params.max_iter, 10..=200000).logarithmic(true),
                        );
                        if res.changed() {
                            self.params_changes.set_breaking();

                            // Avoid leaving max slider at a low value when
                            // max_iter is increased.
                            if prev_max_iter < self.params.max_iter {
                                if let ColoringMode::MinMaxNorm {
                                    max: Extremum::Custom(max),
                                    ..
                                } = &mut self.params.coloring_mode
                                {
                                    *max = self.params.max_iter as F;
                                }
                            }
                        }
                    });

                    c1.add_space(SPACE_SIZE);
                    c1.heading("Controls");
                    c1.separator();

                    {
                        const N_DECIMALS: usize = 8;

                        c1.scope(|ui| {
                            ui.horizontal(|ui| {
                                let label_width = ui.label("zoom:").rect.width();
                                ui.spacing_mut().slider_width = SLIDER_END_POS - label_width;
                                let res = ui.add(
                                    Slider::new(&mut self.params.zoom, 0.000000000001..=50.)
                                        .logarithmic(true)
                                        .min_decimals(N_DECIMALS),
                                );
                                if res.changed() {
                                    self.params_changes.set_breaking();
                                }
                            });
                        });

                        let speed = 0.001 * self.params.zoom;

                        let mut changed = false;

                        const FIXED_LABEL_WIDTH: f32 = 20.;

                        c1.horizontal(|ui| {
                            let label_width = ui.label("re:").rect.width();
                            ui.add_space(FIXED_LABEL_WIDTH - label_width);
                            let res = ui.add(
                                DragValue::new(&mut self.params.center_x)
                                    .speed(speed)
                                    .min_decimals(N_DECIMALS),
                            );
                            changed |= res.changed();
                        });
                        c1.horizontal(|ui| {
                            let label_width = ui.label("im:").rect.width();
                            ui.add_space(FIXED_LABEL_WIDTH - label_width);
                            let res = ui.add(
                                DragValue::new(&mut self.params.center_y)
                                    .speed(speed)
                                    .min_decimals(N_DECIMALS),
                            );
                            changed |= res.changed();
                        });

                        c1.horizontal(|ui| {
                            ui.label("rotate:");
                            let mut rotate = self.params.rotate.unwrap_or(0.);
                            let res = ui.add(
                                DragValue::new(&mut rotate)
                                    .speed(0.01)
                                    .range(0. ..=TAU as F)
                                    .custom_parser(|s| {
                                        s.parse::<F>()
                                            .ok()
                                            .map(|degrees| degrees.floor() * PI as F / 180.)
                                    })
                                    .custom_formatter(|rad, _| {
                                        let degrees = rad * 180. / (PI as F);
                                        degrees.floor().to_string()
                                    }),
                            );
                            ui.label("deg");
                            if res.changed() {
                                self.params.rotate = if rotate > 0. { Some(rotate) } else { None };
                            }
                            changed |= res.changed();
                        });

                        if changed {
                            self.params_changes.set_breaking();
                        }
                    }

                    c1.add_space(SPACE_SIZE);
                    c1.heading("Coloring");
                    c1.separator();

                    c1.horizontal(|ui| {
                        ui.label("coloring mode:");

                        ComboBox::from_id_salt("coloring_mode")
                            .selected_text(match self.params.coloring_mode {
                                ColoringMode::MinMaxNorm { .. } => "MinMaxNorm",
                                ColoringMode::CumulativeHistogram { .. } => "CumulativeHistogram",
                            })
                            .show_ui(ui, |ui| {
                                let selected = matches!(
                                    self.params.coloring_mode,
                                    ColoringMode::MinMaxNorm { .. }
                                );
                                if ui.selectable_label(selected, "MinMaxNorm").clicked()
                                    && !selected
                                {
                                    self.params.coloring_mode = ColoringMode::MinMaxNorm {
                                        min: Extremum::Auto,
                                        max: Extremum::Auto,
                                        map: MapValue::Linear,
                                    };
                                    self.params_changes.set_non_breaking();
                                };

                                let selected = matches!(
                                    self.params.coloring_mode,
                                    ColoringMode::CumulativeHistogram { .. }
                                );
                                if ui
                                    .selectable_label(selected, "CumulativeHistogram")
                                    .clicked()
                                    && !selected
                                {
                                    self.params.coloring_mode = ColoringMode::CumulativeHistogram {
                                        map: MapValue::Linear,
                                    };
                                    self.params_changes.set_non_breaking();
                                };
                            });
                    });

                    c1.horizontal(|ui| {
                        ui.label("map value:");

                        let (ColoringMode::CumulativeHistogram { map }
                        | ColoringMode::MinMaxNorm { map, .. }) = &mut self.params.coloring_mode;

                        ComboBox::from_id_salt("map_value")
                            .selected_text(match map {
                                MapValue::Linear => "Linear",
                                MapValue::Squared => "Squared",
                                MapValue::Powf(_) => "Powf",
                            })
                            .show_ui(ui, |ui| {
                                let selected = matches!(map, MapValue::Linear);
                                if ui.selectable_label(selected, "Linear").clicked() && !selected {
                                    *map = MapValue::Linear;
                                    self.params_changes.set_non_breaking();
                                };

                                let selected = matches!(map, MapValue::Squared);
                                if ui.selectable_label(selected, "Squared").clicked() && !selected {
                                    *map = MapValue::Squared;
                                    self.params_changes.set_non_breaking();
                                };

                                let selected = matches!(map, MapValue::Powf(_));
                                if ui.selectable_label(selected, "Powf").clicked() && !selected {
                                    *map = MapValue::Powf(1.);
                                    self.params_changes.set_non_breaking();
                                };
                            });

                        if let MapValue::Powf(exp) = map {
                            let res = ui.add(Slider::new(exp, 0.01..=20.).logarithmic(true));
                            if res.changed() {
                                self.params_changes.set_non_breaking();
                            }
                        }
                    });

                    if let ColoringMode::MinMaxNorm { min, max, .. } =
                        &mut self.params.coloring_mode
                    {
                        const FIXED_LABEL_WIDTH: f32 = 30.;

                        c1.horizontal(|ui| {
                            let label_width = ui.label("min:").rect.width();
                            ui.add_space(FIXED_LABEL_WIDTH - label_width);

                            let mut auto = min.is_auto();
                            let res = ui.checkbox(&mut auto, "auto");
                            if res.changed() {
                                *min = if auto {
                                    Extremum::Auto
                                } else {
                                    Extremum::Custom(0.)
                                };
                                self.params_changes.set_non_breaking();
                            }

                            ui.spacing_mut().slider_width =
                                SLIDER_END_POS - FIXED_LABEL_WIDTH - res.rect.width();

                            if let Extremum::Custom(min) = min {
                                let res = ui.add(
                                    Slider::new(min, 0. ..=self.params.max_iter as F)
                                        .fixed_decimals(0),
                                );
                                if res.changed() {
                                    self.params_changes.set_non_breaking();
                                }
                            }
                        });

                        c1.horizontal(|ui| {
                            let label_width = ui.label("max:").rect.width();
                            ui.add_space(FIXED_LABEL_WIDTH - label_width);

                            let mut auto = max.is_auto();
                            let res = ui.checkbox(&mut auto, "auto");
                            if res.changed() {
                                *max = if auto {
                                    Extremum::Auto
                                } else {
                                    Extremum::Custom(self.params.max_iter as F)
                                };
                                self.params_changes.set_non_breaking();
                            }

                            ui.spacing_mut().slider_width =
                                SLIDER_END_POS - FIXED_LABEL_WIDTH - res.rect.width();

                            if let Extremum::Custom(max) = max {
                                let res = ui.add(
                                    Slider::new(max, 0. ..=self.params.max_iter as F)
                                        .fixed_decimals(0),
                                );
                                if res.changed() {
                                    self.params_changes.set_non_breaking();
                                }
                            }
                        });
                    }

                    c1.add_space(SPACE_SIZE);
                    c1.heading("Parameter file");
                    c1.separator();

                    c1.horizontal(|ui| {
                        if ui.button("revert all edits").clicked() {
                            self.revert_edits();
                            self.params_changes.set_breaking();
                        }
                        if ui.button("save parameter file").clicked() {
                            match self.save_parameter_file() {
                                Ok(_) => self.notify("saved"),
                                Err(_) => self.notify("failed to save parameter file"),
                            }
                        }
                        ui.menu_button("load preset", |ui| {
                            ScrollArea::vertical()
                                .max_width(200.)
                                .max_height(100.)
                                .show(ui, |ui| {
                                    for p in PRESETS {
                                        if let ParamsKind::Frame(params) =
                                            ron::from_str(p.1).unwrap()
                                        {
                                            if ui.button(p.0).clicked() {
                                                self.params = params;
                                                self.params_changes.set_breaking();
                                                self.notify(format!("loaded {}", p.0));
                                                ui.close_menu();
                                            };
                                        }
                                    }
                                })
                        });
                    });

                    // Second column

                    c2.heading("Render");
                    c2.separator();

                    c2.horizontal(|ui| {
                        ui.label("image width:");
                        let res1 = ui.add(
                            DragValue::new(&mut self.params.img_width)
                                .range(32..=20000)
                                .speed(4.),
                        );
                        ui.label("image height:");
                        let res2 = ui.add(
                            DragValue::new(&mut self.params.img_height)
                                .range(32..=20000)
                                .speed(4.),
                        );

                        if res1.changed() || res2.changed() {
                            self.params_changes.set_breaking();
                        }
                    });

                    c2.horizontal(|ui| {
                        ui.label("current spp:")
                            .on_hover_text("number of samples per pixel of the internal image");
                        ui.code(format!(" {} ", self.samples_per_pixel))
                    });

                    c2.horizontal(|ui| {
                        let inner_res = ComboBox::from_id_salt("sampling_level")
                            .selected_text(Self::format_label_ron(self.params.sampling.level))
                            .show_ui(ui, |ui| {
                                self.show_combobox_sampling_level(ui);
                            });
                        inner_res.response.on_hover_text("sampling level");

                        let res = ui
                            .button(format!(
                                "sample fractal (+{} spp)",
                                self.params.sampling.sample_count()
                            ))
                            .on_hover_text("collect new samples");
                        if res.clicked() {
                            self.render_info = Some(self.render_and_save());
                        };

                        ui.add_enabled_ui(self.samples_per_pixel > 0, |ui| {
                            let res = ui.button("save image").on_disabled_hover_text(
                                "sample the fractal before saving the image",
                            );

                            self.should_save_image |= res.clicked();
                        });
                    });

                    c2.add_space(SPACE_SIZE);
                    c2.heading("Preview");
                    c2.separator();

                    if let Some(preview_bytes) = &self.preview_bytes {
                        if let Some(preview_size) = self.preview_size {
                            let d = 0.5 * (Gui::PREVIEW_SIZE as f32 - preview_size.y);
                            c2.add_space(d);
                            c2.add_sized(
                                preview_size,
                                Image::from_bytes(
                                    "bytes://fractal_preview".to_string()
                                        + &self.preview_id.to_string(),
                                    preview_bytes.to_owned(),
                                )
                                .maintain_aspect_ratio(true)
                                .corner_radius(2),
                            );
                            c2.add_space(d);
                        }
                    }
                });
            });

            ui.add_space(SPACE_SIZE);

            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    if let Some((_, progress)) = &self.render_info {
                        ui.add(
                            ProgressBar::new(progress.get_progress())
                                .desired_height(4.)
                                .desired_width(128.)
                                .corner_radius(0.)
                                .fill(Color32::WHITE),
                        );
                    } else if let Some((text, start)) = self.message.as_mut() {
                        const MESSAGE_DISPLAY_TIME: Duration = Duration::from_secs(5);
                        ui.label(text.as_str());
                        if start.elapsed() > MESSAGE_DISPLAY_TIME {
                            self.message = None;
                        }
                    }
                },
            );
        });

        self.handle_update(ctx);
    }
}

impl Gui {
    fn handle_update(&mut self, ctx: &egui::Context) {
        if self.render_info.is_some() {
            ctx.request_repaint();
        }

        if self.params_changes.breaking() {
            // Params relative to fractal and position have
            // changed: stored raw_image is no longer valid.
            self.raw_image = None;
            self.samples_per_pixel = 0;
        }

        if self.params_changes.changed() {
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
            if let Some(raw_image) = &self.raw_image {
                let output_image = color_raw_image(
                    &self.params,
                    self.params.coloring_mode,
                    self.params.custom_gradient.as_ref(),
                    raw_image.to_owned(),
                );

                match output_image.save(self.output_image_path.as_str()) {
                    Ok(_) => self.notify("image saved"),
                    Err(_) => self.notify("failed to save image"),
                }
            }

            self.should_save_image = false;
        }
    }

    fn render_and_save(&mut self) -> (JoinHandle<(Mat2D<F>, Duration)>, Progress) {
        let progress = Progress::new((self.params.img_width * self.params.img_height) as usize);

        let params_clone = self.params.clone();
        let sampling_points_clone = self.params.sampling.generate_sampling_points();
        let progress_clone = progress.clone();
        (
            thread::spawn(move || {
                let start = Instant::now();
                let raw_image =
                    render_raw_image(&params_clone, &sampling_points_clone, Some(progress_clone));
                (raw_image, start.elapsed())
            }),
            progress,
        )
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

        self.preview_size = Some(Vec2::new(preview_width as f32, preview_height as f32));

        let preview_params = FrameParams {
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

        let output_image = color_raw_image(
            &preview_params,
            preview_params.coloring_mode,
            preview_params.custom_gradient.as_ref(),
            raw_image,
        );

        let mut buf = Vec::new();
        output_image
            .write_with_encoder(PngEncoder::new(&mut buf))
            .unwrap();

        self.preview_id += 1;
        self.preview_bytes = Some(buf);
    }

    fn save_parameter_file(&mut self) -> Result<()> {
        self.init_params = self.params.clone();
        fs::write(
            self.param_file_path.as_str(),
            ron::ser::to_string_pretty(
                &ParamsKind::Frame(self.params.clone()),
                PrettyConfig::default(),
            )
            .map_err(ErrorKind::EncodeParameterFile)?,
        )
        .map_err(ErrorKind::WriteParameterFile)
    }

    fn revert_edits(&mut self) {
        self.params = self.init_params.clone();
    }

    fn notify<S: ToString>(&mut self, msg: S) {
        self.message = Some((msg.to_string(), Instant::now()));
    }

    // Gui display related stuff

    fn format_label_ron(value: impl Serialize) -> String {
        ron::to_string(&value)
            .unwrap_or_default()
            .replace(":", ": ")
            .replace(",", ", ")
    }

    fn show_combobox_fractal(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;

        let selected = matches!(self.params.fractal, Fractal::Mandelbrot);
        if ui.selectable_label(selected, "Mandelbrot").clicked() && !selected {
            self.params.fractal = Fractal::Mandelbrot;
            changed = true;
        };

        let selected = matches!(self.params.fractal, Fractal::MandelbrotCustomExp { .. });
        if ui
            .selectable_label(selected, "MandelbrotCustomExp(exp)")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::MandelbrotCustomExp { exp: 2. };
            changed = true;
        };

        let selected = matches!(self.params.fractal, Fractal::Sdrge);
        if ui
            .selectable_label(selected, "Sdrge")
            .on_hover_text("second degree recursive sequence with growing exponent")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::Sdrge;
            changed = true;
        };

        let selected = matches!(self.params.fractal, Fractal::SdrgeCustomExp { .. });
        if ui
            .selectable_label(selected, "SdrgeCustomExp(exp)")
            .on_hover_text("second degree recursive sequence with growing custom exponent")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::SdrgeCustomExp { exp: 2. };
            changed = true;
        };

        let selected = matches!(self.params.fractal, Fractal::SdrgeParam { .. });
        if ui
            .selectable_label(selected, "SdrgeParam(a_re, a_im)")
            .on_hover_text("parameterized second degree recursive sequence with growing exponent")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::SdrgeParam { a_re: 1., a_im: 0. };
            changed = true;
        };

        let selected = matches!(self.params.fractal, Fractal::Sdrage);
        if ui
            .selectable_label(selected, "Sdrage")
            .on_hover_text("second degree recursive alternating sequence with growing exponent")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::Sdrage;
            changed = true;
        };

        let selected = matches!(self.params.fractal, Fractal::Tdrge);
        if ui
            .selectable_label(selected, "Tdrge")
            .on_hover_text("third degree recursive sequence with growing exponent")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::Tdrge;
            changed = true;
        };

        let selected = matches!(self.params.fractal, Fractal::NthDrge(_));
        if ui
            .selectable_label(selected, "NthDrge(n)")
            .on_hover_text("nth degree recursive sequence with growing exponent")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::NthDrge(4);
            changed = true;
        };

        let selected = matches!(self.params.fractal, Fractal::ThirdDegreeRecPairs);
        if ui
            .selectable_label(selected, "ThirdDegreeRecPairs")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::ThirdDegreeRecPairs;
            changed = true;
        };

        let selected = matches!(self.params.fractal, Fractal::SecondDegreeThirtySevenBlend);
        if ui
            .selectable_label(selected, "SecondDegreeThirtySevenBlend")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::SecondDegreeThirtySevenBlend;
            changed = true;
        };

        let selected = matches!(self.params.fractal, Fractal::ComplexLogisticMapLike { .. });
        if ui
            .selectable_label(selected, "ComplexLogisticMapLike(a_re, a_im)")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::ComplexLogisticMapLike { a_re: 1., a_im: 0. };
            changed = true;
        };

        let selected = matches!(self.params.fractal, Fractal::Vshqwj);
        if ui.selectable_label(selected, "Vshqwj").clicked() && !selected {
            self.params.fractal = Fractal::Vshqwj;
            changed = true;
        };

        let selected = matches!(self.params.fractal, Fractal::Wmriho { .. });
        if ui
            .selectable_label(selected, "Wmriho(a_re, a_im)")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::Wmriho { a_re: 0., a_im: 0. };
            changed = true;
        };

        let selected = matches!(self.params.fractal, Fractal::Iigdzh { .. });
        if ui
            .selectable_label(selected, "Iigdzh(a_re, a_im)")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::Iigdzh { a_re: 0., a_im: 0. };
            changed = true;
        };

        let selected = matches!(self.params.fractal, Fractal::Fxdicq);
        if ui.selectable_label(selected, "Fxdicq").clicked() && !selected {
            self.params.fractal = Fractal::Fxdicq;
            changed = true;
        };

        let selected = matches!(self.params.fractal, Fractal::Mjygzr);
        if ui.selectable_label(selected, "Mjygzr").clicked() && !selected {
            self.params.fractal = Fractal::Mjygzr;
            changed = true;
        };

        let selected = matches!(self.params.fractal, Fractal::Sfwypc { .. });
        if ui
            .selectable_label(selected, "Sfwypc(alpha, beta, gamma)")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::Sfwypc {
                alpha: (0., 0.),
                beta: (0., 0.),
                gamma: (0., 0.),
            };
            changed = true;
        };

        changed
    }

    fn show_fractal_parameters(&mut self, ui: &mut egui::Ui) -> bool {
        const SPEED: f64 = 0.0001;
        const N_DECIMALS: usize = 8;

        let mut changed = false;

        if let Fractal::MandelbrotCustomExp { exp } = &mut self.params.fractal {
            ui.horizontal(|ui| {
                ui.label("exp:");
                let res = ui.add(
                    DragValue::new(exp)
                        .speed(SPEED)
                        .range(0.001..=20.)
                        .fixed_decimals(N_DECIMALS),
                );
                changed |= res.changed();
            });
        }

        if let Fractal::SdrgeCustomExp { exp } = &mut self.params.fractal {
            ui.horizontal(|ui| {
                ui.label("exp:");
                let res = ui.add(
                    DragValue::new(exp)
                        .speed(SPEED)
                        .range(1..=10)
                        .fixed_decimals(N_DECIMALS),
                );
                changed |= res.changed();
            });
        }

        if let Fractal::SdrgeParam { a_re, a_im }
        | Fractal::ComplexLogisticMapLike { a_re, a_im }
        | Fractal::Wmriho { a_re, a_im }
        | Fractal::Iigdzh { a_re, a_im } = &mut self.params.fractal
        {
            ui.horizontal(|ui| {
                ui.label("a_re:");
                let res1 = ui.add(DragValue::new(a_re).speed(SPEED).fixed_decimals(N_DECIMALS));
                ui.label("a_im:");
                let res2 = ui.add(DragValue::new(a_im).speed(SPEED).fixed_decimals(N_DECIMALS));

                changed |= res1.changed() || res2.changed();
            });
        }

        if let Fractal::NthDrge(n) = &mut self.params.fractal {
            ui.horizontal(|ui| {
                ui.label("n:");
                let res = ui.add(Slider::new(n, 2..=20));
                changed |= res.changed();
            });
        }

        if let Fractal::Sfwypc { alpha, beta, gamma } = &mut self.params.fractal {
            Grid::new("param grid").show(ui, |ui| {
                [(alpha, "alpha"), (beta, "beta"), (gamma, "gamma")]
                    .iter_mut()
                    .for_each(|(v, name)| {
                        ui.label(name.to_string() + "_re:");
                        changed |= ui
                            .add(
                                DragValue::new(&mut v.0)
                                    .speed(SPEED)
                                    .fixed_decimals(N_DECIMALS),
                            )
                            .changed();
                        ui.label(name.to_string() + "_im:");
                        changed |= ui
                            .add(
                                DragValue::new(&mut v.1)
                                    .speed(SPEED)
                                    .fixed_decimals(N_DECIMALS),
                            )
                            .changed();
                        ui.end_row();
                    });
            });
        }

        changed
    }

    fn show_combobox_sampling_level(&mut self, ui: &mut egui::Ui) {
        ui.selectable_value(
            &mut self.params.sampling.level,
            SamplingLevel::Exploration,
            "Exploration",
        );
        ui.selectable_value(&mut self.params.sampling.level, SamplingLevel::Low, "Low");
        ui.selectable_value(
            &mut self.params.sampling.level,
            SamplingLevel::Medium,
            "Medium",
        );
        ui.selectable_value(&mut self.params.sampling.level, SamplingLevel::High, "High");
        ui.selectable_value(
            &mut self.params.sampling.level,
            SamplingLevel::Ultra,
            "Ultra",
        );
        ui.selectable_value(
            &mut self.params.sampling.level,
            SamplingLevel::Extreme,
            "Extreme",
        );
        ui.selectable_value(
            &mut self.params.sampling.level,
            SamplingLevel::Extreme1,
            "Extreme1",
        );
        ui.selectable_value(
            &mut self.params.sampling.level,
            SamplingLevel::Extreme2,
            "Extreme2",
        );
        ui.selectable_value(
            &mut self.params.sampling.level,
            SamplingLevel::Extreme3,
            "Extreme3",
        );
    }
}
