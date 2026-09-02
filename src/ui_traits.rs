use crate::traits::IODevice;
use eframe::egui;

pub trait DeviceWithUi: IODevice {
    fn draw(&mut self, ctx: &egui::Context);
    fn get_name(&self) -> String;
    fn get_window_open_state(&self) -> bool;
    fn set_window_open_state(&mut self, open: bool);
    fn reset(&mut self) {}
}
