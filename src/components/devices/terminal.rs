use crate::traits::IODevice;
use crate::ui_traits::DeviceWithUi;
use eframe::egui;
use std::collections::VecDeque;

pub struct VirtualTerminal {
    port: u16,
    rx_queue: VecDeque<u8>,
    tx_buffer: String,
    control_sequence: Vec<u8>,
    // UI temporary state for user input
    input_string: String,
    is_open: bool,
}

impl VirtualTerminal {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            rx_queue: VecDeque::new(),
            tx_buffer: String::new(),
            control_sequence: Vec::new(),
            input_string: String::new(),
            is_open: true,
        }
    }
}

impl IODevice for VirtualTerminal {
    fn read_in(&mut self, port: u16) -> Option<u8> {
        let p = port & 0xFF;
        let base = self.port & 0xFF;
        let status_port = base.wrapping_add(1) & 0xFF;

        if p == base {
            // Data Port: Pop the next character from the user's input
            Some(self.rx_queue.pop_front().unwrap_or(0))
        } else if p == status_port {
            // Status Port: Tell the OS if it should bother reading
            let mut status = 0x02; // Bit 1 = Ready to transmit (always true here)
            if !self.rx_queue.is_empty() {
                status |= 0x01; // Bit 0 = Data available to receive
            }
            Some(status)
        } else {
            None
        }
    }

    fn write_out(&mut self, port: u16, data: u8) -> bool {
        let p = port & 0xFF;
        let base = self.port & 0xFF;
        let status_port = base.wrapping_add(1) & 0xFF;

        if p == base {
            self.write_terminal_byte(data);
            true
        } else if p == status_port {
            // Ignore writes to the status port, but acknowledge them as handled
            true
        } else {
            false
        }
    }
}

impl VirtualTerminal {
    fn write_terminal_byte(&mut self, data: u8) {
        // TODO: Make this more robust to handle other sequences and partial sequences
        const CLEAR_SCREEN: &[u8] = b"\x1b[2J\x1b[H";
        if self.control_sequence.is_empty() && data != 0x1B {
            self.append_display_byte(data);
            return;
        }

        self.control_sequence.push(data);
        if self.control_sequence == CLEAR_SCREEN {
            self.tx_buffer.clear();
            self.control_sequence.clear();
        } else if !CLEAR_SCREEN.starts_with(&self.control_sequence) {
            let sequence = std::mem::take(&mut self.control_sequence);
            for byte in sequence {
                self.append_display_byte(byte);
            }
        }
    }

    fn append_display_byte(&mut self, data: u8) {
        let ch = data as char;
        if ch == '\n' || ch == '\r' || (' '..='~').contains(&ch) {
            self.tx_buffer.push(ch);
        } else {
            self.tx_buffer.push('.');
        }
    }
}

impl DeviceWithUi for VirtualTerminal {
    fn get_name(&self) -> String {
        format!(
            "OS Terminal (Ports 0x{:02X}-0x{:02X})",
            self.port & 0xFF,
            (self.port.wrapping_add(1)) & 0xFF
        )
    }

    fn get_window_open_state(&self) -> bool {
        self.is_open
    }

    fn set_window_open_state(&mut self, open: bool) {
        self.is_open = open;
    }

    fn draw(&mut self, ctx: &egui::Context) {
        self.show_window(ctx, Some(350.0), &mut |this, ui| {
            ui.label("OS Output:");

            // Render the text sent by the Z80 OS
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut this.tx_buffer.as_str())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .interactive(false),
                    );
                });

            if ui.button("Clear Output").clicked() {
                this.tx_buffer.clear();
            }

            ui.separator();
            ui.label("Send Input to OS:");

            ui.horizontal(|ui| {
                let response = ui.add(
                    egui::TextEdit::singleline(&mut this.input_string)
                        .hint_text("Type and press Enter…")
                        .desired_width(200.0),
                );

                let send_clicked = ui.button("Send").clicked();

                // Trigger when the user clicks 'Send' or hits the Enter key
                if send_clicked
                    || (response.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter)))
                {
                    // Push typed bytes into the hardware queue
                    for byte in this.input_string.bytes() {
                        this.rx_queue.push_back(byte);
                    }
                    // Add a carriage return so the OS knows the command is done
                    this.rx_queue.push_back(b'\r');

                    this.input_string.clear();
                    response.request_focus(); // Keep focus so you can keep typing
                }
            });
        });
    }

    fn reset(&mut self) {
        self.rx_queue.clear();
        self.tx_buffer.clear();
        self.control_sequence.clear();
        self.input_string.clear();
    }
}
