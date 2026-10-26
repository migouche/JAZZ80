use eframe::egui;

use super::super::Z80App;

pub(crate) fn show(app: &mut Z80App, ui: &mut egui::Ui) {
    let theme = app.theme;
    ui.horizontal(|ui| {
        let name = app.workspace.active_name();
        let path = app.workspace.active_path_string();
        let dirty = app
            .workspace
            .tabs
            .get(app.workspace.active_tab)
            .is_some_and(|tab| tab.is_dirty);

        let mut name_label = egui::RichText::new(name).strong();
        if dirty {
            name_label = name_label.color(theme.warning);
        }
        let resp = ui.label(name_label);
        if let Some(path) = path {
            resp.on_hover_text(path);
        }
        if dirty {
            ui.label(
                egui::RichText::new("● unsaved")
                    .small()
                    .color(theme.warning),
            )
            .on_hover_text("This document has unsaved changes");
        }

        ui.separator();
        ui.monospace(format!("PC 0x{:04X}", app.session.snapshot().pc))
            .on_hover_text("Program counter");
        ui.monospace(format!("SP 0x{:04X}", app.session.snapshot().sp))
            .on_hover_text("Stack pointer");

        if app.workspace.is_assembly_stale {
            ui.label(
                egui::RichText::new("source changed since last assemble")
                    .small()
                    .color(theme.stale),
            )
            .on_hover_text("Run uses the previously assembled program until you assemble again");
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(err) = app.last_error.clone() {
                if ui.small_button("✕").on_hover_text("Dismiss").clicked() {
                    app.last_error = None;
                }
                let text = format!("⚠ {err}");
                ui.label(egui::RichText::new(&text).color(theme.danger).small())
                    .on_hover_text(err);
            } else if let Some(msg) = app.status_message.clone() {
                if ui.small_button("✕").on_hover_text("Dismiss").clicked() {
                    app.status_message = None;
                }
                ui.label(egui::RichText::new(&msg).color(theme.muted).small())
                    .on_hover_text(msg);
            }
        });
    });
}
