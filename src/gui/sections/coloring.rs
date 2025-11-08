use eframe::egui::{
    color_picker::color_edit_button_srgb, Button, CollapsingHeader, ComboBox, DragValue, Slider, Ui,
};

use crate::{
    coloring::{ColoringMode, Extremum, MapValue},
    gui::{FileDialogAction, FileDialogKind, Gui},
    F,
};

impl Gui {
    pub fn show_section_coloring(&mut self, ui: &mut Ui) {
        let mut changed = false;

        ui.add_enabled_ui(self.render_info.is_none(), |ui| {
            ui.horizontal(|ui| {
                ui.label("coloring mode:");

                ComboBox::from_id_salt("coloring_mode")
                    .selected_text(match self.params.coloring_mode {
                        ColoringMode::MinMaxNorm { .. } => "MinMaxNorm",
                        ColoringMode::Repeating { .. } => "Repeating",
                    })
                    .show_ui(ui, |ui| changed |= self.show_combobox_coloring_mode(ui));
            });

            ui.horizontal(|ui| {
                let (ColoringMode::MinMaxNorm { map, .. } | ColoringMode::Repeating { map, .. }) =
                    &mut self.params.coloring_mode;

                ui.label("map value:");

                ComboBox::from_id_salt("map_value")
                    .selected_text(match map {
                        MapValue::Linear => "Linear",
                        MapValue::Powf(_) => "Powf",
                    })
                    .show_ui(ui, |ui| changed |= Self::show_combobox_map_value(ui, map));

                if let MapValue::Powf(p) = map {
                    changed |= ui
                        .add(Slider::new(p, 0.001..=20.).logarithmic(true))
                        .changed();
                }
            });

            #[allow(irrefutable_let_patterns)]
            if let ColoringMode::MinMaxNorm { min, max, .. } = &mut self.params.coloring_mode {
                const FIXED_LABEL_WIDTH: f32 = 30.;

                let upper_bound = self
                    .params
                    .fractal
                    .max_iter()
                    .map(|x| x as F)
                    .unwrap_or(F::INFINITY);

                ui.horizontal(|ui| {
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
                        changed = true;
                    }

                    ui.spacing_mut().slider_width =
                        Self::SLIDER_END_POS - FIXED_LABEL_WIDTH - res.rect.width();

                    if let Extremum::Custom(min) = min {
                        changed |= ui.add(Slider::new(min, 0. ..=upper_bound)).changed();
                    }
                });

                ui.horizontal(|ui| {
                    let label_width = ui.label("max:").rect.width();
                    ui.add_space(FIXED_LABEL_WIDTH - label_width);

                    let mut auto = max.is_auto();
                    let res = ui.checkbox(&mut auto, "auto");
                    if res.changed() {
                        *max = if auto {
                            Extremum::Auto
                        } else {
                            self.params
                                .fractal
                                .max_iter()
                                .map(|max_iter| Extremum::Custom(max_iter as F))
                                .unwrap_or(Extremum::Auto)
                        };
                        changed = true;
                    }

                    ui.spacing_mut().slider_width =
                        Self::SLIDER_END_POS - FIXED_LABEL_WIDTH - res.rect.width();

                    if let Extremum::Custom(max) = max {
                        changed |= ui.add(Slider::new(max, 0. ..=upper_bound)).changed();
                    }
                });
            }

            if let ColoringMode::Repeating { frac, rotate, .. } = &mut self.params.coloring_mode {
                ui.horizontal(|ui| {
                    ui.label("period:");
                    changed |= ui
                        .add(Slider::new(frac, 0. ..=10000.).logarithmic(true))
                        .changed();
                });
                ui.horizontal(|ui| {
                    ui.label("rotate:");
                    changed |= ui
                        .add(DragValue::new(rotate).speed(0.01).range(0. ..=1.))
                        .changed();
                });
            }
        });

        CollapsingHeader::new("Gradient")
            .default_open(false)
            .show(ui, |ui| {
                ui.add_enabled_ui(self.render_info.is_none(), |ui| {
                    changed |= self.show_gradient_ui(ui)
                });
            });

        if changed {
            self.params_changes.set_non_breaking();
        }
    }

    fn show_combobox_coloring_mode(&mut self, ui: &mut Ui) -> bool {
        [
            (
                "MinMaxNorm",
                matches!(self.params.coloring_mode, ColoringMode::MinMaxNorm { .. }),
                ColoringMode::MinMaxNorm {
                    min: Extremum::Auto,
                    max: Extremum::Auto,
                    map: MapValue::Linear,
                },
            ),
            (
                "Repeating",
                matches!(self.params.coloring_mode, ColoringMode::Repeating { .. }),
                ColoringMode::Repeating {
                    frac: 1000.,
                    rotate: 0.,
                    map: MapValue::Linear,
                },
            ),
        ]
        .iter()
        .any(|&(name, selected, default)| {
            let res = ui.selectable_label(selected, name);
            let clicked = res.clicked() && !selected;
            if clicked {
                self.params.coloring_mode = default;
            }
            clicked
        })
    }

    fn show_combobox_map_value(ui: &mut Ui, map: &mut MapValue) -> bool {
        [
            ("Linear", matches!(map, MapValue::Linear), MapValue::Linear),
            ("Powf", matches!(map, MapValue::Powf(_)), MapValue::Powf(1.)),
        ]
        .iter()
        .any(|&(name, selected, default)| {
            let res = ui.selectable_label(selected, name);
            let clicked = res.clicked() && !selected;
            if clicked {
                *map = default;
            }
            clicked
        })
    }

    fn show_gradient_ui(&mut self, ui: &mut Ui) -> bool {
        let mut changed = false;

        let l = self.params.gradient.len();
        let t_values = self
            .params
            .gradient
            .iter()
            .map(|&(t, _)| t)
            .collect::<Vec<_>>();

        let mut reorder = (0..l).map(Some).collect::<Vec<_>>();

        for (i, (t, c)) in &mut self.params.gradient.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                let is_start = i == 0;
                let is_end = i + 1 == l;
                let is_start_or_end = is_start || is_end;

                if is_start {
                    *t = 0.;
                }
                if is_end {
                    *t = 1.;
                }

                let range = if !is_start_or_end {
                    t_values.get(i - 1).copied().unwrap_or(0.)
                        ..=t_values.get(i + 1).copied().unwrap_or(1.)
                } else {
                    0. ..=1.
                };
                changed |= ui
                    .add_enabled(
                        !is_start_or_end,
                        DragValue::new(t).range(range).fixed_decimals(2).speed(0.01),
                    )
                    .changed();
                changed |= color_edit_button_srgb(ui, c).changed();
                if ui.add_enabled(!is_start, Button::new("up")).clicked() {
                    reorder.swap(i - 1, i);
                    changed = true;
                }
                if ui.add_enabled(!is_end, Button::new("down")).clicked() {
                    reorder.swap(i, i + 1);
                    changed = true;
                }
                if ui.add_enabled(l > 2, Button::new("remove")).clicked() {
                    reorder[i] = None;
                    changed = true;
                }
                if ui.button("duplicate").clicked() {
                    reorder.insert(i, Some(i));
                }
            });
        }

        self.params.gradient = reorder
            .iter()
            .filter_map(|&v| v)
            .map(|i| self.params.gradient[i])
            .collect::<Vec<_>>();

        if ui
            .button("load gradient from other parameter file")
            .clicked()
        {
            self.open_file_dialog(
                self.param_file_path
                    .as_ref()
                    .or(self.output_image_path.as_ref())
                    .and_then(|p| p.parent())
                    .map(|p| p.to_path_buf()),
                FileDialogKind::PickFile,
                FileDialogAction::LoadGradientFromParameterFile,
            );
        }

        changed
    }
}
