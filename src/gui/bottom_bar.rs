use std::time::Duration;

use eframe::egui::{Align, Color32, Label, Layout, ProgressBar, Ui};

use crate::gui::Gui;

impl Gui {
    pub fn show_bottom_bar(&mut self, ui: &mut Ui) {
        fn special_label(text: &str) -> Label {
            Label::new(text).selectable(false)
        }

        if let Some(path) = &self.param_file_path {
            let text = if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                format!("parameter file: {}", file_name)
            } else {
                "parameter file".to_string()
            };
            ui.add(special_label(&text))
                .on_hover_text(path.to_string_lossy());
        } else {
            let _ = ui
                .add(special_label("no parameter file"))
                .on_hover_text("first load parameter file from menu");
        }

        ui.separator();

        if let Some(path) = &self.output_image_path {
            let text = if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                format!("output image: {}", file_name)
            } else {
                "output image".to_string()
            };
            ui.add(special_label(&text))
                .on_hover_text(path.to_string_lossy());
        } else {
            let _ = ui
                .add(special_label("no output image"))
                .on_hover_text("first set output image from menu");
        }

        ui.separator();

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
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
        });
    }
}
