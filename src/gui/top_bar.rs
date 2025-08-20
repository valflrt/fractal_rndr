use eframe::egui::{Button, ScrollArea, Ui, ViewportCommand};

use crate::{
    error::ErrorKind,
    gui::{FileDialogAction, FileDialogKind, Gui},
    params::ParamsKind,
    presets::PRESETS,
};

impl Gui {
    pub fn show_top_bar(&mut self, ui: &mut Ui) {
        ui.menu_button("file", |ui| {
            if ui
                .add_enabled(
                    self.file_dialog_handle.is_none(),
                    Button::new("open parameter file"),
                )
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
                ScrollArea::vertical()
                    .max_width(200.)
                    .max_height(100.)
                    .show(ui, |ui| {
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
                    })
            });

            ui.separator();

            ui.add_enabled_ui(self.param_file_path.is_some(), |ui| {
                let res = ui.button("overwrite parameter file");
                let res = if self.param_file_path.is_none() {
                    res.on_disabled_hover_text("no path was provided for the parameter file")
                } else {
                    res
                };
                if res.clicked() {
                    self.save_parameter_file();
                }
            });

            if ui.button("save parameter file as ...").clicked() {
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

            if ui.button("revert unsaved changes").clicked() {
                self.params = self.last_saved_params.clone();
                self.params_changes.set_breaking();
            }

            ui.separator();

            if ui.button("exit").clicked() {
                ui.ctx().send_viewport_cmd(ViewportCommand::Close);
            }
        });

        ui.menu_button("output", |ui| {
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
    }
}
