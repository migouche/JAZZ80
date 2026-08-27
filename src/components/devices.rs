use crate::traits::IODevice;
use crate::ui_traits::DeviceWithUi;
use eframe::egui;
use std::collections::VecDeque;

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
        let mut open = self.is_open;
        egui::Window::new(self.get_name())
            .open(&mut open)
            .show(ctx, |ui| {
                ui.group(|ui| {
                    ui.label(format!("Port: 0x{:02X}", self.get_port()));
                    ui.separator();

                    // Simple text representation of 7-segment
                    let val = self.get_value();

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
        self.is_open = open;
    }
}

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
        let mut open = self.is_open;
        egui::Window::new(self.get_name())
            .open(&mut open)
            .show(ctx, |ui| {
                ui.group(|ui| {
                    ui.label(format!("Port: 0x{:02X}", self.get_port()));
                    ui.separator();

                    ui.label(format!("Last Key: {:02X}", self.get_last_key()));

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
                                        .add(
                                            egui::Button::new(label)
                                                .min_size(egui::vec2(30.0, 30.0)),
                                        )
                                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                                        .clicked()
                                    {
                                        self.press_key(code);
                                    }
                                }
                                ui.end_row();
                            }
                        });
                });
            });
        self.is_open = open;
    }
}
pub struct GenericInterruptDevice {
    port: u16,
    vector: u8,
    interrupt_pending: bool,
    is_open: bool,
    // UI temporary state
    vector_input: String,
}

impl GenericInterruptDevice {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            vector: 0xFF, // Default RST 38h
            interrupt_pending: false,
            is_open: true,
            vector_input: "FF".to_string(),
        }
    }

    pub fn trigger(&mut self) {
        self.interrupt_pending = true;
    }
}

impl IODevice for GenericInterruptDevice {
    fn read_in(&mut self, _port: u16) -> Option<u8> {
        None
    }

    fn write_out(&mut self, _port: u16, _data: u8) -> bool {
        false
    }

    fn poll_interrupt(&self) -> bool {
        self.interrupt_pending
    }

    fn ack_interrupt(&mut self) -> u8 {
        self.interrupt_pending = false;
        self.vector
    }
}

impl DeviceWithUi for GenericInterruptDevice {
    fn get_name(&self) -> String {
        format!("Int Controller (Port 0x{:02X})", self.port)
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
                ui.group(|ui| {
                    ui.label("Control interrupt generation for Mode 0 or 2.");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Vector/Opcode (Hex):");
                        if ui.text_edit_singleline(&mut self.vector_input).changed() {
                            if let Ok(val) = u8::from_str_radix(&self.vector_input, 16) {
                                self.vector = val;
                            }
                        }
                    });

                    ui.label(format!("Current Vector: 0x{:02X}", self.vector));

                    if ui
                        .button("TRIGGER INTERRUPT")
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        self.trigger();
                    }

                    if self.interrupt_pending {
                        ui.colored_label(egui::Color32::RED, "Interrupt Pending...");
                    } else {
                        ui.label("Idle");
                    }
                });
            });
        self.is_open = open;
    }
}

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
            if ch == '\n' || (ch >= ' ' && ch <= '~') {
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
        let mut open = self.is_open;
        egui::Window::new(self.get_name())
            .open(&mut open)
            .min_width(300.0)
            .show(ctx, |ui| {
                ui.label("Sent from CPU:");

                let text: String = self.display_buffer.iter().collect();
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
                    self.display_buffer.clear();
                    self.input_index = 0; // Reset input too maybe?
                }

                ui.separator();
                ui.label(format!("Input Index: {}", self.input_index));
            });
        self.is_open = open;
    }
}

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
        if ch == '\n' || ch == '\r' || (ch >= ' ' && ch <= '~') {
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
        let mut open = self.is_open;
        egui::Window::new(self.get_name())
            .open(&mut open)
            .min_width(350.0)
            .show(ctx, |ui| {
                ui.label("OS Output:");

                // Render the text sent by the Z80 OS
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut self.tx_buffer.as_str())
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .interactive(false),
                        );
                    });

                if ui.button("Clear Output").clicked() {
                    self.tx_buffer.clear();
                }

                ui.separator();
                ui.label("Send Input to OS:");

                ui.horizontal(|ui| {
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.input_string)
                            .hint_text("Type and press Enter...")
                            .desired_width(200.0),
                    );

                    let send_clicked = ui.button("Send").clicked();

                    // Trigger when the user clicks 'Send' or hits the Enter key
                    if send_clicked
                        || (response.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter)))
                    {
                        // Push typed bytes into the hardware queue
                        for byte in self.input_string.bytes() {
                            self.rx_queue.push_back(byte);
                        }
                        // Add a carriage return so the OS knows the command is done
                        self.rx_queue.push_back(b'\r');

                        self.input_string.clear();
                        response.request_focus(); // Keep focus so you can keep typing
                    }
                });
            });
        self.is_open = open;
    }
}
