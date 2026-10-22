//! Central application theme.
//!
//! Every colour used by the shell (panels, gutters, status bar, flags,
//! breakpoints, dialogs) comes from here instead of being written inline in the
//! widget code. The palette is derived from One Dark, the same palette used by
//! the syntax highlighter, so the editor and the surrounding chrome agree.

use eframe::egui;

#[derive(Clone, Copy, Debug)]
pub struct AppTheme {
    pub bg: egui::Color32,
    pub panel: egui::Color32,
    pub faint: egui::Color32,
    pub gutter: egui::Color32,
    pub fg: egui::Color32,
    pub muted: egui::Color32,
    pub accent: egui::Color32,
    pub address: egui::Color32,
    pub data: egui::Color32,
    pub stale: egui::Color32,
    pub success: egui::Color32,
    pub warning: egui::Color32,
    pub danger: egui::Color32,
    pub breakpoint: egui::Color32,
    pub pc_row: egui::Color32,
    pub pc_marker: egui::Color32,
    pub flag_on: egui::Color32,
    pub flag_on_text: egui::Color32,
    pub flag_off: egui::Color32,
    pub flag_off_text: egui::Color32,
    pub tab_active_bg: egui::Color32,
    pub tab_active_fg: egui::Color32,
}

impl AppTheme {
    pub const ONE_DARK: Self = Self {
        // Surfaces
        bg: egui::Color32::from_rgb(0x21, 0x22, 0x27),
        panel: egui::Color32::from_rgb(0x28, 0x2c, 0x34),
        faint: egui::Color32::from_rgb(0x2c, 0x31, 0x3a),
        gutter: egui::Color32::from_rgb(0x7f, 0x84, 0x8e),
        // Text
        fg: egui::Color32::from_rgb(0xab, 0xb2, 0xbf),
        muted: egui::Color32::from_rgb(0x7f, 0x84, 0x8e),
        // Semantic
        accent: egui::Color32::from_rgb(0x61, 0xaf, 0xef),
        address: egui::Color32::from_rgb(0x61, 0xaf, 0xef),
        data: egui::Color32::from_rgb(0x98, 0xc3, 0x79),
        stale: egui::Color32::from_rgb(0xe5, 0xc0, 0x7b),
        success: egui::Color32::from_rgb(0x98, 0xc3, 0x79),
        warning: egui::Color32::from_rgb(0xe5, 0xc0, 0x7b),
        danger: egui::Color32::from_rgb(0xe0, 0x6c, 0x75),
        breakpoint: egui::Color32::from_rgb(0xe0, 0x6c, 0x75),
        // Editor states
        pc_row: egui::Color32::from_rgb(0x3a, 0x40, 0x4b),
        pc_marker: egui::Color32::from_rgb(0x98, 0xc3, 0x79),
        // Flags
        flag_on: egui::Color32::from_rgb(0x98, 0xc3, 0x79),
        flag_on_text: egui::Color32::from_rgb(0x21, 0x22, 0x27),
        flag_off: egui::Color32::from_rgb(0x3a, 0x3f, 0x49),
        flag_off_text: egui::Color32::from_rgb(0x7f, 0x84, 0x8e),
        // Tabs
        tab_active_bg: egui::Color32::from_rgb(0x3a, 0x40, 0x4b),
        tab_active_fg: egui::Color32::from_rgb(0xdc, 0xe0, 0xe6),
    };
}

/// Install the application visuals and widget spacing on a context.
pub fn apply(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = AppTheme::ONE_DARK.bg;
    visuals.window_fill = AppTheme::ONE_DARK.panel;
    visuals.extreme_bg_color = AppTheme::ONE_DARK.bg;
    visuals.faint_bg_color = AppTheme::ONE_DARK.faint;
    visuals.selection.bg_fill = AppTheme::ONE_DARK.accent.gamma_multiply(0.55);
    visuals.selection.stroke.color = AppTheme::ONE_DARK.fg;
    visuals.window_corner_radius = egui::CornerRadius::same(8);
    visuals.menu_corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.noninteractive.bg_stroke.color = egui::Color32::from_rgb(0x3a, 0x3f, 0x49);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(4);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(4);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(4);
    // The palette above is a dark palette. Pin the theme so a light-mode OS
    // setting cannot mix dark text with our dark surfaces.
    ctx.set_theme(egui::Theme::Dark);
    ctx.set_visuals_of(egui::Theme::Dark, visuals);

    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    ctx.set_style_of(egui::Theme::Dark, style);
}
