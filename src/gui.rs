use std::{
    fs,
    thread::{self, JoinHandle},
    time::Instant,
};

use eframe::{
    egui::{self, Button, Color32, ComboBox, DragValue, Image, ProgressBar, Slider, Vec2},
    App, CreationContext, Frame as EFrame,
};
use image::codecs::png::PngEncoder;
use ron::ser::PrettyConfig;
use uni_path::PathBuf;

use crate::{
    coloring::{color_raw_image, ColoringMode, Extremum, MapValue},
    error::{ErrorKind, Result},
    fractal::Fractal,
    params::{FrameParams, ParamsKind},
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

    render_info: Option<(JoinHandle<Result<()>>, Progress, Instant)>,
}

impl Gui {
    pub const PREVIEW_WIDTH: u32 = 256;

    pub fn new(
        cc: &CreationContext,
        params: FrameParams,
        view: View,
        output_image_path: PathBuf,
        param_file_path: PathBuf,
    ) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut slf = Gui {
            init_params: params.clone(),
            params,
            view,

            output_image_path,
            param_file_path,

            preview_bytes: None,
            preview_size: None,
            preview_id: 0,

            render_info: None,
        };

        slf.update_preview();

        slf
    }
}

impl App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut EFrame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns_const(|[c1, c2]| {
                const SPACE_SIZE: f32 = 8.;

                let mut should_update_preview = false;

                // First column

                c1.heading("Fractal");
                c1.separator();
                c1.horizontal(|ui| {
                    ui.label("fractal: ");

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
                        _ => unimplemented!(), // Fractal::MoireTest => 15,
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
                            _ => unreachable!(),
                        };

                        // Reset view
                        self.params.center_x = 0.;
                        self.params.center_y = 0.;
                        self.params.zoom = DEFAULT_ZOOM;

                        should_update_preview = true;
                    }
                });

                {
                    if let Fractal::MandelbrotCustomExp { exp } = &mut self.params.fractal {
                        c1.horizontal(|ui| {
                            ui.label("exp: ");
                            let res = ui.add(Slider::new(exp, 0.001..=20.));
                            if res.changed() {
                                should_update_preview = true;
                            }
                        });
                    }

                    if let Fractal::SecondDegreeRecWithGrowingExponentParam { a_re, a_im }
                    | Fractal::ComplexLogisticMapLike { a_re, a_im }
                    | Fractal::Wmriho { a_re, a_im }
                    | Fractal::Iigdzh { a_re, a_im } = &mut self.params.fractal
                    {
                        c1.scope(|ui| {
                            ui.spacing_mut().slider_width = 250.;
                            const FRACTAL_PARAM_RANGE: F = 5.;

                            ui.horizontal(|ui| {
                                ui.label("a_re: ");
                                let res = ui.add(Slider::new(
                                    a_re,
                                    -FRACTAL_PARAM_RANGE..=FRACTAL_PARAM_RANGE,
                                ));
                                if res.changed() {
                                    should_update_preview = true;
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label("a_im: ");
                                let res = ui.add(Slider::new(
                                    a_im,
                                    -FRACTAL_PARAM_RANGE..=FRACTAL_PARAM_RANGE,
                                ));
                                if res.changed() {
                                    should_update_preview = true;
                                }
                            });
                        });
                    }

                    if let Fractal::NthDegreeRecWithGrowingExponent(n) = &mut self.params.fractal {
                        c1.horizontal(|ui| {
                            ui.label("n: ");
                            let res = ui.add(Slider::new(n, 2..=20));
                            if res.changed() {
                                should_update_preview = true;
                            }
                        });
                    }
                }

                c1.horizontal(|ui| {
                    ui.label("max_iter: ");
                    let res = ui
                        .add(Slider::new(&mut self.params.max_iter, 10..=200000).logarithmic(true));
                    if res.changed() {
                        should_update_preview = true;
                    }
                });

                c1.add_space(SPACE_SIZE);
                c1.heading("Controls");
                c1.separator();

                c1.scope(|ui| {
                    ui.spacing_mut().slider_width = 250.;
                    ui.horizontal(|ui| {
                        ui.label("zoom: ");
                        let res = ui.add(
                            Slider::new(&mut self.params.zoom, 0.000000000001..=50.)
                                .logarithmic(true),
                        );
                        if res.changed() {
                            should_update_preview = true;
                        }
                    });
                });

                {
                    let z = self.params.zoom;
                    c1.horizontal(|ui| {
                        ui.label("re: ");
                        let res = ui.add(DragValue::new(&mut self.params.center_x).speed(0.01 * z));
                        if res.changed() {
                            should_update_preview = true;
                        }
                    });
                    c1.horizontal(|ui| {
                        ui.label("im: ");
                        let res = ui.add(DragValue::new(&mut self.params.center_y).speed(0.01 * z));
                        if res.changed() {
                            should_update_preview = true;
                        }
                    });
                }

                c1.add_space(SPACE_SIZE);
                c1.heading("Coloring");
                c1.separator();

                c1.horizontal(|ui| {
                    ui.label("coloring mode: ");

                    let mut selected_mode_i = match self.params.coloring_mode {
                        ColoringMode::CumulativeHistogram { .. } => 0,
                        ColoringMode::MinMaxNorm { .. } => 1,
                        ColoringMode::BlackAndWhite { .. } => 2,
                    };
                    const MODES: &[&str] = &["CumulativeHistogram", "MinMaxNorm", "BlackAndWhite"];
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
                        should_update_preview = true;
                    }
                });

                match &mut self.params.coloring_mode {
                    ColoringMode::CumulativeHistogram { map }
                    | ColoringMode::MinMaxNorm { map, .. } => {
                        c1.horizontal(|ui| {
                            ui.label("map value: ");

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
                                should_update_preview = true;
                            }

                            if let MapValue::Powf(exp) = map {
                                let res = ui.add(Slider::new(exp, 0.01..=20.).logarithmic(true));
                                if res.changed() {
                                    should_update_preview = true;
                                }
                            }
                        });
                    }
                    _ => (),
                }

                if let ColoringMode::MinMaxNorm { min, max, .. } = &mut self.params.coloring_mode {
                    let (init_min, init_max) =
                        if let ColoringMode::MinMaxNorm { min, max, .. } =
                            self.init_params.coloring_mode
                        {
                            (min, max)
                        } else {
                            (Extremum::Auto, Extremum::Auto)
                        };

                    c1.horizontal(|ui| {
                        ui.label("min: ");

                        let mut auto = min.is_auto();
                        let res = ui.checkbox(&mut auto, "auto");
                        if res.changed() {
                            *min = if auto {
                                Extremum::Auto
                            } else {
                                Extremum::Custom(init_min.unwrap_custom_or(0.))
                            };
                            should_update_preview = true;
                        }

                        if let Extremum::Custom(min) = min {
                            let res = ui.add(Slider::new(min, 0. ..=self.params.max_iter as F));
                            if res.changed() {
                                should_update_preview = true;
                            }
                        }
                    });
                    c1.horizontal(|ui| {
                        ui.label("max: ");

                        let mut auto = max.is_auto();
                        let res = ui.checkbox(&mut auto, "auto");
                        if res.changed() {
                            *max = if auto {
                                Extremum::Auto
                            } else {
                                Extremum::Custom(
                                    init_max.unwrap_custom_or(self.params.max_iter as F),
                                )
                            };
                            should_update_preview = true;
                        }

                        if let Extremum::Custom(max) = max {
                            let res = ui.add(Slider::new(max, 0. ..=self.params.max_iter as F));
                            if res.changed() {
                                should_update_preview = true;
                            }
                        }
                    });
                }

                c1.add_space(SPACE_SIZE);
                c1.heading("Parameter file");
                c1.separator();

                c1.horizontal(|ui| {
                    if ui.button("revert edits").clicked() {
                        self.revert_edits();
                        self.update_preview();
                    }
                    if ui.button("write parameter file").clicked() {
                        self.save_parameter_file();
                    }
                });

                if should_update_preview {
                    self.update_view();
                    self.update_preview();
                }

                // Second column

                c2.heading("Preview");
                c2.separator();

                if let Some(preview_bytes) = &self.preview_bytes {
                    if let Some(preview_size) = self.preview_size {
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
                    }
                }

                c2.add_space(SPACE_SIZE);
                c2.heading("Render");
                c2.separator();

                c2.add_enabled_ui(self.render_info.is_none(), |ui| {
                    ui.horizontal(|ui| {
                        let mut selected_sampling_level_i = match self.params.sampling.level {
                            SamplingLevel::Exploration => 0,
                            SamplingLevel::Low => 1,
                            SamplingLevel::Medium => 2,
                            SamplingLevel::High => 3,
                            SamplingLevel::Ultra => 4,
                            SamplingLevel::Extreme => 5,
                            SamplingLevel::Extreme1 => 6,
                            SamplingLevel::Extreme2 => 7,
                            SamplingLevel::Extreme3 => 8,
                        };
                        const SAMPLING_LEVEL: &[&str] = &[
                            "Exploration",
                            "Low",
                            "Medium",
                            "High",
                            "Ultra",
                            "Extreme",
                            "Extreme1",
                            "Extreme2",
                            "Extreme3",
                        ];
                        let res = ComboBox::from_id_salt("sampling_level").show_index(
                            ui,
                            &mut selected_sampling_level_i,
                            SAMPLING_LEVEL.len(),
                            |i| SAMPLING_LEVEL[i],
                        );

                        if res.changed() {
                            self.params.sampling.level = match selected_sampling_level_i {
                                0 => SamplingLevel::Exploration,
                                1 => SamplingLevel::Low,
                                2 => SamplingLevel::Medium,
                                3 => SamplingLevel::High,
                                4 => SamplingLevel::Ultra,
                                5 => SamplingLevel::Extreme,
                                6 => SamplingLevel::Extreme1,
                                7 => SamplingLevel::Extreme2,
                                8 => SamplingLevel::Extreme3,
                                _ => unimplemented!(),
                            }
                        }
                    });
                });

                c2.horizontal(|ui| {
                    let btn: egui::Response = ui.add_enabled(
                        self.render_info.is_none(),
                        Button::new("render and save image"),
                    );
                    if btn.clicked() {
                        let (progress, handle) = self.render_and_save();
                        self.render_info = Some((handle, progress, Instant::now()));
                    };
                    if let Some((handle, progress, start)) = &self.render_info {
                        let progress_bar = ProgressBar::new(progress.get_progress())
                            .desired_height(4.)
                            .desired_width(64.)
                            .corner_radius(0.)
                            .fill(Color32::WHITE);
                        ui.add(progress_bar);
                        ui.label(format!(
                            "{:.0}%  –  {:.1}s",
                            100. * progress.get_progress(),
                            start.elapsed().as_secs_f32()
                        ));
                        ctx.request_repaint();

                        if handle.is_finished() {
                            self.render_info = None;
                        }
                    }
                });
            });
        });
    }
}

impl Gui {
    fn revert_edits(&mut self) {
        self.params = self.init_params.clone();
        self.update_view();
    }

    fn save_parameter_file(&self) {
        fs::write(
            self.param_file_path.as_str(),
            ron::ser::to_string_pretty(
                &ParamsKind::Frame(self.params.clone()),
                PrettyConfig::default(),
            )
            .map_err(ErrorKind::EncodeParameterFile)
            .unwrap(),
        )
        .map_err(ErrorKind::WriteParameterFile)
        .unwrap();
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
            center_x,
            center_y,
            ..
        } = self.params;

        self.view = View::new(img_width, img_height, zoom, center_x, center_y);
    }

    fn update_preview(&mut self) {
        let preview_width = Gui::PREVIEW_WIDTH;
        let preview_height = (self.params.img_height * Gui::PREVIEW_WIDTH) / self.params.img_width;

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
}
