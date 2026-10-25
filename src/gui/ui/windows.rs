use eframe::egui;

use super::super::{Z80App, parse_hex_u16};

pub(crate) fn show_shortcuts(app: &mut Z80App, ctx: &egui::Context) {
    if app.show_shortcuts {
        egui::Window::new("Keyboard Shortcuts")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .open(&mut app.show_shortcuts)
            .show(ctx, |ui| {
                let rows = [
                    ("Run / Stop", "F5"),
                    ("Step one instruction", "F8"),
                    ("New file", "Ctrl+N"),
                    ("Open file", "Ctrl+O"),
                    ("Save", "Ctrl+S"),
                    ("Save as", "Ctrl+Shift+S"),
                    ("Close tab", "Ctrl+W"),
                    ("Toggle breakpoint", "Click the gutter"),
                ];
                egui::Grid::new("shortcuts_grid")
                    .num_columns(2)
                    .spacing([24.0, 6.0])
                    .striped(true)
                    .show(ui, |ui| {
                        for (action, keys) in rows {
                            ui.label(action);
                            ui.monospace(keys);
                            ui.end_row();
                        }
                    });
            });
    }
}

pub(crate) fn show_memory_viewer(app: &mut Z80App, ui: &mut egui::Ui) {
    let mut show_mem = app.show_memory_viewer;
    if show_mem {
        egui::Window::new("Memory Viewer")
            .open(&mut show_mem)
            .resizable(true)
            .default_width(560.0)
            .default_height(430.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Start (hex):");
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut app.memory_viewer_start)
                            .desired_width(64.0)
                            .font(egui::TextStyle::Monospace)
                            .hint_text("0000"),
                    );
                    if ui
                        .small_button("◀")
                        .on_hover_text("Previous page (−0x100)")
                        .clicked()
                    {
                        let start = parse_hex_u16(&app.memory_viewer_start).unwrap_or(0);
                        app.memory_viewer_start = format!("{:04X}", start.wrapping_sub(0x100));
                    }
                    if ui
                        .small_button("▶")
                        .on_hover_text("Next page (+0x100)")
                        .clicked()
                    {
                        let start = parse_hex_u16(&app.memory_viewer_start).unwrap_or(0);
                        app.memory_viewer_start = format!("{:04X}", start.wrapping_add(0x100));
                    }
                    if ui
                        .small_button("Goto PC")
                        .on_hover_text("Jump to the program counter")
                        .clicked()
                    {
                        app.memory_viewer_start = format!("{:04X}", app.session.snapshot().pc);
                        response.request_focus();
                    }
                    ui.checkbox(&mut app.memory_follow_pc, "Follow PC")
                        .on_hover_text("Scroll along with the program counter");
                    ui.label(
                        egui::RichText::new("read-only")
                            .small()
                            .color(app.theme.muted),
                    );
                });

                let valid = parse_hex_u16(&app.memory_viewer_start).is_some();
                if !valid {
                    ui.label(
                        egui::RichText::new("Enter a hexadecimal address (0000–FFFF)")
                            .small()
                            .color(app.theme.danger),
                    );
                }

                ui.separator();

                if app.memory_follow_pc {
                    app.memory_viewer_start = format!("{:04X}", app.session.snapshot().pc);
                }
                let start_addr = parse_hex_u16(&app.memory_viewer_start).unwrap_or(0);
                let pc = app.session.snapshot().pc;

                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("mem_dump_grid")
                        .num_columns(3)
                        .spacing([20.0, 4.0])
                        .show(ui, |ui| {
                            for r in 0..16u16 {
                                let row_addr = start_addr.wrapping_add(r * 16);
                                let has_pc = pc.wrapping_sub(row_addr) < 16;
                                let addr_text = egui::RichText::new(format!("{row_addr:04X}"))
                                    .monospace()
                                    .color(if has_pc {
                                        app.theme.pc_marker
                                    } else {
                                        app.theme.address
                                    });
                                ui.label(addr_text);

                                let mut hex_str = String::new();
                                let mut ascii_str = String::new();
                                for c in 0..16u16 {
                                    let addr = row_addr.wrapping_add(c);
                                    let val = app.session.snapshot().memory[addr as usize];
                                    hex_str.push_str(&format!("{:02X} ", val));
                                    if (32..=126).contains(&val) {
                                        ascii_str.push(val as char);
                                    } else {
                                        ascii_str.push('.');
                                    }
                                }

                                ui.monospace(hex_str)
                                    .on_hover_text("Memory is read-only in this view");
                                ui.monospace(ascii_str);
                                ui.end_row();
                            }
                        });
                });
            });
        app.show_memory_viewer = show_mem;
    }
}

pub(crate) fn show_devices(app: &mut Z80App, ctx: &egui::Context) {
    for device in &app.attached_devices {
        device.lock().unwrap().draw(ctx);
    }
}
