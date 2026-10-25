use eframe::egui;

use super::super::{HeaderAction, ModalType, Z80App};
use crate::gui::exec::{Command, Runner};

pub(crate) fn show(
    app: &mut Z80App,
    frame: &mut eframe::Frame,
    ui: &mut egui::Ui,
) -> Option<HeaderAction> {
    // `frame` is only needed by the wasm-only Examples menu.
    #[cfg(not(target_arch = "wasm32"))]
    let _ = frame;

    let mut action = None;

    egui::MenuBar::new().ui(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui
                .add(egui::Button::new("New").shortcut_text("Ctrl+N"))
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                action = Some(HeaderAction::NewFile);
                ui.close();
            }
            if ui
                .add(egui::Button::new("Open...").shortcut_text("Ctrl+O"))
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                action = Some(HeaderAction::OpenFileDialog);
                ui.close();
            }

            ui.menu_button("Open Recent", |ui| {
                if app.workspace.recent_files.is_empty() {
                    ui.label("No recent files");
                } else {
                    let mut to_open = None;
                    for path in &app.workspace.recent_files {
                        if ui
                            .button(path.display().to_string())
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            to_open = Some(path.clone());
                        }
                    }
                    if let Some(path) = to_open {
                        action = Some(HeaderAction::OpenFile(path));
                        ui.close();
                    }
                }
            })
            .response
            .on_hover_cursor(egui::CursorIcon::PointingHand);

            ui.separator();

            if ui
                .add(egui::Button::new("Save").shortcut_text("Ctrl+S"))
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                action = Some(HeaderAction::SaveFile);
                ui.close();
            }
            if ui
                .add(egui::Button::new("Save As...").shortcut_text("Ctrl+Shift+S"))
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                action = Some(HeaderAction::SaveFileAs);
                ui.close();
            }

            ui.separator();
            if ui
                .button("Assemble Binary...")
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                action = Some(HeaderAction::AssembleBinary);
                ui.close();
            }
            if ui
                .button("Load Binary…")
                .on_hover_text("Open a .bin/.rom image as a read-only tab")
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                action = Some(HeaderAction::LoadBinary);
                ui.close();
            }

            ui.separator();
            if ui
                .button("Quit")
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                action = Some(HeaderAction::Quit);
                ui.close();
            }
        })
        .response
        .on_hover_cursor(egui::CursorIcon::PointingHand);

        ui.menu_button("View", |ui| {
            ui.checkbox(&mut app.workspace.show_location_counter, "Location Counter");
            ui.checkbox(&mut app.show_memory_viewer, "Memory Viewer");
            ui.separator();
            if ui.button("Keyboard Shortcuts…").clicked() {
                app.show_shortcuts = true;
                ui.close();
            }
        })
        .response
        .on_hover_cursor(egui::CursorIcon::PointingHand);

        ui.menu_button("Devices", |ui| {
            ui.label(egui::RichText::new("Add").strong());
            for definition in crate::components::devices::device_definitions() {
                if ui
                    .button(definition.menu_name)
                    .on_hover_text(format!("Attach on I/O port {}", definition.default_port))
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    app.pending_modal = Some(ModalType::AddDevice(
                        definition,
                        definition.default_port.to_string(),
                    ));
                    ui.close();
                }
            }

            if !app.attached_devices.is_empty() {
                ui.separator();
                ui.label(egui::RichText::new("Attached").strong());
                let mut remove = None;
                for (idx, dev) in app.attached_devices.iter().enumerate() {
                    let name = dev.lock().unwrap().get_name();
                    ui.horizontal(|ui| {
                        let mut dev_ref = dev.lock().unwrap();
                        let mut open = dev_ref.get_window_open_state();
                        if ui.checkbox(&mut open, &name).changed() {
                            dev_ref.set_window_open_state(open);
                        }
                        drop(dev_ref);
                        if ui
                            .small_button("🗑")
                            .on_hover_text(format!("Remove {name}"))
                            .clicked()
                        {
                            remove = Some(idx);
                        }
                    });
                }
                if let Some(idx) = remove {
                    app.remove_device(idx);
                    ui.close();
                }
            }
        })
        .response
        .on_hover_cursor(egui::CursorIcon::PointingHand);

        ui.menu_button("Memory", |ui| {
            if ui
                .checkbox(
                    &mut app.show_memory_viewer,
                    "Memory Viewer Window (read-only)",
                )
                .changed()
            {
                ui.close();
            }
        })
        .response
        .on_hover_cursor(egui::CursorIcon::PointingHand);

        #[cfg(target_arch = "wasm32")]
        {
            ui.menu_button("Examples", |ui| {
                if app.examples_list.is_empty() {
                    let message = if app.files.examples_pending() {
                        "Loading examples…"
                    } else {
                        "No examples found"
                    };
                    ui.label(message);
                } else {
                    let mut to_load = None;
                    for ex in &app.examples_list {
                        if ui
                            .button(ex)
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            to_load = Some(ex.clone());
                        }
                    }
                    if let Some(ex) = to_load {
                        app.load_example(ex, ui.ctx(), frame.storage_mut());
                        ui.close();
                    }
                }
            })
            .response
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        }

        ui.menu_button("Help", |ui| {
            if ui.button("Keyboard Shortcuts…").clicked() {
                app.show_shortcuts = true;
                ui.close();
            }
            ui.separator();
            ui.label(format!("JAZZ80 {}", env!("CARGO_PKG_VERSION")));
            ui.label("Z80 simulator");
        })
        .response
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    });
    ui.separator();

    // Toolbar
    ui.horizontal_wrapped(|ui| {
        if ui
            .button("⟳ Assemble & Reset")
            .on_hover_text(
                "Assemble the active source, reset the CPU and load the program at 0x0000",
            )
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
        {
            app.last_error = None;
            app.load_and_reset();
        }

        ui.separator();

        let asset_ok = app.workspace.program_loaded || app.session.is_running();
        let step_resp = ui
            .add_enabled(asset_ok, egui::Button::new("⏭ Step"))
            .on_hover_text(if asset_ok {
                "Execute one instruction (F8)"
            } else {
                "Assemble or load a program first"
            })
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        if step_resp.clicked() {
            app.step_instruction();
        }

        let halted = app.session.snapshot().halted;
        let run_label = if app.session.is_running() {
            "⏸ Stop"
        } else if halted {
            "▶ Resume"
        } else if app.session.snapshot().pc > 0 {
            "▶ Continue"
        } else {
            "▶ Run"
        };
        let run_resp = ui
            .add_enabled(asset_ok, egui::Button::new(run_label))
            .on_hover_text(if asset_ok {
                "Run until a breakpoint or HALT (F5)"
            } else {
                "Assemble or load a program first"
            })
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        if run_resp.clicked() {
            app.toggle_execution();
        }

        ui.separator();

        if ui
            .button("⚡ INT")
            .on_hover_text("Raise a maskable interrupt")
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
        {
            if app.session.is_running() {
                app.session
                    .runner_mut()
                    .send_command(Command::PulseInterrupt);
            } else if let Some(machine) = app.session.machine_mut() {
                machine.cpu.pulse_interrupt();
                app.session.refresh_snapshot();
            }
        }

        if ui
            .button("⚡ NMI")
            .on_hover_text("Raise a non-maskable interrupt")
            .on_hover_cursor(egui::CursorIcon::PointingHand)
            .clicked()
        {
            if app.session.is_running() {
                app.session.runner_mut().send_command(Command::Nmi);
            } else if let Some(machine) = app.session.machine_mut() {
                machine.cpu.nmi();
                app.session.refresh_snapshot();
            }
        }

        ui.separator();

        let (state_text, state_color) = if app.session.snapshot().halted {
            ("HALTED", app.theme.stale)
        } else if app.session.is_running() {
            ("RUNNING", app.theme.success)
        } else if app.session.last_stop_addr().is_some() {
            ("AT BREAKPOINT", app.theme.warning)
        } else {
            ("PAUSED", app.theme.muted)
        };
        ui.label(egui::RichText::new(state_text).color(state_color).strong());
        if let Some(addr) = app.session.last_stop_addr() {
            ui.label(
                egui::RichText::new(format!("0x{addr:04X}"))
                    .monospace()
                    .color(app.theme.warning),
            );
        }

        if app.session.snapshot().halted && ui.button("Resume").clicked() {
            app.resume_from_halt();
        }
    });

    action
}
