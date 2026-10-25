use eframe::egui;

use super::super::{DeviceSpec, ModalType, Z80App, parse_hex_u16};

pub(crate) fn show(app: &mut Z80App, ctx: &egui::Context, frame: &mut eframe::Frame) {
    if let Some(mut modal_type) = app.pending_modal.take() {
        let mut should_close_modal = false;

        let title = match &modal_type {
            ModalType::AddDevice(_, _) => "Attach Device",
            ModalType::CloseTab(_) => "Close Unsaved Tab",
            ModalType::Quit => "Quit JAZZ80",
        };

        let modal = egui::Modal::new(egui::Id::unique("modal")).show(ctx, |ui| {
            ui.set_width(380.0);
            ui.heading(title);
            ui.separator();

            match &mut modal_type {
                ModalType::AddDevice(definition, port_text) => {
                    ui.label("I/O port (hexadecimal):");
                    ui.add(
                        egui::TextEdit::singleline(port_text)
                            .hint_text("e.g. 20")
                            .desired_width(110.0),
                    );
                    let parsed = parse_hex_u16(port_text);
                    if parsed.is_none() {
                        ui.label(
                            egui::RichText::new("Enter a hexadecimal port, e.g. 20")
                                .small()
                                .color(app.theme.danger),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("Devices listen on the low byte of the port.")
                                .small()
                                .color(app.theme.muted),
                        );
                    }
                    if let Some(port) = parsed
                        && port > 0xFF
                    {
                        ui.label(
                            egui::RichText::new("Port must be 0–255")
                                .small()
                                .color(app.theme.danger),
                        );
                    }

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        let valid_port = parse_hex_u16(port_text).filter(|port| *port <= 0xFF);
                        if ui
                            .add_enabled(valid_port.is_some(), egui::Button::new("Add"))
                            .clicked()
                        {
                            let port = valid_port.unwrap();
                            let dev = (definition.create)(port);
                            if let Some(machine) = app.session.machine_mut() {
                                machine.cpu.attach_device(dev.clone());
                            }
                            app.attached_devices.push(dev);
                            app.attached_device_specs.push(DeviceSpec {
                                menu_name: definition.menu_name.to_string(),
                                port,
                            });
                            should_close_modal = true;
                        }
                        if ui.button("Cancel").clicked() {
                            should_close_modal = true;
                        }
                    });
                }
                ModalType::CloseTab(idx) => {
                    let idx = *idx;
                    let name = app
                        .workspace
                        .tabs
                        .get(idx)
                        .and_then(|t| t.path.as_ref())
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .unwrap_or("Untitled");

                    ui.label(format!(
                        "Save the changes you made to \"{name}\" before closing?"
                    ));
                    ui.label(
                        egui::RichText::new("Your changes will be lost if you don't save them.")
                            .color(app.theme.muted),
                    );

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            app.save_file(idx, frame.storage_mut());
                            if app.workspace.tabs.get(idx).is_some_and(|t| !t.is_dirty) {
                                app.close_tab(idx, frame.storage_mut());
                                should_close_modal = true;
                            }
                        }
                        if ui.button("Don't Save").clicked() {
                            if let Some(tab) = app.workspace.tabs.get_mut(idx) {
                                tab.is_dirty = false;
                            }
                            app.close_tab(idx, frame.storage_mut());
                            should_close_modal = true;
                        }
                        if ui.button("Cancel").clicked() {
                            should_close_modal = true;
                        }
                    });
                }
                ModalType::Quit => {
                    ui.label("You have unsaved changes in this workspace.");
                    ui.label("Save everything before quitting?");

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if ui.button("Save All").clicked() && app.save_all(frame.storage_mut()) {
                            should_close_modal = true;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("Quit Without Saving").clicked() {
                            for tab in &mut app.workspace.tabs {
                                tab.is_dirty = false;
                            }
                            should_close_modal = true;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("Cancel").clicked() {
                            should_close_modal = true;
                        }
                    });
                }
            }
        });

        if modal.should_close() {
            should_close_modal = true;
        }

        if !should_close_modal {
            app.pending_modal = Some(modal_type);
        }
    }
}
