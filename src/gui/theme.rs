use eframe::egui;

/// The One Dark palette. Every distinct colour lives here exactly once; the
/// roles that use them live in [`AppTheme`].
mod palette {
    use eframe::egui::Color32;

    pub(super) const BACKGROUND: Color32 = Color32::from_rgb(0x21, 0x22, 0x27);
    pub(super) const PANEL: Color32 = Color32::from_rgb(0x28, 0x2c, 0x34);
    pub(super) const FAINT: Color32 = Color32::from_rgb(0x2c, 0x31, 0x3a);
    pub(super) const PC_ROW: Color32 = Color32::from_rgb(0x3a, 0x40, 0x4b);
    pub(super) const FLAG_OFF: Color32 = Color32::from_rgb(0x3a, 0x3f, 0x49);
    pub(super) const TAB_ACTIVE_TEXT: Color32 = Color32::from_rgb(0xdc, 0xe0, 0xe6);
    pub(super) const GREY: Color32 = Color32::from_rgb(0x7f, 0x84, 0x8e);
    pub(super) const FOREGROUND: Color32 = Color32::from_rgb(0xab, 0xb2, 0xbf);
    pub(super) const BLUE: Color32 = Color32::from_rgb(0x61, 0xaf, 0xef);
    pub(super) const GREEN: Color32 = Color32::from_rgb(0x98, 0xc3, 0x79);
    pub(super) const YELLOW: Color32 = Color32::from_rgb(0xe5, 0xc0, 0x7b);
    pub(super) const RED: Color32 = Color32::from_rgb(0xe0, 0x6c, 0x75);
    pub(super) const PURPLE: Color32 = Color32::from_rgb(0xc6, 0x78, 0xdd);
    pub(super) const ORANGE: Color32 = Color32::from_rgb(0xd1, 0x9a, 0x66);
    pub(super) const CYAN: Color32 = Color32::from_rgb(0x56, 0xb6, 0xc2);
    pub(super) const WHITE: Color32 = Color32::WHITE;
}

/// Syntax-highlighting colours for the source editor.
#[derive(Clone, Copy, Debug)]
pub struct SyntaxColors {
    pub comment: egui::Color32,
    pub mnemonic: egui::Color32,
    pub register: egui::Color32,
    pub condition: egui::Color32,
    pub number: egui::Color32,
    pub label: egui::Color32,
    pub directive: egui::Color32,
    pub string: egui::Color32,
    pub plain: egui::Color32,
}

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
    pub breakpoint_text: egui::Color32,
    pub pc_row: egui::Color32,
    pub pc_marker: egui::Color32,
    pub flag_on: egui::Color32,
    pub flag_on_text: egui::Color32,
    pub flag_off: egui::Color32,
    pub flag_off_text: egui::Color32,
    pub tab_active_bg: egui::Color32,
    pub tab_active_fg: egui::Color32,
    pub syntax: SyntaxColors,
}

impl AppTheme {
    pub const ONE_DARK: Self = Self {
        // Surfaces
        bg: palette::BACKGROUND,
        panel: palette::PANEL,
        faint: palette::FAINT,
        gutter: palette::GREY,
        // Text
        fg: palette::FOREGROUND,
        muted: palette::GREY,
        // Semantic
        accent: palette::BLUE,
        address: palette::BLUE,
        data: palette::GREEN,
        stale: palette::YELLOW,
        success: palette::GREEN,
        warning: palette::YELLOW,
        danger: palette::RED,
        breakpoint: palette::RED,
        breakpoint_text: palette::WHITE,
        // Editor states
        pc_row: palette::PC_ROW,
        pc_marker: palette::GREEN,
        // Flags
        flag_on: palette::GREEN,
        flag_on_text: palette::BACKGROUND,
        flag_off: palette::FLAG_OFF,
        flag_off_text: palette::GREY,
        // Tabs
        tab_active_bg: palette::PC_ROW,
        tab_active_fg: palette::TAB_ACTIVE_TEXT,
        // Syntax
        syntax: SyntaxColors {
            comment: palette::GREY,
            mnemonic: palette::PURPLE,
            register: palette::YELLOW,
            condition: palette::RED,
            number: palette::ORANGE,
            label: palette::BLUE,
            directive: palette::CYAN,
            string: palette::GREEN,
            plain: palette::FOREGROUND,
        },
    };

    pub const MONO_FONT_SIZE: f32 = 14.0;
    pub const DIRTY_DOT_FONT_SIZE: f32 = 10.0;
    pub const TAB_CLOSE_FONT_SIZE: f32 = 14.0;
    pub const COLUMN_GAP: f32 = 4.0;
    pub const WINDOW_CORNER_RADIUS: u8 = 8;
    pub const MENU_CORNER_RADIUS: u8 = 6;
    pub const WIDGET_CORNER_RADIUS: u8 = 4;
    pub const TAB_CORNER_RADIUS: u8 = 5;
    pub const TAB_MARGIN_X: i8 = 8;
    pub const TAB_MARGIN_Y: i8 = 4;
    pub const EDITOR_MARGIN_X: i8 = 4;
}

pub fn apply(ctx: &egui::Context) {
    let theme = AppTheme::ONE_DARK;

    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = theme.bg;
    visuals.window_fill = theme.panel;
    visuals.extreme_bg_color = theme.bg;
    visuals.faint_bg_color = theme.faint;
    visuals.selection.bg_fill = theme.accent.gamma_multiply(0.55);
    visuals.selection.stroke.color = theme.fg;
    visuals.window_corner_radius = egui::CornerRadius::same(AppTheme::WINDOW_CORNER_RADIUS);
    visuals.menu_corner_radius = egui::CornerRadius::same(AppTheme::MENU_CORNER_RADIUS);
    visuals.widgets.noninteractive.bg_stroke.color = theme.flag_off;
    visuals.widgets.inactive.corner_radius =
        egui::CornerRadius::same(AppTheme::WIDGET_CORNER_RADIUS);
    visuals.widgets.hovered.corner_radius =
        egui::CornerRadius::same(AppTheme::WIDGET_CORNER_RADIUS);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(AppTheme::WIDGET_CORNER_RADIUS);
    ctx.set_theme(egui::Theme::Dark);
    ctx.set_visuals_of(egui::Theme::Dark, visuals);

    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    ctx.set_style_of(egui::Theme::Dark, style);
}

#[cfg(test)]
mod tests {
    use super::AppTheme;
    use eframe::egui;

    #[test]
    fn default_fonts_include_the_bundled_icon_fonts() {
        let fonts = egui::FontDefinitions::default();
        assert!(
            fonts.font_data.contains_key("NotoEmoji-Regular"),
            "NotoEmoji-Regular is required for the toolbar icons"
        );
        assert!(
            fonts.font_data.contains_key("emoji-icon-font"),
            "emoji-icon-font is required for the toolbar icons"
        );
    }

    #[test]
    fn one_dark_palette_keeps_its_colours() {
        let t = AppTheme::ONE_DARK;
        let rgb = egui::Color32::from_rgb;

        // Surfaces
        assert_eq!(t.bg, rgb(0x21, 0x22, 0x27));
        assert_eq!(t.panel, rgb(0x28, 0x2c, 0x34));
        assert_eq!(t.faint, rgb(0x2c, 0x31, 0x3a));
        // Text
        assert_eq!(t.gutter, rgb(0x7f, 0x84, 0x8e));
        assert_eq!(t.fg, rgb(0xab, 0xb2, 0xbf));
        assert_eq!(t.muted, rgb(0x7f, 0x84, 0x8e));
        // Semantic
        assert_eq!(t.accent, rgb(0x61, 0xaf, 0xef));
        assert_eq!(t.address, rgb(0x61, 0xaf, 0xef));
        assert_eq!(t.data, rgb(0x98, 0xc3, 0x79));
        assert_eq!(t.stale, rgb(0xe5, 0xc0, 0x7b));
        assert_eq!(t.success, rgb(0x98, 0xc3, 0x79));
        assert_eq!(t.warning, rgb(0xe5, 0xc0, 0x7b));
        assert_eq!(t.danger, rgb(0xe0, 0x6c, 0x75));
        assert_eq!(t.breakpoint, rgb(0xe0, 0x6c, 0x75));
        assert_eq!(t.breakpoint_text, rgb(0xff, 0xff, 0xff));
        // Editor states
        assert_eq!(t.pc_row, rgb(0x3a, 0x40, 0x4b));
        assert_eq!(t.pc_marker, rgb(0x98, 0xc3, 0x79));
        // Flags
        assert_eq!(t.flag_on, rgb(0x98, 0xc3, 0x79));
        assert_eq!(t.flag_on_text, rgb(0x21, 0x22, 0x27));
        assert_eq!(t.flag_off, rgb(0x3a, 0x3f, 0x49));
        assert_eq!(t.flag_off_text, rgb(0x7f, 0x84, 0x8e));
        // Tabs
        assert_eq!(t.tab_active_bg, rgb(0x3a, 0x40, 0x4b));
        assert_eq!(t.tab_active_fg, rgb(0xdc, 0xe0, 0xe6));
        // Syntax
        assert_eq!(t.syntax.comment, rgb(0x7f, 0x84, 0x8e));
        assert_eq!(t.syntax.mnemonic, rgb(0xc6, 0x78, 0xdd));
        assert_eq!(t.syntax.register, rgb(0xe5, 0xc0, 0x7b));
        assert_eq!(t.syntax.condition, rgb(0xe0, 0x6c, 0x75));
        assert_eq!(t.syntax.number, rgb(0xd1, 0x9a, 0x66));
        assert_eq!(t.syntax.label, rgb(0x61, 0xaf, 0xef));
        assert_eq!(t.syntax.directive, rgb(0x56, 0xb6, 0xc2));
        assert_eq!(t.syntax.string, rgb(0x98, 0xc3, 0x79));
        assert_eq!(t.syntax.plain, rgb(0xab, 0xb2, 0xbf));
    }
}
