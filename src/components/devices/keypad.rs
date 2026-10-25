use crate::traits::IODevice;
use crate::ui_traits::DeviceWithUi;
use eframe::egui;

pub struct Keypad {
    port: u16,
    current_key: u8,
    interrupt_pending: bool,
    is_open: bool,
}

impl Keypad {
    pub fn new(port: u16) -> Self {
        Keypad {
            port,
            current_key: 0,
            interrupt_pending: false,
            is_open: true,
        }
    }

    pub fn press_key(&mut self, key: u8) {
        self.current_key = key;
        self.interrupt_pending = true;
    }

    pub fn get_last_key(&self) -> u8 {
        self.current_key
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }
}

impl IODevice for Keypad {
    fn read_in(&mut self, port: u16) -> Option<u8> {
        // Ignore upper 8 bits of port addressing for simple I/O decoding
        if (port & 0xFF) == (self.get_port() & 0xFF) {
            self.interrupt_pending = false;
            Some(self.get_last_key())
        } else {
            None
        }
    }

    fn write_out(&mut self, _port: u16, _data: u8) -> bool {
        false
    }

    fn poll_interrupt(&self) -> bool {
        self.interrupt_pending
    }

    fn ack_interrupt(&mut self) -> u8 {
        // Simple default vector 0xFF, effectively RST 38h in Mode 0, or vector FF in Mode 2
        self.interrupt_pending = false;
        0xFF
    }
}

impl DeviceWithUi for Keypad {
    fn get_name(&self) -> String {
        format!("Keypad (Port 0x{:02X})", self.get_port())
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

                ui.label(format!("Last Key: {:02X}", this.get_last_key()));

                egui::Grid::new("keypad_grid")
                    .spacing(egui::vec2(5.0, 5.0))
                    .show(ui, |ui| {
                        let keys = [
                            [(0x1, "1"), (0x2, "2"), (0x3, "3"), (0xA, "A")],
                            [(0x4, "4"), (0x5, "5"), (0x6, "6"), (0xB, "B")],
                            [(0x7, "7"), (0x8, "8"), (0x9, "9"), (0xC, "C")],
                            [(0xE, "*"), (0x0, "0"), (0xF, "#"), (0xD, "D")],
                        ];

                        for row in keys {
                            let row_iter = row.into_iter();
                            for (code, label) in row_iter {
                                if ui
                                    .add(egui::Button::new(label).min_size(egui::vec2(30.0, 30.0)))
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    this.press_key(code);
                                }
                            }
                            ui.end_row();
                        }
                    });
            });
        });
    }
}
