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
use image::{codecs::png::PngEncoder, RgbImage};
use ron::ser::PrettyConfig;
use uni_path::PathBuf;

use crate::{
    coloring::{color_raw_image, ColoringMode, Extremum, MapValue},
    error::{ErrorKind, Result},
    fractal::Fractal,
    params::{FrameParams, ParamsKind},
    presets::PRESETS,
    progress::Progress,
    rendering::render_raw_image,
    sampling::generate_sampling_points,
    View, F, SAMPLE_MUL,
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
    preview_render_handle: Option<JoinHandle<RgbImage>>,
    should_render_preview: bool,

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
            preview_render_handle: None,
            should_render_preview: true,

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

                        let mut selected_fractal_i = match self.params.fractal {
                            Fractal::Mandelbrot => 0,
                            Fractal::MandelbrotCustomExp { .. } => 1,
                            Fractal::SecondDegreeRecWithGrowingExponent => 2,
                            Fractal::SecondDegreeRecWithGrowingExponentParam { .. } => 3,
                            Fractal::SecondDegreeRecAlternating1WithGrowingExponent => 4,
                            Fractal::ThirdDegreeRecWithGrowingExponent => 5,
                            Fractal::NthDegreeRecWithGrowingExponent(_) => 6,
                            Fractal::ThirdDegreeRecPairs => 7,
                            Fractal::SecondDegreeThirtySevenBlend => 8,
                            Fractal::ComplexLogisticMapLike { .. } => 9,
                            Fractal::Vshqwj => 10,
                            Fractal::Wmriho { .. } => 11,
                            Fractal::Iigdzh { .. } => 12,
                            Fractal::Fxdicq => 13,
                            Fractal::Mjygzr => 14,
                            Fractal::Zqcqvm => 15,
                        };
                        const MODES: &[&str] = &[
                            "Mandelbrot",
                            "MandelbrotCustomExp(exp)",
                            "SecondDegreeRecWithGrowingExponent",
                            "SecondDegreeRecWithGrowingExponentParam(a_re, a_im)",
                            "SecondDegreeRecAlternating1WithGrowingExponent",
                            "ThirdDegreeRecWithGrowingExponent",
                            "NthDegreeRecWithGrowingExponent(n)",
                            "ThirdDegreeRecPairs",
                            "SecondDegreeThirtySevenBlend",
                            "ComplexLogisticMapLike(a_re, a_im)",
                            "Vshqwj",
                            "Wmriho(a_re, a_im)",
                            "Iigdzh(a_re, a_im)",
                            "Fxdicq",
                            "Mjygzr",
                            "Zqcqvm",
                        ];
                        let res = ComboBox::from_id_salt("fractal").show_index(
                            ui,
                            &mut selected_fractal_i,
                            MODES.len(),
                            |i| MODES[i],
                        );

                        if res.changed() {
                            self.params.fractal = match selected_fractal_i {
                                0 => Fractal::Mandelbrot,
                                1 => Fractal::MandelbrotCustomExp {
                                    exp: if let Fractal::MandelbrotCustomExp { exp } =
                                        self.init_params.fractal
                                    {
                                        exp
                                    } else {
                                        2.
                                    },
                                },
                                2 => Fractal::SecondDegreeRecWithGrowingExponent,
                                3 => {
                                    let (a_re, a_im) =
                                        if let Fractal::SecondDegreeRecWithGrowingExponentParam {
                                            a_re,
                                            a_im,
                                        } = self.init_params.fractal
                                        {
                                            (a_re, a_im)
                                        } else {
                                            (1., 0.)
                                        };
                                    Fractal::SecondDegreeRecWithGrowingExponentParam { a_re, a_im }
                                }
                                4 => Fractal::SecondDegreeRecAlternating1WithGrowingExponent,
                                5 => Fractal::ThirdDegreeRecWithGrowingExponent,
                                6 => Fractal::NthDegreeRecWithGrowingExponent(
                                    if let Fractal::NthDegreeRecWithGrowingExponent(n) =
                                        self.init_params.fractal
                                    {
                                        n
                                    } else {
                                        4
                                    },
                                ),
                                7 => Fractal::ThirdDegreeRecPairs,
                                8 => Fractal::SecondDegreeThirtySevenBlend,
                                9 => {
                                    let (a_re, a_im) =
                                        if let Fractal::ComplexLogisticMapLike { a_re, a_im } =
                                            self.init_params.fractal
                                        {
                                            (a_re, a_im)
                                        } else {
                                            (1., 0.)
                                        };
                                    Fractal::ComplexLogisticMapLike { a_re, a_im }
                                }
                                10 => Fractal::Vshqwj,
                                11 => {
                                    let (a_re, a_im) = if let Fractal::Wmriho { a_re, a_im } =
                                        self.init_params.fractal
                                    {
                                        (a_re, a_im)
                                    } else {
                                        (0., 0.)
                                    };
                                    Fractal::Wmriho { a_re, a_im }
                                }
                                12 => {
                                    let (a_re, a_im) = if let Fractal::Iigdzh { a_re, a_im } =
                                        self.init_params.fractal
                                    {
                                        (a_re, a_im)
                                    } else {
                                        (0., 0.)
                                    };
                                    Fractal::Iigdzh { a_re, a_im }
                                }
                                13 => Fractal::Fxdicq,
                                14 => Fractal::Mjygzr,
                                15 => Fractal::Zqcqvm,
                                _ => unreachable!(),
                            };

                            // Reset view
                            self.params.center_x = 0.;
                            self.params.center_y = 0.;
                            self.params.zoom = DEFAULT_ZOOM;

                            self.should_render_preview = true;
                        }
                    });

                    {
                        const SPEED: f64 = 0.0001;

                        if let Fractal::MandelbrotCustomExp { exp } = &mut self.params.fractal {
                            c1.horizontal(|ui| {
                                ui.label("exp:");
                                let res =
                                    ui.add(DragValue::new(exp).speed(SPEED).range(0.001..=20.));
                                if res.changed() {
                                    self.should_render_preview = true;
                                }
                            });
                        }

                        if let Fractal::SecondDegreeRecWithGrowingExponentParam { a_re, a_im }
                        | Fractal::ComplexLogisticMapLike { a_re, a_im }
                        | Fractal::Wmriho { a_re, a_im }
                        | Fractal::Iigdzh { a_re, a_im } = &mut self.params.fractal
                        {
                            c1.horizontal(|ui| {
                                ui.label("a_re:");
                                let res1 = ui.add(DragValue::new(a_re).speed(SPEED));
                                ui.label("a_im:");
                                let res2 = ui.add(DragValue::new(a_im).speed(SPEED));

                                if res1.changed() || res2.changed() {
                                    self.should_render_preview = true;
                                }
                            });
                        }

                        if let Fractal::NthDegreeRecWithGrowingExponent(n) =
                            &mut self.params.fractal
                        {
                            c1.horizontal(|ui| {
                                ui.label("n:");
                                let res = ui.add(Slider::new(n, 2..=20));
                                if res.changed() {
                                    self.should_render_preview = true;
                                }
                            });
                        }
                    }

                    c1.horizontal(|ui| {
                        ui.label("max_iter:");
                        let res = ui.add(
                            Slider::new(&mut self.params.max_iter, 10..=200000).logarithmic(true),
                        );
                        if res.changed() {
                            self.should_render_preview = true;
                        }
                    });

                    c1.add_space(SPACE_SIZE);
                    c1.heading("Controls");
                    c1.separator();

                    c1.scope(|ui| {
                        ui.spacing_mut().slider_width = 250.;
                        ui.horizontal(|ui| {
                            ui.label("zoom:");
                            let res = ui.add(
                                Slider::new(&mut self.params.zoom, 0.000000000001..=50.)
                                    .logarithmic(true),
                            );
                            if res.changed() {
                                self.should_render_preview = true;
                            }
                        });
                    });

                    {
                        let z = self.params.zoom;
                        c1.horizontal(|ui| {
                            ui.label("re:");
                            let res =
                                ui.add(DragValue::new(&mut self.params.center_x).speed(0.01 * z));
                            if res.changed() {
                                self.should_render_preview = true;
                            }
                        });
                        c1.horizontal(|ui| {
                            ui.label("im:");
                            let res =
                                ui.add(DragValue::new(&mut self.params.center_y).speed(0.01 * z));
                            if res.changed() {
                                self.should_render_preview = true;
                            }
                        });

                        let rotate = self.view.rotate;
                        c1.horizontal(|ui| {
                            ui.label("rotate:");
                            let mut rotate = rotate;
                            let res = ui.add(
                                DragValue::new(&mut rotate)
                                    .speed(0.1)
                                    .range(-TAU as F..=TAU as F),
                            );
                            if res.changed() {
                                self.params.rotate = Some(rotate);
                                self.should_render_preview = true;
                            }
                        });
                    }

                    c1.add_space(SPACE_SIZE);
                    c1.heading("Coloring");
                    c1.separator();

                    c1.horizontal(|ui| {
                        ui.label("coloring mode:");

                        let mut selected_mode_i = match self.params.coloring_mode {
                            ColoringMode::CumulativeHistogram { .. } => 0,
                            ColoringMode::MinMaxNorm { .. } => 1,
                            ColoringMode::BlackAndWhite { .. } => 2,
                        };
                        const MODES: &[&str] =
                            &["CumulativeHistogram", "MinMaxNorm", "BlackAndWhite"];
                        let res = ComboBox::from_id_salt("coloring_mode").show_index(
                            ui,
                            &mut selected_mode_i,
                            MODES.len(),
                            |i| MODES[i],
                        );

                        if res.changed() {
                            self.params.coloring_mode = match selected_mode_i {
                                0 => ColoringMode::CumulativeHistogram {
                                    map: MapValue::Linear,
                                },
                                1 => {
                                    let (init_min, init_max) =
                                        if let ColoringMode::MinMaxNorm { min, max, .. } =
                                            self.init_params.coloring_mode
                                        {
                                            (min, max)
                                        } else {
                                            (Extremum::Auto, Extremum::Auto)
                                        };
                                    ColoringMode::MinMaxNorm {
                                        min: init_min,
                                        max: init_max,
                                        map: MapValue::Linear,
                                    }
                                }
                                2 => ColoringMode::BlackAndWhite,
                                _ => unreachable!(),
                            };
                            self.should_render_preview = true;
                        }
                    });

                    match &mut self.params.coloring_mode {
                        ColoringMode::CumulativeHistogram { map }
                        | ColoringMode::MinMaxNorm { map, .. } => {
                            c1.horizontal(|ui| {
                                ui.label("map value:");

                                let mut selected_map_value_i = match map {
                                    MapValue::Linear => 0,
                                    MapValue::Squared => 1,
                                    MapValue::Powf(_) => 2,
                                };
                                const MAP_VALUE: &[&str] = &["Linear", "Squared", "Powf"];
                                let res = ComboBox::from_id_salt("map_value").show_index(
                                    ui,
                                    &mut selected_map_value_i,
                                    MAP_VALUE.len(),
                                    |i| MAP_VALUE[i],
                                );

                                if res.changed() {
                                    *map = match selected_map_value_i {
                                        0 => MapValue::Linear,
                                        1 => MapValue::Squared,
                                        2 => MapValue::Powf(1.),
                                        _ => unimplemented!(),
                                    };
                                    self.should_render_preview = true;
                                }

                                if let MapValue::Powf(exp) = map {
                                    let res =
                                        ui.add(Slider::new(exp, 0.01..=20.).logarithmic(true));
                                    if res.changed() {
                                        self.should_render_preview = true;
                                    }
                                }
                            });
                        }
                        _ => (),
                    }

                    if let ColoringMode::MinMaxNorm { min, max, .. } =
                        &mut self.params.coloring_mode
                    {
                        let (init_min, init_max) =
                            if let ColoringMode::MinMaxNorm { min, max, .. } =
                                self.init_params.coloring_mode
                            {
                                (min, max)
                            } else {
                                (Extremum::Auto, Extremum::Auto)
                            };

                        c1.horizontal(|ui| {
                            ui.label("min:");

                            let mut auto = min.is_auto();
                            let res = ui.checkbox(&mut auto, "auto");
                            if res.changed() {
                                *min = if auto {
                                    Extremum::Auto
                                } else {
                                    Extremum::Custom(init_min.unwrap_custom_or(0.))
                                };
                                self.should_render_preview = true;
                            }

                            if let Extremum::Custom(min) = min {
                                let res = ui.add(DragValue::new(min));
                                if res.changed() {
                                    self.should_render_preview = true;
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
                                    Extremum::Custom(init_max.unwrap_custom_or(1.))
                                };
                                self.should_render_preview = true;
                            }

                            if let Extremum::Custom(max) = max {
                                let res = ui.add(DragValue::new(max));
                                if res.changed() {
                                    self.should_render_preview = true;
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
                            self.render_preview();
                        }
                        if ui.button("save parameter file").clicked() {
                            self.notify(if self.save_parameter_file().is_ok() {
                                "saved"
                            } else {
                                "failed to save parameter file"
                            });
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
                                                self.should_render_preview = true;
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
                            self.should_render_preview = true;
                        }
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

                    c2.allocate_ui(c2.available_size() - 32. * Vec2::Y, |ui| {
                        if self.preview_render_handle.is_none() {
                            if let Some(preview_bytes) = &self.preview_bytes {
                                if let Some(preview_size) = self.preview_size {
                                    let d = 0.5 * (ui.available_height() - preview_size.y);
                                    ui.add_space(d);
                                    ui.add_sized(
                                        preview_size,
                                        Image::from_bytes(
                                            "bytes://fractal_preview".to_string()
                                                + &self.preview_id.to_string(),
                                            preview_bytes.to_owned(),
                                        )
                                        .maintain_aspect_ratio(true)
                                        .corner_radius(2),
                                    );
                                    ui.add_space(d);
                                }
                            }
                        } else {
                            ui.centered_and_justified(|ui| {
                                ui.spinner();
                            });
                        }
                    });
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

        if self.should_render_preview {
            self.round_floats();
            self.update_view();

            if self.preview_render_handle.is_none() {
                self.render_preview();
                self.should_render_preview = false;
            }
        }

        if let Some(handle) = self.preview_render_handle.as_ref() {
            if handle.is_finished() {
                let handle = self.preview_render_handle.take().unwrap();
                if let Ok(output_image) = handle.join() {
                    let mut buf = Vec::new();
                    output_image
                        .write_with_encoder(PngEncoder::new(&mut buf))
                        .unwrap();

                    self.preview_id += 1;
                    self.preview_bytes = Some(buf);
                }
            }
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

    fn save_parameter_file(&self) -> Result<()> {
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
        let sampling_points = generate_sampling_points(
            SAMPLE_MUL * (self.params.img_width * self.params.img_height) as usize,
        );
        let progress = Progress::new(sampling_points.len());

        let params_clone = self.params.clone();
        let view = self.view;
        let output_image_path_clone = self.output_image_path.clone();
        (
            progress.clone(),
            thread::spawn(move || {
                let raw_image =
                    render_raw_image(&params_clone, &view, &sampling_points, Some(progress));

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

    fn round_floats(&mut self) {
        fn truncate_to_significant_digits(value: F, digits: usize) -> F {
            if value.is_subnormal() {
                return value;
            }
            let factor = (10. as F).powi(digits as i32 - value.abs().log10().floor() as i32 - 1);
            (value * factor).round() / factor
        }

        self.params.zoom = truncate_to_significant_digits(self.params.zoom, 4);

        if let Some(rotate) = self.params.rotate.as_mut() {
            *rotate = truncate_to_significant_digits(*rotate, 3);
        }
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

    fn render_preview(&mut self) {
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
            ..self.params.clone()
        };

        let sampling_points = generate_sampling_points(
            SAMPLE_MUL * (preview_params.img_width * preview_params.img_height) as usize,
        );

        let view = self.view;
        self.preview_render_handle = Some(thread::spawn(move || {
            let raw_image = render_raw_image(&preview_params, &view, &sampling_points, None);

            let output_image = color_raw_image(
                &preview_params,
                preview_params.coloring_mode,
                preview_params.custom_gradient.as_ref(),
                raw_image,
            );

            output_image
        }));
    }
}
