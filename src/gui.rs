use std::{
    fs,
    thread::{self, JoinHandle},
    time::Instant,
};

use eframe::{
    egui::{self, Button, Color32, DragValue, Image, ProgressBar, Slider, Vec2},
    App, CreationContext, Frame as EFrame,
};
use image::codecs::png::PngEncoder;
use ron::ser::PrettyConfig;
use uni_path::PathBuf;

use crate::{
    coloring::color_raw_image,
    error::{ErrorKind, Result},
    params::{FrameParams, ParamsKind},
    progress::Progress,
    rendering::render_raw_image,
    sampling::{generate_sampling_points, Sampling},
    View, F,
};

pub struct Gui {
    params: FrameParams,
    view: View,
    sampling_points: Vec<(F, F)>,

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
        sampling_points: Vec<(F, F)>,
        output_image_path: PathBuf,
        param_file_path: PathBuf,
    ) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut slf = Gui {
            params,
            view,
            sampling_points,

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
                c1.heading("Controls");
                c1.separator();

                let mut params_updated = false;

                c1.scope(|ui| {
                    ui.spacing_mut().slider_width = 250.;
                    ui.horizontal(|ui| {
                        ui.label("zoom: ");
                        let res = ui.add(
                            Slider::new(&mut self.params.zoom, 0.000000000001..=50.)
                                .logarithmic(true),
                        );
                        if res.changed() {
                            params_updated = true;
                        }
                    });
                });

                {
                    let z = self.params.zoom;
                    c1.horizontal(|ui| {
                        ui.label("re: ");
                        let res = ui.add(DragValue::new(&mut self.params.center_x).speed(0.01 * z));
                        if res.changed() {
                            params_updated = true;
                        }
                    });
                    c1.horizontal(|ui| {
                        ui.label("im: ");
                        let res = ui.add(DragValue::new(&mut self.params.center_y).speed(0.01 * z));
                        if res.changed() {
                            params_updated = true;
                        }
                    });
                }

                c1.horizontal(|ui| {
                    if ui.button("revert to parameter file").clicked() {
                        self.revert_to_parameter_file();
                        self.update_preview();
                    }
                    if ui.button("write parameter file").clicked() {
                        self.save_parameter_file();
                    }
                });
                c1.horizontal(|ui| {
                    let btn: egui::Response =
                        ui.add_enabled(self.render_info.is_none(), Button::new("render and save"));
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
                if params_updated {
                    self.update_view();
                    self.update_preview();
                }

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
            });
        });
    }
}

impl Gui {
    fn revert_to_parameter_file(&mut self) {
        if let ParamsKind::Frame(params) = ron::from_str::<ParamsKind>(
            &fs::read_to_string(self.param_file_path.as_str())
                .map_err(ErrorKind::ReadParameterFile)
                .unwrap(),
        )
        .map_err(ErrorKind::DecodeParameterFile)
        .unwrap()
        {
            self.params = params;
        } else {
            unimplemented!()
        }
        self.update_view();
    }
    fn save_parameter_file(&self) {
        fs::write(
            self.param_file_path.to_str(),
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
        let sampling_points_clone = self.sampling_points.clone();
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
                level: crate::sampling::SamplingLevel::Low,
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
