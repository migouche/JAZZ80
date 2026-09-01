use crate::assembler::{Symbol, SymbolType, assemble, assemble_absolute, assemble_binary};
use eframe::egui::{self, TextBuffer};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::rc::Rc;

use crate::components::memories::mem_64k::Mem64k;
use crate::cpu::{Flag, GPR, Z80A};
use crate::traits::{MemoryMapper, SyncronousComponent};

use crate::components::devices::DeviceDefinition;
use crate::ui_traits::DeviceWithUi;

mod highlighting;

#[cfg(target_arch = "wasm32")]
use std::sync::mpsc::{Receiver, Sender, channel};

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
")]
extern "C" {
    fn download_file(filename: &str, text: &str);
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

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Z80App {
    #[serde(skip)]
    cpu: Z80A,
    #[serde(skip)]
    memory: Rc<RefCell<dyn MemoryMapper>>,

    tabs: Vec<EditorTab>,
    active_tab: usize,

    #[serde(skip)]
    last_error: Option<String>,
    #[serde(skip)]
    symbol_table: HashMap<String, Symbol>,
    #[serde(skip)]
    address_to_line: HashMap<u16, usize>,
    #[serde(skip)]
    line_to_address: HashMap<usize, u16>,

    #[serde(default)]
    symbol_display_prefs: HashMap<String, SymbolDisplayFormat>,

    #[serde(skip)]
    is_running: bool,
    #[serde(skip)]
    pending_modal: Option<ModalType>,
    #[serde(skip)]
    loaded_file_name: String,
    #[serde(skip)]
    code_theme: highlighting::CodeTheme,

    #[serde(skip)]
    attached_devices: Vec<Rc<RefCell<dyn DeviceWithUi>>>,

    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    file_receiver: Option<Receiver<(String, String)>>,

    #[serde(skip)]
    examples_list: Vec<String>,
    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    examples_receiver: Option<Receiver<Vec<String>>>,

    recent_files: Vec<PathBuf>,

    #[serde(skip)]
    show_memory_viewer: bool,
    #[serde(skip)]
    memory_viewer_start: String,

    // Location Counter toggle and state tracking
    #[serde(default)]
    show_location_counter: bool,
    #[serde(skip)]
    is_assembly_stale: bool,
}

const APP_ID: &str = "jazz80-simulator";
const APP_NAME: &str = "JAZZ80 - Z80 Simulator";
const STORAGE_KEY: &str = "z80_workspace";

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id(APP_ID)
            .with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        APP_NAME,
        options,
        Box::new(|cc| Ok(Box::new(Z80App::new(cc)))),
    )
}

#[cfg(target_arch = "wasm32")]
pub fn run() -> eframe::Result<()> {
    // Placeholder to satisfy main.rs, but main.rs will ignore it on wasm usually
    Ok(())
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

    fn load_and_reset(&mut self) {
        // Reset
        let memory: Rc<RefCell<dyn MemoryMapper>> = Rc::new(RefCell::new(Mem64k::new()));
        self.memory = memory.clone();
        self.cpu = Z80A::new(self.memory.clone());
        for device in &self.attached_devices {
            device.borrow_mut().reset();
            self.cpu.attach_device(device.clone());
        }

        self.is_running = false;

        if self.tabs.is_empty() {
            return;
        }

        let active = self.active_tab;
        let active_tab = &self.tabs[active];
        self.loaded_file_name = active_tab
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
            .to_string();

        if active_tab.kind == EditorTabKind::Binary {
            self.symbol_table.clear();
            self.address_to_line.clear();
            self.line_to_address.clear();
            self.is_assembly_stale = false;
            self.last_error = None;

            if let Some(bytes) = active_tab.binary_bytes.clone() {
                let mut mem = self.memory.borrow_mut();
                for (i, b) in bytes.iter().enumerate() {
                    mem.write(i as u16, *b);
                }
            } else {
                self.last_error = Some("Binary tab is missing raw bytes".to_string());
            }
            return;
        }

        let code = &active_tab.code;
        let (bytes, symbols, addr_map, line_map, image, error) =
            match (assemble(code), assemble_absolute(code)) {
                (Ok((b, s, m, l)), Ok(image)) => (b, s, m, l, image, None),
                (Err(e), _) => (
                    Vec::new(),
                    HashMap::new(),
                    HashMap::new(),
                    HashMap::new(),
                    Vec::new(),
                    Some(e),
                ),
                (_, Err(e)) => (
                    Vec::new(),
                    HashMap::new(),
                    HashMap::new(),
                    HashMap::new(),
                    Vec::new(),
                    Some(e),
                ),
            };

        self.symbol_table = symbols;
        self.address_to_line = addr_map;
        self.line_to_address = line_map;
        self.is_assembly_stale = false;

        if let Some(err) = error {
            self.last_error = Some(err);
        } else {
            self.last_error = None;
            let mut mem = self.memory.borrow_mut();
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
    }

    fn save_to_storage(&self, storage: Option<&mut (dyn eframe::Storage + 'static)>) {
        if let Some(storage) = storage {
            eframe::set_value(storage, STORAGE_KEY, self);
            storage.flush();
        }
    }

    pub fn new(cc: &eframe::CreationContext) -> Self {
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

        app.load_and_reset();
        app
    }

    fn open_file_dialog(&mut self, storage: Option<&mut (dyn eframe::Storage + 'static)>) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Assembly", &["asm", "z80"])
            .add_filter("Binary", &["bin", "rom", "com"])
            .add_filter("All files", &["*"])
            .pick_file()
        {
            self.open_file(path, storage);
        }

        #[cfg(target_arch = "wasm32")]
        {
            let task = rfd::AsyncFileDialog::new()
                .add_filter("Assembly", &["asm", "z80"])
                .add_filter("All files", &["*"])
                .pick_file();

            let (sender, receiver) = channel();
            self.file_receiver = Some(receiver);

            wasm_bindgen_futures::spawn_local(async move {
                let file = task.await;
                if let Some(file) = file {
                    let name = file.file_name();
                    let content = file.read().await;
                    if let Ok(text) = String::from_utf8(content) {
                        sender.send((name, text)).ok();
                    }
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
        if index < self.tabs.len() {
            self.tabs.remove(index);
            if self.tabs.is_empty() {
                self.new_file(storage); // Ensure at least one tab
            } else {
                if self.active_tab >= self.tabs.len() {
                    self.active_tab = self.tabs.len() - 1;
                }
                self.is_assembly_stale = true;
                self.save_to_storage(storage);
            }
        }
    }

    fn binary_hexdump(bytes: &[u8]) -> String {
        let mut lines = Vec::new();
        for (offset, chunk) in bytes.chunks(16).enumerate() {
            let start = offset * 16;
            let hex = chunk
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");
            let ascii = chunk
                .iter()
                .map(|b| {
                    let c = *b as char;
                    if c.is_ascii_graphic() || c == ' ' {
                        c
                    } else {
                        '.'
                    }
                })
                .collect::<String>();
            lines.push(format!("{:04X}: {:<47} {}", start as u16, hex, ascii));
        }
        if lines.is_empty() {
            "".to_string()
        } else {
            lines.join("\n")
        }
    }

    fn open_binary_tab(
        &mut self,
        path: PathBuf,
        bytes: Vec<u8>,
        storage: Option<&mut (dyn eframe::Storage + 'static)>,
    ) {
        let hexdump = Self::binary_hexdump(&bytes);
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
        let binary_kind = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "bin" | "rom" | "com"))
            .unwrap_or(false);

        if current_is_disposable {
            self.tabs[self.active_tab] = EditorTab {
                path: Some(path.clone()),
                code: content,
                kind: if binary_kind {
                    EditorTabKind::Binary
                } else {
                    EditorTabKind::Source
                },
                is_dirty: false,
                breakpoints: HashSet::new(),
                binary_bytes: None,
            };
        } else {
            self.tabs.push(EditorTab {
                path: Some(path.clone()),
                code: content,
                kind: if binary_kind {
                    EditorTabKind::Binary
                } else {
                    EditorTabKind::Source
                },
                is_dirty: false,
                breakpoints: HashSet::new(),
                binary_bytes: None,
            });
            self.active_tab = self.tabs.len() - 1;
        }

        self.add_recent_file(path);
        self.load_and_reset();
        self.save_to_storage(storage);
    }

    fn open_file(&mut self, path: PathBuf, storage: Option<&mut (dyn eframe::Storage + 'static)>) {
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

        let is_binary = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "bin" | "rom" | "com"))
            .unwrap_or(false);

        #[cfg(not(target_arch = "wasm32"))]
        if is_binary {
            match std::fs::read(&path) {
                Ok(bytes) => {
                    self.open_binary_tab(path, bytes, storage);
                }
                Err(err) => {
                    self.last_error = Some(format!("Failed to open binary file: {}", err));
                }
            }
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                self.load_file_content(path, content, storage);
            }
            Err(err) => {
                self.last_error = Some(format!("Failed to open file: {}", err));
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.last_error = Some("Opening local files by path not supported on web".to_string());
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn load_example(&mut self, filename: String, ctx: egui::Context) {
        let (sender, receiver) = channel();
        self.file_receiver = Some(receiver);

        let location = web_sys::window().unwrap().location();
        let origin = location.origin().unwrap();
        let pathname = location.pathname().unwrap();
        // Ensure pathname ends in slash or strip filename if present (basic heuristic)
        let base_path = if pathname.ends_with('/') {
            pathname
        } else if let Some(last_slash) = pathname.rfind('/') {
            pathname[0..=last_slash].to_string()
        } else {
            "/".to_string()
        };

        let url = format!("{}{}z80%20files/{}", origin, base_path, filename);
        let name = filename;

        wasm_bindgen_futures::spawn_local(async move {
            match reqwest::get(&url).await {
                Ok(resp) => {
                    if let Ok(text) = resp.text().await {
                        let _ = sender.send((name, text));
                        ctx.request_repaint();
                    }
                }
                Err(_) => {}
            }
        });
    }

    fn save_file(&mut self, tab_idx: usize, storage: Option<&mut (dyn eframe::Storage + 'static)>) {
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
                self.last_error = Some("Can only save active file on web".to_string());
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
                }
            } else {
                if tab_idx == self.active_tab {
                    self.save_file_as(storage);
                } else {
                    self.last_error = Some("Cannot save untitled file in background".to_string());
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
                    let hex = bytes
                        .iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join("");
                    download_file(&name, &hex);
                    self.last_error = None;
                    self.save_to_storage(storage);
                    return;
                }

                self.last_error = Some("Binary export was cancelled".to_string());
            }
            Err(err) => {
                self.last_error = Some(format!("Failed to assemble binary: {}", err));
            }
        }
    }

    fn load_binary_file(
        &mut self,
        path: PathBuf,
        storage: Option<&mut (dyn eframe::Storage + 'static)>,
    ) {
        match std::fs::read(&path) {
            Ok(bytes) => {
                self.open_binary_tab(path.clone(), bytes, storage);
                self.loaded_file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("program.bin")
                    .to_string();
                self.is_running = false;
                self.last_error = None;
            }
            Err(err) => {
                self.last_error = Some(format!("Failed to load binary: {}", err));
            }
        }
    }

    fn load_binary_dialog(&mut self, storage: Option<&mut (dyn eframe::Storage + 'static)>) {
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
            self.last_error = Some("Binary import is not supported in the web build".to_string());
        }
    }

    fn add_recent_file(&mut self, path: PathBuf) {
        // Remove if exists to move it to the top
        println!("Adding recent file: {:?}", path);
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
        let memory: Rc<RefCell<dyn MemoryMapper>> = Rc::new(RefCell::new(Mem64k::new()));
        let cpu = Z80A::new(memory.clone());

        #[cfg(target_arch = "wasm32")]
        let (examples_sender, examples_receiver) = channel();

        #[cfg(target_arch = "wasm32")]
        {
            wasm_bindgen_futures::spawn_local(async move {
                // Fetch examples.json
                let location = web_sys::window().unwrap().location();
                let origin = location.origin().unwrap();
                let pathname = location.pathname().unwrap();
                // Ensure pathname ends in slash or strip filename if present (basic heuristic)
                let base_path = if pathname.ends_with('/') {
                    pathname
                } else if let Some(last_slash) = pathname.rfind('/') {
                    pathname[0..=last_slash].to_string()
                } else {
                    "/".to_string()
                };

                let url = format!("{}{}z80%20files/examples.json", origin, base_path);

                match reqwest::get(&url).await {
                    Ok(resp) => {
                        if !resp.status().is_success() {
                            return;
                        }
                        if let Ok(list) = resp.json::<Vec<String>>().await {
                            let _ = examples_sender.send(list);
                        }
                    }
                    Err(_) => {}
                }
            });
        }

        Self {
            cpu,
            memory,
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
            symbol_table: HashMap::new(),
            address_to_line: HashMap::new(),
            line_to_address: HashMap::new(),
            symbol_display_prefs: HashMap::new(),
            is_running: false,
            pending_modal: None,
            loaded_file_name: "Untitled".to_string(),
            code_theme: highlighting::CodeTheme::one_dark_pro_vivid(),
            recent_files: Vec::new(),
            attached_devices: Vec::new(),
            #[cfg(target_arch = "wasm32")]
            file_receiver: None,
            examples_list: Vec::new(),
            #[cfg(target_arch = "wasm32")]
            examples_receiver: Some(examples_receiver),
            show_memory_viewer: false,
            memory_viewer_start: "0000".to_string(),
            show_location_counter: true,
            is_assembly_stale: false,
        }
    }
}

impl eframe::App for Z80App {
    /// Called by the frame work to save state before shutdown.
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

            if let Some(receiver) = self.examples_receiver.take() {
                if let Ok(list) = receiver.try_recv() {
                    self.examples_list = list;
                }
                self.examples_receiver = Some(receiver);
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
                        .button("Load Binary...")
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

                // Added View Menu Option
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_location_counter, "Location Counter");
                })
                .response
                .on_hover_cursor(egui::CursorIcon::PointingHand);

                ui.menu_button("Devices", |ui| {
                    for definition in crate::components::devices::device_definitions() {
                        if ui
                            .button(definition.menu_name)
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
                        ui.label("Manage Windows:");
                        for dev in &self.attached_devices {
                            let mut dev_ref = dev.borrow_mut();
                            let name = dev_ref.get_name();
                            let mut open = dev_ref.get_window_open_state();
                            if ui.checkbox(&mut open, name).changed() {
                                dev_ref.set_window_open_state(open);
                            }
                        }
                    }
                })
                .response
                .on_hover_cursor(egui::CursorIcon::PointingHand);

                ui.menu_button("Memory", |ui| {
                    if ui
                        .checkbox(&mut self.show_memory_viewer, "Memory Viewer Window")
                        .changed()
                    {
                        ui.close();
                    }
                })
                .response
                .on_hover_cursor(egui::CursorIcon::PointingHand);

                #[cfg(target_arch = "wasm32")]
                ui.menu_button("Examples", |ui| {
                    if self.examples_list.is_empty() {
                        ui.label("No examples found");
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
                            self.load_example(ex, ctx.clone());
                            ui.close();
                        }
                    }
                })
                .response
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            });
            ui.separator();

            // Toolbar
            ui.horizontal(|ui| {
                if ui
                    .button("⟳ Load & Reset")
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    self.load_and_reset();
                }

                ui.separator();

                if ui
                    .add_enabled(self.last_error.is_none(), egui::Button::new("⏭ Step"))
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    self.is_running = false;
                    self.cpu.tick();
                }

                let run_label = if self.is_running {
                    "⏸ Stop"
                } else if self.cpu.get_pc() > 0 && !self.cpu.is_halted() {
                    "▶ Resume"
                } else {
                    "▶ Run"
                };
                if ui
                    .add_enabled(self.last_error.is_none(), egui::Button::new(run_label))
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    if !self.is_running && !self.cpu.is_halted() {
                        // Check if we are at a breakpoint, and if so, step once
                        let pc = self.cpu.get_pc();
                        let mut at_bp = false;
                        if let Some(tab) = self.tabs.get(self.active_tab) {
                            for &bp_line in &tab.breakpoints {
                                if let Some(&addr) = self.line_to_address.get(&bp_line)
                                    && addr == pc
                                {
                                    at_bp = true;
                                    break;
                                }
                            }
                        }
                        if at_bp {
                            self.cpu.tick();
                        }
                    }
                    self.is_running = !self.is_running;
                }

                ui.separator();

                if ui
                    .button("⚡ INT")
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    self.cpu.set_interrupt(true);
                }

                if ui
                    .button("⚡ NMI")
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    self.cpu.nmi();
                }

                ui.separator();

                if let Some(err) = &self.last_error {
                    ui.colored_label(egui::Color32::RED, format!("⚠ {}", err));
                } else if self.cpu.is_halted() {
                    ui.colored_label(egui::Color32::RED, "HALTED");
                    if ui
                        .button("Resume")
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        self.cpu.set_halted(false);
                        self.is_running = true;
                    }
                } else if self.is_running {
                    ui.label(egui::RichText::new("Running").color(egui::Color32::GREEN));
                } else {
                    ui.label(egui::RichText::new("Paused").color(egui::Color32::YELLOW));
                }
            });
            ui.separator();
            ui.vertical_centered(|ui| {
                ui.heading(format!("Loaded: {}", self.loaded_file_name));
            });
        });

        // Tab Bar Logic (pre-processing to find if we need to close tabs)
        // We'll define the closure later inside central panel?
        // No, let's process actions first. but Tab Bar is part of Central Panel UI.

        // Right Panel: Registers and Flags
        egui::Panel::right("right_panel")
            .resizable(true)
            .default_size(280.0)
            .show(ui, |ui| {
                ui.heading("Registers");
                ui.separator();

                let can_edit = !self.is_running || self.cpu.is_halted();

                egui::Grid::new("regs_grid")
                    .striped(true)
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| {
                        macro_rules! edit_16 {
                            ($name:expr, $getter:expr, $setter:expr) => {
                                ui.label($name);
                                let val = $getter;
                                let id = egui::Id::new($name);
                                let mut s = ui.data_mut(|d| {
                                    d.get_temp_mut_or_insert_with(id, || format!("{:04X}", val))
                                        .clone()
                                });
                                let resp = ui.add_enabled(
                                    can_edit,
                                    egui::TextEdit::singleline(&mut s)
                                        .desired_width(45.0)
                                        .font(egui::TextStyle::Monospace),
                                );
                                if resp.changed() {
                                    if let Ok(v) = u16::from_str_radix(s.trim(), 16) {
                                        $setter(v);
                                    }
                                    ui.data_mut(|d| d.insert_temp(id, s));
                                } else if !resp.has_focus() {
                                    ui.data_mut(|d| d.insert_temp(id, format!("{:04X}", val)));
                                }
                            };
                        }

                        macro_rules! edit_8 {
                            ($name:expr, $id:expr, $getter:expr, $setter:expr) => {
                                ui.label($name);
                                let val = $getter;
                                let id = egui::Id::new($id);
                                let mut s = ui.data_mut(|d| {
                                    d.get_temp_mut_or_insert_with(id, || format!("{:02X}", val))
                                        .clone()
                                });
                                let resp = ui.add_enabled(
                                    can_edit,
                                    egui::TextEdit::singleline(&mut s)
                                        .desired_width(25.0)
                                        .font(egui::TextStyle::Monospace),
                                );
                                if resp.changed() {
                                    if let Ok(v) = u8::from_str_radix(s.trim(), 16) {
                                        $setter(v);
                                    }
                                    ui.data_mut(|d| d.insert_temp(id, s));
                                } else if !resp.has_focus() {
                                    ui.data_mut(|d| d.insert_temp(id, format!("{:02X}", val)));
                                }
                            };
                        }

                        edit_16!("PC", self.cpu.get_pc(), |v| self.cpu.set_pc(v));
                        ui.label("");
                        ui.label("");
                        ui.end_row();
                        edit_16!("SP", self.cpu.get_sp(), |v| self.cpu.set_sp(v));
                        ui.label("");
                        ui.label("");
                        ui.end_row();
                        edit_16!("IX", self.cpu.get_ix(), |v| self.cpu.set_ix(v));
                        ui.label("");
                        ui.label("");
                        ui.end_row();
                        edit_16!("IY", self.cpu.get_iy(), |v| self.cpu.set_iy(v));
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
                            edit_8!(name, name, self.cpu.get_register(gpr), |v| self
                                .cpu
                                .set_register(gpr, v));
                            let shadow_name = format!("{}'", name);
                            edit_8!(
                                shadow_name.clone(),
                                shadow_name,
                                self.cpu.get_shadow_register(gpr),
                                |v| self.cpu.set_shadow_register(gpr, v)
                            );
                            ui.end_row();
                        }

                        ui.separator();
                        ui.separator();
                        ui.separator();
                        ui.separator();
                        ui.end_row();

                        let mono = |text: String| egui::RichText::new(text).monospace();
                        ui.label("IFF1");
                        ui.label(mono(format!("{}", self.cpu.get_iff1())));
                        ui.label("");
                        ui.label("");
                        ui.end_row();
                        ui.label("IFF2");
                        ui.label(mono(format!("{}", self.cpu.get_iff2())));
                        ui.label("");
                        ui.label("");
                        ui.end_row();
                        ui.label("IM");
                        ui.label(mono(format!("{}", self.cpu.get_interrupt_mode())));
                        ui.label("");
                        ui.label("");
                        ui.end_row();
                    });

                ui.add_space(20.0);
                ui.heading("Flags");
                ui.separator();

                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    for (flag, label) in [
                        (Flag::S, "S"),
                        (Flag::Z, "Z"),
                        (Flag::Y, "Y"),
                        (Flag::H, "H"),
                        (Flag::X, "X"),
                        (Flag::PV, "PV"),
                        (Flag::N, "N"),
                        (Flag::C, "C"),
                    ] {
                        let on = self.cpu.get_flag(flag);
                        let color = if on {
                            egui::Color32::from_rgb(0, 255, 0)
                        } else {
                            egui::Color32::from_rgb(60, 60, 60)
                        };
                        let text_color = if on {
                            egui::Color32::BLACK
                        } else {
                            egui::Color32::WHITE
                        };

                        // Drawn as a small badge
                        let text = egui::RichText::new(label).strong().color(text_color);
                        let frame_resp = egui::Frame::new()
                            .fill(color)
                            .corner_radius(4.0)
                            .inner_margin(4.0)
                            .show(ui, |ui| {
                                ui.label(text);
                            });

                        let interact_resp = ui.interact(
                            frame_resp.response.rect,
                            ui.id().with(label),
                            egui::Sense::click(),
                        );

                        if can_edit {
                            if interact_resp.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                            if interact_resp.clicked() {
                                self.cpu.set_flag(!on, flag);
                            }
                        }
                    }
                });
            });

        let mut sorted_symbols: Vec<_> = self.symbol_table.iter().collect();
        sorted_symbols.sort_by_key(|item| item.1.source_order);

        egui::Panel::right("vars_panel")
            .resizable(true)
            .default_size(280.0)
            .show(ui, |ui| {
                ui.heading("Variables");
                ui.separator();

                egui::ScrollArea::vertical()
                    .id_salt("vars_scroll")
                    .max_height(200.0)
                    .show(ui, |ui| {
                        egui::Grid::new("vars_grid")
                            .striped(true)
                            .spacing([20.0, 4.0])
                            .show(ui, |ui| {
                                ui.label(egui::RichText::new("Name").strong());
                                ui.label(egui::RichText::new("Addr").strong());
                                ui.label(egui::RichText::new("Value").strong());
                                ui.end_row();

                                for (name, symbol) in sorted_symbols.iter().
                                filter(|s|
                                    !matches!(s.1.kind, SymbolType::Label | SymbolType::Constant)) {

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

                                    // Context menu for format selection
                                    let label_resp = ui.label(display_name);
                                    label_resp.context_menu(|ui| {
                                        ui.label(egui::RichText::new("Display Format").strong());

                                        // "Default" implies removing the override
                                        if ui.button("Default").clicked() {
                                            self.symbol_display_prefs.remove(*name);
                                            ui.close();
                                        }

                                        // Always offer String and Array/Hex options regardless of defined type, as requested
                                        if ui.button("String").clicked() {
                                            self.symbol_display_prefs.insert(name.to_string(), SymbolDisplayFormat::String);
                                            ui.close();
                                        }
                                        if ui.button("Hex Bytes").clicked() {
                                            self.symbol_display_prefs.insert(name.to_string(), SymbolDisplayFormat::HexBytes);
                                            ui.close();
                                        }
                                        if ui.button("Hex Words").clicked() {
                                            self.symbol_display_prefs.insert(name.to_string(), SymbolDisplayFormat::HexWords);
                                            ui.close();
                                        }
                                    });

                                    ui.monospace(format!("0x{:04X}", addr));

                                    // Render based on format
                                    match format {
                                        SymbolDisplayFormat::Default => {
                                            match symbol.kind {
                                                SymbolType::Byte => {
                                                    let val = self.memory.borrow().read(addr);
                                                    ui.monospace(format!("0x{:02X}", val));
                                                }
                                                SymbolType::Word => {
                                                    let low = self.memory.borrow().read(addr);
                                                    let high = self.memory.borrow().read(addr.wrapping_add(1));
                                                    let val = (low as u16) | ((high as u16) << 8);
                                                    ui.monospace(format!("0x{:04X}", val));
                                                }
                                                SymbolType::String(len) => {
                                                    // Default String render
                                                    let mut bytes = Vec::new();
                                                    // Read at most 20 chars for preview, but check for null terminator
                                                    let display_len = len.min(20);
                                                    for i in 0..display_len {
                                                        let b = self.memory.borrow().read(addr.wrapping_add(i as u16));
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
                                                            let b = self.memory.borrow().read(addr.wrapping_add(i as u16));
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
                                                        bytes.push(self.memory.borrow().read(addr.wrapping_add(i as u16)));
                                                    }
                                                    let hex_str: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
                                                    if len > 8 {
                                                        // Read full content for tooltip
                                                        let mut full_bytes = Vec::new();
                                                        for i in 0..len {
                                                            full_bytes.push(self.memory.borrow().read(addr.wrapping_add(i as u16)));
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
                                                let b = self.memory.borrow().read(addr.wrapping_add(i as u16));
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
                                                     let b = self.memory.borrow().read(addr.wrapping_add(i as u16));
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
                                            let display_len = len.min(8).max(1); // At least 1 byte
                                            for i in 0..display_len {
                                                bytes.push(self.memory.borrow().read(addr.wrapping_add(i as u16)));
                                            }
                                            let hex_str: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
                                            if len > 8 {
                                                 let mut full_bytes = Vec::new();
                                                 for i in 0..len {
                                                     full_bytes.push(self.memory.borrow().read(addr.wrapping_add(i as u16)));
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
                                                 let val = self.memory.borrow().read(addr);
                                                 ui.monospace(format!("[00{:02X}:inv]", val))
                                                    .on_hover_text("Odd length, cannot display as words correctly");
                                            } else {
                                                let mut words = Vec::new();
                                                let display_count = word_count.min(4);
                                                for i in 0..display_count {
                                                    let offset = (i * 2) as u16;
                                                    let low = self.memory.borrow().read(addr.wrapping_add(offset));
                                                    let high = self.memory.borrow().read(addr.wrapping_add(offset + 1));
                                                    words.push(format!("{:04X}", (low as u16) | ((high as u16) << 8)));
                                                }
                                                if word_count > 4 {
                                                     let mut full_words = Vec::new();
                                                     for i in 0..word_count {
                                                         let offset = (i * 2) as u16;
                                                         let low = self.memory.borrow().read(addr.wrapping_add(offset));
                                                         let high = self.memory.borrow().read(addr.wrapping_add(offset + 1));
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
                                    ui.visuals().selection.bg_fill
                                } else {
                                    ui.visuals().faint_bg_color
                                };
                                let fg_color = if is_active {
                                    ui.visuals().selection.stroke.color
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

                                ui.add_space(2.0);
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
                ui.heading("Assembly Source");
                if self.is_assembly_stale {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("⚠ Out of date (re-assemble with Load & Reset)")
                            .color(egui::Color32::GOLD)
                            .small(),
                    );
                }
            });

            if self.active_tab < self.tabs.len() {
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
                                font(font_id.clone()).strong().color(egui::Color32::GRAY));

                            for i in 1..=num_lines {
                                let is_bp = current_tab.breakpoints.contains(&i);
                                let bg = if is_bp {
                                    egui::Color32::RED
                                } else {
                                    egui::Color32::TRANSPARENT
                                };
                                let text_color = if is_bp {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::GRAY
                                };

                                let label = egui::Label::new(
                                    egui::RichText::new(format!("{: >4}", i))
                                        .font(font_id.clone())
                                        .color(text_color)
                                        .background_color(bg),
                                )
                                .sense(egui::Sense::click());

                                if ui
                                    .add(label)
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
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
                                font(font_id.clone()).strong().color(egui::Color32::GRAY));

                                let addr_color = if self.is_assembly_stale {
                                    egui::Color32::from_rgb(180, 150, 80)
                                } else {
                                    egui::Color32::from_rgb(100, 180, 240)
                                };

                                for i in 1..=num_lines {
                                    let line_text = lines.get(i - 1).unwrap_or(&"");

                                    // Extract the actual instruction ignoring comments and labels
                                    let mut clean = line_text.split(';').next().unwrap_or("").trim();
                                    if let Some(idx) = clean.find(':') {
                                        clean = clean[idx + 1..].trim();
                                    }
                                    let upper = clean.to_uppercase();

                                    // Determine if this line actually emits code
                                    let emits_code = !clean.is_empty()
                                        && !upper.starts_with("ORG ") 
                                        && !upper.starts_with("EQU ") 
                                        && !upper.contains(" EQU ");

                                    let mut addr_str = "    ".to_string();

                                    if emits_code
                                        && let Some(&addr) = self.line_to_address.get(&i) {
                                            addr_str = format!("{:04X}", addr);
                                        }

                                    let label = egui::Label::new(
                                        egui::RichText::new(addr_str)
                                            .font(font_id.clone())
                                            .color(addr_color),
                                    );
                                    let resp = ui.add(label);

                                    if self.is_assembly_stale {
                                        resp.on_hover_text("Location counter is out of date. Click 'Load & Reset' to reassemble.");
                                    }
                                }
                            });

                            ui.add_space(4.0);

                            // Data Column
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 0.0;
                                ui.label(egui::RichText::new("Data").font(font_id.clone()).strong().color(egui::Color32::GRAY));

                                let data_color = if self.is_assembly_stale {
                                    egui::Color32::from_rgb(180, 150, 80)
                                } else {
                                    egui::Color32::from_rgb(150, 220, 150)
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
                                        && let Some(&addr) = self.line_to_address.get(&i) {

                                            // 1. Z80 length decoder to fetch the EXACT required bytes 
                                            let len = if upper.starts_with("DB ") || upper.starts_with("DEFB ") {
                                                (clean.split(',').count() as u16).min(8) // Cap visualizer to 8 bytes
                                            } else if upper.starts_with("DW ") || upper.starts_with("DEFW ") {
                                                (clean.split(',').count() as u16 * 2).min(8)
                                            } else if upper.starts_with("DS ") || upper.starts_with("DEFS ") {
                                                0
                                            } else {
                                                let mem = self.memory.borrow();
                                                let b0 = mem.read(addr);
                                                match b0 {
                                                    0xCB => 2,
                                                    0xDD | 0xFD => {
                                                        let b1 = mem.read(addr.wrapping_add(1));
                                                        match b1 {
                                                            0xCB => 4,
                                                            0x21 | 0x22 | 0x2A | 0x36 => 4,
                                                            0x34 | 0x35 => 3,
                                                            0x46 | 0x4E | 0x56 | 0x5E | 0x66 | 0x6E | 0x7E => 3,
                                                            0x70..=0x75 | 0x77 => 3,
                                                            0x86 | 0x8E | 0x96 | 0x9E | 0xA6 | 0xAE | 0xB6 | 0xBE => 3,
                                                            _ => 2,
                                                        }
                                                    }
                                                    0xED => {
                                                        let b1 = mem.read(addr.wrapping_add(1));
                                                        match b1 {
                                                            0x43 | 0x4B | 0x53 | 0x5B | 0x73 | 0x7B => 4,
                                                            _ => 2,
                                                        }
                                                    }
                                                    0xC2 | 0xC3 | 0xCA | 0xD2 | 0xDA | 0xE2 | 0xEA | 0xF2 | 0xFA | 0xCD | 0xCC | 0xC4 | 0xD4 | 0xDC | 0xE4 | 0xEC | 0xF4 | 0xFC => 3,
                                                    0x01 | 0x11 | 0x21 | 0x31 | 0x22 | 0x2A | 0x32 | 0x3A => 3,
                                                    0xC6 | 0xCE | 0xD6 | 0xDE | 0xE6 | 0xEE | 0xF6 | 0xFE => 2,
                                                    0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x3E => 2,
                                                    0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 => 2,
                                                    0xDB | 0xD3 => 2,
                                                    _ => 1,
                                                }
                                            };

                                            // 2. Fetch the bytes
                                            let mut bytes = Vec::new();
                                            let mem = self.memory.borrow();
                                            for offset in 0..len {
                                                bytes.push(mem.read(addr.wrapping_add(offset)));
                                            }

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
                            ui.label(egui::RichText::new("Code").font(font_id.clone()).strong().color(egui::Color32::GRAY));

                            let highlight_line = if !self.is_running || self.cpu.is_halted() {
                                let pc = self.cpu.get_pc();
                                if self.cpu.is_halted() {
                                    let mut best_match = None;
                                    let mut max_addr = -1i32;

                                    for (&addr, &line) in &self.address_to_line {
                                        if addr < pc
                                            && (addr as i32) > max_addr {
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
                            let editor = if is_binary_tab {
                                egui::TextEdit::multiline(&mut current_tab.code)
                                    .font(egui::TextStyle::Monospace)
                                    .layouter(&mut layouter)
                                    .code_editor()
                                    .desired_width(f32::INFINITY)
                                    .desired_rows(25)
                                    .lock_focus(true)
                                    .interactive(false)
                            } else {
                                egui::TextEdit::multiline(&mut current_tab.code)
                                    .font(egui::TextStyle::Monospace)
                                    .layouter(&mut layouter)
                                    .code_editor()
                                    .desired_width(f32::INFINITY)
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
            } else {
                ui.label("No files open");
            }
        });

        if let Some(action) = action {
            match action {
                HeaderAction::NewFile => self.new_file(frame.storage_mut()),
                HeaderAction::OpenFileDialog => self.open_file_dialog(frame.storage_mut()),
                HeaderAction::OpenFile(path) => self.open_file(path, frame.storage_mut()),
                HeaderAction::SaveFile => self.save_file(self.active_tab, frame.storage_mut()),
                HeaderAction::SaveFileAs => self.save_file_as(frame.storage_mut()),
                HeaderAction::AssembleBinary => self.assemble_binary_to_file(frame.storage_mut()),
                HeaderAction::LoadBinary => self.load_binary_dialog(frame.storage_mut()),
                HeaderAction::CloseTab(idx) => {
                    if self.tabs[idx].is_dirty {
                        self.pending_modal = Some(ModalType::CloseTab(idx));
                    } else {
                        self.close_tab(idx, frame.storage_mut());
                    }
                }
                HeaderAction::Quit => {
                    // This triggers on_close_event
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }

        // Handle Modal Dialogs
        if let Some(mut modal_type) = self.pending_modal.take() {
            let mut open = true;
            let mut should_close_modal = false;

            let title = match modal_type {
                ModalType::AddDevice(_, _) => "Add Device",
                _ => "Unsaved Changes",
            };

            // We use a centered window to simulate a modal
            egui::Window::new(title)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .open(&mut open)
                .show(ui, |ui| {
                    match &mut modal_type {
                        ModalType::AddDevice(definition, port_text) => {
                            ui.label("Enter Port Number (Hex):");
                            ui.text_edit_singleline(port_text);

                            ui.horizontal(|ui| {
                                if ui
                                    .button("Add")
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    // Parse port
                                    let port_res = u16::from_str_radix(port_text, 16);
                                    match port_res {
                                        Ok(port) => {
                                            let dev = (definition.create)(port);
                                            self.cpu.attach_device(dev.clone());
                                            self.attached_devices.push(dev);
                                            should_close_modal = true;
                                        }
                                        Err(_) => {
                                            ui.label("Invalid Hex Number");
                                        }
                                    }
                                }
                                if ui
                                    .button("Cancel")
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    should_close_modal = true;
                                }
                            });
                        }
                        ModalType::CloseTab(idx) => {
                            let idx = *idx; // Copy index
                            let name = self
                                .tabs
                                .get(idx)
                                .and_then(|t| t.path.as_ref())
                                .and_then(|p| p.file_name())
                                .and_then(|n| n.to_str())
                                .unwrap_or("Untitled");

                            ui.label(format!(
                                "Do you want to save the changes you made to {}?",
                                name
                            ));
                            ui.label("Your changes will be lost if you don't save them.");

                            ui.horizontal(|ui| {
                                if ui
                                    .button("Save")
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    self.save_file(idx, frame.storage_mut());

                                    // Check if save succeeded (is_dirty false)
                                    if !self.tabs[idx].is_dirty {
                                        self.close_tab(idx, frame.storage_mut());
                                        should_close_modal = true;
                                    }
                                }
                                if ui
                                    .button("Don't Save")
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    if idx < self.tabs.len() {
                                        self.tabs[idx].is_dirty = false; // Forced clear
                                        self.close_tab(idx, frame.storage_mut());
                                    }
                                    should_close_modal = true;
                                }
                                if ui
                                    .button("Cancel")
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    should_close_modal = true;
                                }
                            });
                        }
                        ModalType::Quit => {
                            ui.label("You have unsaved changes in your workspace.");
                            ui.label("Do you want to save all changes before quitting?");

                            ui.horizontal(|ui| {
                                if ui
                                    .button("Save All")
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    // Iterate and save all dirty tabs
                                    let prev_active = self.active_tab;
                                    for i in 0..self.tabs.len() {
                                        if self.tabs[i].is_dirty {
                                            self.save_file(i, frame.storage_mut());
                                        }
                                    }

                                    for i in 0..self.tabs.len() {
                                        if self.tabs[i].is_dirty {
                                            // Ensure tab is active if it might need a dialog (untitled)
                                            // The implementation of save_file checks `tab_idx == self.active_tab` for untitled files.
                                            // So we must switch.
                                            if self.tabs[i].path.is_none() {
                                                self.active_tab = i;
                                            }
                                            self.save_file(i, frame.storage_mut());
                                        }
                                    }
                                    self.active_tab = prev_active;

                                    // If all clear, close
                                    if !self.tabs.iter().any(|t| t.is_dirty) {
                                        // Force close by clearing modal first to prevent loop
                                        should_close_modal = true;
                                        // self.pending_modal is already None because we took it
                                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                    }
                                }
                                if ui
                                    .button("Quit Without Saving")
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    // Clear all dirty flags to bypass on_close_event check
                                    for tab in &mut self.tabs {
                                        tab.is_dirty = false;
                                    }
                                    should_close_modal = true;
                                    // self.pending_modal is already None
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                }
                                if ui
                                    .button("Cancel")
                                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    should_close_modal = true;
                                }
                            });
                        }
                    }
                });

            if !open || should_close_modal {
                self.pending_modal = None;
            } else {
                // Restore logic
                self.pending_modal = Some(modal_type);
            }
        }

        if self.is_running {
            let mut steps = 0;
            // Optimization: Clone breakpoints once before the tight loop.
            // This prevents looking up the tab and the hashset 10,000 times per frame.
            let active_breakpoints = self
                .tabs
                .get(self.active_tab)
                .map(|t| t.breakpoints.clone())
                .unwrap_or_default();

            // Execute in batches to keep UI responsive
            while steps < 10000 {
                if self.cpu.is_halted() {
                    // Even if halted, we must continue ticking to handle interrupts!
                    // tick() checks interrupts.
                    self.cpu.tick();

                    // If we just woke up, we continue normal execution in this loop.
                    if !self.cpu.is_halted() {
                        steps += 1;
                        continue;
                    }

                    // If still halted, we stop this batch to avoid spinning 10000 times on a halted cpu per frame.
                    // We need to request repaint though (handled below)
                    break;
                }

                // Check for breakpoint
                let pc = self.cpu.get_pc();
                if self
                    .line_to_address
                    .iter()
                    .any(|(line, &addr)| addr == pc && active_breakpoints.contains(line))
                {
                    self.is_running = false;
                    break;
                }

                self.cpu.tick();
                steps += 1;
            }
            if self.is_running {
                ctx.request_repaint();
                // If halted, we still want to repaint/re-check because UI interaction (keypad)
                // might change device state, which needs to be picked up by next tick.
            }
        }

        // Draw Memory Viewer Window
        let mut show_mem = self.show_memory_viewer;
        if show_mem {
            egui::Window::new("Memory Viewer")
                .open(&mut show_mem)
                .resizable(true)
                .default_width(500.0)
                .default_height(400.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Start Address (Hex):");
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.memory_viewer_start)
                                .desired_width(60.0)
                                .font(egui::TextStyle::Monospace),
                        );
                        if ui.button("Goto PC").clicked() {
                            self.memory_viewer_start = format!("{:04X}", self.cpu.get_pc());
                            response.request_focus();
                        }
                    });

                    ui.separator();

                    let start_addr =
                        u16::from_str_radix(self.memory_viewer_start.trim(), 16).unwrap_or(0);

                    // We render 16 rows of 16 bytes for a classic Hex Dump
                    let rows = 16;

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("mem_dump_grid")
                            .num_columns(3)
                            .spacing([20.0, 4.0])
                            .show(ui, |ui| {
                                for r in 0..rows {
                                    let row_addr = start_addr.wrapping_add((r * 16) as u16);

                                    // Print Address
                                    ui.monospace(format!("{:04X}", row_addr));

                                    // Print Hex Bytes
                                    let mut hex_str = String::new();
                                    let mut ascii_str = String::new();

                                    for c in 0..16 {
                                        let addr = row_addr.wrapping_add(c);
                                        let val = self.memory.borrow().read(addr);
                                        hex_str.push_str(&format!("{:02X} ", val));

                                        // Ascii representation
                                        if (32..=126).contains(&val) {
                                            ascii_str.push(val as char);
                                        } else {
                                            ascii_str.push('.');
                                        }
                                    }

                                    ui.monospace(hex_str);
                                    ui.monospace(ascii_str);
                                    ui.end_row();
                                }
                            });
                    });
                });
            self.show_memory_viewer = show_mem;
        }

        // Draw separate windows for devices
        for device in &self.attached_devices {
            device.borrow_mut().draw(&ctx);
        }
    }
}
