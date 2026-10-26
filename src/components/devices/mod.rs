use self::nmi_trigger::NmiTrigger;
use crate::ui_traits::DeviceWithUi;
use std::sync::{Arc, Mutex};

mod display;
mod dos;
mod interrupt;
mod keypad;
mod lcd;
mod nmi_trigger;
mod terminal;

pub use display::SevenSegmentDisplay;
pub use dos::VirtualDOS;
pub use interrupt::GenericInterruptDevice;
pub use keypad::Keypad;
pub use lcd::LcdDisplay;
pub use terminal::VirtualTerminal;

pub struct DeviceDefinition {
    pub menu_name: &'static str,
    pub default_port: &'static str,
    pub create: fn(u16) -> Arc<Mutex<dyn DeviceWithUi>>,
}

fn device<T: DeviceWithUi + 'static>(device: T) -> Arc<Mutex<dyn DeviceWithUi>> {
    Arc::new(Mutex::new(device))
}

fn create_keypad(port: u16) -> Arc<Mutex<dyn DeviceWithUi>> {
    device(Keypad::new(port))
}

fn create_display(port: u16) -> Arc<Mutex<dyn DeviceWithUi>> {
    device(SevenSegmentDisplay::new(port))
}

fn create_lcd_display(port: u16) -> Arc<Mutex<dyn DeviceWithUi>> {
    device(LcdDisplay::new(port))
}

fn create_interrupt_controller(port: u16) -> Arc<Mutex<dyn DeviceWithUi>> {
    device(GenericInterruptDevice::new(port))
}

fn create_virtual_terminal(port: u16) -> Arc<Mutex<dyn DeviceWithUi>> {
    device(VirtualTerminal::new(port))
}

fn create_virtual_file_system(port: u16) -> Arc<Mutex<dyn DeviceWithUi>> {
    device(VirtualDOS::new(port))
}

fn create_nmi_trigger(_port: u16) -> Arc<Mutex<dyn DeviceWithUi>> {
    device(NmiTrigger::new())
}

pub fn device_definitions() -> &'static [DeviceDefinition] {
    &[
        DeviceDefinition {
            menu_name: "Add Keypad",
            default_port: "1",
            create: create_keypad,
        },
        DeviceDefinition {
            menu_name: "Add Display",
            default_port: "2",
            create: create_display,
        },
        DeviceDefinition {
            menu_name: "Add LCD Display",
            default_port: "3",
            create: create_lcd_display,
        },
        DeviceDefinition {
            menu_name: "Add Interrupt Controller",
            default_port: "0",
            create: create_interrupt_controller,
        },
        DeviceDefinition {
            menu_name: "Add Virtual Terminal",
            default_port: "0",
            create: create_virtual_terminal,
        },
        DeviceDefinition {
            menu_name: "Add DOS Controller",
            default_port: "20",
            create: create_virtual_file_system,
        },
        DeviceDefinition {
            menu_name: "Add NMI Trigger",
            default_port: "0",
            create: create_nmi_trigger,
        },
    ]
}
