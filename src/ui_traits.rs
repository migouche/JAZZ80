use crate::traits::IODevice;
use eframe::egui;

pub trait DeviceWithUi: IODevice + Send {
    fn draw(&mut self, ctx: &egui::Context);
    fn get_name(&self) -> String;
    fn get_window_open_state(&self) -> bool;
    fn set_window_open_state(&mut self, open: bool);

    fn show_window(
        &mut self,
        ctx: &egui::Context,
        min_width: Option<f32>,
        contents: &mut dyn FnMut(&mut Self, &mut egui::Ui),
    ) where
        Self: Sized,
    {
        let mut open = self.get_window_open_state();
        let title = self.get_name();
        let mut window = egui::Window::new(title).open(&mut open);
        if let Some(min_width) = min_width {
            window = window.min_width(min_width);
        }
        window.show(ctx, |ui| contents(self, ui));
        self.set_window_open_state(open);
    }

    fn reset(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::Harness;

    struct TestDevice {
        is_open: bool,
        draws: usize,
    }

    impl TestDevice {
        fn new(is_open: bool) -> Self {
            Self { is_open, draws: 0 }
        }
    }

    impl IODevice for TestDevice {
        fn read_in(&mut self, _port: u16) -> Option<u8> {
            None
        }

        fn write_out(&mut self, _port: u16, _data: u8) -> bool {
            false
        }
    }

    impl DeviceWithUi for TestDevice {
        fn draw(&mut self, ctx: &egui::Context) {
            self.show_window(ctx, None, &mut |this, ui| {
                this.draws += 1;
                ui.label("test");
            });
        }

        fn get_name(&self) -> String {
            "Test Device".to_string()
        }

        fn get_window_open_state(&self) -> bool {
            self.is_open
        }

        fn set_window_open_state(&mut self, open: bool) {
            self.is_open = open;
        }
    }

    #[test]
    fn open_window_runs_contents() {
        let mut harness = Harness::new_ui_state(
            |ui, device: &mut TestDevice| device.draw(ui.ctx()),
            TestDevice::new(true),
        );
        harness.run();
        assert!(harness.state().draws > 0);
    }

    #[test]
    fn closed_window_does_not_run_contents() {
        let mut harness = Harness::new_ui_state(
            |ui, device: &mut TestDevice| device.draw(ui.ctx()),
            TestDevice::new(false),
        );
        harness.run();
        assert_eq!(harness.state().draws, 0);
    }

    #[test]
    fn open_state_survives_a_frame() {
        let mut harness = Harness::new_ui_state(
            |ui, device: &mut TestDevice| device.draw(ui.ctx()),
            TestDevice::new(true),
        );
        harness.run();
        assert!(harness.state().is_open);
    }
}
