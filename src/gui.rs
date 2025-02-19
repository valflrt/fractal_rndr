use std::fs;

use eframe::{
    egui::{self, Image, Vec2},
    App, CreationContext, Frame as EFrame,
};
use image::codecs::png::PngEncoder;
use ron::ser::PrettyConfig;
use uni_path::PathBuf;

use crate::{
    coloring::color_raw_image, params::FrameParams, rendering::render_raw_image,
    sampling::Sampling, RenderCtx, View,
};

pub struct Gui {
    fractal_params: FrameParams,
    render_ctx: RenderCtx,
    view: View,

    param_file_path: PathBuf,

    preview_bytes: Option<Vec<u8>>,
    preview_size: Option<Vec2>,
    preview_id: u128,
}

impl Gui {
    pub const PREVIEW_WIDTH: u32 = 256;

    pub fn new(
        cc: &CreationContext,
        fractal_params: FrameParams,
        render_ctx: RenderCtx,
        view: View,
        param_file_path: PathBuf,
    ) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let mut slf = Gui {
            fractal_params,
            render_ctx,
            view,

            param_file_path,

            preview_bytes: None,
            preview_size: None,
            preview_id: 0,
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

                let mut preview_needs_update = false;
                c1.horizontal(|ui| {
                    let step = 0.1;
                    if ui.button("left").clicked() {
                        self.fractal_params.center_x -= step * self.fractal_params.zoom;
                        preview_needs_update = true;
                    }
                    if ui.button("right").clicked() {
                        self.fractal_params.center_x += step * self.fractal_params.zoom;
                        preview_needs_update = true;
                    }
                    if ui.button("up").clicked() {
                        self.fractal_params.center_y += step * self.fractal_params.zoom;
                        preview_needs_update = true;
                    }
                    if ui.button("down").clicked() {
                        self.fractal_params.center_y -= step * self.fractal_params.zoom;
                        preview_needs_update = true;
                    }
                });
                c1.horizontal(|ui| {
                    let step = 2.;
                    if ui.button("zoom in").clicked() {
                        self.fractal_params.zoom /= step;
                        preview_needs_update = true;
                    }
                    if ui.button("zoom out").clicked() {
                        self.fractal_params.zoom *= step;
                        preview_needs_update = true;
                    }
                });
                c1.horizontal(|ui| {
                    if ui.button("write parameter file").clicked() {
                        fs::write(
                            self.param_file_path.to_str(),
                            ron::ser::to_string_pretty(&self.fractal_params, PrettyConfig::new())
                                .unwrap(),
                        )
                        .unwrap();
                    }
                });
                if preview_needs_update {
                    let FrameParams {
                        img_width,
                        img_height,
                        zoom,
                        center_x,
                        center_y,
                        ..
                    } = self.fractal_params;

                    self.view = View::new(img_width, img_height, zoom, center_x, center_y);
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
    fn update_preview(&mut self) {
        let preview_width = Gui::PREVIEW_WIDTH;
        let preview_height =
            (self.fractal_params.img_height * Gui::PREVIEW_WIDTH) / self.fractal_params.img_width;

        self.preview_size = Some(Vec2::new(preview_width as f32, preview_height as f32));

        let preview_render_ctx = {
            RenderCtx::new(&FrameParams {
                img_width: preview_width,
                img_height: preview_height,
                sampling: Sampling {
                    level: crate::sampling::SamplingLevel::Low,
                    random_offsets: true,
                },
                ..self.fractal_params.clone()
            })
            .unwrap()
        };

        let raw_image = render_raw_image(
            self.fractal_params.fractal,
            &self.view,
            &preview_render_ctx,
            None,
        );

        let output_image = color_raw_image(
            &preview_render_ctx,
            self.fractal_params.coloring_mode,
            self.fractal_params.custom_gradient.as_ref(),
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
