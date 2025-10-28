use eframe::egui::{ComboBox, DragValue, Grid, Slider, Ui};

use crate::{
    coloring::{ColoringMode, Extremum},
    fractal::Fractal,
    gui::{Gui, DEFAULT_ZOOM},
    F,
};

impl Gui {
    pub fn show_section_fractal(&mut self, ui: &mut Ui) {
        ui.add_enabled_ui(self.render_info.is_none(), |ui| {
            ui.horizontal(|ui| {
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

            if self.show_fractal_parameters(ui) {
                self.params_changes.set_breaking();
            }
        });
    }

    fn show_fractal_parameters(&mut self, ui: &mut Ui) -> bool {
        const N_DECIMALS: usize = 8;

        let mut changed = false;

        if let Some(max_iter) = self.params.fractal.max_iter_mut() {
            ui.horizontal(|ui| {
                let label_width = ui.label("max iter:").rect.width();
                ui.spacing_mut().slider_width = Self::SLIDER_END_POS - label_width;
                let prev_max_iter = *max_iter;
                let res = ui.add(Slider::new(max_iter, 10..=200000).logarithmic(true));
                if res.changed() {
                    changed = true;

                    // Avoid leaving max slider at a low value when
                    // max_iter is increased.
                    if prev_max_iter < *max_iter {
                        if let ColoringMode::MinMaxNorm {
                            max: Extremum::Custom(max),
                            ..
                        } = &mut self.params.coloring_mode
                        {
                            *max = *max_iter as F;
                        }
                    }
                }
            });
        }

        if let Some(bailout) = self.params.fractal.bailout_mut() {
            ui.horizontal(|ui| {
                let label_width = ui.label("bailout:").rect.width();
                ui.spacing_mut().slider_width = Self::SLIDER_END_POS - label_width;
                let res = ui.add(Slider::new(bailout, 0.01..=100.).logarithmic(true));
                changed |= res.changed();
            });
        }

        if let Fractal::MandelbrotCustomExp { exp, .. } = &mut self.params.fractal {
            ui.horizontal(|ui| {
                ui.label("exp:");
                let res = ui.add(
                    DragValue::new(exp)
                        .speed(self.fractal_param_precision)
                        .range(0.001..=20.)
                        .fixed_decimals(N_DECIMALS),
                );
                changed |= res.changed();
            });
        }

        if let Fractal::SdrgeCustomIntExp { exp, .. } = &mut self.params.fractal {
            ui.horizontal(|ui| {
                ui.label("exp:");
                let res = ui.add(DragValue::new(exp).range(1..=10));
                changed |= res.changed();
            });
        }
        if let Fractal::SdrgeCustomExp { exp, .. } = &mut self.params.fractal {
            ui.horizontal(|ui| {
                ui.label("exp:");
                let res = ui.add(
                    DragValue::new(exp)
                        .speed(self.fractal_param_precision)
                        .range(1..=10)
                        .fixed_decimals(N_DECIMALS),
                );
                changed |= res.changed();
            });
        }

        if let Fractal::SdrgeParam { a_re, a_im, .. }
        | Fractal::ComplexLogisticMapLike { a_re, a_im, .. }
        | Fractal::Wmriho { a_re, a_im, .. }
        | Fractal::Iigdzh { a_re, a_im, .. } = &mut self.params.fractal
        {
            ui.horizontal(|ui| {
                ui.label("a_re:");
                let res1 = ui.add(
                    DragValue::new(a_re)
                        .speed(self.fractal_param_precision)
                        .fixed_decimals(N_DECIMALS),
                );
                ui.label("a_im:");
                let res2 = ui.add(
                    DragValue::new(a_im)
                        .speed(self.fractal_param_precision)
                        .fixed_decimals(N_DECIMALS),
                );

                changed |= res1.changed() || res2.changed();
            });
        }

        if let Fractal::NthDrge { n, .. } = &mut self.params.fractal {
            ui.horizontal(|ui| {
                ui.label("n:");
                let res = ui.add(Slider::new(n, 2..=20));
                changed |= res.changed();
            });
        }

        if let Fractal::Sfwypc {
            alpha, beta, gamma, ..
        } = &mut self.params.fractal
        {
            Grid::new("param grid").show(ui, |ui| {
                [(alpha, "alpha"), (beta, "beta"), (gamma, "gamma")]
                    .iter_mut()
                    .for_each(|(v, name)| {
                        ui.label(name.to_string() + "_re:");
                        changed |= ui
                            .add(
                                DragValue::new(&mut v.0)
                                    .speed(self.fractal_param_precision)
                                    .fixed_decimals(N_DECIMALS),
                            )
                            .changed();
                        ui.label(name.to_string() + "_im:");
                        changed |= ui
                            .add(
                                DragValue::new(&mut v.1)
                                    .speed(self.fractal_param_precision)
                                    .fixed_decimals(N_DECIMALS),
                            )
                            .changed();
                        ui.end_row();
                    });
            });
        }

        ui.horizontal(|ui| {
            ui.label("precision:");
            ui.add(Slider::new(&mut self.fractal_param_precision, 1e-9..=1e-3).logarithmic(true));
        });

        changed
    }

    fn show_combobox_fractal(&mut self, ui: &mut Ui) -> bool {
        [
            (
                matches!(self.params.fractal, Fractal::Mandelbrot { .. }),
                "Mandelbrot",
                None,
                Fractal::Mandelbrot {
                    max_iter: 500,
                    bailout: 10.,
                },
            ),
            (
                matches!(self.params.fractal, Fractal::MandelbrotCustomExp { .. }),
                "MandelbrotCustomExp(exp)",
                None,
                Fractal::MandelbrotCustomExp {
                    max_iter: 500,
                    bailout: 10.,
                    exp: 2.,
                },
            ),
            (
                matches!(self.params.fractal, Fractal::Sdrge { .. }),
                "Sdrge",
                Some("second degree recursive sequence with growing exponent"),
                Fractal::Sdrge {
                    max_iter: 500,
                    bailout: 10.,
                },
            ),
            (
                matches!(self.params.fractal, Fractal::SdrgeCustomIntExp { .. }),
                "SdrgeCustomIntExp(exp)",
                Some("second degree recursive sequence with growing custom integer exponent"),
                Fractal::SdrgeCustomIntExp {
                    max_iter: 500,
                    bailout: 10.,
                    exp: 2,
                },
            ),
            (
                matches!(self.params.fractal, Fractal::SdrgeCustomExp { .. }),
                "SdrgeCustomExp(exp)",
                Some("second degree recursive sequence with growing custom exponent"),
                Fractal::SdrgeCustomExp {
                    max_iter: 500,
                    bailout: 10.,
                    exp: 2.,
                },
            ),
            (
                matches!(self.params.fractal, Fractal::SdrgeParam { .. }),
                "SdrgeParam(a_re, a_im)",
                Some("parameterized second degree recursive sequence with growing exponent"),
                Fractal::SdrgeParam {
                    max_iter: 500,
                    bailout: 10.,
                    a_re: 1.,
                    a_im: 0.,
                },
            ),
            (
                matches!(self.params.fractal, Fractal::Sdrage { .. }),
                "Sdrage",
                Some("second degree recursive alternating sequence with growing exponent"),
                Fractal::Sdrage {
                    max_iter: 500,
                    bailout: 10.,
                },
            ),
            (
                matches!(self.params.fractal, Fractal::Tdrge { .. }),
                "Tdrge",
                Some("third degree recursive sequence with growing exponent"),
                Fractal::Tdrge {
                    max_iter: 500,
                    bailout: 10.,
                },
            ),
            (
                matches!(self.params.fractal, Fractal::NthDrge { .. }),
                "NthDrge(n)",
                Some("nth degree recursive sequence with growing exponent"),
                Fractal::NthDrge {
                    max_iter: 500,
                    bailout: 10.,
                    n: 4,
                },
            ),
            (
                matches!(self.params.fractal, Fractal::ThirdDegreeRecPairs { .. }),
                "ThirdDegreeRecPairs",
                None,
                Fractal::ThirdDegreeRecPairs {
                    max_iter: 500,
                    bailout: 10.,
                },
            ),
            (
                matches!(
                    self.params.fractal,
                    Fractal::SecondDegreeThirtySevenBlend { .. }
                ),
                "SecondDegreeThirtySevenBlend",
                None,
                Fractal::SecondDegreeThirtySevenBlend {
                    max_iter: 500,
                    bailout: 10.,
                },
            ),
            (
                matches!(self.params.fractal, Fractal::ComplexLogisticMapLike { .. }),
                "ComplexLogisticMapLike(a_re, a_im)",
                None,
                Fractal::ComplexLogisticMapLike {
                    max_iter: 500,
                    bailout: 10.,
                    a_re: 1.,
                    a_im: 0.,
                },
            ),
            (
                matches!(self.params.fractal, Fractal::Vshqwj { .. }),
                "Vshqwj",
                None,
                Fractal::Vshqwj {
                    max_iter: 500,
                    bailout: 10.,
                },
            ),
            (
                matches!(self.params.fractal, Fractal::Wmriho { .. }),
                "Wmriho(a_re, a_im)",
                None,
                Fractal::Wmriho {
                    max_iter: 500,
                    bailout: 10.,
                    a_re: 0.,
                    a_im: 0.,
                },
            ),
            (
                matches!(self.params.fractal, Fractal::Iigdzh { .. }),
                "Iigdzh(a_re, a_im)",
                None,
                Fractal::Iigdzh {
                    max_iter: 500,
                    bailout: 10.,
                    a_re: 0.,
                    a_im: 0.,
                },
            ),
            (
                matches!(self.params.fractal, Fractal::Fxdicq { .. }),
                "Fxdicq",
                None,
                Fractal::Fxdicq {
                    max_iter: 500,
                    bailout: 10.,
                },
            ),
            (
                matches!(self.params.fractal, Fractal::Mjygzr { .. }),
                "Mjygzr",
                None,
                Fractal::Mjygzr {
                    max_iter: 500,
                    bailout: 10.,
                },
            ),
            (
                matches!(self.params.fractal, Fractal::Sfwypc { .. }),
                "Sfwypc(alpha, beta, gamma)",
                None,
                Fractal::Sfwypc {
                    max_iter: 500,
                    bailout: 10.,
                    alpha: (0., 0.),
                    beta: (0., 0.),
                    gamma: (0., 0.),
                },
            ),
            // (
            //     matches!(self.params.fractal, Fractal::Test { .. }),
            //     "Test",
            //     None,
            //     Fractal::Test {
            //         max_iter: 500,
            //         bailout: 10.,
            //     },
            // ),
        ]
        .iter()
        .any(|&(selected, name, description, default)| {
            let mut res = ui.selectable_label(selected, name);
            if let Some(description) = description {
                res = res.on_hover_text(description);
            }
            let clicked = res.clicked() && !selected;
            if clicked {
                self.params.fractal = default;
            }
            clicked
        })
    }
}
