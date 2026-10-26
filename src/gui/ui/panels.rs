use crate::assembler::SymbolType;
use crate::cpu::{Flag, GPR};
use eframe::egui;

use super::super::{
    SymbolDisplayFormat, Z80App, flag_description, format_constant_value, gpr_index, parse_hex_u8,
    parse_hex_u16, snapshot_flag, theme::AppTheme,
};

pub(crate) fn show_registers(app: &mut Z80App, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.heading("Registers");
        ui.label(egui::RichText::new("hex").small().color(app.theme.muted))
            .on_hover_text("All values are hexadecimal. A 0x prefix is accepted.");
    });
    ui.separator();

    // While a run is in progress the machine lives in the runner thread and
    // `exec.machine` is `None`, so edits would be silently dropped.
    let can_edit = app.session.machine().is_some();
    if !can_edit {
        ui.label(
            egui::RichText::new("Stop the CPU to edit registers")
                .small()
                .color(app.theme.muted),
        );
    }

    // Copy the small scalar fields instead of cloning the whole snapshot
    // (which carries a 64 KiB memory buffer) every frame.
    let pc = app.session.snapshot().pc;
    let sp = app.session.snapshot().sp;
    let ix = app.session.snapshot().ix;
    let iy = app.session.snapshot().iy;
    let iff1 = app.session.snapshot().iff1;
    let iff2 = app.session.snapshot().iff2;
    let interrupt_mode = app.session.snapshot().interrupt_mode;
    let reg_values = app.session.snapshot().regs;
    let shadow_values = app.session.snapshot().shadow_regs;
    let flag_states: [(Flag, &str, bool); 8] = [
        (Flag::S, "S", snapshot_flag(app.session.snapshot(), Flag::S)),
        (Flag::Z, "Z", snapshot_flag(app.session.snapshot(), Flag::Z)),
        (Flag::Y, "Y", snapshot_flag(app.session.snapshot(), Flag::Y)),
        (Flag::H, "H", snapshot_flag(app.session.snapshot(), Flag::H)),
        (Flag::X, "X", snapshot_flag(app.session.snapshot(), Flag::X)),
        (
            Flag::PV,
            "PV",
            snapshot_flag(app.session.snapshot(), Flag::PV),
        ),
        (Flag::N, "N", snapshot_flag(app.session.snapshot(), Flag::N)),
        (Flag::C, "C", snapshot_flag(app.session.snapshot(), Flag::C)),
    ];

    let theme = app.theme;
    egui::Grid::new("regs_grid")
        .striped(true)
        .spacing([10.0, 8.0])
        .show(ui, |ui| {
            macro_rules! edit_16 {
                ($name:expr, $getter:expr, $setter:expr) => {
                    ui.label($name);
                    let val = $getter;
                    let id = egui::Id::unique($name);
                    let mut s = ui.data_mut(|d| {
                        d.get_temp_mut_or_insert_with(id, || format!("{:04X}", val))
                            .clone()
                    });
                    let resp = ui
                        .add_enabled(
                            can_edit,
                            egui::TextEdit::singleline(&mut s)
                                .desired_width(45.0)
                                .font(egui::TextStyle::Monospace),
                        )
                        .on_hover_text("Hexadecimal; 0x is optional");
                    if resp.changed() {
                        if let Some(v) = parse_hex_u16(&s) {
                            $setter(v);
                        }
                        ui.data_mut(|d| d.insert_temp(id, s.clone()));
                    } else if !resp.has_focus() {
                        ui.data_mut(|d| d.insert_temp(id, format!("{:04X}", val)));
                    }
                    if resp.has_focus() && parse_hex_u16(&s).is_none() {
                        ui.painter().rect_stroke(
                            resp.rect,
                            egui::CornerRadius::same(3),
                            egui::Stroke::new(1.5, theme.danger),
                            egui::StrokeKind::Inside,
                        );
                    }
                };
            }

            macro_rules! edit_8 {
                ($name:expr, $id:expr, $getter:expr, $setter:expr) => {
                    ui.label($name);
                    let val = $getter;
                    let id = egui::Id::unique($id);
                    let mut s = ui.data_mut(|d| {
                        d.get_temp_mut_or_insert_with(id, || format!("{:02X}", val))
                            .clone()
                    });
                    let resp = ui
                        .add_enabled(
                            can_edit,
                            egui::TextEdit::singleline(&mut s)
                                .desired_width(30.0)
                                .font(egui::TextStyle::Monospace),
                        )
                        .on_hover_text("Hexadecimal; 0x is optional");
                    if resp.changed() {
                        if let Some(v) = parse_hex_u8(&s) {
                            $setter(v);
                        }
                        ui.data_mut(|d| d.insert_temp(id, s.clone()));
                    } else if !resp.has_focus() {
                        ui.data_mut(|d| d.insert_temp(id, format!("{:02X}", val)));
                    }
                    if resp.has_focus() && parse_hex_u8(&s).is_none() {
                        ui.painter().rect_stroke(
                            resp.rect,
                            egui::CornerRadius::same(3),
                            egui::Stroke::new(1.5, theme.danger),
                            egui::StrokeKind::Inside,
                        );
                    }
                };
            }

            edit_16!("PC", pc, |v| app.edit_cpu(|m| m.cpu.set_pc(v)));
            ui.label("");
            ui.label("");
            ui.end_row();
            edit_16!("SP", sp, |v| app.edit_cpu(|m| m.cpu.set_sp(v)));
            ui.label("");
            ui.label("");
            ui.end_row();
            edit_16!("IX", ix, |v| app.edit_cpu(|m| m.cpu.set_ix(v)));
            ui.label("");
            ui.label("");
            ui.end_row();
            edit_16!("IY", iy, |v| app.edit_cpu(|m| m.cpu.set_iy(v)));
            ui.label("");
            ui.label("");
            ui.end_row();

            ui.separator();
            ui.separator();
            ui.separator();
            ui.separator();
            ui.end_row();

            let regs = [
                ("A", GPR::A),
                ("F", GPR::F),
                ("B", GPR::B),
                ("C", GPR::C),
                ("D", GPR::D),
                ("E", GPR::E),
                ("H", GPR::H),
                ("L", GPR::L),
            ];

            for (name, gpr) in regs {
                let idx = gpr_index(gpr);
                edit_8!(name, name, reg_values[idx], |v| app
                    .edit_cpu(|m| m.cpu.set_register(gpr, v)));
                let shadow_name = format!("{}'", name);
                edit_8!(shadow_name.clone(), shadow_name, shadow_values[idx], |v| {
                    app.edit_cpu(|m| m.cpu.set_shadow_register(gpr, v))
                });
                ui.end_row();
            }

            ui.separator();
            ui.separator();
            ui.separator();
            ui.separator();
            ui.end_row();

            let mono = |text: String| egui::RichText::new(text).monospace();
            ui.label("IFF1");
            ui.label(mono(format!("{}", iff1)));
            ui.label("");
            ui.label("");
            ui.end_row();
            ui.label("IFF2");
            ui.label(mono(format!("{}", iff2)));
            ui.label("");
            ui.label("");
            ui.end_row();
            ui.label("IM");
            ui.label(mono(format!("{}", interrupt_mode)));
            ui.label("");
            ui.label("");
            ui.end_row();
        });

    ui.add_space(16.0);
    ui.heading("Flags");
    ui.separator();

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        for (flag, label, on) in flag_states {
            let (fill, text_color) = if on {
                (theme.flag_on, theme.flag_on_text)
            } else {
                (theme.flag_off, theme.flag_off_text)
            };

            let text = egui::RichText::new(label).strong().color(text_color);
            let frame_resp = egui::Frame::new()
                .fill(fill)
                .corner_radius(AppTheme::WIDGET_CORNER_RADIUS)
                .inner_margin(4.0)
                .show(ui, |ui| {
                    ui.label(text);
                });

            let interact_resp = ui.interact(
                frame_resp.response.rect,
                ui.scope_id().with(label),
                egui::Sense::click(),
            );

            let description = flag_description(flag);
            interact_resp.clone().on_hover_text(format!(
                "{label} — {description}\n{}",
                if on { "Set" } else { "Clear" }
            ));

            if can_edit {
                if interact_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if interact_resp.clicked() {
                    app.edit_cpu(|m| m.cpu.set_flag(!on, flag));
                }
            }
        }
    });
}

pub(crate) fn show_variables(app: &mut Z80App, ui: &mut egui::Ui) {
    let mut sorted_symbols: Vec<_> = app.workspace.symbol_table.iter().collect();
    sorted_symbols.sort_by_key(|item| item.1.source_order);

    ui.horizontal(|ui| {
        ui.heading("Variables")
            .on_hover_text("Right-click a variable name to change how its value is displayed");
        if !app.workspace.symbol_display_prefs.is_empty()
            && ui
                .small_button("Reset formats")
                .on_hover_text("Show every variable using its declared type")
                .clicked()
        {
            app.workspace.symbol_display_prefs.clear();
        }
    });
    ui.separator();

    egui::ScrollArea::both()
        .id_salt("vars_scroll")
        .max_height(200.0)
        .show(ui, |ui| {
            egui::Grid::new("vars_grid")
                .striped(true)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Name").strong());
                    ui.label(egui::RichText::new("Addr").strong());
                    ui.label(egui::RichText::new("Value").strong());
                    ui.label(egui::RichText::new("Format").strong());
                    ui.end_row();

                    for (name, symbol) in sorted_symbols
                        .iter()
                        .filter(|s| !matches!(s.1.kind, SymbolType::Label))
                    {
                        let addr = symbol.address;

                        // Determine display format
                        let format = app
                            .workspace
                            .symbol_display_prefs
                            .get(*name)
                            .cloned()
                            .unwrap_or(SymbolDisplayFormat::Default);

                        // Continuation rows use an internal suffix only to keep map keys unique.
                        let display_name = if name.ends_with(']') && name.rfind('[').is_some() {
                            ""
                        } else {
                            *name
                        };

                        // Format selection lives in its own column so it
                        // can never displace the address or value.
                        ui.label(display_name);

                        let is_constant = symbol.kind == SymbolType::Constant;
                        if is_constant {
                            ui.label(egui::RichText::new("—").color(app.theme.muted))
                                .on_hover_text("EQU constant: has no memory address");
                        } else {
                            ui.monospace(format!("0x{:04X}", addr));
                        }

                        if is_constant {
                            ui.monospace(format_constant_value(addr, format))
                                .on_hover_text(format!(
                                    "Assembler constant (EQU) = 0x{:04X}",
                                    addr
                                ));
                        } else {
                            // Render based on format
                            match format {
                                SymbolDisplayFormat::Default => {
                                    match symbol.kind {
                                        SymbolType::Byte => {
                                            let val = app.session.snapshot().memory[addr as usize];
                                            ui.monospace(format!("0x{:02X}", val));
                                        }
                                        SymbolType::Word => {
                                            let low = app.session.snapshot().memory[addr as usize];
                                            let high = app.session.snapshot().memory
                                                [addr.wrapping_add(1) as usize];
                                            let val = (low as u16) | ((high as u16) << 8);
                                            ui.monospace(format!("0x{:04X}", val));
                                        }
                                        SymbolType::String(len) => {
                                            // Default String render
                                            let mut bytes = Vec::new();
                                            // Read at most 20 chars for preview, but check for null terminator
                                            let display_len = len.min(20);
                                            for i in 0..display_len {
                                                let b = app.session.snapshot().memory
                                                    [addr.wrapping_add(i as u16) as usize];
                                                if b == 0 {
                                                    break;
                                                }
                                                bytes.push(b);
                                            }
                                            let s: String = bytes
                                                .iter()
                                                .map(|&b| {
                                                    if (32..=126).contains(&b) {
                                                        b as char
                                                    } else {
                                                        '.'
                                                    }
                                                })
                                                .collect();

                                            // If we hit a null or length is long, check full content for tooltip
                                            if len > 20 || bytes.len() < len.min(20) {
                                                // Read full content for tooltip
                                                let mut full_bytes = Vec::new();
                                                for i in 0..len {
                                                    let b = app.session.snapshot().memory
                                                        [addr.wrapping_add(i as u16) as usize];
                                                    if b == 0 {
                                                        break;
                                                    }
                                                    full_bytes.push(b);
                                                }
                                                let full_s: String = full_bytes
                                                    .iter()
                                                    .map(|&b| {
                                                        if (32..=126).contains(&b) {
                                                            b as char
                                                        } else {
                                                            '.'
                                                        }
                                                    })
                                                    .collect();

                                                if bytes.len() < full_bytes.len() {
                                                    ui.monospace(format!("\"{}\"…", s))
                                                        .on_hover_text(format!(
                                                            "Length: {}\nFull: \"{}\"",
                                                            len, full_s
                                                        ));
                                                } else {
                                                    ui.monospace(format!("\"{}\"", s));
                                                }
                                            } else {
                                                ui.monospace(format!("\"{}\"", s));
                                            }
                                        }
                                        SymbolType::Array(len) => {
                                            // Default Array render (Hex Bytes)
                                            let mut bytes = Vec::new();
                                            let display_len = len.min(8);
                                            for i in 0..display_len {
                                                bytes.push(
                                                    app.session.snapshot().memory
                                                        [addr.wrapping_add(i as u16) as usize],
                                                );
                                            }
                                            let hex_str: Vec<String> = bytes
                                                .iter()
                                                .map(|b| format!("{:02X}", b))
                                                .collect();
                                            if len > 8 {
                                                // Read full content for tooltip
                                                let mut full_bytes = Vec::new();
                                                for i in 0..len {
                                                    full_bytes.push(
                                                        app.session.snapshot().memory
                                                            [addr.wrapping_add(i as u16) as usize],
                                                    );
                                                }
                                                let full_hex: Vec<String> = full_bytes
                                                    .iter()
                                                    .map(|b| format!("{:02X}", b))
                                                    .collect();

                                                ui.monospace(format!("[{}…]", hex_str.join(" ")))
                                                    .on_hover_text(format!(
                                                        "Length: {}\nFull: [{}]",
                                                        len,
                                                        full_hex.join(" ")
                                                    ));
                                            } else {
                                                ui.monospace(format!("[{}]", hex_str.join(" ")));
                                            }
                                        }
                                        _ => {
                                            ui.label("");
                                        }
                                    }
                                }
                                SymbolDisplayFormat::String => {
                                    let len = match symbol.kind {
                                        SymbolType::String(l) | SymbolType::Array(l) => l,
                                        SymbolType::Word => 2,
                                        _ => 1,
                                    };

                                    let max_scan_len = if len > 1 { len } else { 40 };
                                    let display_limit = max_scan_len.min(40);

                                    let mut bytes = Vec::new();
                                    for i in 0..display_limit {
                                        let b = app.session.snapshot().memory
                                            [addr.wrapping_add(i as u16) as usize];
                                        if b == 0 {
                                            break;
                                        }
                                        bytes.push(b);
                                    }

                                    let s: String = bytes
                                        .iter()
                                        .map(|&b| {
                                            if (32..=126).contains(&b) {
                                                b as char
                                            } else {
                                                '.'
                                            }
                                        })
                                        .collect();

                                    // Determine if we need to show truncation or tooltip with full content
                                    // bytes.len() is the effective string length (stopped at null or limit)

                                    // If we stopped at the display limit (40) but allocated size is larger, it might be truncated.
                                    let truncated_visual = bytes.len() == 40 && max_scan_len > 40;

                                    // Read full string for tooltip only if visually truncated
                                    let mut full_s = s.clone();
                                    if truncated_visual {
                                        let mut full_bytes = Vec::new();
                                        // Scan up to max_scan_len (which is allocated length)
                                        for i in 0..max_scan_len {
                                            let b = app.session.snapshot().memory
                                                [addr.wrapping_add(i as u16) as usize];
                                            if b == 0 {
                                                break;
                                            }
                                            full_bytes.push(b);
                                        }
                                        full_s = full_bytes
                                            .iter()
                                            .map(|&b| {
                                                if (32..=126).contains(&b) {
                                                    b as char
                                                } else {
                                                    '.'
                                                }
                                            })
                                            .collect();
                                    }

                                    if truncated_visual {
                                        ui.monospace(format!("\"{}\"…", s)).on_hover_text(format!(
                                            "Allocated: {}\nString: \"{}\"",
                                            len, full_s
                                        ));
                                    } else {
                                        // Not truncated visually, so 's' is the full string (or at least as much as fits in allocated)
                                        ui.monospace(format!("\"{}\"", s)).on_hover_text(format!(
                                            "Allocated: {}\nString: \"{}\"",
                                            len, s
                                        ));
                                    }
                                }
                                SymbolDisplayFormat::HexBytes => {
                                    let len = match symbol.kind {
                                        SymbolType::String(l) | SymbolType::Array(l) => l,
                                        SymbolType::Word => 2,
                                        _ => 1,
                                    };
                                    let mut bytes = Vec::new();
                                    let display_len = len.clamp(1, 8); // At least 1 byte
                                    for i in 0..display_len {
                                        bytes.push(
                                            app.session.snapshot().memory
                                                [addr.wrapping_add(i as u16) as usize],
                                        );
                                    }
                                    let hex_str: Vec<String> =
                                        bytes.iter().map(|b| format!("{:02X}", b)).collect();
                                    if len > 8 {
                                        let mut full_bytes = Vec::new();
                                        for i in 0..len {
                                            full_bytes.push(
                                                app.session.snapshot().memory
                                                    [addr.wrapping_add(i as u16) as usize],
                                            );
                                        }
                                        let full_hex: Vec<String> = full_bytes
                                            .iter()
                                            .map(|b| format!("{:02X}", b))
                                            .collect();
                                        ui.monospace(format!("[{}…]", hex_str.join(" ")))
                                            .on_hover_text(format!(
                                                "Length: {}\nFull: [{}]",
                                                len,
                                                full_hex.join(" ")
                                            ));
                                    } else {
                                        ui.monospace(format!("[{}]", hex_str.join(" ")))
                                            .on_hover_text(format!(
                                                "Length: {}\nFull: [{}]",
                                                len,
                                                hex_str.join(" ")
                                            ));
                                    }
                                }
                                SymbolDisplayFormat::HexWords => {
                                    let len = match symbol.kind {
                                        SymbolType::String(l) | SymbolType::Array(l) => l,
                                        SymbolType::Word => 2,
                                        _ => 1,
                                    };
                                    // If length is odd, maybe show up to last even?
                                    let word_count = len / 2;
                                    if word_count == 0 {
                                        // Fallback for single byte if forced?
                                        let val = app.session.snapshot().memory[addr as usize];
                                        ui.monospace(format!("[00{:02X}:inv]", val)).on_hover_text(
                                            "Odd length, cannot display as words correctly",
                                        );
                                    } else {
                                        let mut words = Vec::new();
                                        let display_count = word_count.min(4);
                                        for i in 0..display_count {
                                            let offset = (i * 2) as u16;
                                            let low = app.session.snapshot().memory
                                                [addr.wrapping_add(offset) as usize];
                                            let high = app.session.snapshot().memory
                                                [addr.wrapping_add(offset + 1) as usize];
                                            words.push(format!(
                                                "{:04X}",
                                                (low as u16) | ((high as u16) << 8)
                                            ));
                                        }
                                        if word_count > 4 {
                                            let mut full_words = Vec::new();
                                            for i in 0..word_count {
                                                let offset = (i * 2) as u16;
                                                let low = app.session.snapshot().memory
                                                    [addr.wrapping_add(offset) as usize];
                                                let high = app.session.snapshot().memory
                                                    [addr.wrapping_add(offset + 1) as usize];
                                                full_words.push(format!(
                                                    "{:04X}",
                                                    (low as u16) | ((high as u16) << 8)
                                                ));
                                            }
                                            ui.monospace(format!("[{}…]", words.join(" ")))
                                                .on_hover_text(format!(
                                                    "Length: {} bytes\nFull: [{}]",
                                                    len,
                                                    full_words.join(" ")
                                                ));
                                        } else {
                                            ui.monospace(format!("[{}]", words.join(" ")))
                                                .on_hover_text(format!(
                                                    "Length: {} bytes\nFull: [{}]",
                                                    len,
                                                    words.join(" ")
                                                ));
                                        }
                                    }
                                }
                            }
                        }

                        let selected = match format {
                            SymbolDisplayFormat::Default => "Default",
                            SymbolDisplayFormat::String => "String",
                            SymbolDisplayFormat::HexBytes => "Hex bytes",
                            SymbolDisplayFormat::HexWords => "Hex words",
                        };
                        egui::ComboBox::from_id_salt(("vars_fmt", name.as_str()))
                            .selected_text(selected)
                            .width(78.0)
                            .show_ui(ui, |ui| {
                                if ui
                                    .selectable_label(
                                        format == SymbolDisplayFormat::Default,
                                        "Default",
                                    )
                                    .clicked()
                                {
                                    app.workspace.symbol_display_prefs.remove(*name);
                                    ui.close();
                                }
                                if ui
                                    .selectable_label(
                                        format == SymbolDisplayFormat::String,
                                        "String",
                                    )
                                    .clicked()
                                {
                                    app.workspace
                                        .symbol_display_prefs
                                        .insert(name.to_string(), SymbolDisplayFormat::String);
                                    ui.close();
                                }
                                if ui
                                    .selectable_label(
                                        format == SymbolDisplayFormat::HexBytes,
                                        "Hex bytes",
                                    )
                                    .clicked()
                                {
                                    app.workspace
                                        .symbol_display_prefs
                                        .insert(name.to_string(), SymbolDisplayFormat::HexBytes);
                                    ui.close();
                                }
                                if ui
                                    .selectable_label(
                                        format == SymbolDisplayFormat::HexWords,
                                        "Hex words",
                                    )
                                    .clicked()
                                {
                                    app.workspace
                                        .symbol_display_prefs
                                        .insert(name.to_string(), SymbolDisplayFormat::HexWords);
                                    ui.close();
                                }
                            });
                        ui.end_row();
                    }
                });
        });

    ui.add_space(20.0);
    ui.heading("Labels");
    ui.separator();

    egui::ScrollArea::vertical()
        .id_salt("labels_scroll")
        .show(ui, |ui| {
            egui::Grid::new("labels_grid")
                .striped(true)
                .spacing([20.0, 4.0])
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Name").strong());
                    ui.label(egui::RichText::new("Addr").strong());
                    ui.end_row();

                    for (name, symbol) in &sorted_symbols {
                        if symbol.kind != SymbolType::Label {
                            continue;
                        }
                        let addr = symbol.address;

                        ui.label(*name);
                        ui.monospace(format!("0x{:04X}", addr));
                        ui.end_row();
                    }
                });
        });
}
