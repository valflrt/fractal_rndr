use eframe::egui::{ComboBox, DragValue, Ui};

use crate::{gui::Gui, sampling::SamplingLevel};

impl Gui {
    pub fn show_section_render(&mut self, ui: &mut Ui) {
        ui.add_enabled_ui(self.render_info.is_none(), |ui| {
            ui.horizontal(|ui| {
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

            ui.horizontal(|ui| {
                ui.label("current spp:")
                    .on_hover_text("number of samples per pixel of the internal image");
                ui.code(format!(" {} ", self.samples_per_pixel));
            });

            ui.horizontal(|ui| {
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
                    self.render_and_save()
                };

                let no_samples = self.samples_per_pixel == 0;
                let no_output_image_path = self.output_image_path.is_none();
                ui.add_enabled_ui(!(no_samples || no_output_image_path), |ui| {
                    let res = {
                        let btn = ui.button("save image");

                        if no_output_image_path {
                            btn.on_disabled_hover_text("no path was provided for the output image")
                        } else if no_samples {
                            btn.on_disabled_hover_text("sample the fractal before saving the image")
                        } else {
                            btn
                        }
                    };

                    self.should_save_image |= res.clicked();
                });
            });
        });
    }

    fn show_combobox_sampling_level(&mut self, ui: &mut Ui) {
        const LEVELS: &[(SamplingLevel, &str)] = &[
            (SamplingLevel::Raw, "Raw"),
            (SamplingLevel::Exploration, "Exploration"),
            (SamplingLevel::Low, "Low"),
            (SamplingLevel::Medium, "Medium"),
            (SamplingLevel::High, "High"),
            (SamplingLevel::Ultra, "Ultra"),
            (SamplingLevel::Extreme, "Extreme"),
        ];

        for &(level, name) in LEVELS {
            ui.selectable_value(&mut self.params.sampling.level, level, name);
        }
    }
}
