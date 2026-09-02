use crate::traits::IODevice;
use crate::ui_traits::DeviceWithUi;
use eframe::egui;

pub struct NmiTrigger {
    triggered: bool,
    is_open: bool,
}

impl NmiTrigger {
    pub fn new() -> Self {
        NmiTrigger {
            triggered: false,
            is_open: true,
        }
    }
}

impl IODevice for NmiTrigger {
    fn read_in(&mut self, _port: u16) -> Option<u8> {
        None
    }

    fn write_out(&mut self, _port: u16, _data: u8) -> bool {
        false
    }

    fn poll_nmi(&self) -> bool {
        self.triggered
    }
}

impl DeviceWithUi for NmiTrigger {
    fn get_name(&self) -> String {
        "NMI Trigger".to_string()
    }

    fn get_window_open_state(&self) -> bool {
        self.is_open
    }

    fn set_window_open_state(&mut self, open: bool) {
        self.is_open = open;
    }

    fn draw(&mut self, ctx: &egui::Context) {
        let mut open = self.is_open;
        egui::Window::new(self.get_name())
            .open(&mut open)
            .show(ctx, |ui| {
                self.triggered = ui
                    .button("TRIGGER NMI")
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked();
            });
        self.is_open = open;
    }
}
