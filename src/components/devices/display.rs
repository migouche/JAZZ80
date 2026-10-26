use crate::traits::IODevice;
use crate::ui_traits::DeviceWithUi;
use eframe::egui;

pub struct SevenSegmentDisplay {
    port: u16,
    value: u8,
    is_open: bool,
}

impl SevenSegmentDisplay {
    pub fn new(port: u16) -> Self {
        SevenSegmentDisplay {
            port,
            value: 0,
            is_open: true,
        }
    }
    pub fn get_value(&self) -> u8 {
        self.value
    }
    pub fn get_port(&self) -> u16 {
        self.port
    }
}

impl IODevice for SevenSegmentDisplay {
    fn read_in(&mut self, _port: u16) -> Option<u8> {
        None
    }

    fn write_out(&mut self, port: u16, data: u8) -> bool {
        // Ignore upper 8 bits of port addressing for simple I/O decoding
        if (port & 0xFF) == (self.port & 0xFF) {
            self.value = data;
            true
        } else {
            false
        }
    }
}

impl DeviceWithUi for SevenSegmentDisplay {
    fn get_name(&self) -> String {
        format!("Seven Segment Display (Port 0x{:02X})", self.get_port())
    }

    fn get_window_open_state(&self) -> bool {
        self.is_open
    }

    fn set_window_open_state(&mut self, open: bool) {
        self.is_open = open;
    }

    fn draw(&mut self, ctx: &egui::Context) {
        self.show_window(ctx, None, &mut |this, ui| {
            ui.group(|ui| {
                ui.label(format!("Port: 0x{:02X}", this.get_port()));
                ui.separator();

                // Simple text representation of 7-segment
                let val = this.get_value();

                // Draw a big number
                ui.vertical_centered(|ui| {
                    let text = egui::RichText::new(format!("{:02X}", val))
                        .size(40.0)
                        .family(egui::FontFamily::Monospace)
                        .strong();
                    ui.label(text);

                    // Binary representation
                    ui.label(format!("Binary: {:08b}", val));
                });
            });
        });
    }
}
