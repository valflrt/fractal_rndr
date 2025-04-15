use std::{
    f64::consts::TAU,
    fs,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use eframe::{
    egui::{self, Color32, ComboBox, DragValue, Image, ProgressBar, ScrollArea, Slider, Vec2},
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
    params::{FrameParams, ParamsKind},
    presets::PRESETS,
    progress::Progress,
    rendering::render_raw_image,
    sampling::{generate_sampling_points, Sampling, SamplingLevel},
    View, F,
};

const DEFAULT_ZOOM: F = 5.;

pub struct Gui {
    params: FrameParams,
    init_params: FrameParams,
    view: View,

    output_image_path: PathBuf,
    param_file_path: PathBuf,

    preview_bytes: Option<Vec<u8>>,
    preview_size: Option<Vec2>,
    preview_id: u128,
    should_update_preview: bool,

    render_info: Option<(JoinHandle<Result<()>>, Progress, Instant)>,
    message: Option<(String, Instant)>,
}

impl Gui {
    pub const PREVIEW_SIZE: u32 = 256;

    pub fn new(
        cc: &CreationContext,
        params: FrameParams,
        view: View,
        output_image_path: PathBuf,
        param_file_path: PathBuf,
    ) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        Gui {
            init_params: params.clone(),
            params,
            view,

            output_image_path,
            param_file_path,

            preview_bytes: None,
            preview_size: None,
            preview_id: 0,
            should_update_preview: true,

            render_info: None,
            message: None,
        }
    }
}

impl App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut EFrame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            const SPACE_SIZE: f32 = 8.;
            ui.spacing_mut().slider_width = 150.;

            ui.add_enabled_ui(self.render_info.is_none(), |ui| {
                ui.columns_const(|[c1, c2]| {
                    // First column

                    c1.heading("Fractal");
                    c1.separator();

                    c1.horizontal(|ui| {
                        ui.label("fractal:");

                        let res = ComboBox::from_id_salt("fractal")
                            .selected_text(Self::format_label_ron(&self.params.fractal))
                            .show_ui(ui, |ui| self.combobox_fractal_selection(ui));

                        if res.inner.unwrap_or(false) {
                            // Reset view
                            self.params.center_x = 0.;
                            self.params.center_y = 0.;
                            self.params.zoom = DEFAULT_ZOOM;

                            self.should_update_preview = true;
                        }
                    });

                    self.fractal_parameters(c1);

                    c1.horizontal(|ui| {
                        ui.label("max_iter:");
                        let res = ui.add(
                            Slider::new(&mut self.params.max_iter, 10..=200000).logarithmic(true),
                        );
                        if res.changed() {
                            self.should_update_preview = true;
                        }
                    });

                    c1.add_space(SPACE_SIZE);
                    c1.heading("Controls");
                    c1.separator();

                    c1.scope(|ui| {
                        ui.spacing_mut().slider_width = 300.;
                        ui.horizontal(|ui| {
                            ui.label("zoom:");
                            let res = ui.add(
                                Slider::new(&mut self.params.zoom, 0.000000000001..=50.)
                                    .logarithmic(true),
                            );
                            if res.changed() {
                                self.should_update_preview = true;
                            }
                        });
                    });

                    {
                        let speed = 0.005 * self.params.zoom;
                        c1.horizontal(|ui| {
                            ui.label("re:");
                            let res =
                                ui.add(DragValue::new(&mut self.params.center_x).speed(speed));
                            if res.changed() {
                                self.should_update_preview = true;
                            }
                        });
                        c1.horizontal(|ui| {
                            ui.label("im:");
                            let res =
                                ui.add(DragValue::new(&mut self.params.center_y).speed(speed));
                            if res.changed() {
                                self.should_update_preview = true;
                            }
                        });

                        let rotate = self.view.rotate;
                        c1.horizontal(|ui| {
                            ui.label("rotate:");
                            let mut rotate = rotate;
                            let res = ui.add(
                                DragValue::new(&mut rotate)
                                    .speed(0.01)
                                    .range(0. ..=TAU as F),
                            );
                            if res.changed() {
                                self.params.rotate = if rotate > 0. { Some(rotate) } else { None };
                                self.should_update_preview = true;
                            }
                        });
                    }

                    c1.add_space(SPACE_SIZE);
                    c1.heading("Coloring");
                    c1.separator();

                    c1.horizontal(|ui| {
                        ui.label("coloring mode:");

                        ComboBox::from_id_salt("coloring_mode")
                            .selected_text(match self.params.coloring_mode {
                                ColoringMode::CumulativeHistogram { .. } => "CumulativeHistogram",
                                ColoringMode::MinMaxNorm { .. } => "MinMaxNorm",
                            })
                            .show_ui(ui, |ui| {
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
                                    self.should_update_preview = true;
                                };

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
                                    self.should_update_preview = true;
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
                                    self.should_update_preview = true;
                                };

                                let selected = matches!(map, MapValue::Squared);
                                if ui.selectable_label(selected, "Squared").clicked() && !selected {
                                    *map = MapValue::Squared;
                                    self.should_update_preview = true;
                                };

                                let selected = matches!(map, MapValue::Powf(_));
                                if ui.selectable_label(selected, "Powf").clicked() && !selected {
                                    *map = MapValue::Powf(1.);
                                    self.should_update_preview = true;
                                };
                            });

                        if let MapValue::Powf(exp) = map {
                            let res = ui.add(Slider::new(exp, 0.01..=20.).logarithmic(true));
                            if res.changed() {
                                self.should_update_preview = true;
                            }
                        }
                    });

                    if let ColoringMode::MinMaxNorm { min, max, .. } =
                        &mut self.params.coloring_mode
                    {
                        c1.horizontal(|ui| {
                            ui.label("min:");

                            let mut auto = min.is_auto();
                            let res = ui.checkbox(&mut auto, "auto");
                            if res.changed() {
                                *min = if auto {
                                    Extremum::Auto
                                } else {
                                    Extremum::Custom(0.)
                                };
                                self.should_update_preview = true;
                            }

                            if let Extremum::Custom(min) = min {
                                let res = ui.add(Slider::new(min, 0. ..=self.params.max_iter as F));
                                if res.changed() {
                                    self.should_update_preview = true;
                                }
                            }
                        });
                        c1.horizontal(|ui| {
                            ui.label("max:");

                            let mut auto = max.is_auto();
                            let res = ui.checkbox(&mut auto, "auto");
                            if res.changed() {
                                *max = if auto {
                                    Extremum::Auto
                                } else {
                                    Extremum::Custom(self.params.max_iter as F)
                                };
                                self.should_update_preview = true;
                            }

                            if let Extremum::Custom(max) = max {
                                let res = ui.add(Slider::new(max, 0. ..=self.params.max_iter as F));
                                if res.changed() {
                                    self.should_update_preview = true;
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
                            self.update_preview();
                        }
                        if ui.button("save parameter file").clicked() {
                            let msg = if self.save_parameter_file().is_ok() {
                                "saved"
                            } else {
                                "failed to save parameter file"
                            };
                            self.notify(msg);
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
                                                self.should_update_preview = true;
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
                            self.should_update_preview = true;
                        }
                    });

                    c2.horizontal(|ui| {
                        ui.label("sampling level:");

                        ComboBox::from_id_salt("sampling_level")
                            .selected_text(Self::format_label_ron(&self.params.sampling.level))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.params.sampling.level,
                                    SamplingLevel::Exploration,
                                    "Exploration",
                                );
                                ui.selectable_value(
                                    &mut self.params.sampling.level,
                                    SamplingLevel::Low,
                                    "Low",
                                );
                                ui.selectable_value(
                                    &mut self.params.sampling.level,
                                    SamplingLevel::Medium,
                                    "Medium",
                                );
                                ui.selectable_value(
                                    &mut self.params.sampling.level,
                                    SamplingLevel::High,
                                    "High",
                                );
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
                            });
                    });

                    c2.horizontal(|ui| {
                        let res = ui.button("render and save image");
                        if res.clicked() {
                            let (progress, handle) = self.render_and_save();
                            self.render_info = Some((handle, progress, Instant::now()));
                        };
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
                    if let Some((handle, progress, start)) = &self.render_info {
                        let progress_bar = ProgressBar::new(progress.get_progress())
                            .desired_height(4.)
                            .desired_width(128.)
                            .corner_radius(0.)
                            .fill(Color32::WHITE);
                        ui.add(progress_bar);
                        ctx.request_repaint();

                        if handle.is_finished() {
                            self.notify(format!("{:.1}s elapsed", start.elapsed().as_secs_f32()));
                            self.render_info = None;
                        }
                    } else if let Some((text, start)) = &self.message {
                        const MESSAGE_DISPLAY_TIME: Duration = Duration::from_secs(5);
                        ui.label(text);
                        if start.elapsed() > MESSAGE_DISPLAY_TIME {
                            self.message = None;
                        }
                    }
                },
            );
        });

        if self.should_update_preview {
            self.update_view();
            self.update_preview();

            self.should_update_preview = false;
        }
    }
}

impl Gui {
    fn notify<S: ToString>(&mut self, msg: S) {
        self.message = Some((msg.to_string(), Instant::now()));
    }

    fn revert_edits(&mut self) {
        self.params = self.init_params.clone();
        self.update_view();
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

    fn render_and_save(&mut self) -> (Progress, JoinHandle<Result<()>>) {
        let progress = Progress::new((self.params.img_width * self.params.img_height) as usize);

        let params_clone = self.params.clone();
        let view = self.view;
        let sampling_points_clone = generate_sampling_points(self.params.sampling.level);
        let output_image_path_clone = self.output_image_path.clone();
        (
            progress.clone(),
            thread::spawn(move || {
                let raw_image =
                    render_raw_image(&params_clone, &view, &sampling_points_clone, Some(progress));

                let output_image = color_raw_image(
                    &params_clone,
                    params_clone.coloring_mode,
                    params_clone.custom_gradient.as_ref(),
                    raw_image,
                );

                output_image
                    .save(output_image_path_clone.as_str())
                    .map_err(ErrorKind::SaveImage)
            }),
        )
    }

    fn update_view(&mut self) {
        let FrameParams {
            img_width,
            img_height,
            zoom,
            rotate,
            center_x,
            center_y,
            ..
        } = self.params;

        self.view = View::new(img_width, img_height, zoom, center_x, center_y, rotate);
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

        let sampling_points = generate_sampling_points(preview_params.sampling.level);

        let raw_image = render_raw_image(&preview_params, &self.view, &sampling_points, None);

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

    // Gui display related stuff

    fn format_label_ron(value: impl Serialize) -> String {
        ron::to_string(&value)
            .unwrap_or_default()
            .replace(":", ": ")
            .replace(",", ", ")
    }

    fn combobox_fractal_selection(&mut self, ui: &mut egui::Ui) -> bool {
        let mut should_reset_view = false;

        let selected = matches!(self.params.fractal, Fractal::Mandelbrot);
        if ui.selectable_label(selected, "Mandelbrot").clicked() && !selected {
            self.params.fractal = Fractal::Mandelbrot;
            should_reset_view = true;
        };

        let selected = matches!(self.params.fractal, Fractal::MandelbrotCustomExp { .. });
        if ui
            .selectable_label(selected, "MandelbrotCustomExp(exp)")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::MandelbrotCustomExp { exp: 2. };
            should_reset_view = true;
        };

        let selected = matches!(self.params.fractal, Fractal::SDRGE);
        if ui
            .selectable_label(selected, "SDRGE")
            .on_hover_text("second degree recursive sequence with growing exponent")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::SDRGE;
            should_reset_view = true;
        };

        let selected = matches!(self.params.fractal, Fractal::SDRGECustomExp { .. });
        if ui
            .selectable_label(selected, "SDRGECustomExp(exp)")
            .on_hover_text("second degree recursive sequence with growing custom exponent")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::SDRGECustomExp { exp: 2. };
            should_reset_view = true;
        };

        let selected = matches!(self.params.fractal, Fractal::SDRGEParam { .. });
        if ui
            .selectable_label(selected, "SDRGEParam(a_re, a_im)")
            .on_hover_text("parameterized second degree recursive sequence with growing exponent")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::SDRGEParam { a_re: 1., a_im: 0. };
            should_reset_view = true;
        };

        let selected = matches!(self.params.fractal, Fractal::SDRAGE);
        if ui
            .selectable_label(selected, "SDRAGE")
            .on_hover_text("second degree recursive alternating sequence with growing exponent")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::SDRAGE;
            should_reset_view = true;
        };

        let selected = matches!(self.params.fractal, Fractal::TDRGE);
        if ui
            .selectable_label(selected, "TDRGE")
            .on_hover_text("third degree recursive sequence with growing exponent")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::TDRGE;
            should_reset_view = true;
        };

        let selected = matches!(self.params.fractal, Fractal::NthDRGE(_));
        if ui
            .selectable_label(selected, "NthDRGE(n)")
            .on_hover_text("nth degree recursive sequence with growing exponent")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::NthDRGE(4);
            should_reset_view = true;
        };

        let selected = matches!(self.params.fractal, Fractal::ThirdDegreeRecPairs);
        if ui
            .selectable_label(selected, "ThirdDegreeRecPairs")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::ThirdDegreeRecPairs;
            should_reset_view = true;
        };

        let selected = matches!(self.params.fractal, Fractal::SecondDegreeThirtySevenBlend);
        if ui
            .selectable_label(selected, "SecondDegreeThirtySevenBlend")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::SecondDegreeThirtySevenBlend;
            should_reset_view = true;
        };

        let selected = matches!(self.params.fractal, Fractal::ComplexLogisticMapLike { .. });
        if ui
            .selectable_label(selected, "ComplexLogisticMapLike(a_re, a_im)")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::ComplexLogisticMapLike { a_re: 1., a_im: 0. };
            should_reset_view = true;
        };

        let selected = matches!(self.params.fractal, Fractal::Vshqwj);
        if ui.selectable_label(selected, "Vshqwj").clicked() && !selected {
            self.params.fractal = Fractal::Vshqwj;
            should_reset_view = true;
        };

        let selected = matches!(self.params.fractal, Fractal::Wmriho { .. });
        if ui
            .selectable_label(selected, "Wmriho(a_re, a_im)")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::Wmriho { a_re: 0., a_im: 0. };
            should_reset_view = true;
        };

        let selected = matches!(self.params.fractal, Fractal::Iigdzh { .. });
        if ui
            .selectable_label(selected, "Iigdzh(a_re, a_im)")
            .clicked()
            && !selected
        {
            self.params.fractal = Fractal::Iigdzh { a_re: 0., a_im: 0. };
            should_reset_view = true;
        };

        let selected = matches!(self.params.fractal, Fractal::Fxdicq);
        if ui.selectable_label(selected, "Fxdicq").clicked() && !selected {
            self.params.fractal = Fractal::Fxdicq;
            should_reset_view = true;
        };

        let selected = matches!(self.params.fractal, Fractal::Mjygzr);
        if ui.selectable_label(selected, "Mjygzr").clicked() && !selected {
            self.params.fractal = Fractal::Mjygzr;
            should_reset_view = true;
        };

        should_reset_view
    }

    fn fractal_parameters(&mut self, ui: &mut egui::Ui) {
        const SPEED: f64 = 0.0001;

        if let Fractal::MandelbrotCustomExp { exp } = &mut self.params.fractal {
            ui.horizontal(|ui| {
                ui.label("exp:");
                let res = ui.add(DragValue::new(exp).speed(SPEED).range(0.001..=20.));
                if res.changed() {
                    self.should_update_preview = true;
                }
            });
        }

        if let Fractal::SDRGECustomExp { exp } = &mut self.params.fractal {
            ui.horizontal(|ui| {
                ui.label("exp:");
                let res = ui.add(DragValue::new(exp).speed(SPEED).range(1..=10));
                if res.changed() {
                    self.should_update_preview = true;
                }
            });
        }

        if let Fractal::SDRGEParam { a_re, a_im }
        | Fractal::ComplexLogisticMapLike { a_re, a_im }
        | Fractal::Wmriho { a_re, a_im }
        | Fractal::Iigdzh { a_re, a_im } = &mut self.params.fractal
        {
            ui.horizontal(|ui| {
                ui.label("a_re:");
                let res1 = ui.add(DragValue::new(a_re).speed(SPEED));
                ui.label("a_im:");
                let res2 = ui.add(DragValue::new(a_im).speed(SPEED));

                if res1.changed() || res2.changed() {
                    self.should_update_preview = true;
                }
            });
        }

        if let Fractal::NthDRGE(n) = &mut self.params.fractal {
            ui.horizontal(|ui| {
                ui.label("n:");
                let res = ui.add(Slider::new(n, 2..=20));
                if res.changed() {
                    self.should_update_preview = true;
                }
            });
        }
    }
}
