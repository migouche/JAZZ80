use crate::assembler::{assemble_binary, assemble_with_metadata};
use crate::cpu::{Flag, GPR};
use crate::emulator::{Machine, MachineSnapshot};
use crate::gui::exec::{Command, Event, Runner};
use crate::traits::SynchronousComponent;
use eframe::egui;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::components::devices::DeviceDefinition;
use crate::ui_traits::DeviceWithUi;

pub mod exec;
mod formatting;
mod highlighting;
mod persistence;
mod session;
mod theme;
mod ui;
mod workspace;

pub use persistence::PersistedWorkspace;
pub use session::DebugSession;
#[allow(unused_imports)]
pub use session::SessionState;
pub(crate) use workspace::EditorTabKind;
pub use workspace::{EditorTab, Workspace};

// Example programs bundled into the web build. `build.rs` generates this
// list from `z80 files/examples.json`, so a missing example fails the build.
#[cfg(target_arch = "wasm32")]
include!(concat!(env!("OUT_DIR"), "/bundled_examples.inc"));

#[cfg(target_arch = "wasm32")]
use std::sync::mpsc::{Receiver, channel};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(inline_js = "
export function download_file(filename, text) {
    var element = document.createElement('a');
    element.setAttribute('href', 'data:text/plain;charset=utf-8,' + encodeURIComponent(text));
    element.setAttribute('download', filename);
    element.style.display = 'none';
    document.body.appendChild(element);
    element.click();
    document.body.removeChild(element);
}

export function download_bytes(filename, data) {
    var blob = new Blob([data], { type: 'application/octet-stream' });
    var url = URL.createObjectURL(blob);
    var element = document.createElement('a');
    element.setAttribute('href', url);
    element.setAttribute('download', filename);
    element.style.display = 'none';
    document.body.appendChild(element);
    element.click();
    document.body.removeChild(element);
    setTimeout(function () { URL.revokeObjectURL(url); }, 1000);
}
")]
extern "C" {
    fn download_file(filename: &str, text: &str);
    fn download_bytes(filename: &str, data: &[u8]);
}

#[derive(Clone)]
enum HeaderAction {
    NewFile,
    OpenFileDialog,
    OpenFile(PathBuf),
    SaveFile,
    SaveFileAs,
    AssembleBinary,
    LoadBinary,
    CloseTab(usize),
    ToggleRun,
    StepInstruction,
    Quit,
}

#[derive(Clone)]
enum ModalType {
    CloseTab(usize),
    Quit,
    AddDevice(&'static DeviceDefinition, String),
}

#[derive(Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
pub(crate) enum SymbolDisplayFormat {
    Default,
    String,
    HexBytes,
    HexWords,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct DeviceSpec {
    pub menu_name: String,
    pub port: u16,
}

pub struct Z80App {
    workspace: Workspace,
    session: DebugSession,

    last_error: Option<String>,
    status_message: Option<String>,

    pending_modal: Option<ModalType>,
    code_theme: highlighting::CodeTheme,

    attached_devices: Vec<Arc<Mutex<dyn DeviceWithUi>>>,

    attached_device_specs: Vec<DeviceSpec>,

    theme: theme::AppTheme,

    #[cfg(target_arch = "wasm32")]
    file_receiver: Option<Receiver<(String, String)>>,

    #[cfg(target_arch = "wasm32")]
    binary_file_receiver: Option<Receiver<(String, Vec<u8>)>>,

    #[cfg(target_arch = "wasm32")]
    examples_list: Vec<String>,

    #[cfg(target_arch = "wasm32")]
    examples_receiver: Option<Receiver<Vec<String>>>,

    #[cfg(target_arch = "wasm32")]
    example_file_receiver: Option<Receiver<Result<(String, Vec<u8>), String>>>,

    show_memory_viewer: bool,
    show_shortcuts: bool,
    memory_viewer_start: String,
    memory_follow_pc: bool,
}

#[cfg(not(target_arch = "wasm32"))]
const APP_ID: &str = "jazz80-simulator";
#[cfg(not(target_arch = "wasm32"))]
const APP_NAME: &str = "JAZZ80 - Z80 Simulator";
const STORAGE_KEY: &str = "z80_workspace";

fn gpr_index(gpr: GPR) -> usize {
    match gpr {
        GPR::A => 0,
        GPR::F => 1,
        GPR::B => 2,
        GPR::C => 3,
        GPR::D => 4,
        GPR::E => 5,
        GPR::H => 6,
        GPR::L => 7,
    }
}

fn parse_hex_u16(text: &str) -> Option<u16> {
    let cleaned = text.trim().replace('_', "");
    let digits = cleaned
        .strip_prefix("0x")
        .or_else(|| cleaned.strip_prefix("0X"))
        .or_else(|| cleaned.strip_prefix('$'))
        .or_else(|| cleaned.strip_prefix('#'))
        .unwrap_or(&cleaned);
    if digits.is_empty() {
        return None;
    }
    u16::from_str_radix(digits, 16).ok()
}

fn parse_hex_u8(text: &str) -> Option<u8> {
    let cleaned = text.trim().replace('_', "");
    let digits = cleaned
        .strip_prefix("0x")
        .or_else(|| cleaned.strip_prefix("0X"))
        .or_else(|| cleaned.strip_prefix('$'))
        .or_else(|| cleaned.strip_prefix('#'))
        .unwrap_or(&cleaned);
    if digits.is_empty() {
        return None;
    }
    u8::from_str_radix(digits, 16).ok()
}

fn snapshot_flag(snapshot: &MachineSnapshot, flag: Flag) -> bool {
    match flag {
        Flag::S => snapshot.flags.s,
        Flag::Z => snapshot.flags.z,
        Flag::Y => snapshot.flags.y,
        Flag::H => snapshot.flags.h,
        Flag::X => snapshot.flags.x,
        Flag::PV => snapshot.flags.pv,
        Flag::N => snapshot.flags.n,
        Flag::C => snapshot.flags.c,
    }
}

#[cfg(target_arch = "wasm32")]
fn z80_files_url(file: &str) -> Option<String> {
    let window = web_sys::window()?;
    let location = window.location();
    let origin = location.origin().ok()?;
    let pathname = location.pathname().ok()?;

    let base = if pathname.ends_with('/') {
        pathname
    } else if let Some(slash) = pathname.rfind('/') {
        pathname[..=slash].to_string()
    } else {
        "/".to_string()
    };

    Some(format!("{origin}{base}z80%20files/{file}"))
}

fn flag_description(flag: Flag) -> &'static str {
    match flag {
        Flag::S => "Sign — copy of bit 7 of the result",
        Flag::Z => "Zero — result was zero",
        Flag::Y => "Undocumented bit 5 copy",
        Flag::H => "Half carry — carry out of bit 3",
        Flag::X => "Undocumented bit 3 copy",
        Flag::PV => "Parity / overflow",
        Flag::N => "Add/subtract — last operation was subtraction",
        Flag::C => "Carry — carry out of bit 7",
    }
}

fn format_constant_value(value: u16, format: SymbolDisplayFormat) -> String {
    let low = (value & 0xFF) as u8;
    let high = (value >> 8) as u8;

    match format {
        SymbolDisplayFormat::Default => format!("0x{value:04X}"),
        SymbolDisplayFormat::String => {
            let mut text = String::new();
            for byte in [low, high] {
                if byte == 0 {
                    break;
                }
                text.push(if (32..=126).contains(&byte) {
                    byte as char
                } else {
                    '.'
                });
            }
            format!("\"{text}\"")
        }
        SymbolDisplayFormat::HexBytes => format!("[{low:02X} {high:02X}]"),
        SymbolDisplayFormat::HexWords => format!("[{value:04X}]"),
    }
}

#[cfg(not(target_arch = "wasm32"))]
const LOGO_PNG: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.png"));

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> eframe::Result<()> {
    let image = image::load_from_memory(LOGO_PNG)
        .expect("Failed to load application logo")
        .into_rgba8();

    let (width, height) = image.dimensions();

    let icon = egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id(APP_ID)
            .with_inner_size([1280.0, 720.0])
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| Ok(Box::new(Z80App::new(cc)))),
    )
}

impl Z80App {
    fn default_code() -> String {
        r"        ORG 0000h
        JP START

; ----------------------------
; Data
; ----------------------------

; Write your variables here

; ----------------------------
; Code
; ----------------------------
START:
; Write your code here:
"
        .to_string()
    }

    fn keybinds() -> Vec<(egui::Modifiers, egui::Key, HeaderAction)> {
        vec![
            (
                egui::Modifiers::COMMAND,
                egui::Key::N,
                HeaderAction::NewFile,
            ),
            (
                egui::Modifiers::COMMAND,
                egui::Key::S,
                HeaderAction::SaveFile,
            ),
            (
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::S,
                HeaderAction::SaveFileAs,
            ),
            (
                egui::Modifiers::COMMAND,
                egui::Key::O,
                HeaderAction::OpenFileDialog,
            ),
            (
                egui::Modifiers::NONE,
                egui::Key::F5,
                HeaderAction::ToggleRun,
            ),
            (
                egui::Modifiers::NONE,
                egui::Key::F8,
                HeaderAction::StepInstruction,
            ),
            (
                egui::Modifiers::COMMAND,
                egui::Key::W,
                HeaderAction::CloseTab(usize::MAX),
            ),
        ]
    }
    /*
        fn line_to_str(&self, line: &str, i: usize) -> String {
            let mut clean = line.split(';').next().unwrap_or("").trim();

            if let Some(idx) = clean.find(':') {
                clean = &clean[idx + 1..].trim();
            }
            let upper = clean.to_uppercase();

            let emits_code = !clean.is_empty() && !upper.starts_with("ORG") && !upper.starts_with("EQU");
            let mut addr_str = "    ".to_string(); // padding, gotta figure out a better way
            if emits_code{
                if let Some(&addr) = self.workspace.line_to_address.get(&i){
                    addr_str = format!("{:04X} ", addr);
                }
            }
            addr_str
        }
    */
    fn check_shortcuts(&self, ctx: &egui::Context) -> Option<HeaderAction> {
        let mut action = None;
        ctx.input_mut(|i| {
            for (modifiers, key, act) in Self::keybinds() {
                if i.consume_key(modifiers, key) {
                    action = Some(act);
                    break;
                }
            }
        });
        action
    }

    fn attach_device_spec(&mut self, spec: &DeviceSpec) {
        let Some(definition) = crate::components::devices::device_definitions()
            .iter()
            .find(|d| d.menu_name == spec.menu_name)
        else {
            return;
        };
        let dev = (definition.create)(spec.port);
        if let Some(machine) = self.session.machine_mut() {
            machine.cpu.attach_device(dev.clone());
        }
        self.attached_devices.push(dev);
    }

    fn load_and_reset(&mut self) {
        if self.session.machine().is_none() {
            let _ = self.session.runner_mut().take_machine();
        }
        self.session.set_machine(Some(Machine::new()));
        {
            let machine = self.session.machine_mut().unwrap();
            for device in &self.attached_devices {
                device.lock().unwrap().reset();
                machine.cpu.attach_device(device.clone());
            }
        }

        self.session.reset();

        if self.workspace.tabs.is_empty() {
            self.workspace.program_loaded = false;
            self.session.refresh_snapshot();
            return;
        }
        self.workspace.clamp_active_tab();

        let active = self.workspace.active_tab;
        let is_binary = self.workspace.tabs[active].kind == EditorTabKind::Binary;

        if is_binary {
            self.workspace.clear_assembly();
            self.last_error = None;

            // `binary_bytes` is not serialised, so a tab restored from storage
            // only has its hexdump. Recover the bytes from the dump instead of
            // leaving a dead tab behind.
            let bytes = {
                let active_tab = &self.workspace.tabs[active];
                active_tab
                    .binary_bytes
                    .clone()
                    .or_else(|| formatting::parse_hexdump(&active_tab.code))
            };
            if let Some(bytes) = bytes {
                let mem = self.session.machine_mut().unwrap().memory_mut();
                for (i, b) in bytes.iter().enumerate() {
                    mem.write(i as u16, *b);
                }
                self.workspace.program_loaded = true;
            } else {
                self.workspace.program_loaded = false;
                self.last_error = Some("Binary tab is missing its raw bytes".to_string());
            }
            self.sync_breakpoints();
            self.session.refresh_snapshot();
            return;
        }

        let assembled = assemble_with_metadata(&self.workspace.tabs[active].code);

        match assembled {
            Ok(mut result) => {
                let bytes = std::mem::take(&mut result.bytes);
                let image = std::mem::take(&mut result.image);
                self.workspace.apply_assembly(result);
                self.last_error = None;

                let mem = self.session.machine_mut().unwrap().memory_mut();
                let image_addr_set: HashSet<u16> = image.iter().map(|(addr, _)| *addr).collect();
                for (addr, b) in &image {
                    mem.write(*addr, *b);
                }
                for (i, b) in bytes.iter().enumerate() {
                    if !image_addr_set.contains(&(i as u16)) {
                        mem.write(i as u16, *b);
                    }
                }
            }
            Err(err) => {
                self.workspace.clear_assembly();
                self.workspace.program_loaded = false;
                self.last_error = Some(err);
            }
        }
        self.sync_breakpoints();
        self.session.refresh_snapshot();
    }

    fn edit_cpu<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Machine) -> R,
    {
        if let Some(machine) = self.session.machine_mut() {
            let result = f(machine);
            self.session.refresh_snapshot();
            result
        } else {
            // Machine is running in the runner; we cannot edit it directly.
            // The UI should send commands instead. This is a fallback that
            // takes the machine back (stopping execution) and applies the edit.
            let mut machine = self.session.runner_mut().take_machine();
            let result = f(&mut machine);
            self.session.set_machine(Some(machine));
            self.session.reset();
            self.session.refresh_snapshot();
            result
        }
    }

    fn sync_breakpoints(&mut self) {
        let addrs = self.workspace.breakpoint_addresses();
        self.session.set_breakpoints(addrs);
    }

    fn start_execution(&mut self) {
        let breakpoints = self.workspace.breakpoint_addresses();
        self.session.set_breakpoints(breakpoints.clone());
        self.session.begin_run(breakpoints);
    }

    fn stop_execution(&mut self) {
        if self.session.machine().is_some() {
            return;
        }
        let machine = self.session.runner_mut().take_machine();
        self.session.set_machine(Some(machine));
        self.session.pause();
        self.session.refresh_snapshot();
    }

    fn step_instruction(&mut self) {
        if !self.workspace.program_loaded && !self.session.is_running() {
            return;
        }
        if self.session.is_running() {
            self.session.runner_mut().send_command(Command::Tick);
        } else if let Some(machine) = self.session.machine_mut() {
            machine.cpu.tick();
            self.session.refresh_snapshot();
        }
    }

    fn resume_from_halt(&mut self) {
        if self.session.is_running() {
            self.session.runner_mut().send_command(Command::Resume);
        } else if let Some(machine) = self.session.machine_mut() {
            machine.cpu.set_halted(false);
            self.session.refresh_snapshot();
        }
    }

    fn toggle_execution(&mut self) {
        if self.session.is_running() {
            self.stop_execution();
            return;
        }
        if !self.workspace.program_loaded {
            return;
        }

        if self.session.snapshot().halted {
            if let Some(machine) = self.session.machine_mut() {
                machine.cpu.set_halted(false);
            }
            self.session.refresh_snapshot();
        }

        if self
            .session
            .breakpoints()
            .contains(&self.session.snapshot().pc)
            && let Some(machine) = self.session.machine_mut()
        {
            // Standing on a breakpoint would stop again immediately; step over.
            machine.cpu.tick();
            self.session.refresh_snapshot();
        }

        self.start_execution();
    }

    fn save_to_storage(&self, storage: Option<&mut (dyn eframe::Storage + 'static)>) {
        if let Some(storage) = storage {
            let persisted =
                PersistedWorkspace::from_parts(&self.workspace, self.attached_device_specs.clone());
            eframe::set_value(storage, STORAGE_KEY, &persisted);
            storage.flush();
        }
    }

    fn restore_from_storage(storage: Option<&dyn eframe::Storage>) -> Self {
        let mut app =
            match storage.and_then(|s| eframe::get_value::<PersistedWorkspace>(s, STORAGE_KEY)) {
                Some(persisted) => {
                    let (mut workspace, specs) = persisted.into_parts();
                    let mut max_id = 0u64;
                    for tab in &mut workspace.tabs {
                        if tab.id == 0 {
                            max_id = max_id.wrapping_add(1);
                            tab.id = max_id;
                        } else if tab.id > max_id {
                            max_id = tab.id;
                        }
                    }
                    if workspace.tabs.is_empty() {
                        max_id = max_id.wrapping_add(1);
                        workspace.tabs.push(EditorTab {
                            id: max_id,
                            path: None,
                            code: Self::default_code(),
                            kind: EditorTabKind::Source,
                            is_dirty: false,
                            breakpoints: HashSet::new(),
                            binary_bytes: None,
                        });
                    }
                    workspace.next_tab_id = if max_id == u64::MAX { 1 } else { max_id + 1 };
                    workspace.clamp_active_tab();
                    Self {
                        workspace,
                        attached_device_specs: specs,
                        ..Self::default()
                    }
                }
                None => Self::default(),
            };

        // Devices are not serialised directly; rebuild them from their specs.
        let specs = app.attached_device_specs.clone();
        for spec in &specs {
            app.attach_device_spec(spec);
        }

        for tab in &mut app.workspace.tabs {
            if tab.kind == EditorTabKind::Binary && tab.binary_bytes.is_none() {
                tab.binary_bytes = formatting::parse_hexdump(&tab.code);
            }
        }

        app
    }

    pub fn new(cc: &eframe::CreationContext) -> Self {
        theme::apply(&cc.egui_ctx);

        let mut app = Self::restore_from_storage(cc.storage);

        #[cfg(target_arch = "wasm32")]
        app.start_examples_fetch(&cc.egui_ctx);

        app.load_and_reset();
        app
    }

    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
    fn open_file_dialog(
        &mut self,
        ctx: &egui::Context,
        storage: Option<&mut (dyn eframe::Storage + 'static)>,
    ) {
        // "Open" is for Z80 assembly source only. Use "Load Binary..." to import
        // a raw binary image.
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Assembly", &["asm", "z80"])
            .add_filter("All files", &["*"])
            .pick_file()
        {
            self.open_file(path, ctx, storage);
        }

        #[cfg(target_arch = "wasm32")]
        {
            let task = rfd::AsyncFileDialog::new()
                .add_filter("Assembly", &["asm", "z80"])
                .add_filter("All files", &["*"])
                .pick_file();

            let (sender, receiver) = channel();
            self.file_receiver = Some(receiver);

            let ctx = ctx.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let file = task.await;
                if let Some(file) = file {
                    let name = file.file_name();
                    let content = file.read().await;
                    if let Ok(text) = String::from_utf8(content) {
                        sender.send((name, text)).ok();
                    }
                    ctx.request_repaint();
                }
            });
        }
    }

    fn new_file(&mut self, storage: Option<&mut (dyn eframe::Storage + 'static)>) {
        self.workspace.add_source_tab(None, Self::default_code());
        self.workspace.is_assembly_stale = true;
        self.save_to_storage(storage);
    }

    fn close_tab(&mut self, index: usize, storage: Option<&mut (dyn eframe::Storage + 'static)>) {
        if index >= self.workspace.tabs.len() {
            return;
        }
        if self.workspace.close(index) {
            self.new_file(storage); // Ensure at least one tab
            self.load_and_reset();
        } else {
            self.save_to_storage(storage);
        }
    }

    fn open_binary_tab(
        &mut self,
        path: PathBuf,
        bytes: Vec<u8>,
        storage: Option<&mut (dyn eframe::Storage + 'static)>,
    ) {
        let hexdump = formatting::binary_hexdump(&bytes);
        self.workspace.open_binary(path.clone(), bytes, hexdump);
        self.add_recent_file(path);
        self.workspace.is_assembly_stale = false;
        self.save_to_storage(storage);
        self.load_and_reset();
    }

    fn load_file_content(
        &mut self,
        path: PathBuf,
        content: String,
        storage: Option<&mut (dyn eframe::Storage + 'static)>,
    ) {
        self.workspace
            .open_source(path.clone(), content, &Self::default_code());
        self.add_recent_file(path);
        self.load_and_reset();
        self.save_to_storage(storage);
    }

    fn open_file(
        &mut self,
        path: PathBuf,
        _ctx: &egui::Context,
        storage: Option<&mut (dyn eframe::Storage + 'static)>,
    ) {
        // Check if already open
        if let Some(idx) = self
            .workspace
            .tabs
            .iter()
            .position(|t| t.path.as_ref() == Some(&path))
        {
            if self.workspace.active_tab != idx {
                self.workspace.active_tab = idx;
                self.workspace.is_assembly_stale = true;
            }
            self.save_to_storage(storage);
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    self.load_file_content(path, content, storage);
                }
                Err(err) => {
                    self.last_error = Some(format!("Failed to open file: {}", err));
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.load_example(path.to_string_lossy().into_owned(), _ctx, storage);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn start_examples_fetch(&mut self, ctx: &egui::Context) {
        let Some(url) = z80_files_url("examples.json") else {
            return;
        };

        let (sender, receiver) = channel();
        self.examples_receiver = Some(receiver);

        let ctx = ctx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match reqwest::get(&url).await {
                Ok(response) if response.status().is_success() => {
                    match response.json::<Vec<String>>().await {
                        Ok(examples) => {
                            sender.send(examples).ok();
                        }
                        Err(err) => {
                            web_sys::console::error_1(
                                &format!("Failed to parse examples.json: {err}").into(),
                            );
                        }
                    }
                }
                Ok(response) => {
                    web_sys::console::error_1(
                        &format!("examples.json returned {}", response.status()).into(),
                    );
                }
                Err(err) => {
                    web_sys::console::error_1(
                        &format!("Failed to fetch examples.json: {err}").into(),
                    );
                }
            }
            ctx.request_repaint();
        });
    }

    #[cfg(target_arch = "wasm32")]
    fn load_example(
        &mut self,
        filename: String,
        ctx: &egui::Context,
        _storage: Option<&mut (dyn eframe::Storage + 'static)>,
    ) {
        let Some(url) = z80_files_url(&filename) else {
            self.last_error = Some("Could not determine the examples URL".into());
            return;
        };

        let (sender, receiver) = channel();
        self.example_file_receiver = Some(receiver);

        let ctx = ctx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = match reqwest::get(&url).await {
                Ok(response) if response.status().is_success() => match response.bytes().await {
                    Ok(bytes) => Ok((filename.clone(), bytes.to_vec())),
                    Err(err) => Err(format!("Failed to read example '{filename}': {err}")),
                },
                Ok(response) => Err(format!(
                    "Failed to load example '{filename}': HTTP {}",
                    response.status()
                )),
                Err(err) => Err(format!("Failed to load example '{filename}': {err}")),
            };
            sender.send(result).ok();
            ctx.request_repaint();
        });
    }

    fn save_all(&mut self, mut storage: Option<&mut (dyn eframe::Storage + 'static)>) -> bool {
        let prev_active = self.workspace.active_tab;
        for i in 0..self.workspace.tabs.len() {
            if self.workspace.tabs[i].is_dirty {
                self.workspace.active_tab = i;
                self.save_file(i, storage.as_deref_mut());
                if self.workspace.tabs[i].is_dirty {
                    break;
                }
            }
        }
        self.workspace.active_tab = prev_active;
        !self.workspace.tabs.iter().any(|tab| tab.is_dirty)
    }

    fn remove_device(&mut self, idx: usize) {
        if idx >= self.attached_devices.len() {
            return;
        }
        let name = self.attached_devices[idx].lock().unwrap().get_name();
        self.attached_devices.remove(idx);
        if idx < self.attached_device_specs.len() {
            self.attached_device_specs.remove(idx);
        }

        // Detach immediately rather than waiting for the next reset: a device
        // the user removed should stop responding to I/O right away.
        if let Some(machine) = self.session.machine_mut() {
            machine.cpu.clear_devices();
            for dev in &self.attached_devices {
                machine.cpu.attach_device(dev.clone());
            }
        }
        self.status_message = Some(format!("Removed {name}"));
    }

    fn save_file(&mut self, tab_idx: usize, storage: Option<&mut (dyn eframe::Storage + 'static)>) {
        // A binary tab's `code` is a hexdump, not source. Writing it back would
        // destroy the .bin file it was loaded from.
        if self
            .workspace
            .tabs
            .get(tab_idx)
            .is_some_and(|tab| tab.kind == EditorTabKind::Binary)
        {
            self.status_message =
                Some("Binary tabs are read-only. Use File → Assemble Binary to export.".into());
            return;
        }

        #[cfg(target_arch = "wasm32")]
        {
            // On Wasm, we always use "Save As" behavior because we can't write to disk silently
            if tab_idx == self.workspace.active_tab {
                self.save_file_as(storage);
            } else {
                // Switching tab on wasm needed? Or just ignore background saves?
                // For now, only save active tab or ignore.
                // Or temporarily switch active tab?
                // Simpler to just warn.
                self.status_message =
                    Some("Only the active file can be saved on the web".to_string());
            }
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let Some(tab) = self.workspace.tabs.get_mut(tab_idx) else {
                return;
            };
            if let Some(path) = &tab.path {
                let path_clone = path.clone(); // Clone to avoid borrow check issues if we needed it later, but here write takes reference
                if let Err(err) = std::fs::write(&path_clone, &tab.code) {
                    self.last_error = Some(format!("Failed to save file: {}", err));
                } else {
                    tab.is_dirty = false;
                    self.status_message = Some(format!("Saved {}", path_clone.display()));
                }
            } else {
                if tab_idx == self.workspace.active_tab {
                    self.save_file_as(storage);
                } else {
                    self.status_message =
                        Some("Only the active untitled file can be saved".to_string());
                }
                return;
            }

            if let Some(path) = self
                .workspace
                .tabs
                .get(tab_idx)
                .and_then(|tab| tab.path.as_ref())
            {
                self.add_recent_file(path.clone());
            }
            self.save_to_storage(storage);
        }
    }

    fn save_file_as(&mut self, storage: Option<&mut (dyn eframe::Storage + 'static)>) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Assembly", &["asm", "z80"])
            .save_file()
        {
            let Some(tab) = self.workspace.tabs.get_mut(self.workspace.active_tab) else {
                return;
            };
            if let Err(err) = std::fs::write(&path, &tab.code) {
                self.last_error = Some(format!("Failed to save file: {}", err));
            } else {
                tab.path = Some(path.clone());
                tab.is_dirty = false;
                self.status_message = Some(format!("Saved {}", path.display()));
                self.add_recent_file(path);
                self.save_to_storage(storage);
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            let Some(tab) = self.workspace.tabs.get_mut(self.workspace.active_tab) else {
                return;
            };
            let name = tab
                .path
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("program.asm");

            download_file(name, &tab.code);
            self.status_message = Some(format!("Downloaded {name}"));

            tab.is_dirty = false;
            self.save_to_storage(storage);
        }
    }

    fn assemble_binary_to_file(&mut self, storage: Option<&mut (dyn eframe::Storage + 'static)>) {
        let code = self
            .workspace
            .tabs
            .get(self.workspace.active_tab)
            .map(|tab| tab.code.clone());
        let Some(code) = code else {
            self.last_error = Some("No active tab to assemble".to_string());
            return;
        };

        match assemble_binary(&code) {
            Ok(bytes) => {
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Binary", &["bin", "rom"])
                    .add_filter("All files", &["*"])
                    .save_file()
                {
                    if let Err(err) = std::fs::write(&path, &bytes) {
                        self.last_error = Some(format!("Failed to export binary: {}", err));
                    } else {
                        self.last_error = None;
                        self.status_message = Some(format!(
                            "Exported {} ({} bytes)",
                            path.display(),
                            bytes.len()
                        ));
                        self.add_recent_file(path);
                        self.save_to_storage(storage);
                    }
                    return;
                }

                #[cfg(target_arch = "wasm32")]
                {
                    let name = self
                        .workspace
                        .tabs
                        .get(self.workspace.active_tab)
                        .and_then(|tab| tab.path.as_ref())
                        .and_then(|p| p.file_stem())
                        .and_then(|n| n.to_str())
                        .unwrap_or("program")
                        .to_string()
                        + ".bin";
                    download_bytes(&name, &bytes);
                    self.last_error = None;
                    self.status_message =
                        Some(format!("Exported {} ({} bytes)", name, bytes.len()));
                    self.save_to_storage(storage);
                    return;
                }

                #[cfg(not(target_arch = "wasm32"))]
                {
                    self.status_message = Some("Binary export cancelled".to_string());
                }
            }
            Err(err) => {
                self.last_error = Some(format!("Failed to assemble binary: {}", err));
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_binary_file(
        &mut self,
        path: PathBuf,
        storage: Option<&mut (dyn eframe::Storage + 'static)>,
    ) {
        match std::fs::read(&path) {
            Ok(bytes) => {
                self.open_binary_tab(path.clone(), bytes, storage);
                self.session.reset();
                self.last_error = None;
            }
            Err(err) => {
                self.last_error = Some(format!("Failed to load binary: {}", err));
            }
        }
    }

    #[cfg_attr(target_arch = "wasm32", allow(unused_variables))]
    fn load_binary_dialog(
        &mut self,
        _ctx: &egui::Context,
        storage: Option<&mut (dyn eframe::Storage + 'static)>,
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Binary", &["bin", "rom", "com"])
            .add_filter("All files", &["*"])
            .pick_file()
        {
            self.load_binary_file(path, storage);
        }

        #[cfg(target_arch = "wasm32")]
        {
            let task = rfd::AsyncFileDialog::new()
                .add_filter("Binary", &["bin", "rom", "com"])
                .add_filter("All files", &["*"])
                .pick_file();

            let (sender, receiver) = channel();
            self.binary_file_receiver = Some(receiver);

            let ctx = _ctx.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Some(file) = task.await {
                    let name = file.file_name();
                    let bytes = file.read().await;
                    sender.send((name, bytes)).ok();
                    ctx.request_repaint();
                }
            });
        }
    }

    fn process_header_action(
        &mut self,
        action: HeaderAction,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
    ) {
        match action {
            HeaderAction::NewFile => self.new_file(frame.storage_mut()),
            HeaderAction::OpenFileDialog => self.open_file_dialog(ctx, frame.storage_mut()),
            HeaderAction::OpenFile(path) => self.open_file(path, ctx, frame.storage_mut()),
            HeaderAction::SaveFile => {
                self.save_file(self.workspace.active_tab, frame.storage_mut())
            }
            HeaderAction::SaveFileAs => self.save_file_as(frame.storage_mut()),
            HeaderAction::AssembleBinary => self.assemble_binary_to_file(frame.storage_mut()),
            HeaderAction::LoadBinary => self.load_binary_dialog(ctx, frame.storage_mut()),
            HeaderAction::CloseTab(idx) => {
                let idx = if idx == usize::MAX {
                    self.workspace.active_tab
                } else {
                    idx
                };
                if idx >= self.workspace.tabs.len() {
                    return;
                }
                if self.workspace.tabs[idx].is_dirty {
                    self.pending_modal = Some(ModalType::CloseTab(idx));
                } else {
                    self.close_tab(idx, frame.storage_mut());
                }
            }
            HeaderAction::ToggleRun => self.toggle_execution(),
            HeaderAction::StepInstruction => self.step_instruction(),
            HeaderAction::Quit => {
                // This triggers on_close_event
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    fn poll_execution(&mut self, ctx: &egui::Context) {
        if self.session.is_running() {
            for event in self.session.runner_mut().poll() {
                match event {
                    Event::Snapshot(delta) => {
                        self.session.apply_delta(delta);
                    }
                    Event::Finished(addr) => {
                        self.session.finish_run(addr);
                    }
                }
            }
            if self.session.is_running() {
                ctx.request_repaint_after(Duration::from_millis(16));
            }
            self.session.ensure_machine_reclaimed();
        }
    }

    fn add_recent_file(&mut self, path: PathBuf) {
        // Embedded examples are opened by name and do not exist on disk on
        // native, so they would produce dead "recent file" entries.
        #[cfg(not(target_arch = "wasm32"))]
        if !path.exists() {
            return;
        }

        // On the web there is no filesystem, and recent entries are reopened by
        // looking up an embedded example by name. Only remember those.
        #[cfg(target_arch = "wasm32")]
        if !self
            .examples_list
            .iter()
            .any(|name| PathBuf::from(name) == path)
        {
            return;
        }

        // Remove if exists to move it to the top
        self.workspace.add_recent_file(path);
    }
}

impl Default for Z80App {
    fn default() -> Self {
        Self {
            workspace: Workspace::with_default_tab(Self::default_code()),
            session: DebugSession::new(),
            last_error: None,
            status_message: None,
            pending_modal: None,
            code_theme: highlighting::CodeTheme::one_dark_pro_vivid(),
            attached_devices: Vec::new(),
            attached_device_specs: Vec::new(),
            theme: theme::AppTheme::ONE_DARK,
            #[cfg(target_arch = "wasm32")]
            file_receiver: None,
            #[cfg(target_arch = "wasm32")]
            binary_file_receiver: None,
            #[cfg(target_arch = "wasm32")]
            examples_list: Vec::new(),
            #[cfg(target_arch = "wasm32")]
            examples_receiver: None,
            #[cfg(target_arch = "wasm32")]
            example_file_receiver: None,
            show_memory_viewer: false,
            show_shortcuts: false,
            memory_viewer_start: "0000".to_string(),
            memory_follow_pc: false,
        }
    }
}

impl eframe::App for Z80App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let persisted =
            PersistedWorkspace::from_parts(&self.workspace, self.attached_device_specs.clone());
        eframe::set_value(storage, STORAGE_KEY, &persisted);
    }

    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(receiver) = self.file_receiver.take() {
                while let Ok((name, content)) = receiver.try_recv() {
                    self.load_file_content(PathBuf::from(name), content, frame.storage_mut());
                }
                self.file_receiver = Some(receiver);
            }
            if let Some(receiver) = self.binary_file_receiver.take() {
                while let Ok((name, bytes)) = receiver.try_recv() {
                    self.open_binary_tab(PathBuf::from(name), bytes, frame.storage_mut());
                    self.last_error = None;
                }
                self.binary_file_receiver = Some(receiver);
            }
            if let Some(receiver) = self.example_file_receiver.take() {
                while let Ok(result) = receiver.try_recv() {
                    match result {
                        Ok((name, bytes)) => {
                            if name.ends_with(".bin") {
                                self.open_binary_tab(
                                    PathBuf::from(&name),
                                    bytes,
                                    frame.storage_mut(),
                                );
                            } else {
                                match String::from_utf8(bytes) {
                                    Ok(text) => {
                                        self.load_file_content(
                                            PathBuf::from(&name),
                                            text,
                                            frame.storage_mut(),
                                        );
                                        self.status_message =
                                            Some(format!("Loaded example {name}"));
                                    }
                                    Err(_) => {
                                        self.last_error =
                                            Some(format!("Example '{name}' is not valid UTF-8"));
                                    }
                                }
                            }
                        }
                        Err(err) => self.last_error = Some(err),
                    }
                }
                self.example_file_receiver = Some(receiver);
            }
            if let Some(receiver) = self.examples_receiver.take() {
                match receiver.try_recv() {
                    Ok(examples) => self.examples_list = examples,
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        self.examples_receiver = Some(receiver);
                    }
                    // The fetch failed (the worker logs the reason). Leave the
                    // list empty so the menu shows "No examples found".
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {}
                }
            }
        }

        // Handle window close request
        if ctx.input(|i| i.viewport().close_requested()) {
            let any_dirty = self.workspace.tabs.iter().any(|t| t.is_dirty);
            if any_dirty {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.pending_modal = Some(ModalType::Quit);
            }
        }

        let mut action = self.check_shortcuts(&ctx);

        // Top Panel: Menu Bar and Control Toolbar
        egui::Panel::top("top_panel").show(ui, |ui| {
            if let Some(header_action) = ui::header::show(self, frame, ui) {
                action = Some(header_action);
            }
        });

        // Bottom Status Bar. Added before the side panels so it spans the full
        // window width, which gives long error messages room to breathe.
        egui::Panel::bottom("status_bar").show(ui, |ui| ui::status_bar::show(self, ui));

        // Right Panel: Registers and Flags
        egui::Panel::right("right_panel")
            .resizable(true)
            .default_size(280.0)
            .show(ui, |ui| ui::panels::show_registers(self, ui));

        egui::Panel::right("vars_panel")
            .resizable(true)
            .default_size(280.0)
            .show(ui, |ui| ui::panels::show_variables(self, ui));

        // Central Panel: Code Editor
        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(editor_action) = ui::editor::show(self, ui) {
                action = Some(editor_action);
            }
        });

        // Breakpoints may have been toggled in the gutter during this frame. Sync
        // them before dispatching so a run started by this frame's action sees
        // the breakpoints the user just set.
        self.sync_breakpoints();

        if let Some(action) = action {
            self.process_header_action(action, &ctx, frame);
        }

        // Handle Modal Dialogs
        ui::modals::show(self, &ctx, frame);

        // Keyboard shortcut reference
        ui::windows::show_shortcuts(self, &ctx);

        // Poll execution events from the runner
        self.poll_execution(&ctx);

        // Draw Memory Viewer Window
        ui::windows::show_memory_viewer(self, ui);

        // Draw separate windows for devices
        ui::windows::show_devices(self, &ctx);
    }
}

#[cfg(test)]
mod register_input_tests {
    use super::{parse_hex_u8, parse_hex_u16};

    #[test]
    fn accepts_plain_and_prefixed_hex() {
        assert_eq!(parse_hex_u16("1A2B"), Some(0x1A2B));
        assert_eq!(parse_hex_u16("0x1a2b"), Some(0x1A2B));
        assert_eq!(parse_hex_u16("$1A2B"), Some(0x1A2B));
        assert_eq!(parse_hex_u16("#1a2b"), Some(0x1A2B));
        assert_eq!(parse_hex_u8("0x00"), Some(0));
        assert_eq!(parse_hex_u8("FF"), Some(0xFF));
    }

    #[test]
    fn ignores_surrounding_space_and_separators() {
        assert_eq!(parse_hex_u16(" 0x1A_2B "), Some(0x1A2B));
    }

    #[test]
    fn rejects_empty_and_out_of_range() {
        assert_eq!(parse_hex_u16(""), None);
        assert_eq!(parse_hex_u16("0x"), None);
        assert_eq!(parse_hex_u16("10000"), None);
        assert_eq!(parse_hex_u8("100"), None);
        assert_eq!(parse_hex_u8("zz"), None);
    }
}

#[cfg(test)]
mod constant_format_tests {
    use super::{SymbolDisplayFormat, format_constant_value};

    #[test]
    fn default_shows_a_hex_word() {
        assert_eq!(
            format_constant_value(0x1234, SymbolDisplayFormat::Default),
            "0x1234"
        );
    }

    #[test]
    fn hex_formats_use_little_endian_bytes() {
        assert_eq!(
            format_constant_value(0x1234, SymbolDisplayFormat::HexBytes),
            "[34 12]"
        );
        assert_eq!(
            format_constant_value(0x1234, SymbolDisplayFormat::HexWords),
            "[1234]"
        );
    }

    #[test]
    fn string_format_stops_at_a_null_byte() {
        assert_eq!(
            format_constant_value(0x4241, SymbolDisplayFormat::String),
            "\"AB\""
        );
        assert_eq!(
            format_constant_value(0x0041, SymbolDisplayFormat::String),
            "\"A\""
        );
    }
}

#[cfg(test)]
mod workspace_load_tests {
    use super::{EditorTab, EditorTabKind, PersistedWorkspace, STORAGE_KEY, Workspace, Z80App};
    use eframe::Storage;
    use std::collections::{HashMap, HashSet};

    #[derive(Default)]
    struct MemoryStorage {
        values: HashMap<String, String>,
    }

    impl eframe::Storage for MemoryStorage {
        fn get_string(&self, key: &str) -> Option<String> {
            self.values.get(key).cloned()
        }

        fn set_string(&mut self, key: &str, value: String) {
            self.values.insert(key.to_string(), value);
        }

        fn remove_string(&mut self, key: &str) {
            self.values.remove(key);
        }

        fn flush(&mut self) {}
    }

    fn extra_source_tab() -> EditorTab {
        EditorTab {
            id: 2,
            path: None,
            code: "NOP".to_string(),
            kind: EditorTabKind::Source,
            is_dirty: false,
            breakpoints: HashSet::new(),
            binary_bytes: None,
        }
    }

    fn persisted_workspace() -> PersistedWorkspace {
        PersistedWorkspace::from_parts(&Workspace::with_default_tab(String::new()), Vec::new())
    }

    fn store(workspace: &PersistedWorkspace) -> MemoryStorage {
        let mut storage = MemoryStorage::default();
        eframe::set_value(&mut storage, STORAGE_KEY, workspace);
        storage
    }

    #[test]
    fn restore_clamps_an_out_of_range_active_tab() {
        let mut persisted = persisted_workspace();
        persisted.tabs.push(extra_source_tab());
        persisted.active_tab = 99;

        let restored = Z80App::restore_from_storage(Some(&store(&persisted)));

        assert_eq!(restored.workspace.tabs.len(), 2);
        assert_eq!(restored.workspace.active_tab, 1);
    }

    #[test]
    fn restore_keeps_an_in_range_active_tab() {
        let mut persisted = persisted_workspace();
        persisted.tabs.push(extra_source_tab());
        persisted.active_tab = 0;

        let restored = Z80App::restore_from_storage(Some(&store(&persisted)));

        assert_eq!(restored.workspace.active_tab, 0);
    }

    #[test]
    fn restore_of_an_empty_workspace_adds_a_tab_and_resets_the_index() {
        let mut persisted = persisted_workspace();
        persisted.tabs.clear();
        persisted.active_tab = 7;

        let restored = Z80App::restore_from_storage(Some(&store(&persisted)));

        assert_eq!(restored.workspace.tabs.len(), 1);
        assert_eq!(restored.workspace.active_tab, 0);
    }

    #[test]
    fn restore_defaults_a_missing_active_tab_to_zero() {
        let persisted = persisted_workspace();
        let storage = store(&persisted);
        let serialized = storage.get_string(STORAGE_KEY).unwrap();
        let field_start = serialized.find("active_tab:").unwrap();
        let field_end = field_start + serialized[field_start..].find(',').unwrap();
        let mut without_active = storage;
        without_active.set_string(
            STORAGE_KEY,
            format!(
                "{}{}",
                &serialized[..field_start],
                &serialized[field_end + 1..]
            ),
        );

        let restored = Z80App::restore_from_storage(Some(&without_active));

        assert_eq!(restored.workspace.active_tab, 0);
    }
}
