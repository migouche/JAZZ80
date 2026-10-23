use crate::assembler::{AssemblyLine, Symbol, SymbolType, assemble_binary, assemble_with_metadata};
use crate::cpu::{Flag, GPR};
use crate::emulator::{Machine, MachineSnapshot, SnapshotDelta};
use crate::gui::exec::{Command, Event, Runner, RunnerImpl};
use crate::traits::SynchronousComponent;
use eframe::egui::{self, TextBuffer};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::components::devices::DeviceDefinition;
use crate::ui_traits::DeviceWithUi;

pub mod exec;
mod formatting;
mod highlighting;
mod theme;

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
enum SymbolDisplayFormat {
    Default,
    String,
    HexBytes,
    HexWords,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum EditorTabKind {
    #[default]
    Source,
    Binary,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct EditorTab {
    pub path: Option<PathBuf>,
    pub code: String,
    #[serde(default)]
    pub kind: EditorTabKind,
    pub is_dirty: bool,
    #[serde(default)]
    pub breakpoints: HashSet<usize>,
    #[serde(skip)]
    pub binary_bytes: Option<Vec<u8>>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct DeviceSpec {
    pub menu_name: String,
    pub port: u16,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Z80App {
    #[serde(skip)]
    exec: ExecutionState,

    tabs: Vec<EditorTab>,
    active_tab: usize,

    #[serde(skip)]
    last_error: Option<String>,
    #[serde(skip)]
    status_message: Option<String>,
    #[serde(skip)]
    symbol_table: HashMap<String, Symbol>,
    #[serde(skip)]
    address_to_line: HashMap<u16, usize>,
    #[serde(skip)]
    line_to_address: HashMap<usize, u16>,
    #[serde(skip)]
    assembly_lines: HashMap<usize, AssemblyLine>,

    #[serde(default)]
    symbol_display_prefs: HashMap<String, SymbolDisplayFormat>,

    #[serde(skip)]
    pending_modal: Option<ModalType>,
    #[serde(skip)]
    code_theme: highlighting::CodeTheme,

    #[serde(skip)]
    attached_devices: Vec<Arc<Mutex<dyn DeviceWithUi>>>,

    #[serde(default)]
    attached_device_specs: Vec<DeviceSpec>,

    #[serde(skip)]
    theme: theme::AppTheme,

    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    file_receiver: Option<Receiver<(String, String)>>,

    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    binary_file_receiver: Option<Receiver<(String, Vec<u8>)>>,

    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    examples_list: Vec<String>,

    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    examples_receiver: Option<Receiver<Vec<String>>>,

    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    example_file_receiver: Option<Receiver<Result<(String, Vec<u8>), String>>>,

    recent_files: Vec<PathBuf>,

    #[serde(skip)]
    show_memory_viewer: bool,
    #[serde(skip)]
    show_shortcuts: bool,
    #[serde(skip)]
    memory_viewer_start: String,
    #[serde(skip)]
    memory_follow_pc: bool,

    #[serde(default)]
    show_location_counter: bool,
    #[serde(skip)]
    is_assembly_stale: bool,
    #[serde(skip)]
    program_loaded: bool,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ExecutionState {
    #[serde(skip)]
    machine: Option<Machine>,
    #[serde(skip)]
    snapshot: MachineSnapshot,
    #[serde(skip)]
    runner: RunnerImpl,
    #[serde(skip)]
    breakpoint_addrs: HashSet<u16>,
    #[serde(skip)]
    is_running: bool,
    #[serde(skip)]
    last_stop_addr: Option<u16>,
}

#[cfg(not(target_arch = "wasm32"))]
const APP_ID: &str = "jazz80-simulator";
#[cfg(not(target_arch = "wasm32"))]
const APP_NAME: &str = "JAZZ80 - Z80 Simulator";
const STORAGE_KEY: &str = "z80_workspace";

impl ExecutionState {
    pub fn new() -> Self {
        Self {
            machine: Some(Machine::new()),
            snapshot: MachineSnapshot::default(),
            runner: crate::gui::exec::create_runner(),
            breakpoint_addrs: HashSet::new(),
            is_running: false,
            last_stop_addr: None,
        }
    }

    pub fn refresh_snapshot(&mut self) {
        if let Some(machine) = self.machine.as_mut() {
            self.snapshot = machine.snapshot();
        }
    }

    pub fn apply_delta(&mut self, delta: SnapshotDelta) {
        for (addr, value) in delta.memory {
            self.snapshot.memory[addr as usize] = value;
        }
        let memory = std::mem::take(&mut self.snapshot.memory);
        self.snapshot = delta.state;
        self.snapshot.memory = memory;
    }

    pub fn sync_breakpoints(
        &mut self,
        line_to_address: &HashMap<usize, u16>,
        active_tab: usize,
        tabs: &[EditorTab],
    ) {
        let mut addrs = HashSet::new();
        if let Some(tab) = tabs.get(active_tab) {
            for &line in &tab.breakpoints {
                if let Some(&addr) = line_to_address.get(&line) {
                    addrs.insert(addr);
                }
            }
        }
        let changed = addrs != self.breakpoint_addrs;
        self.breakpoint_addrs = addrs;
        if changed && self.is_running {
            self.runner
                .send_command(Command::SetBreakpoints(self.breakpoint_addrs.clone()));
        }
    }

    pub fn ensure_machine_reclaimed(&mut self) {
        if self.machine.is_none() && !self.runner.is_running() {
            self.machine = Some(self.runner.take_machine());
            self.refresh_snapshot();
        }
    }
}

impl Default for ExecutionState {
    fn default() -> Self {
        Self::new()
    }
}

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
                if let Some(&addr) = self.line_to_address.get(&i){
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

    fn active_tab_name(&self) -> String {
        self.tabs
            .get(self.active_tab)
            .and_then(|t| t.path.as_ref())
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
            .to_string()
    }

    fn active_tab_path(&self) -> Option<String> {
        self.tabs
            .get(self.active_tab)
            .and_then(|t| t.path.as_ref())
            .map(|p| p.display().to_string())
    }

    fn attach_device_spec(&mut self, spec: &DeviceSpec) {
        let Some(definition) = crate::components::devices::device_definitions()
            .iter()
            .find(|d| d.menu_name == spec.menu_name)
        else {
            return;
        };
        let dev = (definition.create)(spec.port);
        if let Some(machine) = self.exec.machine.as_mut() {
            machine.cpu.attach_device(dev.clone());
        }
        self.attached_devices.push(dev);
    }

    fn load_and_reset(&mut self) {
        if self.exec.machine.is_none() {
            let _ = self.exec.runner.take_machine();
        }
        self.exec.machine = Some(Machine::new());
        let machine = self.exec.machine.as_mut().unwrap();
        for device in &self.attached_devices {
            device.lock().unwrap().reset();
            machine.cpu.attach_device(device.clone());
        }

        self.exec.is_running = false;
        self.exec.last_stop_addr = None;

        if self.tabs.is_empty() {
            self.program_loaded = false;
            self.refresh_snapshot();
            return;
        }

        let active = self.active_tab;
        let active_tab = &self.tabs[active];

        if active_tab.kind == EditorTabKind::Binary {
            self.symbol_table.clear();
            self.address_to_line.clear();
            self.line_to_address.clear();
            self.assembly_lines.clear();
            self.is_assembly_stale = false;
            self.last_error = None;

            // `binary_bytes` is not serialised, so a tab restored from storage
            // only has its hexdump. Recover the bytes from the dump instead of
            // leaving a dead tab behind.
            let bytes = active_tab
                .binary_bytes
                .clone()
                .or_else(|| formatting::parse_hexdump(&active_tab.code));
            if let Some(bytes) = bytes {
                let mem = machine.memory_mut();
                for (i, b) in bytes.iter().enumerate() {
                    mem.write(i as u16, *b);
                }
                self.program_loaded = true;
            } else {
                self.program_loaded = false;
                self.last_error = Some("Binary tab is missing its raw bytes".to_string());
            }
            self.sync_breakpoints();
            self.refresh_snapshot();
            return;
        }

        let code = &active_tab.code;
        let (bytes, symbols, addr_map, line_map, image, assembly_lines, error) =
            match assemble_with_metadata(code) {
                Ok(result) => (
                    result.bytes,
                    result.symbols,
                    result.address_to_line,
                    result.line_to_address,
                    result.image,
                    result.lines,
                    None,
                ),
                Err(e) => (
                    Vec::new(),
                    HashMap::new(),
                    HashMap::new(),
                    HashMap::new(),
                    Vec::new(),
                    HashMap::new(),
                    Some(e),
                ),
            };

        self.symbol_table = symbols;
        self.address_to_line = addr_map;
        self.line_to_address = line_map;
        self.assembly_lines = assembly_lines;
        self.is_assembly_stale = false;

        if let Some(err) = error {
            self.program_loaded = false;
            self.last_error = Some(err);
        } else {
            self.program_loaded = true;
            self.last_error = None;
            let mem = machine.memory_mut();
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
        self.sync_breakpoints();
        self.refresh_snapshot();
    }

    fn edit_cpu<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Machine) -> R,
    {
        if let Some(machine) = self.exec.machine.as_mut() {
            let result = f(machine);
            self.refresh_snapshot();
            result
        } else {
            // Machine is running in the runner; we cannot edit it directly.
            // The UI should send commands instead. This is a fallback that
            // takes the machine back (stopping execution) and applies the edit.
            let mut machine = self.exec.runner.take_machine();
            let result = f(&mut machine);
            self.exec.machine = Some(machine);
            self.exec.is_running = false;
            self.exec.last_stop_addr = None;
            self.refresh_snapshot();
            result
        }
    }

    fn refresh_snapshot(&mut self) {
        if let Some(machine) = self.exec.machine.as_mut() {
            self.exec.snapshot = machine.snapshot();
        }
    }

    fn sync_breakpoints(&mut self) {
        let mut addrs = HashSet::new();
        if let Some(tab) = self.tabs.get(self.active_tab) {
            for &line in &tab.breakpoints {
                if let Some(&addr) = self.line_to_address.get(&line) {
                    addrs.insert(addr);
                }
            }
        }
        let changed = addrs != self.exec.breakpoint_addrs;
        self.exec.breakpoint_addrs = addrs;
        if changed && self.exec.is_running {
            self.exec
                .runner
                .send_command(Command::SetBreakpoints(self.exec.breakpoint_addrs.clone()));
        }
    }

    fn start_execution(&mut self) {
        let machine = self.exec.machine.take().unwrap_or_default();
        self.exec.is_running = true;
        self.exec.last_stop_addr = None;
        self.exec
            .runner
            .start(machine, self.exec.breakpoint_addrs.clone());
    }

    fn stop_execution(&mut self) {
        if self.exec.machine.is_some() {
            return;
        }
        self.exec.machine = Some(self.exec.runner.take_machine());
        self.exec.is_running = false;
        self.refresh_snapshot();
    }

    fn step_instruction(&mut self) {
        if !self.program_loaded && !self.exec.is_running {
            return;
        }
        if self.exec.is_running {
            self.exec.runner.send_command(Command::Tick);
        } else if let Some(machine) = self.exec.machine.as_mut() {
            machine.cpu.tick();
            self.refresh_snapshot();
        }
    }

    fn resume_from_halt(&mut self) {
        if self.exec.is_running {
            self.exec.runner.send_command(Command::Resume);
        } else if let Some(machine) = self.exec.machine.as_mut() {
            machine.cpu.set_halted(false);
            self.refresh_snapshot();
        }
    }

    fn toggle_execution(&mut self) {
        if self.exec.is_running {
            self.stop_execution();
            return;
        }
        if !self.program_loaded {
            return;
        }

        if self.exec.snapshot.halted {
            if let Some(machine) = self.exec.machine.as_mut() {
                machine.cpu.set_halted(false);
            }
            self.refresh_snapshot();
        }

        if self.exec.breakpoint_addrs.contains(&self.exec.snapshot.pc)
            && let Some(machine) = self.exec.machine.as_mut()
        {
            // Standing on a breakpoint would stop again immediately; step over.
            machine.cpu.tick();
            self.refresh_snapshot();
        }

        self.start_execution();
    }

    fn save_to_storage(&self, storage: Option<&mut (dyn eframe::Storage + 'static)>) {
        if let Some(storage) = storage {
            eframe::set_value(storage, STORAGE_KEY, self);
            storage.flush();
        }
    }

    pub fn new(cc: &eframe::CreationContext) -> Self {
        theme::apply(&cc.egui_ctx);

        let mut app = if let Some(storage) = cc.storage {
            match eframe::get_value::<Z80App>(storage, STORAGE_KEY) {
                Some(mut loaded_app) => {
                    if loaded_app.tabs.is_empty() {
                        loaded_app.tabs.push(EditorTab {
                            path: None,
                            code: Self::default_code(),
                            kind: EditorTabKind::Source,
                            is_dirty: false,
                            breakpoints: HashSet::new(),
                            binary_bytes: None,
                        });
                    }
                    loaded_app
                }
                None => Self::default(),
            }
        } else {
            Self::default()
        };

        // Devices are not serialised directly; rebuild them from their specs.
        let specs = app.attached_device_specs.clone();
        for spec in &specs {
            app.attach_device_spec(spec);
        }

        for tab in &mut app.tabs {
            if tab.kind == EditorTabKind::Binary && tab.binary_bytes.is_none() {
                tab.binary_bytes = formatting::parse_hexdump(&tab.code);
            }
        }

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
        self.tabs.push(EditorTab {
            path: None,
            code: Self::default_code(),
            kind: EditorTabKind::Source,
            is_dirty: false,
            breakpoints: HashSet::new(),
            binary_bytes: None,
        });
        self.active_tab = self.tabs.len() - 1;
        self.is_assembly_stale = true;
        self.save_to_storage(storage);
    }

    fn close_tab(&mut self, index: usize, storage: Option<&mut (dyn eframe::Storage + 'static)>) {
        if index >= self.tabs.len() {
            return;
        }
        let closed_active = index == self.active_tab;
        self.tabs.remove(index);

        if self.tabs.is_empty() {
            self.new_file(storage); // Ensure at least one tab
            self.load_and_reset();
        } else {
            if closed_active {
                // Show the neighbouring tab; its source is not what is loaded.
                self.active_tab = index.min(self.tabs.len() - 1);
                self.is_assembly_stale = true;
            } else if index < self.active_tab {
                // Keep pointing at the same document.
                self.active_tab -= 1;
            }
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
        if let Some(idx) = self
            .tabs
            .iter()
            .position(|tab| tab.kind == EditorTabKind::Binary && tab.path.as_ref() == Some(&path))
        {
            self.active_tab = idx;
        } else {
            self.tabs.push(EditorTab {
                path: Some(path.clone()),
                code: hexdump,
                kind: EditorTabKind::Binary,
                is_dirty: false,
                breakpoints: HashSet::new(),
                binary_bytes: Some(bytes),
            });
            self.active_tab = self.tabs.len() - 1;
        }

        self.add_recent_file(path);
        self.is_assembly_stale = false;
        self.save_to_storage(storage);
        self.load_and_reset();
    }

    fn load_file_content(
        &mut self,
        path: PathBuf,
        content: String,
        storage: Option<&mut (dyn eframe::Storage + 'static)>,
    ) {
        // If current tab is empty (default code or empty) and untitled, replace it
        let current_tab = &self.tabs[self.active_tab];
        let default_code = Self::default_code();
        let current_is_disposable = current_tab.path.is_none()
            && (current_tab.code.trim().is_empty() || current_tab.code == default_code)
            && !current_tab.is_dirty;
        let tab = EditorTab {
            path: Some(path.clone()),
            code: content,
            kind: EditorTabKind::Source,
            is_dirty: false,
            breakpoints: HashSet::new(),
            binary_bytes: None,
        };

        if current_is_disposable {
            self.tabs[self.active_tab] = tab;
        } else {
            self.tabs.push(tab);
            self.active_tab = self.tabs.len() - 1;
        }

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
            .tabs
            .iter()
            .position(|t| t.path.as_ref() == Some(&path))
        {
            if self.active_tab != idx {
                self.active_tab = idx;
                self.is_assembly_stale = true;
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
        let prev_active = self.active_tab;
        for i in 0..self.tabs.len() {
            if self.tabs[i].is_dirty {
                self.active_tab = i;
                self.save_file(i, storage.as_deref_mut());
                if self.tabs[i].is_dirty {
                    break;
                }
            }
        }
        self.active_tab = prev_active;
        !self.tabs.iter().any(|tab| tab.is_dirty)
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
        if let Some(machine) = self.exec.machine.as_mut() {
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
            if tab_idx == self.active_tab {
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
            let tab = &mut self.tabs[tab_idx];
            if let Some(path) = &tab.path {
                let path_clone = path.clone(); // Clone to avoid borrow check issues if we needed it later, but here write takes reference
                if let Err(err) = std::fs::write(&path_clone, &tab.code) {
                    self.last_error = Some(format!("Failed to save file: {}", err));
                } else {
                    tab.is_dirty = false;
                    self.status_message = Some(format!("Saved {}", path_clone.display()));
                }
            } else {
                if tab_idx == self.active_tab {
                    self.save_file_as(storage);
                } else {
                    self.status_message =
                        Some("Only the active untitled file can be saved".to_string());
                }
                return;
            }

            if let Some(path) = &self.tabs[tab_idx].path {
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
            let tab = &mut self.tabs[self.active_tab];
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
            let tab = &mut self.tabs[self.active_tab];
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
        let code = self.tabs.get(self.active_tab).map(|tab| tab.code.clone());
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
                    let name = self.tabs[self.active_tab]
                        .path
                        .as_ref()
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
                self.exec.is_running = false;
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
            HeaderAction::SaveFile => self.save_file(self.active_tab, frame.storage_mut()),
            HeaderAction::SaveFileAs => self.save_file_as(frame.storage_mut()),
            HeaderAction::AssembleBinary => self.assemble_binary_to_file(frame.storage_mut()),
            HeaderAction::LoadBinary => self.load_binary_dialog(ctx, frame.storage_mut()),
            HeaderAction::CloseTab(idx) => {
                let idx = if idx == usize::MAX {
                    self.active_tab
                } else {
                    idx
                };
                if idx >= self.tabs.len() {
                    return;
                }
                if self.tabs[idx].is_dirty {
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
        if self.exec.is_running {
            for event in self.exec.runner.poll() {
                match event {
                    Event::Snapshot(delta) => {
                        self.exec.apply_delta(delta);
                    }
                    Event::Finished(addr) => {
                        self.exec.is_running = false;
                        self.exec.last_stop_addr = Some(addr);
                    }
                }
            }
            if self.exec.is_running {
                ctx.request_repaint_after(Duration::from_millis(16));
            }
            self.exec.ensure_machine_reclaimed();
        }
    }

    fn show_memory_viewer(&mut self, ui: &mut egui::Ui) {
        let mut show_mem = self.show_memory_viewer;
        if show_mem {
            egui::Window::new("Memory Viewer")
                .open(&mut show_mem)
                .resizable(true)
                .default_width(560.0)
                .default_height(430.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Start (hex):");
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.memory_viewer_start)
                                .desired_width(64.0)
                                .font(egui::TextStyle::Monospace)
                                .hint_text("0000"),
                        );
                        if ui
                            .small_button("◀")
                            .on_hover_text("Previous page (−0x100)")
                            .clicked()
                        {
                            let start = parse_hex_u16(&self.memory_viewer_start).unwrap_or(0);
                            self.memory_viewer_start = format!("{:04X}", start.wrapping_sub(0x100));
                        }
                        if ui
                            .small_button("▶")
                            .on_hover_text("Next page (+0x100)")
                            .clicked()
                        {
                            let start = parse_hex_u16(&self.memory_viewer_start).unwrap_or(0);
                            self.memory_viewer_start = format!("{:04X}", start.wrapping_add(0x100));
                        }
                        if ui
                            .small_button("Goto PC")
                            .on_hover_text("Jump to the program counter")
                            .clicked()
                        {
                            self.memory_viewer_start = format!("{:04X}", self.exec.snapshot.pc);
                            response.request_focus();
                        }
                        ui.checkbox(&mut self.memory_follow_pc, "Follow PC")
                            .on_hover_text("Scroll along with the program counter");
                        ui.label(
                            egui::RichText::new("read-only")
                                .small()
                                .color(self.theme.muted),
                        );
                    });

                    let valid = parse_hex_u16(&self.memory_viewer_start).is_some();
                    if !valid {
                        ui.label(
                            egui::RichText::new("Enter a hexadecimal address (0000–FFFF)")
                                .small()
                                .color(self.theme.danger),
                        );
                    }

                    ui.separator();

                    if self.memory_follow_pc {
                        self.memory_viewer_start = format!("{:04X}", self.exec.snapshot.pc);
                    }
                    let start_addr = parse_hex_u16(&self.memory_viewer_start).unwrap_or(0);
                    let pc = self.exec.snapshot.pc;

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("mem_dump_grid")
                            .num_columns(3)
                            .spacing([20.0, 4.0])
                            .show(ui, |ui| {
                                for r in 0..16u16 {
                                    let row_addr = start_addr.wrapping_add(r * 16);
                                    let has_pc = pc.wrapping_sub(row_addr) < 16;
                                    let addr_text = egui::RichText::new(format!("{row_addr:04X}"))
                                        .monospace()
                                        .color(if has_pc {
                                            self.theme.pc_marker
                                        } else {
                                            self.theme.address
                                        });
                                    ui.label(addr_text);

                                    let mut hex_str = String::new();
                                    let mut ascii_str = String::new();
                                    for c in 0..16u16 {
                                        let addr = row_addr.wrapping_add(c);
                                        let val = self.exec.snapshot.memory[addr as usize];
                                        hex_str.push_str(&format!("{:02X} ", val));
                                        if (32..=126).contains(&val) {
                                            ascii_str.push(val as char);
                                        } else {
                                            ascii_str.push('.');
                                        }
                                    }

                                    ui.monospace(hex_str)
                                        .on_hover_text("Memory is read-only in this view");
                                    ui.monospace(ascii_str);
                                    ui.end_row();
                                }
                            });
                    });
                });
            self.show_memory_viewer = show_mem;
        }
    }

    fn show_device_windows(&mut self, ctx: &egui::Context) {
        for device in &self.attached_devices {
            device.lock().unwrap().draw(ctx);
        }
    }

    fn render_registers_panel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Registers");
            ui.label(egui::RichText::new("hex").small().color(self.theme.muted))
                .on_hover_text("All values are hexadecimal. A 0x prefix is accepted.");
        });
        ui.separator();

        // While a run is in progress the machine lives in the runner thread and
        // `exec.machine` is `None`, so edits would be silently dropped.
        let can_edit = self.exec.machine.is_some();
        if !can_edit {
            ui.label(
                egui::RichText::new("Stop the CPU to edit registers")
                    .small()
                    .color(self.theme.muted),
            );
        }

        // Copy the small scalar fields instead of cloning the whole snapshot
        // (which carries a 64 KiB memory buffer) every frame.
        let pc = self.exec.snapshot.pc;
        let sp = self.exec.snapshot.sp;
        let ix = self.exec.snapshot.ix;
        let iy = self.exec.snapshot.iy;
        let iff1 = self.exec.snapshot.iff1;
        let iff2 = self.exec.snapshot.iff2;
        let interrupt_mode = self.exec.snapshot.interrupt_mode;
        let reg_values = self.exec.snapshot.regs;
        let shadow_values = self.exec.snapshot.shadow_regs;
        let flag_states: [(Flag, &str, bool); 8] = [
            (Flag::S, "S", snapshot_flag(&self.exec.snapshot, Flag::S)),
            (Flag::Z, "Z", snapshot_flag(&self.exec.snapshot, Flag::Z)),
            (Flag::Y, "Y", snapshot_flag(&self.exec.snapshot, Flag::Y)),
            (Flag::H, "H", snapshot_flag(&self.exec.snapshot, Flag::H)),
            (Flag::X, "X", snapshot_flag(&self.exec.snapshot, Flag::X)),
            (Flag::PV, "PV", snapshot_flag(&self.exec.snapshot, Flag::PV)),
            (Flag::N, "N", snapshot_flag(&self.exec.snapshot, Flag::N)),
            (Flag::C, "C", snapshot_flag(&self.exec.snapshot, Flag::C)),
        ];

        let theme = self.theme;
        egui::Grid::new("regs_grid")
            .striped(true)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                macro_rules! edit_16 {
                    ($name:expr, $getter:expr, $setter:expr) => {
                        ui.label($name);
                        let val = $getter;
                        let id = egui::Id::unique($name);
                        let mut s = ui.data_mut(|d| {
                            d.get_temp_mut_or_insert_with(id, || format!("{:04X}", val))
                                .clone()
                        });
                        let resp = ui
                            .add_enabled(
                                can_edit,
                                egui::TextEdit::singleline(&mut s)
                                    .desired_width(45.0)
                                    .font(egui::TextStyle::Monospace),
                            )
                            .on_hover_text("Hexadecimal; 0x is optional");
                        if resp.changed() {
                            if let Some(v) = parse_hex_u16(&s) {
                                $setter(v);
                            }
                            ui.data_mut(|d| d.insert_temp(id, s.clone()));
                        } else if !resp.has_focus() {
                            ui.data_mut(|d| d.insert_temp(id, format!("{:04X}", val)));
                        }
                        if resp.has_focus() && parse_hex_u16(&s).is_none() {
                            ui.painter().rect_stroke(
                                resp.rect,
                                egui::CornerRadius::same(3),
                                egui::Stroke::new(1.5, theme.danger),
                                egui::StrokeKind::Inside,
                            );
                        }
                    };
                }

                macro_rules! edit_8 {
                    ($name:expr, $id:expr, $getter:expr, $setter:expr) => {
                        ui.label($name);
                        let val = $getter;
                        let id = egui::Id::unique($id);
                        let mut s = ui.data_mut(|d| {
                            d.get_temp_mut_or_insert_with(id, || format!("{:02X}", val))
                                .clone()
                        });
                        let resp = ui
                            .add_enabled(
                                can_edit,
                                egui::TextEdit::singleline(&mut s)
                                    .desired_width(30.0)
                                    .font(egui::TextStyle::Monospace),
                            )
                            .on_hover_text("Hexadecimal; 0x is optional");
                        if resp.changed() {
                            if let Some(v) = parse_hex_u8(&s) {
                                $setter(v);
                            }
                            ui.data_mut(|d| d.insert_temp(id, s.clone()));
                        } else if !resp.has_focus() {
                            ui.data_mut(|d| d.insert_temp(id, format!("{:02X}", val)));
                        }
                        if resp.has_focus() && parse_hex_u8(&s).is_none() {
                            ui.painter().rect_stroke(
                                resp.rect,
                                egui::CornerRadius::same(3),
                                egui::Stroke::new(1.5, theme.danger),
                                egui::StrokeKind::Inside,
                            );
                        }
                    };
                }

                edit_16!("PC", pc, |v| self.edit_cpu(|m| m.cpu.set_pc(v)));
                ui.label("");
                ui.label("");
                ui.end_row();
                edit_16!("SP", sp, |v| self.edit_cpu(|m| m.cpu.set_sp(v)));
                ui.label("");
                ui.label("");
                ui.end_row();
                edit_16!("IX", ix, |v| self.edit_cpu(|m| m.cpu.set_ix(v)));
                ui.label("");
                ui.label("");
                ui.end_row();
                edit_16!("IY", iy, |v| self.edit_cpu(|m| m.cpu.set_iy(v)));
                ui.label("");
                ui.label("");
                ui.end_row();

                ui.separator();
                ui.separator();
                ui.separator();
                ui.separator();
                ui.end_row();

                let regs = [
                    ("A", GPR::A),
                    ("F", GPR::F),
                    ("B", GPR::B),
                    ("C", GPR::C),
                    ("D", GPR::D),
                    ("E", GPR::E),
                    ("H", GPR::H),
                    ("L", GPR::L),
                ];

                for (name, gpr) in regs {
                    let idx = gpr_index(gpr);
                    edit_8!(name, name, reg_values[idx], |v| self
                        .edit_cpu(|m| m.cpu.set_register(gpr, v)));
                    let shadow_name = format!("{}'", name);
                    edit_8!(shadow_name.clone(), shadow_name, shadow_values[idx], |v| {
                        self.edit_cpu(|m| m.cpu.set_shadow_register(gpr, v))
                    });
                    ui.end_row();
                }

                ui.separator();
                ui.separator();
                ui.separator();
                ui.separator();
                ui.end_row();

                let mono = |text: String| egui::RichText::new(text).monospace();
                ui.label("IFF1");
                ui.label(mono(format!("{}", iff1)));
                ui.label("");
                ui.label("");
                ui.end_row();
                ui.label("IFF2");
                ui.label(mono(format!("{}", iff2)));
                ui.label("");
                ui.label("");
                ui.end_row();
                ui.label("IM");
                ui.label(mono(format!("{}", interrupt_mode)));
                ui.label("");
                ui.label("");
                ui.end_row();
            });

        ui.add_space(16.0);
        ui.heading("Flags");
        ui.separator();

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;
            for (flag, label, on) in flag_states {
                let (fill, text_color) = if on {
                    (theme.flag_on, theme.flag_on_text)
                } else {
                    (theme.flag_off, theme.flag_off_text)
                };

                let text = egui::RichText::new(label).strong().color(text_color);
                let frame_resp = egui::Frame::new()
                    .fill(fill)
                    .corner_radius(4.0)
                    .inner_margin(4.0)
                    .show(ui, |ui| {
                        ui.label(text);
                    });

                let interact_resp = ui.interact(
                    frame_resp.response.rect,
                    ui.scope_id().with(label),
                    egui::Sense::click(),
                );

                let description = flag_description(flag);
                interact_resp.clone().on_hover_text(format!(
                    "{label} — {description}\n{}",
                    if on { "Set" } else { "Clear" }
                ));

                if can_edit {
                    if interact_resp.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    if interact_resp.clicked() {
                        self.edit_cpu(|m| m.cpu.set_flag(!on, flag));
                    }
                }
            }
        });
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
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path);

        // Keep only last 10
        if self.recent_files.len() > 10 {
            self.recent_files.truncate(10);
        }
    }
}

impl Default for Z80App {
    fn default() -> Self {
        Self {
            exec: ExecutionState::new(),
            tabs: vec![EditorTab {
                path: None,
                breakpoints: HashSet::new(),
                code: Self::default_code(),
                kind: EditorTabKind::Source,
                is_dirty: false,
                binary_bytes: None,
            }],
            active_tab: 0,
            last_error: None,
            status_message: None,
            symbol_table: HashMap::new(),
            address_to_line: HashMap::new(),
            line_to_address: HashMap::new(),
            assembly_lines: HashMap::new(),
            symbol_display_prefs: HashMap::new(),
            pending_modal: None,
            code_theme: highlighting::CodeTheme::one_dark_pro_vivid(),
            recent_files: Vec::new(),
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
            show_location_counter: true,
            is_assembly_stale: false,
            program_loaded: false,
        }
    }
}

impl eframe::App for Z80App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "z80_workspace", self);
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
            let any_dirty = self.tabs.iter().any(|t| t.is_dirty);
            if any_dirty {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.pending_modal = Some(ModalType::Quit);
            }
        }

        let mut action = self.check_shortcuts(&ctx);

        // Top Panel: Menu Bar and Control Toolbar
        egui::Panel::top("top_panel").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui
                        .add(egui::Button::new("New").shortcut_text("Ctrl+N"))
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        action = Some(HeaderAction::NewFile);
                        ui.close();
                    }
                    if ui
                        .add(egui::Button::new("Open...").shortcut_text("Ctrl+O"))
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        action = Some(HeaderAction::OpenFileDialog);
                        ui.close();
                    }

                    ui.menu_button("Open Recent", |ui| {
                        if self.recent_files.is_empty() {
                            ui.label("No recent files");
                        } else {
                            let mut to_open = None;
                            for path in &self.recent_files {
                                if ui
                                    .button(path.display().to_string())
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    to_open = Some(path.clone());
                                }
                            }
                            if let Some(path) = to_open {
                                action = Some(HeaderAction::OpenFile(path));
                                ui.close();
                            }
                        }
                    })
                    .response
                    .on_hover_cursor(egui::CursorIcon::PointingHand);

                    ui.separator();

                    if ui
                        .add(egui::Button::new("Save").shortcut_text("Ctrl+S"))
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        action = Some(HeaderAction::SaveFile);
                        ui.close();
                    }
                    if ui
                        .add(egui::Button::new("Save As...").shortcut_text("Ctrl+Shift+S"))
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        action = Some(HeaderAction::SaveFileAs);
                        ui.close();
                    }

                    ui.separator();
                    if ui
                        .button("Assemble Binary...")
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        action = Some(HeaderAction::AssembleBinary);
                        ui.close();
                    }
                    if ui
                        .button("Load Binary…")
                        .on_hover_text("Open a .bin/.rom image as a read-only tab")
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        action = Some(HeaderAction::LoadBinary);
                        ui.close();
                    }

                    ui.separator();
                    if ui
                        .button("Quit")
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        action = Some(HeaderAction::Quit);
                        ui.close();
                    }
                })
                .response
                .on_hover_cursor(egui::CursorIcon::PointingHand);

                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_location_counter, "Location Counter");
                    ui.checkbox(&mut self.show_memory_viewer, "Memory Viewer");
                    ui.separator();
                    if ui.button("Keyboard Shortcuts…").clicked() {
                        self.show_shortcuts = true;
                        ui.close();
                    }
                })
                .response
                .on_hover_cursor(egui::CursorIcon::PointingHand);

                ui.menu_button("Devices", |ui| {
                    ui.label(egui::RichText::new("Add").strong());
                    for definition in crate::components::devices::device_definitions() {
                        if ui
                            .button(definition.menu_name)
                            .on_hover_text(format!(
                                "Attach on I/O port {}",
                                definition.default_port
                            ))
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            self.pending_modal = Some(ModalType::AddDevice(
                                definition,
                                definition.default_port.to_string(),
                            ));
                            ui.close();
                        }
                    }

                    if !self.attached_devices.is_empty() {
                        ui.separator();
                        ui.label(egui::RichText::new("Attached").strong());
                        let mut remove = None;
                        for (idx, dev) in self.attached_devices.iter().enumerate() {
                            let name = dev.lock().unwrap().get_name();
                            ui.horizontal(|ui| {
                                let mut dev_ref = dev.lock().unwrap();
                                let mut open = dev_ref.get_window_open_state();
                                if ui.checkbox(&mut open, &name).changed() {
                                    dev_ref.set_window_open_state(open);
                                }
                                drop(dev_ref);
                                if ui
                                    .small_button("🗑")
                                    .on_hover_text(format!("Remove {name}"))
                                    .clicked()
                                {
                                    remove = Some(idx);
                                }
                            });
                        }
                        if let Some(idx) = remove {
                            self.remove_device(idx);
                            ui.close();
                        }
                    }
                })
                .response
                .on_hover_cursor(egui::CursorIcon::PointingHand);

                ui.menu_button("Memory", |ui| {
                    if ui
                        .checkbox(
                            &mut self.show_memory_viewer,
                            "Memory Viewer Window (read-only)",
                        )
                        .changed()
                    {
                        ui.close();
                    }
                })
                .response
                .on_hover_cursor(egui::CursorIcon::PointingHand);

                #[cfg(target_arch = "wasm32")]
                {
                    ui.menu_button("Examples", |ui| {
                        if self.examples_list.is_empty() {
                            let message = if self.examples_receiver.is_some() {
                                "Loading examples…"
                            } else {
                                "No examples found"
                            };
                            ui.label(message);
                        } else {
                            let mut to_load = None;
                            for ex in &self.examples_list {
                                if ui
                                    .button(ex)
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    to_load = Some(ex.clone());
                                }
                            }
                            if let Some(ex) = to_load {
                                self.load_example(ex, ui.ctx(), frame.storage_mut());
                                ui.close();
                            }
                        }
                    })
                    .response
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                }

                ui.menu_button("Help", |ui| {
                    if ui.button("Keyboard Shortcuts…").clicked() {
                        self.show_shortcuts = true;
                        ui.close();
                    }
                    ui.separator();
                    ui.label(format!("JAZZ80 {}", env!("CARGO_PKG_VERSION")));
                    ui.label("Z80 simulator");
                })
                .response
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            });
            ui.separator();

            // Toolbar
            ui.horizontal_wrapped(|ui| {
                if ui
                    .button("⟳ Assemble & Reset")
                    .on_hover_text(
                        "Assemble the active source, reset the CPU and load the program at 0x0000",
                    )
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    self.last_error = None;
                    self.load_and_reset();
                }

                ui.separator();

                let asset_ok = self.program_loaded || self.exec.is_running;
                let step_resp = ui
                    .add_enabled(asset_ok, egui::Button::new("⏭ Step"))
                    .on_hover_text(if asset_ok {
                        "Execute one instruction (F8)"
                    } else {
                        "Assemble or load a program first"
                    })
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if step_resp.clicked() {
                    self.step_instruction();
                }

                let halted = self.exec.snapshot.halted;
                let run_label = if self.exec.is_running {
                    "⏸ Stop"
                } else if halted {
                    "▶ Resume"
                } else if self.exec.snapshot.pc > 0 {
                    "▶ Continue"
                } else {
                    "▶ Run"
                };
                let run_resp = ui
                    .add_enabled(asset_ok, egui::Button::new(run_label))
                    .on_hover_text(if asset_ok {
                        "Run until a breakpoint or HALT (F5)"
                    } else {
                        "Assemble or load a program first"
                    })
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if run_resp.clicked() {
                    self.toggle_execution();
                }

                ui.separator();

                if ui
                    .button("⚡ INT")
                    .on_hover_text("Raise a maskable interrupt")
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    if self.exec.is_running {
                        self.exec.runner.send_command(Command::SetInterrupt(true));
                    } else if let Some(machine) = self.exec.machine.as_mut() {
                        machine.cpu.set_interrupt(true);
                        self.refresh_snapshot();
                    }
                }

                if ui
                    .button("⚡ NMI")
                    .on_hover_text("Raise a non-maskable interrupt")
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    if self.exec.is_running {
                        self.exec.runner.send_command(Command::Nmi);
                    } else if let Some(machine) = self.exec.machine.as_mut() {
                        machine.cpu.nmi();
                        self.refresh_snapshot();
                    }
                }

                ui.separator();

                let (state_text, state_color) = if self.exec.snapshot.halted {
                    ("HALTED", self.theme.stale)
                } else if self.exec.is_running {
                    ("RUNNING", self.theme.success)
                } else if self.exec.last_stop_addr.is_some() {
                    ("AT BREAKPOINT", self.theme.warning)
                } else {
                    ("PAUSED", self.theme.muted)
                };
                ui.label(egui::RichText::new(state_text).color(state_color).strong());
                if let Some(addr) = self.exec.last_stop_addr {
                    ui.label(
                        egui::RichText::new(format!("0x{addr:04X}"))
                            .monospace()
                            .color(self.theme.warning),
                    );
                }

                if self.exec.snapshot.halted && ui.button("Resume").clicked() {
                    self.resume_from_halt();
                }
            });
        });

        // Tab Bar Logic (pre-processing to find if we need to close tabs)
        // We'll define the closure later inside central panel?
        // No, let's process actions first. but Tab Bar is part of Central Panel UI.

        // Bottom Status Bar. Added before the side panels so it spans the full
        // window width, which gives long error messages room to breathe.
        egui::Panel::bottom("status_bar").show(ui, |ui| {
            let theme = self.theme;
            ui.horizontal(|ui| {
                let name = self.active_tab_name();
                let path = self.active_tab_path();
                let dirty = self
                    .tabs
                    .get(self.active_tab)
                    .is_some_and(|tab| tab.is_dirty);

                let mut name_label = egui::RichText::new(name).strong();
                if dirty {
                    name_label = name_label.color(theme.warning);
                }
                let resp = ui.label(name_label);
                if let Some(path) = path {
                    resp.on_hover_text(path);
                }
                if dirty {
                    ui.label(
                        egui::RichText::new("● unsaved")
                            .small()
                            .color(theme.warning),
                    )
                    .on_hover_text("This document has unsaved changes");
                }

                ui.separator();
                ui.monospace(format!("PC 0x{:04X}", self.exec.snapshot.pc))
                    .on_hover_text("Program counter");
                ui.monospace(format!("SP 0x{:04X}", self.exec.snapshot.sp))
                    .on_hover_text("Stack pointer");

                if self.is_assembly_stale {
                    ui.label(
                        egui::RichText::new("source changed since last assemble")
                            .small()
                            .color(theme.stale),
                    )
                    .on_hover_text(
                        "Run uses the previously assembled program until you assemble again",
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(err) = self.last_error.clone() {
                        if ui.small_button("✕").on_hover_text("Dismiss").clicked() {
                            self.last_error = None;
                        }
                        let text = format!("⚠ {err}");
                        ui.label(egui::RichText::new(&text).color(theme.danger).small())
                            .on_hover_text(err);
                    } else if let Some(msg) = self.status_message.clone() {
                        if ui.small_button("✕").on_hover_text("Dismiss").clicked() {
                            self.status_message = None;
                        }
                        ui.label(egui::RichText::new(&msg).color(theme.muted).small())
                            .on_hover_text(msg);
                    }
                });
            });
        });

        // Right Panel: Registers and Flags
        egui::Panel::right("right_panel")
            .resizable(true)
            .default_size(280.0)
            .show(ui, |ui| self.render_registers_panel(ui));

        let mut sorted_symbols: Vec<_> = self.symbol_table.iter().collect();
        sorted_symbols.sort_by_key(|item| item.1.source_order);

        egui::Panel::right("vars_panel")
            .resizable(true)
            .default_size(280.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Variables").on_hover_text(
                        "Right-click a variable name to change how its value is displayed",
                    );
                    if !self.symbol_display_prefs.is_empty()
                        && ui
                            .small_button("Reset formats")
                            .on_hover_text("Show every variable using its declared type")
                            .clicked()
                    {
                        self.symbol_display_prefs.clear();
                    }
                });
                ui.separator();

                egui::ScrollArea::both()
                    .id_salt("vars_scroll")
                    .max_height(200.0)
                    .show(ui, |ui| {
                        egui::Grid::new("vars_grid")
                            .striped(true)
                            .spacing([8.0, 4.0])
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new("Name").strong());
                                ui.label(egui::RichText::new("Addr").strong());
                                ui.label(egui::RichText::new("Value").strong());
                                ui.label(egui::RichText::new("Format").strong());
                                ui.end_row();

                                for (name, symbol) in sorted_symbols.iter().
                                filter(|s| !matches!(s.1.kind, SymbolType::Label)) {

                                    let addr = symbol.address;

                                    // Determine display format
                                    let format = self.symbol_display_prefs.get(*name).cloned().unwrap_or(SymbolDisplayFormat::Default);

                                    // Continuation rows use an internal suffix only to keep map keys unique.
                                    let display_name = if name.ends_with(']')
                                        && name.rfind('[').is_some()
                                    {
                                        ""
                                    } else {
                                        *name
                                    };

                                    // Format selection lives in its own column so it
                                    // can never displace the address or value.
                                    ui.label(display_name);

                                    let is_constant = symbol.kind == SymbolType::Constant;
                                    if is_constant {
                                        ui.label(egui::RichText::new("—").color(self.theme.muted))
                                            .on_hover_text("EQU constant: has no memory address");
                                    } else {
                                        ui.monospace(format!("0x{:04X}", addr));
                                    }

                                    if is_constant {
                                        ui.monospace(format_constant_value(addr, format))
                                            .on_hover_text(format!(
                                                "Assembler constant (EQU) = 0x{:04X}",
                                                addr
                                            ));
                                    } else {
                                    // Render based on format
                                    match format {
                                        SymbolDisplayFormat::Default => {
                                            match symbol.kind {
                                                SymbolType::Byte => {
                                                    let val = self.exec.snapshot.memory[addr as usize];
                                                    ui.monospace(format!("0x{:02X}", val));
                                                }
                                                SymbolType::Word => {
                                                    let low = self.exec.snapshot.memory[addr as usize];
                                                    let high = self.exec.snapshot.memory[addr.wrapping_add(1) as usize];
                                                    let val = (low as u16) | ((high as u16) << 8);
                                                    ui.monospace(format!("0x{:04X}", val));
                                                }
                                                SymbolType::String(len) => {
                                                    // Default String render
                                                    let mut bytes = Vec::new();
                                                    // Read at most 20 chars for preview, but check for null terminator
                                                    let display_len = len.min(20);
                                                    for i in 0..display_len {
                                                        let b = self.exec.snapshot.memory[addr.wrapping_add(i as u16) as usize];
                                                        if b == 0 { break; }
                                                        bytes.push(b);
                                                    }
                                                    let s: String = bytes.iter()
                                                        .map(|&b| if (32..=126).contains(&b) { b as char } else { '.' })
                                                        .collect();

                                                        // If we hit a null or length is long, check full content for tooltip
                                                    if len > 20 || bytes.len() < len.min(20) {
                                                        // Read full content for tooltip
                                                        let mut full_bytes = Vec::new();
                                                        for i in 0..len {
                                                            let b = self.exec.snapshot.memory[addr.wrapping_add(i as u16) as usize];
                                                            if b == 0 { break; }
                                                            full_bytes.push(b);
                                                        }
                                                        let full_s: String = full_bytes.iter()
                                                            .map(|&b| if (32..=126).contains(&b) { b as char } else { '.' })
                                                            .collect();

                                                        if bytes.len() < full_bytes.len() {
                                                             ui.monospace(format!("\"{}\"...", s))
                                                                .on_hover_text(format!("Length: {}\nFull: \"{}\"", len, full_s));
                                                        } else {
                                                             ui.monospace(format!("\"{}\"", s));
                                                        }
                                                    } else {
                                                        ui.monospace(format!("\"{}\"", s));
                                                    }
                                                }
                                                SymbolType::Array(len) => {
                                                    // Default Array render (Hex Bytes)
                                                    let mut bytes = Vec::new();
                                                    let display_len = len.min(8);
                                                    for i in 0..display_len {
                                                        bytes.push(self.exec.snapshot.memory[addr.wrapping_add(i as u16) as usize]);
                                                    }
                                                    let hex_str: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
                                                    if len > 8 {
                                                        // Read full content for tooltip
                                                        let mut full_bytes = Vec::new();
                                                        for i in 0..len {
                                                            full_bytes.push(self.exec.snapshot.memory[addr.wrapping_add(i as u16) as usize]);
                                                        }
                                                        let full_hex: Vec<String> = full_bytes.iter().map(|b| format!("{:02X}", b)).collect();

                                                        ui.monospace(format!("[{}...]", hex_str.join(" ")))
                                                            .on_hover_text(format!("Length: {}\nFull: [{}]", len, full_hex.join(" ")));
                                                    } else {
                                                        ui.monospace(format!("[{}]", hex_str.join(" ")));
                                                    }
                                                }
                                                _ => { ui.label(""); }
                                            }
                                        },
                                        SymbolDisplayFormat::String => {
                                            let len = match symbol.kind {
                                                SymbolType::String(l) | SymbolType::Array(l) => l,
                                                SymbolType::Word => 2,
                                                _ => 1,
                                            };

                                            let max_scan_len = if len > 1 { len } else { 40 };
                                            let display_limit = max_scan_len.min(40);

                                            let mut bytes = Vec::new();
                                            for i in 0..display_limit {
                                                let b = self.exec.snapshot.memory[addr.wrapping_add(i as u16) as usize];
                                                if b == 0 { break; }
                                                bytes.push(b);
                                            }

                                            let s: String = bytes.iter()
                                                .map(|&b| if (32..=126).contains(&b) { b as char } else { '.' })
                                                .collect();

                                            // Determine if we need to show truncation or tooltip with full content
                                            // bytes.len() is the effective string length (stopped at null or limit)

                                            // If we stopped at the display limit (40) but allocated size is larger, it might be truncated.
                                            let truncated_visual = bytes.len() == 40 && max_scan_len > 40;

                                            // Read full string for tooltip only if visually truncated
                                            let mut full_s = s.clone();
                                            if truncated_visual {
                                                 let mut full_bytes = Vec::new();
                                                 // Scan up to max_scan_len (which is allocated length)
                                                 for i in 0..max_scan_len {
                                                     let b = self.exec.snapshot.memory[addr.wrapping_add(i as u16) as usize];
                                                     if b == 0 { break; }
                                                     full_bytes.push(b);
                                                 }
                                                 full_s = full_bytes.iter()
                                                    .map(|&b| if (32..=126).contains(&b) { b as char } else { '.' })
                                                    .collect();
                                            }

                                            if truncated_visual {
                                                ui.monospace(format!("\"{}\"...", s))
                                                    .on_hover_text(format!("Allocated: {}\nString: \"{}\"", len, full_s));
                                            } else {
                                                // Not truncated visually, so 's' is the full string (or at least as much as fits in allocated)
                                                ui.monospace(format!("\"{}\"", s))
                                                    .on_hover_text(format!("Allocated: {}\nString: \"{}\"", len, s));
                                            }
                                        },
                                        SymbolDisplayFormat::HexBytes => {
                                            let len = match symbol.kind {
                                                SymbolType::String(l) | SymbolType::Array(l) => l,
                                                SymbolType::Word => 2,
                                                _ => 1,
                                            };
                                            let mut bytes = Vec::new();
                                            let display_len = len.clamp(1, 8); // At least 1 byte
                                            for i in 0..display_len {
                                                bytes.push(self.exec.snapshot.memory[addr.wrapping_add(i as u16) as usize]);
                                            }
                                            let hex_str: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
                                            if len > 8 {
                                                 let mut full_bytes = Vec::new();
                                                 for i in 0..len {
                                                     full_bytes.push(self.exec.snapshot.memory[addr.wrapping_add(i as u16) as usize]);
                                                 }
                                                 let full_hex: Vec<String> = full_bytes.iter().map(|b| format!("{:02X}", b)).collect();
                                                ui.monospace(format!("[{}...]", hex_str.join(" ")))
                                                    .on_hover_text(format!("Length: {}\nFull: [{}]", len, full_hex.join(" ")));
                                            } else {
                                                ui.monospace(format!("[{}]", hex_str.join(" ")))
                                                    .on_hover_text(format!("Length: {}\nFull: [{}]", len, hex_str.join(" ")));
                                            }
                                        },
                                        SymbolDisplayFormat::HexWords => {
                                            let len = match symbol.kind {
                                                SymbolType::String(l) | SymbolType::Array(l) => l,
                                                SymbolType::Word => 2,
                                                _ => 1,
                                            };
                                            // If length is odd, maybe show up to last even?
                                            let word_count = len / 2;
                                            if word_count == 0 {
                                                 // Fallback for single byte if forced?
                                                 let val = self.exec.snapshot.memory[addr as usize];
                                                 ui.monospace(format!("[00{:02X}:inv]", val))
                                                    .on_hover_text("Odd length, cannot display as words correctly");
                                            } else {
                                                let mut words = Vec::new();
                                                let display_count = word_count.min(4);
                                                for i in 0..display_count {
                                                    let offset = (i * 2) as u16;
                                                    let low = self.exec.snapshot.memory[addr.wrapping_add(offset) as usize];
                                                    let high = self.exec.snapshot.memory[addr.wrapping_add(offset + 1) as usize];
                                                    words.push(format!("{:04X}", (low as u16) | ((high as u16) << 8)));
                                                }
                                                if word_count > 4 {
                                                     let mut full_words = Vec::new();
                                                     for i in 0..word_count {
                                                         let offset = (i * 2) as u16;
                                                         let low = self.exec.snapshot.memory[addr.wrapping_add(offset) as usize];
                                                         let high = self.exec.snapshot.memory[addr.wrapping_add(offset + 1) as usize];
                                                         full_words.push(format!("{:04X}", (low as u16) | ((high as u16) << 8)));
                                                     }
                                                    ui.monospace(format!("[{}...]", words.join(" ")))
                                                        .on_hover_text(format!("Length: {} bytes\nFull: [{}]", len, full_words.join(" ")));
                                                } else {
                                                    ui.monospace(format!("[{}]", words.join(" ")))
                                                        .on_hover_text(format!("Length: {} bytes\nFull: [{}]", len, words.join(" ")));
                                                }
                                            }
                                        }
                                    }
                                    }

                                    let selected = match format {
                                        SymbolDisplayFormat::Default => "Default",
                                        SymbolDisplayFormat::String => "String",
                                        SymbolDisplayFormat::HexBytes => "Hex bytes",
                                        SymbolDisplayFormat::HexWords => "Hex words",
                                    };
                                    egui::ComboBox::from_id_salt(("vars_fmt", name.as_str()))
                                        .selected_text(selected)
                                        .width(78.0)
                                        .show_ui(ui, |ui| {
                                            if ui
                                                .selectable_label(
                                                    format == SymbolDisplayFormat::Default,
                                                    "Default",
                                                )
                                                .clicked()
                                            {
                                                self.symbol_display_prefs.remove(*name);
                                                ui.close();
                                            }
                                            if ui
                                                .selectable_label(
                                                    format == SymbolDisplayFormat::String,
                                                    "String",
                                                )
                                                .clicked()
                                            {
                                                self.symbol_display_prefs.insert(
                                                    name.to_string(),
                                                    SymbolDisplayFormat::String,
                                                );
                                                ui.close();
                                            }
                                            if ui
                                                .selectable_label(
                                                    format == SymbolDisplayFormat::HexBytes,
                                                    "Hex bytes",
                                                )
                                                .clicked()
                                            {
                                                self.symbol_display_prefs.insert(
                                                    name.to_string(),
                                                    SymbolDisplayFormat::HexBytes,
                                                );
                                                ui.close();
                                            }
                                            if ui
                                                .selectable_label(
                                                    format == SymbolDisplayFormat::HexWords,
                                                    "Hex words",
                                                )
                                                .clicked()
                                            {
                                                self.symbol_display_prefs.insert(
                                                    name.to_string(),
                                                    SymbolDisplayFormat::HexWords,
                                                );
                                                ui.close();
                                            }
                                        });
                                    ui.end_row();
                                }
                            });
                    });

                ui.add_space(20.0);
                ui.heading("Labels");
                ui.separator();

                egui::ScrollArea::vertical()
                    .id_salt("labels_scroll")
                    .show(ui, |ui| {
                        egui::Grid::new("labels_grid")
                            .striped(true)
                            .spacing([20.0, 4.0])
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new("Name").strong());
                                ui.label(egui::RichText::new("Addr").strong());
                                ui.end_row();

                                for (name, symbol) in &sorted_symbols {
                                    if symbol.kind != SymbolType::Label {
                                        continue;
                                    }
                                    let addr = symbol.address;

                                    ui.label(*name);
                                    ui.monospace(format!("0x{:04X}", addr));
                                    ui.end_row();
                                }
                            });
                    });
            });

        // Central Panel: Code Editor
        egui::CentralPanel::default().show(ui, |ui| {
            // Tab Bar
            ui.horizontal(|ui| {
                egui::ScrollArea::horizontal()
                    .id_salt("tabs_scroll")
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let mut to_activate = None;
                            let mut to_close = None;

                            for (i, tab) in self.tabs.iter().enumerate() {
                                let name = tab
                                    .path
                                    .as_ref()
                                    .and_then(|p| p.file_name())
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("Untitled");

                                let is_active = i == self.active_tab;

                                // Visuals
                                let bg_color = if is_active {
                                    self.theme.tab_active_bg
                                } else {
                                    ui.visuals().faint_bg_color
                                };
                                let fg_color = if is_active {
                                    self.theme.tab_active_fg
                                } else {
                                    ui.visuals().text_color()
                                };

                                let resp = egui::Frame::new()
                                    .fill(bg_color)
                                    .inner_margin(egui::Margin::symmetric(8, 4))
                                    .corner_radius(egui::CornerRadius {
                                        nw: 5,
                                        ne: 5,
                                        ..Default::default()
                                    })
                                    .stroke(egui::Stroke::new(
                                        1.0_f32,
                                        ui.visuals().widgets.noninteractive.bg_stroke.color,
                                    ))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            if ui
                                                .add(
                                                    egui::Label::new(
                                                        egui::RichText::new(name).color(fg_color),
                                                    )
                                                    .sense(egui::Sense::click()),
                                                )
                                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                                .clicked()
                                            {
                                                to_activate = Some(i);
                                            }

                                            if tab.is_dirty {
                                                ui.add(
                                                    egui::Label::new(
                                                        egui::RichText::new("●").size(10.0),
                                                    )
                                                    .sense(egui::Sense::hover()),
                                                );
                                            }

                                            // Close button (x)
                                            if ui
                                                .add(
                                                    egui::Button::new(
                                                        egui::RichText::new("×").size(14.0),
                                                    )
                                                    .frame(false),
                                                )
                                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                                .clicked()
                                            {
                                                to_close = Some(i);
                                            }
                                        });
                                    });

                                if resp.response.clicked() {
                                    to_activate = Some(i);
                                }
                                if let Some(path) = tab.path.as_ref() {
                                    resp.response.on_hover_text(path.display().to_string());
                                }

                                ui.add_space(2.0);
                            }

                            if ui
                                .button("+")
                                .on_hover_text("New file (Ctrl+N)")
                                .on_hover_cursor(egui::CursorIcon::PointingHand)
                                .clicked()
                            {
                                action = Some(HeaderAction::NewFile);
                            }

                            if let Some(i) = to_activate
                                && self.active_tab != i {
                                    self.active_tab = i;
                                    self.is_assembly_stale = true;
                                }
                            if let Some(i) = to_close {
                                action = Some(HeaderAction::CloseTab(i));
                            }
                        });
                    });
            });
            ui.separator();

            ui.horizontal(|ui| {
                let is_binary = self
                    .tabs
                    .get(self.active_tab)
                    .is_some_and(|t| t.kind == EditorTabKind::Binary);
                ui.heading(if is_binary {
                    "Binary Image"
                } else {
                    "Assembly Source"
                });
                if is_binary {
                    ui.label(
                        egui::RichText::new("read-only hexdump")
                            .small()
                            .color(self.theme.muted),
                    )
                    .on_hover_text(
                        "These are the bytes loaded into memory. Use File → Assemble Binary to export a .bin.",
                    );
                }
                if self.is_assembly_stale {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("⚠ Out of date (assemble to refresh)")
                            .color(self.theme.stale)
                            .small(),
                    )
                    .on_hover_text(
                        "The source changed since the last assemble. Run still executes the old program.",
                    );
                }
            });

            if self.active_tab < self.tabs.len() {
                let highlight_line = if !self.exec.is_running || self.exec.snapshot.halted {
                    let pc = self.exec.snapshot.pc;
                    if self.exec.snapshot.halted {
                        let mut best_match = None;
                        let mut max_addr = -1i32;
                        for (&addr, &line) in &self.address_to_line {
                            if addr < pc && (addr as i32) > max_addr {
                                max_addr = addr as i32;
                                best_match = Some(line);
                            }
                        }
                        best_match.or_else(|| self.address_to_line.get(&pc).copied())
                    } else {
                        self.address_to_line.get(&pc).copied()
                    }
                } else {
                    None
                };

                let current_tab = &mut self.tabs[self.active_tab];

                egui::ScrollArea::vertical().show(ui, |ui| {

                    ui.horizontal_top(|ui| {
                        let num_lines = if current_tab.code.is_empty() {
                            1
                        } else {
                            current_tab.code.lines().count()
                                + if current_tab.code.ends_with('\n') {
                                    1
                                } else {
                                    0
                                }
                        };

                        let font_id = egui::FontId::monospace(14.0);

                        // 1. Line numbers column
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;

                            // Header for Line
                            ui.label(egui::RichText::new("Line").
                                font(font_id.clone()).strong().color(self.theme.muted));

                            for i in 1..=num_lines {
                                let is_bp = current_tab.breakpoints.contains(&i);
                                let is_pc = highlight_line == Some(i);
                                let bg = if is_bp {
                                    self.theme.breakpoint
                                } else if is_pc {
                                    self.theme.pc_row
                                } else {
                                    egui::Color32::TRANSPARENT
                                };
                                let text_color = if is_bp {
                                    egui::Color32::WHITE
                                } else if is_pc {
                                    self.theme.pc_marker
                                } else {
                                    self.theme.gutter
                                };

                                let marker = if is_pc { "▶" } else { " " };
                                let label = egui::Label::new(
                                    egui::RichText::new(format!(
                                        "{}{: >4}",
                                        marker, i
                                    ))
                                    .font(font_id.clone())
                                    .color(text_color)
                                    .background_color(bg),
                                )
                                .sense(egui::Sense::click());

                                if ui
                                    .add(label)
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .on_hover_text(if is_bp {
                                        "Breakpoint set — click to remove"
                                    } else {
                                        "Click to set a breakpoint"
                                    })
                                    .clicked()
                                {
                                    // Toggle breakpoint
                                    if is_bp {
                                        current_tab.breakpoints.remove(&i);
                                    } else {
                                        current_tab.breakpoints.insert(i);
                                    }
                                }
                            }
                        });

                        // 2. Location Counter Columns
                        if self.show_location_counter {
                            ui.add_space(4.0);

                            // Extract lines once so we can parse their content
                            let lines: Vec<&str> = current_tab.code.lines().collect();

                            // Address Column
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 0.0;
                                ui.label(egui::RichText::new("Address").
                                font(font_id.clone()).strong().color(self.theme.muted));

                                let addr_color = if self.is_assembly_stale {
                                    self.theme.stale
                                } else {
                                    self.theme.address
                                };

                                for i in 1..=num_lines {
                                    let mut addr_str = "    ".to_string();

                                    if let Some(line) = self.assembly_lines.get(&i)
                                        && !line.bytes.is_empty()
                                    {
                                        addr_str = format!("{:04X}", line.address);
                                    }

                                    let label = egui::Label::new(
                                        egui::RichText::new(addr_str)
                                            .font(font_id.clone())
                                            .color(addr_color),
                                    );
                                    let resp = ui.add(label);

                                    if self.is_assembly_stale {
                                        resp.on_hover_text(
                                            "Location counter is out of date. Assemble again to refresh.",
                                        );
                                    }
                                }
                            });

                            ui.add_space(4.0);

                            // Data Column
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 0.0;
                                ui.label(egui::RichText::new("Data").font(font_id.clone()).strong().color(self.theme.muted));

                                let data_color = if self.is_assembly_stale {
                                    self.theme.stale
                                } else {
                                    self.theme.data
                                };

                                for i in 1..=num_lines {
                                    let line_text = lines.get(i - 1).unwrap_or(&"");

                                    // Parse again for the Data column
                                    let mut clean = line_text.split(';').next().unwrap_or("").trim();
                                    if let Some(idx) = clean.find(':') {
                                        clean = clean[idx + 1..].trim();
                                    }
                                    let upper = clean.to_uppercase();

                                    let emits_code = !clean.is_empty()
                                        && !upper.starts_with("ORG ") 
                                        && !upper.starts_with("EQU ") 
                                        && !upper.contains(" EQU ");

                                    let mut data_str = "".to_string();

                                    if emits_code
                                        && let Some(line) = self.assembly_lines.get(&i) {
                                            let bytes = line.bytes.iter().take(8).copied().collect::<Vec<_>>();

                                            // 3. String formatting based on text structure 
                                            if !bytes.is_empty() {
                                                if upper.starts_with("DB ") || upper.starts_with("DEFB ") {
                                                    data_str = bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(", ");
                                                } else if upper.starts_with("DW ") || upper.starts_with("DEFW ") {
                                                    let mut words = Vec::new();
                                                    for chunk in bytes.chunks(2) {
                                                        if chunk.len() == 2 {
                                                            words.push(format!("{:02X}{:02X}", chunk[1], chunk[0])); // Little-endian format for display
                                                        } else {
                                                            words.push(format!("{:02X}", chunk[0]));
                                                        }
                                                    }
                                                    data_str = words.join(", ");
                                                } else {
                                                    let mut op_len = 1;
                                                    if bytes.len() > 1 && (bytes[0] == 0xCB || bytes[0] == 0xDD || bytes[0] == 0xED || bytes[0] == 0xFD) {
                                                        op_len = 2; // Keep prefixes glued to the main Opcode
                                                    }
                                                    let op_hex = bytes[0..op_len].iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join("");

                                                    if bytes.len() <= op_len {
                                                        data_str = op_hex;
                                                    } else {
                                                        let arg_hex = bytes[op_len..].iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ");
                                                        let has_comma = clean.contains(',');
                                                        let has_parens = clean.contains('(') && clean.contains(')');

                                                        // Adapt the Data String punctuation based on the instruction
                                                        if has_comma && has_parens {
                                                            let comma_pos = clean.find(',').unwrap_or(usize::MAX);
                                                            let paren_pos = clean.find('(').unwrap_or(usize::MAX);
                                                            if paren_pos < comma_pos {
                                                                data_str = format!("{} ({})", op_hex, arg_hex); // i.e. LD (nn), A
                                                            } else {
                                                                data_str = format!("{}, ({})", op_hex, arg_hex); // i.e. LD A, (nn)
                                                            }
                                                        } else if has_comma {
                                                            data_str = format!("{}, {}", op_hex, arg_hex);
                                                        } else if has_parens {
                                                            data_str = format!("{} ({})", op_hex, arg_hex);
                                                        } else {
                                                            data_str = format!("{} {}", op_hex, arg_hex);
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                    // Pad output heavily so multi-byte values (e.g. DD21, 34 12) have a fixed column space
                                    let formatted_str = format!("{: <14}", data_str);

                                    let label = egui::Label::new(
                                        egui::RichText::new(formatted_str)
                                            .font(font_id.clone())
                                            .color(data_color),
                                    );
                                    ui.add(label);
                                }
                            });

                            ui.add_space(4.0);
                        }
                        // 3. Code Editor Column
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            // Add a matching header so the text edit component starts exactly on the same row!
                            ui.label(egui::RichText::new("Code").font(font_id.clone()).strong().color(self.theme.muted));

                            let theme = &self.code_theme;
                            let mut layouter =
                                |ui: &egui::Ui, string: &dyn TextBuffer, _wrap_width: f32| {
                                    let layout_job = highlighting::highlight(
                                        ui.ctx(),
                                        theme,
                                        string.as_str(),
                                        highlight_line,
                                    );
                                    ui.painter().layout_job(layout_job)
                                };

                            let is_binary_tab = current_tab.kind == EditorTabKind::Binary;

                            // Long lines are reachable via a horizontal scrollbar
                            // instead of wrapping: wrapping would add visual rows
                            // and desynchronise the line-number/address gutters.
                            let font_id = egui::FontId::monospace(14.0);
                            let longest = current_tab
                                .code
                                .split('\n')
                                .map(|line| line.chars().count())
                                .max()
                                .unwrap_or(0);
                            let text_width = ui
                                .painter()
                                .layout_no_wrap(
                                    "M".repeat(longest.max(1)),
                                    font_id,
                                    egui::Color32::PLACEHOLDER,
                                )
                                .size()
                                .x;
                            let desired_width =
                                (text_width + 24.0).max(ui.available_width());

                            egui::ScrollArea::horizontal()
                                .id_salt("code_hscroll")
                                .auto_shrink([false, true])
                                .show(ui, |ui| {
                                    let editor = if is_binary_tab {
                                        egui::TextEdit::multiline(&mut current_tab.code)
                                            .font(egui::TextStyle::Monospace)
                                            .layouter(&mut layouter)
                                            .code_editor()
                                            .margin(egui::Margin::symmetric(4, 0))
                                            .desired_width(desired_width)
                                            .desired_rows(25)
                                            .lock_focus(true)
                                            .interactive(false)
                                    } else {
                                        egui::TextEdit::multiline(&mut current_tab.code)
                                            .font(egui::TextStyle::Monospace)
                                            .layouter(&mut layouter)
                                            .code_editor()
                                            .margin(egui::Margin::symmetric(4, 0))
                                            .desired_width(desired_width)
                                            .desired_rows(25)
                                            .lock_focus(true)
                                    };

                                    if ui.add(editor).changed() && !is_binary_tab {
                                        current_tab.is_dirty = true;
                                        self.is_assembly_stale = true;
                                    }
                                });
                        });
                    });
                });
            } else {
                ui.label("No files open");
            }
        });

        if let Some(action) = action {
            self.process_header_action(action, &ctx, frame);
        }

        // Handle Modal Dialogs
        if let Some(mut modal_type) = self.pending_modal.take() {
            let mut should_close_modal = false;

            let title = match &modal_type {
                ModalType::AddDevice(_, _) => "Attach Device",
                ModalType::CloseTab(_) => "Close Unsaved Tab",
                ModalType::Quit => "Quit JAZZ80",
            };

            let modal = egui::Modal::new(egui::Id::unique("modal")).show(ui.ctx(), |ui| {
                ui.set_width(380.0);
                ui.heading(title);
                ui.separator();

                match &mut modal_type {
                    ModalType::AddDevice(definition, port_text) => {
                        ui.label("I/O port (hexadecimal):");
                        ui.add(
                            egui::TextEdit::singleline(port_text)
                                .hint_text("e.g. 20")
                                .desired_width(110.0),
                        );
                        let parsed = parse_hex_u16(port_text);
                        if parsed.is_none() {
                            ui.label(
                                egui::RichText::new("Enter a hexadecimal port, e.g. 20")
                                    .small()
                                    .color(self.theme.danger),
                            );
                        } else {
                            ui.label(
                                egui::RichText::new("Devices listen on the low byte of the port.")
                                    .small()
                                    .color(self.theme.muted),
                            );
                        }
                        if let Some(port) = parsed
                            && port > 0xFF
                        {
                            ui.label(
                                egui::RichText::new("Port must be 0–255")
                                    .small()
                                    .color(self.theme.danger),
                            );
                        }

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            let valid_port = parse_hex_u16(port_text).filter(|port| *port <= 0xFF);
                            if ui
                                .add_enabled(valid_port.is_some(), egui::Button::new("Add"))
                                .clicked()
                            {
                                let port = valid_port.unwrap();
                                let dev = (definition.create)(port);
                                if let Some(machine) = self.exec.machine.as_mut() {
                                    machine.cpu.attach_device(dev.clone());
                                }
                                self.attached_devices.push(dev);
                                self.attached_device_specs.push(DeviceSpec {
                                    menu_name: definition.menu_name.to_string(),
                                    port,
                                });
                                should_close_modal = true;
                            }
                            if ui.button("Cancel").clicked() {
                                should_close_modal = true;
                            }
                        });
                    }
                    ModalType::CloseTab(idx) => {
                        let idx = *idx;
                        let name = self
                            .tabs
                            .get(idx)
                            .and_then(|t| t.path.as_ref())
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .unwrap_or("Untitled");

                        ui.label(format!(
                            "Save the changes you made to \"{name}\" before closing?"
                        ));
                        ui.label(
                            egui::RichText::new(
                                "Your changes will be lost if you don't save them.",
                            )
                            .color(self.theme.muted),
                        );

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            if ui.button("Save").clicked() {
                                self.save_file(idx, frame.storage_mut());
                                if self.tabs.get(idx).is_some_and(|t| !t.is_dirty) {
                                    self.close_tab(idx, frame.storage_mut());
                                    should_close_modal = true;
                                }
                            }
                            if ui.button("Don't Save").clicked() {
                                if let Some(tab) = self.tabs.get_mut(idx) {
                                    tab.is_dirty = false;
                                }
                                self.close_tab(idx, frame.storage_mut());
                                should_close_modal = true;
                            }
                            if ui.button("Cancel").clicked() {
                                should_close_modal = true;
                            }
                        });
                    }
                    ModalType::Quit => {
                        ui.label("You have unsaved changes in this workspace.");
                        ui.label("Save everything before quitting?");

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            if ui.button("Save All").clicked() && self.save_all(frame.storage_mut())
                            {
                                should_close_modal = true;
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            if ui.button("Quit Without Saving").clicked() {
                                for tab in &mut self.tabs {
                                    tab.is_dirty = false;
                                }
                                should_close_modal = true;
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            if ui.button("Cancel").clicked() {
                                should_close_modal = true;
                            }
                        });
                    }
                }
            });

            if modal.should_close() {
                should_close_modal = true;
            }

            if !should_close_modal {
                self.pending_modal = Some(modal_type);
            }
        }

        // Keyboard shortcut reference
        if self.show_shortcuts {
            egui::Window::new("Keyboard Shortcuts")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .open(&mut self.show_shortcuts)
                .show(ui.ctx(), |ui| {
                    let rows = [
                        ("Run / Stop", "F5"),
                        ("Step one instruction", "F8"),
                        ("New file", "Ctrl+N"),
                        ("Open file", "Ctrl+O"),
                        ("Save", "Ctrl+S"),
                        ("Save as", "Ctrl+Shift+S"),
                        ("Close tab", "Ctrl+W"),
                        ("Toggle breakpoint", "Click the gutter"),
                    ];
                    egui::Grid::new("shortcuts_grid")
                        .num_columns(2)
                        .spacing([24.0, 6.0])
                        .striped(true)
                        .show(ui, |ui| {
                            for (action, keys) in rows {
                                ui.label(action);
                                ui.monospace(keys);
                                ui.end_row();
                            }
                        });
                });
        }

        // Poll execution events from the runner
        self.poll_execution(&ctx);

        // Draw Memory Viewer Window
        self.show_memory_viewer(ui);

        // Draw separate windows for devices
        self.show_device_windows(&ctx);

        self.exec
            .sync_breakpoints(&self.line_to_address, self.active_tab, &self.tabs);
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
mod execution_state_tests {
    use super::{ExecutionState, Machine};

    #[test]
    fn applying_a_delta_keeps_unchanged_memory() {
        let mut state = ExecutionState::new();
        state.snapshot.memory[0x1000] = 0x11;
        state.snapshot.memory[0x1001] = 0x22;

        let mut machine = Machine::new();
        machine.memory_mut().write(0x1001, 0x33);
        state.apply_delta(machine.snapshot_delta());

        assert_eq!(state.snapshot.memory[0x1000], 0x11);
        assert_eq!(state.snapshot.memory[0x1001], 0x33);
    }
}
