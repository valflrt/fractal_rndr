use std::time::Duration;

use eframe::egui::{Color32, Direction, Image, Layout, ProgressBar, Ui};

use crate::gui::Gui;

impl Gui {
    pub fn show_preview(&mut self, ui: &mut Ui) {
        const INFO_AREA_HEIGHT: f32 = 48.;

        let texture_size = self.preview_texture.size_vec2();
        let d = 0.5 * (ui.available_height() - texture_size.y - INFO_AREA_HEIGHT);
        ui.add_space(d);
        ui.add_sized(
            texture_size,
            Image::from_texture((self.preview_texture.id(), texture_size))
                .show_loading_spinner(false)
                .maintain_aspect_ratio(true)
                .corner_radius(2),
        );
        ui.add_space(d);

        ui.with_layout(
            Layout::centered_and_justified(Direction::LeftToRight),
            |ui| {
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
            },
        );
    }
}
