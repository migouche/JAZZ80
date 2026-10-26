use eframe::egui::{self, TextBuffer};

use super::super::theme::AppTheme;
use super::super::{EditorTabKind, HeaderAction, Z80App};

pub(crate) fn show(app: &mut Z80App, ui: &mut egui::Ui) -> Option<HeaderAction> {
    let action = show_tab_bar(app, ui);
    ui.separator();
    show_title_row(app, ui);
    show_listing(app, ui);
    action
}

fn show_tab_bar(app: &mut Z80App, ui: &mut egui::Ui) -> Option<HeaderAction> {
    let mut action = None;
    // Tab Bar
    ui.horizontal(|ui| {
        egui::ScrollArea::horizontal()
            .id_salt("tabs_scroll")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let mut to_activate = None;
                    let mut to_close = None;

                    for (i, tab) in app.workspace.tabs.iter().enumerate() {
                        let name = tab
                            .path
                            .as_ref()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .unwrap_or("Untitled");

                        let is_active = i == app.workspace.active_tab;

                        // Visuals
                        let bg_color = if is_active {
                            app.theme.tab_active_bg
                        } else {
                            ui.visuals().faint_bg_color
                        };
                        let fg_color = if is_active {
                            app.theme.tab_active_fg
                        } else {
                            ui.visuals().text_color()
                        };

                        let resp = egui::Frame::new()
                            .fill(bg_color)
                            .inner_margin(egui::Margin::symmetric(
                                AppTheme::TAB_MARGIN_X,
                                AppTheme::TAB_MARGIN_Y,
                            ))
                            .corner_radius(egui::CornerRadius {
                                nw: AppTheme::TAB_CORNER_RADIUS,
                                ne: AppTheme::TAB_CORNER_RADIUS,
                                ..Default::default()
                            })
                            .stroke(egui::Stroke::new(
                                1.0_f32,
                                ui.visuals().widgets.noninteractive.bg_stroke.color,
                            ))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    if ui
                                        .add(
                                            egui::Label::new(
                                                egui::RichText::new(name).color(fg_color),
                                            )
                                            .sense(egui::Sense::click()),
                                        )
                                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                                        .clicked()
                                    {
                                        to_activate = Some(i);
                                    }

                                    if tab.is_dirty {
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new("●")
                                                    .size(AppTheme::DIRTY_DOT_FONT_SIZE),
                                            )
                                            .sense(egui::Sense::hover()),
                                        );
                                    }

                                    // Close button (x)
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("×")
                                                    .size(AppTheme::TAB_CLOSE_FONT_SIZE),
                                            )
                                            .frame(false),
                                        )
                                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                                        .clicked()
                                    {
                                        to_close = Some(i);
                                    }
                                });
                            });

                        if resp.response.clicked() {
                            to_activate = Some(i);
                        }
                        if let Some(path) = tab.path.as_ref() {
                            resp.response.on_hover_text(path.display().to_string());
                        }

                        ui.add_space(2.0);
                    }

                    if ui
                        .button("+")
                        .on_hover_text(format!(
                            "New file ({})",
                            HeaderAction::NewFile.shortcut_text(ui.ctx())
                        ))
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        action = Some(HeaderAction::NewFile);
                    }

                    if let Some(i) = to_activate
                        && app.workspace.active_tab != i
                    {
                        app.workspace.active_tab = i;
                        app.workspace.is_assembly_stale = true;
                    }
                    if let Some(i) = to_close {
                        action = Some(HeaderAction::CloseTab(i));
                    }
                });
            });
    });
    action
}

fn show_title_row(app: &Z80App, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let is_binary = app
            .workspace
            .tabs
            .get(app.workspace.active_tab)
            .is_some_and(|t| t.kind == EditorTabKind::Binary);
        ui.heading(if is_binary {
            "Binary Image"
        } else {
            "Assembly Source"
        });
        if is_binary {
            ui.label(
                egui::RichText::new("read-only hexdump")
                    .small()
                    .color(app.theme.muted),
            )
            .on_hover_text(
                "These are the bytes loaded into memory. Use File → Assemble Binary to export a .bin.",
            );
        }
        if app.workspace.is_assembly_stale {
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("⚠ Out of date (assemble to refresh)")
                    .color(app.theme.stale)
                    .small(),
            )
            .on_hover_text(
                "The source changed since the last assemble. Run still executes the old program.",
            );
        }
    });
}

fn show_listing(app: &mut Z80App, ui: &mut egui::Ui) {
    if app.workspace.active_tab < app.workspace.tabs.len() {
        let highlight_line = if !app.session.is_running() || app.session.snapshot().halted {
            let pc = app.session.snapshot().pc;
            if app.session.snapshot().halted {
                let mut best_match = None;
                let mut max_addr = -1i32;
                for (&addr, &line) in &app.workspace.address_to_line {
                    if addr < pc && (addr as i32) > max_addr {
                        max_addr = addr as i32;
                        best_match = Some(line);
                    }
                }
                best_match.or_else(|| app.workspace.address_to_line.get(&pc).copied())
            } else {
                app.workspace.address_to_line.get(&pc).copied()
            }
        } else {
            None
        };

        let tab_id = app.workspace.tabs[app.workspace.active_tab].id;
        let current_tab = &mut app.workspace.tabs[app.workspace.active_tab];

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal_top(|ui| {
                let num_lines = if current_tab.code.is_empty() {
                    1
                } else {
                    current_tab.code.lines().count()
                        + if current_tab.code.ends_with('\n') {
                            1
                        } else {
                            0
                        }
                };

                let font_id = egui::FontId::monospace(AppTheme::MONO_FONT_SIZE);

                // 1. Line numbers column
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;

                    // Header for Line
                    ui.label(
                        egui::RichText::new("Line")
                            .font(font_id.clone())
                            .strong()
                            .color(app.theme.muted),
                    );

                    for i in 1..=num_lines {
                        let is_bp = current_tab.breakpoints.contains(&i);
                        let is_pc = highlight_line == Some(i);
                        let bg = if is_bp {
                            app.theme.breakpoint
                        } else if is_pc {
                            app.theme.pc_row
                        } else {
                            egui::Color32::TRANSPARENT
                        };
                        let text_color = if is_bp {
                            app.theme.breakpoint_text
                        } else if is_pc {
                            app.theme.pc_marker
                        } else {
                            app.theme.gutter
                        };

                        let marker = if is_pc { "▶" } else { " " };
                        let label = egui::Label::new(
                            egui::RichText::new(format!("{}{: >4}", marker, i))
                                .font(font_id.clone())
                                .color(text_color)
                                .background_color(bg),
                        )
                        .sense(egui::Sense::click());

                        if ui
                            .add(label)
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .on_hover_text(if is_bp {
                                "Breakpoint set — click to remove"
                            } else {
                                "Click to set a breakpoint"
                            })
                            .clicked()
                        {
                            // Toggle breakpoint
                            if is_bp {
                                current_tab.breakpoints.remove(&i);
                            } else {
                                current_tab.breakpoints.insert(i);
                            }
                        }
                    }
                });

                // 2. Location Counter Columns
                if app.workspace.show_location_counter {
                    ui.add_space(AppTheme::COLUMN_GAP);

                    // Address Column
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        ui.label(
                            egui::RichText::new("Address")
                                .font(font_id.clone())
                                .strong()
                                .color(app.theme.muted),
                        );

                        let addr_color = if app.workspace.is_assembly_stale {
                            app.theme.stale
                        } else {
                            app.theme.address
                        };

                        for i in 1..=num_lines {
                            let mut addr_str = "    ".to_string();

                            if let Some(line) = app.workspace.assembly_lines.get(&i)
                                && !line.bytes.is_empty()
                            {
                                addr_str = format!("{:04X}", line.address);
                            }

                            let label = egui::Label::new(
                                egui::RichText::new(addr_str)
                                    .font(font_id.clone())
                                    .color(addr_color),
                            );
                            let resp = ui.add(label);

                            if app.workspace.is_assembly_stale {
                                resp.on_hover_text(
                                    "Location counter is out of date. Assemble again to refresh.",
                                );
                            }
                        }
                    });

                    ui.add_space(AppTheme::COLUMN_GAP);

                    // Data Column
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        ui.label(
                            egui::RichText::new("Data")
                                .font(font_id.clone())
                                .strong()
                                .color(app.theme.muted),
                        );

                        let data_color = if app.workspace.is_assembly_stale {
                            app.theme.stale
                        } else {
                            app.theme.data
                        };

                        for i in 1..=num_lines {
                            let data = app
                                .workspace
                                .assembly_lines
                                .get(&i)
                                .map_or("", |line| line.data_column.as_str());
                            let label = egui::Label::new(
                                egui::RichText::new(format!("{data: <14}"))
                                    .font(font_id.clone())
                                    .color(data_color),
                            );
                            ui.add(label);
                        }
                    });

                    ui.add_space(AppTheme::COLUMN_GAP);
                }
                // 3. Code Editor Column
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    // Add a matching header so the text edit component starts exactly on the same row!
                    ui.label(
                        egui::RichText::new("Code")
                            .font(font_id.clone())
                            .strong()
                            .color(app.theme.muted),
                    );

                    let theme = app.theme;
                    let cache = &mut app.highlight_cache;
                    let mut layouter =
                        |ui: &egui::Ui, string: &dyn TextBuffer, _wrap_width: f32| {
                            cache.galley(ui, &theme, string.as_str(), highlight_line)
                        };

                    let is_binary_tab = current_tab.kind == EditorTabKind::Binary;

                    // Long lines are reachable via a horizontal scrollbar
                    // instead of wrapping: wrapping would add visual rows
                    // and desynchronise the line-number/address gutters.
                    let font_id = egui::FontId::monospace(AppTheme::MONO_FONT_SIZE);
                    let longest = current_tab
                        .code
                        .split('\n')
                        .map(|line| line.chars().count())
                        .max()
                        .unwrap_or(0);
                    let text_width = ui
                        .painter()
                        .layout_no_wrap(
                            "M".repeat(longest.max(1)),
                            font_id,
                            egui::Color32::PLACEHOLDER,
                        )
                        .size()
                        .x;
                    let desired_width = (text_width + 24.0).max(ui.available_width());

                    egui::ScrollArea::horizontal()
                        .id_salt(("code_hscroll", tab_id))
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            ui.push_id(tab_id, |ui| {
                                let editor = if is_binary_tab {
                                    egui::TextEdit::multiline(&mut current_tab.code)
                                        .font(egui::TextStyle::Monospace)
                                        .layouter(&mut layouter)
                                        .code_editor()
                                        .margin(egui::Margin::symmetric(
                                            AppTheme::EDITOR_MARGIN_X,
                                            0,
                                        ))
                                        .desired_width(desired_width)
                                        .desired_rows(25)
                                        .lock_focus(true)
                                        .interactive(false)
                                } else {
                                    egui::TextEdit::multiline(&mut current_tab.code)
                                        .font(egui::TextStyle::Monospace)
                                        .layouter(&mut layouter)
                                        .code_editor()
                                        .margin(egui::Margin::symmetric(
                                            AppTheme::EDITOR_MARGIN_X,
                                            0,
                                        ))
                                        .desired_width(desired_width)
                                        .desired_rows(25)
                                        .lock_focus(true)
                                };

                                if ui.add(editor).changed() && !is_binary_tab {
                                    current_tab.is_dirty = true;
                                    app.workspace.is_assembly_stale = true;
                                }
                            });
                        });
                });
            });
        });
    } else {
        ui.label("No files open");
    }
}
