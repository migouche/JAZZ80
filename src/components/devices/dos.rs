use crate::traits::IODevice;
use crate::ui_traits::DeviceWithUi;
use eframe::egui;
use std::collections::HashMap;
use std::collections::VecDeque;
#[cfg(target_arch = "wasm32")]
use std::sync::mpsc::{self, Receiver};

#[derive(Clone)]
pub(super) enum Inode {
    File(Vec<u8>),
    Directory,
}

pub struct VirtualDOS {
    port: u16,
    fs: HashMap<String, Inode>,
    cwd: String,
    gui_cwd: String,

    // Hardware communication buffers
    arg_buffer: String,
    out_queue: VecDeque<u8>,
    status: u8, // 0 = OK, 1 = Error, 2 = Data Ready
    is_open: bool,
    new_directory: String,
    #[cfg(target_arch = "wasm32")]
    upload_receiver: Option<Receiver<(String, Vec<u8>)>>,
    #[cfg(target_arch = "wasm32")]
    show_upload_menu: bool,
}

impl VirtualDOS {
    pub fn new(port: u16) -> Self {
        let mut fs = HashMap::new();
        fs.insert("/".to_string(), Inode::Directory); // Root always exists

        Self {
            port,
            fs,
            cwd: "/".to_string(),
            gui_cwd: "/".to_string(),
            arg_buffer: String::new(),
            out_queue: VecDeque::new(),
            status: 0,
            is_open: true,
            new_directory: String::new(),
            #[cfg(target_arch = "wasm32")]
            upload_receiver: None,
            #[cfg(target_arch = "wasm32")]
            show_upload_menu: false,
        }
    }

    // Helper to resolve paths like "FOO", "../BAR", or "/ROOT"
    fn resolve_path_from(&self, path: &str, cwd: &str) -> String {
        let path = path.trim();
        if path == "/" {
            return "/".to_string();
        }

        let mut parts: Vec<&str> = if path.starts_with('/') {
            Vec::new()
        } else {
            cwd.split('/').filter(|s| !s.is_empty()).collect()
        };

        for p in path.split('/') {
            match p {
                "" | "." => continue,
                ".." => {
                    parts.pop();
                }
                _ => parts.push(p),
            }
        }

        if parts.is_empty() {
            return "/".to_string();
        }
        format!("/{}", parts.join("/"))
    }

    fn resolve_path(&self, path: &str) -> String {
        self.resolve_path_from(path, &self.cwd)
    }

    fn child_path(&self, name: &str) -> String {
        self.resolve_path_from(name.trim_matches('/'), &self.gui_cwd)
    }

    fn create_directory(&mut self, name: &str) {
        let name = name.trim();
        if !name.is_empty() && !name.contains('/') {
            self.fs.insert(self.child_path(name), Inode::Directory);
        }
    }

    fn direct_children(&self) -> Vec<(String, bool, usize)> {
        let prefix = if self.gui_cwd == "/" {
            "/".to_string()
        } else {
            format!("{}/", self.gui_cwd)
        };
        let mut children = Vec::new();

        for (path, inode) in &self.fs {
            if path.starts_with(&prefix) {
                let remainder = &path[prefix.len()..];
                if !remainder.is_empty() && !remainder.contains('/') {
                    let size = match inode {
                        Inode::Directory => 0,
                        Inode::File(contents) => contents.len(),
                    };
                    children.push((
                        remainder.to_string(),
                        matches!(inode, Inode::Directory),
                        size,
                    ));
                }
            }
        }
        children.sort_by_key(|left| left.0.to_lowercase());
        children
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn upload_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file()
            && let Ok(contents) = std::fs::read(&path)
        {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("upload");
            self.fs.insert(self.child_path(name), Inode::File(contents));
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn upload_file(&mut self) {
        if self.upload_receiver.is_some() {
            return;
        }

        let task = rfd::AsyncFileDialog::new().pick_file();
        let (sender, receiver) = mpsc::channel();
        self.upload_receiver = Some(receiver);
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(file) = task.await {
                let name = file.file_name();
                let contents = file.read().await;
                let _ = sender.send((name, contents));
            }
        });
    }

    #[cfg(target_arch = "wasm32")]
    fn finish_upload(&mut self) {
        let result = self
            .upload_receiver
            .as_ref()
            .and_then(|receiver| receiver.try_recv().ok());
        if let Some((name, contents)) = result {
            self.fs
                .insert(self.child_path(&name), Inode::File(contents));
            self.upload_receiver = None;
        }
    }

    fn write_argument(&mut self, data: u8) -> bool {
        self.arg_buffer.push(data as char);
        true
    }

    fn clear_command_buffers(&mut self) {
        self.arg_buffer.clear();
        self.out_queue.clear();
        self.status = 0;
    }

    fn make_directory_from_argument(&mut self) {
        let target = self.resolve_path(&self.arg_buffer);
        self.fs.insert(target, Inode::Directory);
        self.status = 0;
    }

    fn change_directory_from_argument(&mut self) {
        let target = self.resolve_path(&self.arg_buffer);
        if let Some(Inode::Directory) = self.fs.get(&target) {
            self.cwd = target;
            self.status = 0;
        } else {
            self.status = 1;
        }
    }

    fn list_directory(&mut self) {
        let mut listing = String::new();
        let prefix = if self.cwd == "/" {
            "/".to_string()
        } else {
            format!("{}/", self.cwd)
        };

        for (path, inode) in &self.fs {
            if path.starts_with(&prefix) {
                let remainder = &path[prefix.len()..];
                // Only list direct children, not subdirectories
                if !remainder.contains('/') && !remainder.is_empty() {
                    let label = match inode {
                        Inode::Directory => "[DIR] ",
                        Inode::File(_) => "[FILE]",
                    };
                    listing.push_str(&format!("{} {}\r\n", label, remainder));
                }
            }
        }
        if listing.is_empty() {
            listing.push_str("Empty\r\n");
        }

        self.out_queue.extend(listing.bytes());
        self.status = if self.out_queue.is_empty() { 0 } else { 2 };
    }

    fn read_file_from_argument(&mut self) {
        let target = self.resolve_path(&self.arg_buffer);
        match self.fs.get(&target) {
            Some(Inode::File(contents)) => {
                self.out_queue.extend(contents.iter().cloned());
                self.status = if self.out_queue.is_empty() { 0 } else { 2 };
            }
            _ => {
                self.status = 1;
            }
        }
    }

    fn dispatch_command(&mut self, command: u8) {
        match command {
            0 => self.clear_command_buffers(),
            1 => self.make_directory_from_argument(),
            2 => self.change_directory_from_argument(),
            3 => self.list_directory(),
            4 => self.read_file_from_argument(),
            _ => {}
        }
    }
}

impl IODevice for VirtualDOS {
    fn read_in(&mut self, port: u16) -> Option<u8> {
        let p = port & 0xFF;
        let base = self.port & 0xFF;

        if p == base {
            // Read output data (e.g., from an LS command)
            if let Some(b) = self.out_queue.pop_front() {
                if self.out_queue.is_empty() {
                    self.status = 0;
                } // Clear Data Ready flag
                Some(b)
            } else {
                Some(0)
            }
        } else if p == base.wrapping_add(2) {
            // Read Status (0=OK, 1=Error, 2=Data Ready)
            Some(self.status)
        } else {
            None
        }
    }

    fn write_out(&mut self, port: u16, data: u8) -> bool {
        let p = port & 0xFF;
        let base = self.port & 0xFF;

        if p == base {
            self.write_argument(data)
        } else if p == base.wrapping_add(1) {
            self.dispatch_command(data);
            true
        } else {
            false
        }
    }
}

impl DeviceWithUi for VirtualDOS {
    fn get_name(&self) -> String {
        format!("DOS Controller (Port 0x{:02X})", self.port & 0xFF)
    }
    fn get_window_open_state(&self) -> bool {
        self.is_open
    }
    fn set_window_open_state(&mut self, open: bool) {
        self.is_open = open;
    }

    fn draw(&mut self, ctx: &egui::Context) {
        #[cfg(target_arch = "wasm32")]
        {
            self.finish_upload();
            if self.upload_receiver.is_some() {
                ctx.request_repaint();
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        let mut upload_requested = false;
        #[cfg(target_arch = "wasm32")]
        let upload_requested = false;
        self.show_window(ctx, Some(420.0), &mut |this, ui| {
            ui.horizontal(|ui| {
                if ui.button("Up").clicked() && this.gui_cwd != "/" {
                    this.gui_cwd = this
                        .gui_cwd
                        .rsplit_once('/')
                        .map(|(parent, _)| if parent.is_empty() { "/" } else { parent })
                        .unwrap_or("/")
                        .to_string();
                }
                ui.monospace(this.gui_cwd.to_string());
                #[cfg(target_arch = "wasm32")]
                {
                    if ui.button("Upload file...").clicked() {
                        this.show_upload_menu = true;
                    }
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if ui.button("Upload file...").clicked() {
                        upload_requested = true;
                    }
                }
            });
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(220.0)
                .show(ui, |ui| {
                    for (name, is_directory, size) in this.direct_children() {
                        ui.horizontal(|ui| {
                            if is_directory {
                                if ui.button(format!("[DIR] {}", name)).clicked() {
                                    this.gui_cwd = this.child_path(&name);
                                }
                            } else {
                                ui.label(format!("[FILE] {} ({} bytes)", name, size));
                            }
                        });
                    }
                });

            ui.separator();
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut this.new_directory);
                if ui.button("Create folder").clicked() {
                    let name = std::mem::take(&mut this.new_directory);
                    this.create_directory(&name);
                }
            });
            #[cfg(target_arch = "wasm32")]
            if this.upload_receiver.is_some() {
                ui.label("Waiting for file...");
            }
        });
        if upload_requested {
            self.upload_file();
        }

        #[cfg(target_arch = "wasm32")]
        if self.show_upload_menu {
            let mut menu_open = true;
            egui::Window::new("Upload file")
                .open(&mut menu_open)
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Choose a bundled example or upload from your computer:");
                    ui.separator();

                    for (name, contents) in crate::gui::BUNDLED_EXAMPLES {
                        if ui.button(*name).clicked() {
                            let path = self.child_path(name);
                            self.fs.insert(path, Inode::File(contents.to_vec()));
                            self.show_upload_menu = false;
                        }
                    }

                    ui.separator();
                    if ui.button("Upload from computer...").clicked() {
                        self.show_upload_menu = false;
                        self.upload_file();
                    }
                    if ui.button("Cancel").clicked() {
                        self.show_upload_menu = false;
                    }
                });
            if !menu_open {
                self.show_upload_menu = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::IODevice;

    fn write_argument(dos: &mut VirtualDOS, argument: &str) {
        for byte in argument.bytes() {
            assert!(dos.write_out(0x20, byte));
        }
    }

    #[test]
    fn virtual_dos_routes_commands_and_preserves_handled_results() {
        let mut dos = VirtualDOS::new(0x20);

        assert!(!dos.write_out(0x21 + 0x10, 0));
        write_argument(&mut dos, "docs");
        assert!(dos.write_out(0x21, 1));
        assert_eq!(dos.read_in(0x22), Some(0));

        assert!(dos.write_out(0x21, 0xFF));
        assert_eq!(dos.read_in(0x22), Some(0));

        assert!(dos.write_out(0x21, 0));
        write_argument(&mut dos, "docs");
        assert!(dos.write_out(0x21, 2));
        assert_eq!(dos.cwd, "/docs");

        assert!(dos.write_out(0x21, 0));
        write_argument(&mut dos, "missing");
        assert!(dos.write_out(0x21, 2));
        assert_eq!(dos.read_in(0x22), Some(1));
    }

    #[test]
    fn virtual_dos_file_command_buffers_data_and_clears_status_after_reads() {
        let mut dos = VirtualDOS::new(0x20);
        dos.fs
            .insert("/readme".to_string(), Inode::File(vec![b'O', b'K']));

        write_argument(&mut dos, "readme");
        assert!(dos.write_out(0x21, 4));
        assert_eq!(dos.read_in(0x22), Some(2));
        assert_eq!(dos.read_in(0x20), Some(b'O'));
        assert_eq!(dos.read_in(0x22), Some(2));
        assert_eq!(dos.read_in(0x20), Some(b'K'));
        assert_eq!(dos.read_in(0x22), Some(0));

        assert!(dos.write_out(0x21, 0));
        assert_eq!(dos.read_in(0x22), Some(0));
    }
}
