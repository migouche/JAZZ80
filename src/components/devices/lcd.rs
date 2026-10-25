use crate::traits::IODevice;
use crate::ui_traits::DeviceWithUi;
use eframe::egui;

pub struct LcdDisplay {
    port: u16,
    display_buffer: Vec<char>,
    input_buffer: Vec<u8>,
    input_index: usize,
    is_open: bool,
}

impl LcdDisplay {
    pub fn new(port: u16) -> Self {
        LcdDisplay {
            port,
            display_buffer: Vec::new(),
            input_buffer: b"Hello Input".to_vec(),
            input_index: 0,
            is_open: true,
        }
    }
}

impl IODevice for LcdDisplay {
    fn read_in(&mut self, port: u16) -> Option<u8> {
        if (port & 0xFF) == (self.port & 0xFF) {
            let val = if self.input_index < self.input_buffer.len() {
                let v = self.input_buffer[self.input_index];
                self.input_index += 1;
                v
            } else {
                0
            };
            Some(val)
        } else {
            None
        }
    }

    fn write_out(&mut self, port: u16, data: u8) -> bool {
        if (port & 0xFF) == (self.port & 0xFF) {
            let ch = data as char;
            if ch == '\n' || (' '..='~').contains(&ch) {
                self.display_buffer.push(ch);
            } else {
                self.display_buffer.push('.');
            }
            true
        } else {
            false
        }
    }
}

impl DeviceWithUi for LcdDisplay {
    fn get_name(&self) -> String {
        format!("LCD Display (Port 0x{:02X})", self.port)
    }

    fn get_window_open_state(&self) -> bool {
        self.is_open
    }

    fn set_window_open_state(&mut self, open: bool) {
        self.is_open = open;
    }

    fn draw(&mut self, ctx: &egui::Context) {
        self.show_window(ctx, Some(300.0), &mut |this, ui| {
            ui.label("Sent from CPU:");

            let text: String = this.display_buffer.iter().collect();
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut text.as_str())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .interactive(false), // Read-only
                    );
                });

            if ui.button("Clear").clicked() {
                this.display_buffer.clear();
                this.input_index = 0; // Reset input too maybe?
            }

            ui.separator();
            ui.label(format!("Input Index: {}", this.input_index));
        });
    }
}
