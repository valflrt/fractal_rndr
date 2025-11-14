use eframe::egui::{Image, Rect, Sense, Ui};

use crate::{gui::Gui, F};

impl Gui {
    pub fn show_preview(&mut self, ui: &mut Ui) {
        let (rect, res) = ui.allocate_exact_size(ui.available_size(), Sense::drag());

        let texture_size = self.preview_texture.size_vec2();

        ui.place(
            Rect::from_center_size(rect.center(), texture_size),
            Image::from_texture((self.preview_texture.id(), texture_size))
                .show_loading_spinner(false)
                .maintain_aspect_ratio(true),
        );

        let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);

        if res.hovered() && scroll_delta != 0. {
            let zoom_factor = 1. - scroll_delta as F * 0.005;
            self.params.zoom *= zoom_factor;

            self.params_changes.set_breaking();
        }

        if res.dragged() {
            let drag_delta = res.drag_delta();
            let height = rect.height();

            let rotate = self.params.rotate.unwrap_or(0.);
            let rotate_cos = rotate.cos();
            let rotate_sin = rotate.sin();

            let drag_delta_x = self.params.zoom * (drag_delta.x / height) as F;
            let drag_delta_y = self.params.zoom * (drag_delta.y / height) as F;

            self.params.center_x -= rotate_cos * drag_delta_x - rotate_sin * drag_delta_y;
            self.params.center_y += rotate_sin * drag_delta_x + rotate_cos * drag_delta_y;

            self.params_changes.set_breaking();
        }
    }
}
