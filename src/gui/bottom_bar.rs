use std::time::Duration;

use eframe::egui::{
    containers::menu::{menu_style, MenuButton},
    Align, Button, Color32, IntoAtoms, Layout, Popup, PopupAnchor, PopupKind, ProgressBar,
    RectAlign, Response, RichText, Shadow, Style, Ui, Widget,
};

use crate::{
    error::ErrorKind,
    gui::{FileDialogAction, FileDialogKind, Gui},
    params::ParamsKind,
    presets::PRESETS,
};

impl Gui {
    pub fn show_bottom_bar(&mut self, ui: &mut Ui) {
        ui.visuals_mut().button_frame = false;

        self.show_parameter_menu(ui);

        ui.separator();

        self.show_image_menu(ui);

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

    fn show_parameter_menu(&mut self, ui: &mut Ui) {
        let current_param_file_path = self.param_file_path.to_owned();
        let param_file_btn_text = current_param_file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .map(|file_name| ("parameters: ", RichText::new(file_name).monospace()).into_atoms())
            .unwrap_or("parameters".into_atoms());
        let param_file_btn_hover_text = current_param_file_path
            .as_ref()
            .and_then(|p| p.to_str())
            .unwrap_or("first load parameter file from menu");

        let res = custom_show_menu_button(ui, MenuButton::new(param_file_btn_text), |ui| {
            if ui
                .add_enabled(self.file_dialog_handle.is_none(), Button::new("open"))
                .clicked()
            {
                self.open_file_dialog(
                    self.param_file_path
                        .as_ref()
                        .or(self.output_image_path.as_ref())
                        .and_then(|p| p.parent())
                        .map(|p| p.to_path_buf()),
                    FileDialogKind::PickFile,
                    FileDialogAction::OpenParameterFile,
                );
            }
            ui.menu_button("load preset", |ui| {
                for &(name, cfg_file) in PRESETS {
                    if let ParamsKind::Frame(params) = ron::from_str(cfg_file)
                        .map_err(ErrorKind::DecodeParameterFile)
                        .unwrap()
                    {
                        if ui.button(name).clicked() {
                            self.params = params;
                            self.params_changes.set_breaking();
                            self.notify(format!("loaded {}", name));
                            ui.close();
                        };
                    }
                }
            });

            ui.separator();

            if ui
                .add_enabled(
                    self.param_file_path.is_some(),
                    Button::new("overwrite parameter file"),
                )
                .on_disabled_hover_text("please open a parameter file first")
                .clicked()
            {
                self.save_parameter_file();
            }

            if ui.button("save as ...").clicked() {
                self.open_file_dialog(
                    self.param_file_path
                        .as_ref()
                        .or(self.output_image_path.as_ref())
                        .and_then(|p| p.parent())
                        .map(|p| p.to_path_buf()),
                    FileDialogKind::SaveFile,
                    FileDialogAction::SaveParameterFileAs,
                );
            }

            ui.separator();

            if ui
                .add_enabled(
                    self.params != self.last_saved_params,
                    Button::new("revert unsaved changes"),
                )
                .clicked()
            {
                self.params = self.last_saved_params.clone(); // FIXME
                self.params_changes.set_breaking();
            }
        });

        res.on_hover_text(param_file_btn_hover_text);
    }

    fn show_image_menu(&mut self, ui: &mut Ui) {
        let current_output_image_path = self.output_image_path.to_owned();
        let output_image_btn_text = current_output_image_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .map(|file_name| ("output image: ", RichText::new(file_name).monospace()).into_atoms())
            .unwrap_or("output image".into_atoms());
        let output_image_btn_hover_text = current_output_image_path
            .as_ref()
            .and_then(|p| p.to_str())
            .unwrap_or("first load parameter file from menu");

        let res = custom_show_menu_button(ui, MenuButton::new(output_image_btn_text), |ui| {
            if ui
                .add_enabled(
                    self.file_dialog_handle.is_none(),
                    Button::new("set output image"),
                )
                .clicked()
            {
                self.open_file_dialog(
                    self.output_image_path
                        .as_ref()
                        .or(self.param_file_path.as_ref())
                        .and_then(|p| p.parent())
                        .map(|p| p.to_path_buf()),
                    FileDialogKind::SaveFile,
                    FileDialogAction::SaveOutputImage,
                );
            }
        });

        res.on_hover_text(output_image_btn_hover_text);
    }
}

pub fn custom_show_menu_button<R>(
    ui: &mut Ui,
    menu_button: MenuButton,
    content: impl FnOnce(&mut Ui) -> R,
) -> Response {
    let response = menu_button.button.ui(ui);

    Popup::from_toggle_button_response(&response)
        .kind(PopupKind::Menu)
        .layout(Layout::top_down_justified(Align::Min))
        .align(RectAlign::TOP)
        .align_alternatives(&[RectAlign::TOP_END])
        .anchor(PopupAnchor::Position(response.rect.center_top()))
        .gap(1.)
        .style(|style: &mut Style| {
            menu_style(style);
            style.visuals.popup_shadow = Shadow::NONE;
            style.visuals.menu_corner_radius.sw = 0;
            style.visuals.menu_corner_radius.se = 0;
        })
        .show(content);

    response
}
