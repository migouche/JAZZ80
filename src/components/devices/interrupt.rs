use crate::traits::IODevice;
use crate::ui_traits::DeviceWithUi;
use eframe::egui;

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
        self.show_window(ctx, None, &mut |this, ui| {
            ui.group(|ui| {
                ui.label("Control interrupt generation for Mode 0 or 2.");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Vector/Opcode (Hex):");
                    if ui.text_edit_singleline(&mut this.vector_input).changed()
                        && let Ok(val) = u8::from_str_radix(&this.vector_input, 16)
                    {
                        this.vector = val;
                    }
                });

                ui.label(format!("Current Vector: 0x{:02X}", this.vector));

                if ui
                    .button("TRIGGER INTERRUPT")
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    this.trigger();
                }

                if this.interrupt_pending {
                    ui.colored_label(egui::Color32::RED, "Interrupt Pending...");
                } else {
                    ui.label("Idle");
                }
            });
        });
    }
}
