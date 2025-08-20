use std::f64::consts::{PI, TAU};

use eframe::egui::{DragValue, Slider, Ui};

use crate::{gui::Gui, F};

impl Gui {
    pub fn show_section_controls(&mut self, ui: &mut Ui) {
        ui.add_enabled_ui(self.render_info.is_none(), |ui| {
            const N_DECIMALS: usize = 8;

            ui.scope(|ui| {
                ui.horizontal(|ui| {
                    let label_width = ui.label("zoom:").rect.width();
                    ui.spacing_mut().slider_width = Self::SLIDER_END_POS - label_width;
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

            ui.horizontal(|ui| {
                let label_width = ui.label("x:").rect.width();
                ui.add_space(FIXED_LABEL_WIDTH - label_width);
                let res = ui.add(
                    DragValue::new(&mut self.params.center_x)
                        .speed(speed)
                        .min_decimals(N_DECIMALS),
                );
                changed |= res.changed();
            });
            ui.horizontal(|ui| {
                let label_width = ui.label("y:").rect.width();
                ui.add_space(FIXED_LABEL_WIDTH - label_width);
                let res = ui.add(
                    DragValue::new(&mut self.params.center_y)
                        .speed(speed)
                        .min_decimals(N_DECIMALS),
                );
                changed |= res.changed();
            });

            ui.horizontal(|ui| {
                ui.label("rotate:");
                let mut rotate = self.params.rotate.unwrap_or(0.);

                const FRAC_PI_180: F = PI as F / 180.;
                let res = ui.add(
                    DragValue::new(&mut rotate)
                        .speed(0.01)
                        .range(0. ..=TAU as F)
                        .custom_parser(|s| {
                            #[allow(clippy::unnecessary_cast)]
                            s.parse::<F>()
                                .ok()
                                .map(|degrees| (degrees.floor() * FRAC_PI_180) as f64)
                        })
                        .custom_formatter(|rad, _| {
                            let degrees = rad as F / FRAC_PI_180;
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
        });
    }
}
