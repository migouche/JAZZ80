mod assembler;
mod components;
mod cpu;
mod emulator;
mod gui;
mod traits;
mod ui_traits;

#[cfg(not(target_arch = "wasm32"))]
#[cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() -> eframe::Result<()> {
    gui::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        use wasm_bindgen::JsCast;
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");
        let canvas = document
            .get_element_by_id("z80-simulator-canvas")
            .expect("Failed to find z80-simulator-canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("z80-simulator-canvas was not a HtmlCanvasElement");

        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(gui::Z80App::new(cc)) as Box<dyn eframe::App>)),
            )
            .await
            .expect("failed to start eframe");
    });
}
