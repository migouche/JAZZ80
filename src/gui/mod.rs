use crate::assembler::{assemble_binary, assemble_with_metadata};
use crate::cpu::{Flag, GPR};
use crate::emulator::{Machine, MachineSnapshot};
use crate::gui::exec::{Command, Event, Runner};
use crate::locking::lock_or_recover;
use crate::traits::SynchronousComponent;
use eframe::egui;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::components::devices::DeviceDefinition;
use crate::ui_traits::DeviceWithUi;

pub mod exec;
mod files;
mod formatting;
mod highlighting;
mod persistence;
mod session;
mod theme;
mod ui;
mod workspace;

use files::{FileEvent, FileOrigin, FileService, SaveMode};

pub use persistence::PersistedWorkspace;
pub use session::DebugSession;
pub(crate) use workspace::EditorTabKind;
pub use workspace::{EditorTab, Workspace};

#[cfg(target_arch = "wasm32")]
include!(concat!(env!("OUT_DIR"), "/bundled_examples.inc"));

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
    CloseActiveTab,
    ToggleRun,
    StepInstruction,
    Quit,
}

impl HeaderAction {
    fn shortcut(&self) -> Option<(egui::Modifiers, egui::Key)> {
        use egui::{Key, Modifiers};
        match self {
            Self::ToggleRun => Some((Modifiers::NONE, Key::F5)),
            Self::StepInstruction => Some((Modifiers::NONE, Key::F8)),
            Self::NewFile => Some((Modifiers::COMMAND, Key::N)),
            Self::OpenFileDialog => Some((Modifiers::COMMAND, Key::O)),
            Self::SaveFile => Some((Modifiers::COMMAND, Key::S)),
            Self::SaveFileAs => Some((Modifiers::COMMAND | Modifiers::SHIFT, Key::S)),
            Self::CloseActiveTab => Some((Modifiers::COMMAND, Key::W)),
            Self::OpenFile(_)
            | Self::AssembleBinary
            | Self::LoadBinary
            | Self::CloseTab(_)
            | Self::Quit => None,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::ToggleRun => "Run / Stop",
            Self::StepInstruction => "Step one instruction",
            Self::NewFile => "New file",
            Self::OpenFileDialog => "Open file",
            Self::SaveFile => "Save",
            Self::SaveFileAs => "Save as",
            Self::CloseActiveTab | Self::CloseTab(_) => "Close tab",
            Self::OpenFile(_) => "Open file",
            Self::AssembleBinary => "Assemble to binary",
            Self::LoadBinary => "Load binary",
            Self::Quit => "Quit",
        }
    }

    fn shortcut_text(&self, ctx: &egui::Context) -> String {
        self.shortcut()
            .map(|(modifiers, key)| {
                ctx.format_shortcut(&egui::KeyboardShortcut::new(modifiers, key))
            })
            .unwrap_or_default()
    }
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

    attached_devices: Vec<Arc<Mutex<dyn DeviceWithUi>>>,

    attached_device_specs: Vec<DeviceSpec>,

    theme: theme::AppTheme,

    highlight_cache: highlighting::HighlightCache,

    files: FileService,

    #[cfg(target_arch = "wasm32")]
    examples_list: Vec<String>,

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
        [
            HeaderAction::ToggleRun,
            HeaderAction::StepInstruction,
            HeaderAction::NewFile,
            HeaderAction::OpenFileDialog,
            HeaderAction::SaveFile,
            HeaderAction::SaveFileAs,
            HeaderAction::CloseActiveTab,
        ]
        .into_iter()
        .filter_map(|action| {
            action
                .shortcut()
                .map(|(modifiers, key)| (modifiers, key, action))
        })
        .collect()
    }

    fn check_shortcuts(&self, ctx: &egui::Context) -> Option<HeaderAction> {
        let mut action = None;
        let mut binds = Self::keybinds();

        binds.sort_by_key(|(modifiers, _, _)| {
            std::cmp::Reverse(u8::from(modifiers.shift) + u8::from(modifiers.alt))
        });
        ctx.input_mut(|i| {
            for (modifiers, key, act) in binds {
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
                lock_or_recover(device).reset();
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

    fn toggle_execution(&mut self) {
        if self.session.snapshot().halted {
            if self.session.is_running() {
                self.session.runner_mut().send_command(Command::Resume);
                return;
            }
            if let Some(machine) = self.session.machine_mut() {
                machine.cpu.set_halted(false);
            }
            self.session.refresh_snapshot();
        } else if self.session.is_running() {
            self.stop_execution();
            return;
        }

        if !self.workspace.program_loaded {
            return;
        }

        if self
            .session
            .breakpoints()
            .contains(&self.session.snapshot().pc)
            && let Some(machine) = self.session.machine_mut()
        {
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
        app.files.start_examples(&cc.egui_ctx);
        app.load_and_reset();
        app
    }

    fn open_file_dialog(
        &mut self,
        ctx: &egui::Context,
        storage: Option<&mut (dyn eframe::Storage + 'static)>,
    ) {
        let events = self.files.pick_source(ctx);
        self.apply_file_events(events, storage);
    }

    fn new_file(&mut self, storage: Option<&mut (dyn eframe::Storage + 'static)>) {
        self.workspace.add_source_tab(None, Self::default_code());
        self.workspace.is_assembly_stale = true;
        self.save_to_storage(storage);
    }

    fn request_close_tab(
        &mut self,
        index: usize,
        storage: Option<&mut (dyn eframe::Storage + 'static)>,
    ) {
        if index >= self.workspace.tabs.len() {
            return;
        }
        if self.workspace.tabs[index].is_dirty {
            self.pending_modal = Some(ModalType::CloseTab(index));
        } else {
            self.close_tab(index, storage);
        }
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
        ctx: &egui::Context,
        storage: Option<&mut (dyn eframe::Storage + 'static)>,
    ) {
        // Check if already open
        if let Some(idx) = self.workspace.find_tab_by_path(&path) {
            self.workspace.focus_tab(idx);
            self.save_to_storage(storage);
            return;
        }

        let events = self.files.open_source(&path, ctx);
        self.apply_file_events(events, storage);
    }

    #[cfg(target_arch = "wasm32")]
    fn load_example(
        &mut self,
        filename: String,
        ctx: &egui::Context,
        storage: Option<&mut (dyn eframe::Storage + 'static)>,
    ) {
        let events = self.files.open_source(std::path::Path::new(&filename), ctx);
        self.apply_file_events(events, storage);
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
        let name = lock_or_recover(&self.attached_devices[idx]).get_name();
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

    fn tab_save_data(tab: &EditorTab) -> (u64, Option<PathBuf>, String, String) {
        let name = tab
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("program.asm")
            .to_string();
        (tab.id, tab.path.clone(), name, tab.code.clone())
    }

    fn save_file(&mut self, tab_idx: usize, storage: Option<&mut (dyn eframe::Storage + 'static)>) {
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

        let Some((tab_id, path, name, text)) =
            self.workspace.tabs.get(tab_idx).map(Self::tab_save_data)
        else {
            return;
        };

        let save_mode = self.files.save_mode();
        if tab_idx != self.workspace.active_tab
            && !(save_mode == SaveMode::InPlace && path.is_some())
        {
            self.status_message = Some(
                match save_mode {
                    SaveMode::InPlace => "Only the active untitled file can be saved",
                    SaveMode::Download => "Only the active file can be saved on the web",
                }
                .to_string(),
            );
            return;
        }

        let events = self
            .files
            .save_source(tab_id, path.as_deref(), &name, &text);
        self.apply_file_events(events, storage);
    }

    fn save_file_as(&mut self, storage: Option<&mut (dyn eframe::Storage + 'static)>) {
        let Some((tab_id, _path, name, text)) = self
            .workspace
            .tabs
            .get(self.workspace.active_tab)
            .map(Self::tab_save_data)
        else {
            return;
        };

        let events = self.files.save_source(tab_id, None, &name, &text);
        self.apply_file_events(events, storage);
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
                let events = self.files.export_binary(&name, &bytes);
                self.apply_file_events(events, storage);
            }
            Err(err) => {
                self.last_error = Some(format!("Failed to assemble binary: {}", err));
            }
        }
    }

    fn load_binary_dialog(
        &mut self,
        ctx: &egui::Context,
        storage: Option<&mut (dyn eframe::Storage + 'static)>,
    ) {
        let events = self.files.pick_binary(ctx);
        self.apply_file_events(events, storage);
    }

    fn apply_file_events(
        &mut self,
        events: Vec<FileEvent>,
        mut storage: Option<&mut (dyn eframe::Storage + 'static)>,
    ) {
        for event in events {
            self.apply_file_event(event, storage.as_deref_mut());
        }
    }

    fn apply_file_event(
        &mut self,
        event: FileEvent,
        storage: Option<&mut (dyn eframe::Storage + 'static)>,
    ) {
        match event {
            FileEvent::SourceOpened {
                origin: FileOrigin::Device,
                path,
                content,
            } => {
                if let Some(idx) = self.workspace.find_tab_by_path(&path) {
                    self.workspace.focus_tab(idx);
                    self.save_to_storage(storage);
                } else {
                    self.load_file_content(path, content, storage);
                }
            }
            FileEvent::SourceOpened {
                origin: FileOrigin::Example,
                path,
                content,
            } => {
                self.load_file_content(path.clone(), content, storage);
                self.status_message = Some(format!("Loaded example {}", path.display()));
            }
            FileEvent::BinaryOpened {
                origin: FileOrigin::Device,
                path,
                bytes,
            } => {
                self.open_binary_tab(path, bytes, storage);
                self.last_error = None;
            }
            FileEvent::BinaryOpened {
                origin: FileOrigin::Example,
                path,
                bytes,
            } => {
                self.open_binary_tab(path, bytes, storage);
            }
            FileEvent::SourceSaved { tab_id, path } => {
                if let Some(tab) = self.workspace.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    tab.path = Some(path.clone());
                    tab.is_dirty = false;
                }
                self.status_message = Some(format!("Saved {}", path.display()));
                self.add_recent_file(path);
                self.save_to_storage(storage);
            }
            FileEvent::SourceDownloaded { tab_id, name } => {
                if let Some(tab) = self.workspace.tabs.iter_mut().find(|tab| tab.id == tab_id) {
                    tab.is_dirty = false;
                }
                self.status_message = Some(format!("Downloaded {name}"));
                self.save_to_storage(storage);
            }
            FileEvent::BinaryExported { target, path, len } => {
                self.last_error = None;
                self.status_message = Some(format!("Exported {target} ({len} bytes)"));
                if let Some(path) = path {
                    self.add_recent_file(path);
                }
                self.save_to_storage(storage);
            }
            FileEvent::ExportCancelled => {
                self.status_message = Some("Binary export cancelled".to_string());
            }
            #[cfg(target_arch = "wasm32")]
            FileEvent::ExamplesListed(list) => self.examples_list = list,
            FileEvent::Failed(message) => self.last_error = Some(message),
        }
    }

    fn poll_files(&mut self, storage: Option<&mut (dyn eframe::Storage + 'static)>) {
        let events = self.files.poll();
        self.apply_file_events(events, storage);
    }

    fn dispatch_frame_action(
        &mut self,
        action: Option<HeaderAction>,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
    ) {
        self.sync_breakpoints();

        if let Some(action) = action {
            self.process_header_action(action, ctx, frame);
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
            HeaderAction::CloseTab(index) => self.request_close_tab(index, frame.storage_mut()),
            HeaderAction::CloseActiveTab => {
                let index = self.workspace.active_tab;
                self.request_close_tab(index, frame.storage_mut())
            }
            HeaderAction::ToggleRun => self.toggle_execution(),
            HeaderAction::StepInstruction => self.step_instruction(),
            HeaderAction::Quit => {
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
            attached_devices: Vec::new(),
            attached_device_specs: Vec::new(),
            theme: theme::AppTheme::ONE_DARK,
            highlight_cache: highlighting::HighlightCache::default(),
            files: FileService::default(),
            #[cfg(target_arch = "wasm32")]
            examples_list: Vec::new(),
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
        self.poll_files(frame.storage_mut());

        if ctx.input(|i| i.viewport().close_requested()) {
            let any_dirty = self.workspace.tabs.iter().any(|t| t.is_dirty);
            if any_dirty {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.pending_modal = Some(ModalType::Quit);
            }
        }

        let mut action = self.check_shortcuts(&ctx);

        egui::Panel::top("top_panel").show(ui, |ui| {
            if let Some(header_action) = ui::header::show(self, frame, ui) {
                action = Some(header_action);
            }
        });

        egui::Panel::bottom("status_bar").show(ui, |ui| ui::status_bar::show(self, ui));

        egui::Panel::right("right_panel")
            .resizable(true)
            .default_size(280.0)
            .show(ui, |ui| ui::panels::show_registers(self, ui));

        egui::Panel::right("vars_panel")
            .resizable(true)
            .default_size(280.0)
            .show(ui, |ui| ui::panels::show_variables(self, ui));

        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(editor_action) = ui::editor::show(self, ui) {
                action = Some(editor_action);
            }
        });

        self.dispatch_frame_action(action, &ctx, frame);

        ui::modals::show(self, &ctx, frame);

        ui::windows::show_shortcuts(self, &ctx);

        self.poll_execution(&ctx);

        ui::windows::show_memory_viewer(self, ui);

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
    use super::files::{FileEvent, FileOrigin};
    use super::{EditorTab, EditorTabKind, PersistedWorkspace, STORAGE_KEY, Workspace, Z80App};
    use eframe::Storage;
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    fn temp_path(name: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock after the epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("jazz80-app-{}-{nanos}-{name}", std::process::id()))
    }

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

    #[test]
    fn source_saved_marks_the_tab_clean_and_sets_its_path() {
        let mut app = Z80App::default();
        let tab_id = app.workspace.tabs[0].id;
        app.workspace.tabs[0].is_dirty = true;

        // The native recent-file gate drops paths that do not exist.
        let path = temp_path("saved.asm");
        std::fs::write(&path, "").unwrap();

        app.apply_file_event(
            FileEvent::SourceSaved {
                tab_id,
                path: path.clone(),
            },
            None,
        );

        assert!(!app.workspace.tabs[0].is_dirty);
        assert_eq!(app.workspace.tabs[0].path, Some(path.clone()));
        assert_eq!(
            app.status_message,
            Some(format!("Saved {}", path.display()))
        );
        assert_eq!(app.workspace.recent_files.first(), Some(&path));

        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn source_downloaded_marks_the_tab_clean() {
        let mut app = Z80App::default();
        let tab_id = app.workspace.tabs[0].id;
        app.workspace.tabs[0].is_dirty = true;

        app.apply_file_event(
            FileEvent::SourceDownloaded {
                tab_id,
                name: "x.asm".to_string(),
            },
            None,
        );

        assert!(!app.workspace.tabs[0].is_dirty);
        assert_eq!(app.status_message, Some("Downloaded x.asm".to_string()));
    }

    #[test]
    fn failed_event_sets_last_error() {
        let mut app = Z80App::default();

        app.apply_file_event(FileEvent::Failed("boom".to_string()), None);

        assert_eq!(app.last_error, Some("boom".to_string()));
    }

    #[test]
    fn binary_exported_sets_status_and_clears_the_error() {
        let mut app = Z80App {
            last_error: Some("stale".to_string()),
            ..Default::default()
        };

        app.apply_file_event(
            FileEvent::BinaryExported {
                target: "x.bin".to_string(),
                path: None,
                len: 3,
            },
            None,
        );

        assert_eq!(app.last_error, None);
        assert_eq!(
            app.status_message,
            Some("Exported x.bin (3 bytes)".to_string())
        );
    }

    #[test]
    fn source_opened_for_a_device_dedupes_by_path() {
        let mut app = Z80App::default();
        let path = PathBuf::from("already-open.asm");
        app.workspace.tabs[0].path = Some(path.clone());
        app.workspace.add_source_tab(None, "NOP".to_string());
        assert_eq!(app.workspace.tabs.len(), 2);
        assert_eq!(app.workspace.active_tab, 1);

        app.apply_file_event(
            FileEvent::SourceOpened {
                origin: FileOrigin::Device,
                path,
                content: "HALT".to_string(),
            },
            None,
        );

        assert_eq!(app.workspace.tabs.len(), 2);
        assert_eq!(app.workspace.active_tab, 0);
    }
}

#[cfg(test)]
mod shortcut_tests {
    use super::*;

    fn press(ctx: &egui::Context, modifiers: egui::Modifiers, key: egui::Key) {
        ctx.input_mut(|input| {
            input.events.push(egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers,
            });
        });
    }

    #[test]
    fn save_as_shortcut_is_not_shadowed_by_save() {
        let ctx = egui::Context::default();
        press(
            &ctx,
            egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
            egui::Key::S,
        );
        assert!(matches!(
            Z80App::default().check_shortcuts(&ctx),
            Some(HeaderAction::SaveFileAs)
        ));
    }

    #[test]
    fn save_shortcut_still_saves() {
        let ctx = egui::Context::default();
        press(&ctx, egui::Modifiers::COMMAND, egui::Key::S);
        assert!(matches!(
            Z80App::default().check_shortcuts(&ctx),
            Some(HeaderAction::SaveFile)
        ));
    }
}

#[cfg(test)]
mod frame_action_tests {
    use super::*;

    #[test]
    fn breakpoints_are_synced_before_dispatch() {
        let mut app = Z80App::default();
        app.workspace.program_loaded = true;
        app.workspace.line_to_address.insert(1, 0);
        app.workspace.tabs[0].breakpoints.insert(1);
        app.session.machine_mut().unwrap().cpu.set_pc(0);
        app.session.refresh_snapshot();
        assert!(app.session.breakpoints().is_empty());

        let ctx = egui::Context::default();
        let mut frame = eframe::Frame::_new_kittest();
        app.dispatch_frame_action(Some(HeaderAction::ToggleRun), &ctx, &mut frame);

        assert_eq!(
            app.session.snapshot().pc,
            1,
            "a breakpoint set this frame must be seen by the run it starts"
        );
        assert_eq!(app.session.breakpoints(), &HashSet::from([0]));
        let _ = app.session.runner_mut().take_machine();
    }
}
