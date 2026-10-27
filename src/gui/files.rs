

use std::path::{Path, PathBuf};

use eframe::egui;

#[cfg(target_arch = "wasm32")]
use std::sync::mpsc::{Receiver, TryRecvError, channel};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileOrigin {
    Device,
    Example,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SaveMode {
    InPlace,
    Download,
}

pub(crate) enum FileEvent {
    SourceOpened {
        origin: FileOrigin,
        path: PathBuf,
        content: String,
    },
    BinaryOpened {
        origin: FileOrigin,
        path: PathBuf,
        bytes: Vec<u8>,
    },
    SourceSaved {
        tab_id: u64,
        path: PathBuf,
    },
    SourceDownloaded {
        tab_id: u64,
        name: String,
    },
    BinaryExported {
        target: String,
        path: Option<PathBuf>,
        len: usize,
    },
    ExportCancelled,
    #[cfg(target_arch = "wasm32")]
    ExamplesListed(Vec<String>),
    Failed(String),
}

#[derive(Default)]
pub(crate) struct FileService {
    #[cfg(target_arch = "wasm32")]
    source_receiver: Option<Receiver<FileEvent>>,

    #[cfg(target_arch = "wasm32")]
    binary_receiver: Option<Receiver<FileEvent>>,

    #[cfg(target_arch = "wasm32")]
    example_receiver: Option<Receiver<FileEvent>>,

    #[cfg(target_arch = "wasm32")]
    examples_receiver: Option<Receiver<FileEvent>>,
}

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

#[cfg(target_arch = "wasm32")]
fn drain(slot: &mut Option<Receiver<FileEvent>>) -> Vec<FileEvent> {
    let mut events = Vec::new();
    let Some(receiver) = slot.take() else {
        return events;
    };
    loop {
        match receiver.try_recv() {
            Ok(event) => events.push(event),
            Err(TryRecvError::Empty) => {
                *slot = Some(receiver);
                break;
            }
            Err(TryRecvError::Disconnected) => break,
        }
    }
    events
}

#[cfg(not(target_arch = "wasm32"))]
impl FileService {
    pub(crate) fn save_mode(&self) -> SaveMode {
        let _ = SaveMode::Download;
        let _ = FileOrigin::Example;
        SaveMode::InPlace
    }

    pub(crate) fn pick_source(&mut self, ctx: &egui::Context) -> Vec<FileEvent> {
        let _ = ctx;
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Assembly", &["asm", "z80"])
            .add_filter("All files", &["*"])
            .pick_file()
        {
            self.open_source(&path, ctx)
        } else {
            Vec::new()
        }
    }

    pub(crate) fn pick_binary(&mut self, ctx: &egui::Context) -> Vec<FileEvent> {
        let _ = ctx;
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Binary", &["bin", "rom", "com"])
            .add_filter("All files", &["*"])
            .pick_file()
        {
            match std::fs::read(&path) {
                Ok(bytes) => vec![FileEvent::BinaryOpened {
                    origin: FileOrigin::Device,
                    path,
                    bytes,
                }],
                Err(err) => vec![FileEvent::Failed(format!("Failed to load binary: {err}"))],
            }
        } else {
            Vec::new()
        }
    }

    pub(crate) fn open_source(&mut self, path: &Path, ctx: &egui::Context) -> Vec<FileEvent> {
        let _ = ctx;
        match std::fs::read_to_string(path) {
            Ok(content) => vec![FileEvent::SourceOpened {
                origin: FileOrigin::Device,
                path: path.to_path_buf(),
                content,
            }],
            Err(err) => vec![FileEvent::Failed(format!("Failed to open file: {err}"))],
        }
    }

    pub(crate) fn save_source(
        &mut self,
        tab_id: u64,
        path: Option<&Path>,
        name: &str,
        text: &str,
    ) -> Vec<FileEvent> {
        let _ = name;
        let _ = FileEvent::SourceDownloaded { tab_id, name: name.to_string() };
        #[cfg(target_arch = "wasm32")]
        let _ = FileEvent::ExamplesListed(Vec::new());
        let path = match path {
            Some(path) => path.to_path_buf(),
            None => match rfd::FileDialog::new()
                .add_filter("Assembly", &["asm", "z80"])
                .save_file()
            {
                Some(path) => path,
                None => return Vec::new(),
            },
        };

        match std::fs::write(&path, text) {
            Ok(()) => vec![FileEvent::SourceSaved { tab_id, path }],
            Err(err) => vec![FileEvent::Failed(format!("Failed to save file: {err}"))],
        }
    }

    pub(crate) fn export_binary(&mut self, name: &str, bytes: &[u8]) -> Vec<FileEvent> {
        let _ = name;
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Binary", &["bin", "rom"])
            .add_filter("All files", &["*"])
            .save_file()
        else {
            return vec![FileEvent::ExportCancelled];
        };

        match std::fs::write(&path, bytes) {
            Ok(()) => vec![FileEvent::BinaryExported {
                target: path.display().to_string(),
                path: Some(path),
                len: bytes.len(),
            }],
            Err(err) => vec![FileEvent::Failed(format!("Failed to export binary: {err}"))],
        }
    }

    pub(crate) fn start_examples(&mut self, ctx: &egui::Context) {
        let _ = ctx;
    }

    pub(crate) fn poll(&mut self) -> Vec<FileEvent> {
        Vec::new()
    }
}

#[cfg(target_arch = "wasm32")]
impl FileService {
    pub(crate) fn save_mode(&self) -> SaveMode {
        SaveMode::Download
    }

    pub(crate) fn pick_source(&mut self, ctx: &egui::Context) -> Vec<FileEvent> {
        let task = rfd::AsyncFileDialog::new()
            .add_filter("Assembly", &["asm", "z80"])
            .add_filter("All files", &["*"])
            .pick_file();

        let (sender, receiver) = channel();
        self.source_receiver = Some(receiver);

        let ctx = ctx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(file) = task.await {
                let name = file.file_name();
                let content = file.read().await;
                if let Ok(text) = String::from_utf8(content) {
                    sender
                        .send(FileEvent::SourceOpened {
                            origin: FileOrigin::Device,
                            path: PathBuf::from(name),
                            content: text,
                        })
                        .ok();
                }
            }
            ctx.request_repaint();
        });
        Vec::new()
    }

    pub(crate) fn pick_binary(&mut self, ctx: &egui::Context) -> Vec<FileEvent> {
        let task = rfd::AsyncFileDialog::new()
            .add_filter("Binary", &["bin", "rom", "com"])
            .add_filter("All files", &["*"])
            .pick_file();

        let (sender, receiver) = channel();
        self.binary_receiver = Some(receiver);

        let ctx = ctx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(file) = task.await {
                let name = file.file_name();
                let bytes = file.read().await;
                sender
                    .send(FileEvent::BinaryOpened {
                        origin: FileOrigin::Device,
                        path: PathBuf::from(name),
                        bytes,
                    })
                    .ok();
            }
            ctx.request_repaint();
        });
        Vec::new()
    }

    pub(crate) fn open_source(&mut self, path: &Path, ctx: &egui::Context) -> Vec<FileEvent> {
        let filename = path.to_string_lossy().into_owned();
        let Some(url) = z80_files_url(&filename) else {
            return vec![FileEvent::Failed(
                "Could not determine the examples URL".into(),
            )];
        };

        let path = path.to_path_buf();
        let (sender, receiver) = channel();
        self.example_receiver = Some(receiver);

        let ctx = ctx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let event = match reqwest::get(&url).await {
                Ok(response) if response.status().is_success() => match response.bytes().await {
                    Ok(bytes) => {
                        let bytes = bytes.to_vec();
                        if filename.ends_with(".bin") {
                            FileEvent::BinaryOpened {
                                origin: FileOrigin::Example,
                                path,
                                bytes,
                            }
                        } else {
                            match String::from_utf8(bytes) {
                                Ok(content) => FileEvent::SourceOpened {
                                    origin: FileOrigin::Example,
                                    path,
                                    content,
                                },
                                Err(_) => FileEvent::Failed(format!(
                                    "Example '{filename}' is not valid UTF-8"
                                )),
                            }
                        }
                    }
                    Err(err) => {
                        FileEvent::Failed(format!("Failed to read example '{filename}': {err}"))
                    }
                },
                Ok(response) => FileEvent::Failed(format!(
                    "Failed to load example '{filename}': HTTP {}",
                    response.status()
                )),
                Err(err) => {
                    FileEvent::Failed(format!("Failed to load example '{filename}': {err}"))
                }
            };
            sender.send(event).ok();
            ctx.request_repaint();
        });
        Vec::new()
    }

    pub(crate) fn save_source(
        &mut self,
        tab_id: u64,
        path: Option<&Path>,
        name: &str,
        text: &str,
    ) -> Vec<FileEvent> {
        let _ = path;
        download_file(name, text);
        vec![FileEvent::SourceDownloaded {
            tab_id,
            name: name.to_string(),
        }]
    }

    pub(crate) fn export_binary(&mut self, name: &str, bytes: &[u8]) -> Vec<FileEvent> {
        download_bytes(name, bytes);
        vec![FileEvent::BinaryExported {
            target: name.to_string(),
            path: None,
            len: bytes.len(),
        }]
    }

    pub(crate) fn start_examples(&mut self, ctx: &egui::Context) {
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
                            sender.send(FileEvent::ExamplesListed(examples)).ok();
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

    pub(crate) fn poll(&mut self) -> Vec<FileEvent> {
        let mut events = Vec::new();
        events.extend(drain(&mut self.source_receiver));
        events.extend(drain(&mut self.binary_receiver));
        events.extend(drain(&mut self.example_receiver));
        events.extend(drain(&mut self.examples_receiver));
        events
    }

    pub(crate) fn examples_pending(&self) -> bool {
        self.examples_receiver.is_some()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::{FileEvent, FileOrigin, FileService, SaveMode};
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after the epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("jazz80-{}-{nanos}-{name}", std::process::id()))
    }

    #[test]
    fn save_source_writes_in_place() {
        let path = temp_path("save.asm");
        let mut service = FileService::default();

        let events = service.save_source(7, Some(&path), "save.asm", "NOP\n");

        assert!(matches!(
            events.as_slice(),
            [FileEvent::SourceSaved { tab_id: 7, path: saved }] if saved == &path
        ));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "NOP\n");
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn save_source_reports_write_failures() {
        let path = temp_path("missing-dir").join("save.asm");
        let mut service = FileService::default();

        let events = service.save_source(1, Some(&path), "save.asm", "NOP\n");

        match events.as_slice() {
            [FileEvent::Failed(message)] => {
                assert!(message.starts_with("Failed to save file: "), "{message}");
            }
            other => panic!("expected one Failed event, got {} events", other.len()),
        }
    }

    #[test]
    fn open_source_reads_utf8_text() {
        let path = temp_path("open.asm");
        std::fs::write(&path, "HALT\n").unwrap();
        let mut service = FileService::default();

        let events = service.open_source(&path, &egui::Context::default());

        match events.as_slice() {
            [
                FileEvent::SourceOpened {
                    origin,
                    path: opened,
                    content,
                },
            ] => {
                assert_eq!(*origin, FileOrigin::Device);
                assert_eq!(opened, &path);
                assert_eq!(content, "HALT\n");
            }
            _ => panic!("expected one SourceOpened event"),
        }
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn open_source_reports_read_failures() {
        let path = temp_path("absent.asm");
        let mut service = FileService::default();

        let events = service.open_source(&path, &egui::Context::default());

        match events.as_slice() {
            [FileEvent::Failed(message)] => {
                assert!(message.starts_with("Failed to open file: "), "{message}");
            }
            _ => panic!("expected one Failed event"),
        }
    }

    #[test]
    fn native_saves_in_place() {
        assert_eq!(FileService::default().save_mode(), SaveMode::InPlace);
    }

    #[test]
    fn poll_is_empty_when_nothing_is_pending() {
        assert!(FileService::default().poll().is_empty());
    }
}
