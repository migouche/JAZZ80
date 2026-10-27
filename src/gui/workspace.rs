use crate::assembler::{AssemblyLine, AssemblyResult, Symbol};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::SymbolDisplayFormat;

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum EditorTabKind {
    #[default]
    Source,
    Binary,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct EditorTab {
    #[serde(default)]
    pub id: u64,
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

pub struct Workspace {
    pub tabs: Vec<EditorTab>,
    pub active_tab: usize,
    pub symbol_table: HashMap<String, Symbol>,
    pub address_to_line: HashMap<u16, usize>,
    pub line_to_address: HashMap<usize, u16>,
    pub assembly_lines: HashMap<usize, AssemblyLine>,
    pub symbol_display_prefs: HashMap<String, SymbolDisplayFormat>,
    pub recent_files: Vec<PathBuf>,
    pub show_location_counter: bool,
    pub is_assembly_stale: bool,
    pub program_loaded: bool,
    pub(crate) next_tab_id: u64,
}

impl Workspace {
    pub fn with_default_tab(default_code: String) -> Self {
        Self {
            tabs: vec![EditorTab {
                id: 1,
                path: None,
                code: default_code,
                kind: EditorTabKind::Source,
                is_dirty: false,
                breakpoints: HashSet::new(),
                binary_bytes: None,
            }],
            active_tab: 0,
            symbol_table: HashMap::new(),
            address_to_line: HashMap::new(),
            line_to_address: HashMap::new(),
            assembly_lines: HashMap::new(),
            symbol_display_prefs: HashMap::new(),
            recent_files: Vec::new(),
            show_location_counter: true,
            is_assembly_stale: false,
            program_loaded: false,
            next_tab_id: 2,
        }
    }

    pub fn alloc_tab_id(&mut self) -> u64 {
        let id = self.next_tab_id;
        self.next_tab_id = self.next_tab_id.wrapping_add(1);
        if id == 0 {
            let id = self.next_tab_id;
            self.next_tab_id = self.next_tab_id.wrapping_add(1);
            id
        } else {
            id
        }
    }

    pub fn clamp_active_tab(&mut self) {
        if self.tabs.is_empty() {
            self.active_tab = 0;
        } else if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        }
    }

    pub fn active(&self) -> Option<&EditorTab> {
        self.tabs.get(self.active_tab)
    }

    pub fn find_tab_by_path(&self, path: &Path) -> Option<usize> {
        self.tabs
            .iter()
            .position(|tab| tab.path.as_deref() == Some(path))
    }

    pub fn focus_tab(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() {
            return false;
        }
        if self.active_tab != index {
            self.active_tab = index;
            self.is_assembly_stale = true;
        }
        true
    }

    pub fn active_name(&self) -> String {
        self.active()
            .and_then(|tab| tab.path.as_ref())
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("Untitled")
            .to_string()
    }

    pub fn active_path_string(&self) -> Option<String> {
        self.active()
            .and_then(|tab| tab.path.as_ref())
            .map(|path| path.display().to_string())
    }

    pub fn add_source_tab(&mut self, path: Option<PathBuf>, code: String) -> usize {
        let id = self.alloc_tab_id();
        self.tabs.push(EditorTab {
            id,
            path,
            code,
            kind: EditorTabKind::Source,
            is_dirty: false,
            breakpoints: HashSet::new(),
            binary_bytes: None,
        });
        self.active_tab = self.tabs.len() - 1;
        self.active_tab
    }

    pub fn open_source(&mut self, path: PathBuf, code: String, default_code: &str) -> usize {
        let replace_active = self.active().is_some_and(|tab| {
            tab.path.is_none()
                && (tab.code.trim().is_empty() || tab.code == default_code)
                && !tab.is_dirty
        });
        let id = if replace_active {
            self.active().map(|tab| tab.id).unwrap_or_default()
        } else {
            self.alloc_tab_id()
        };
        let tab = EditorTab {
            id,
            path: Some(path),
            code,
            kind: EditorTabKind::Source,
            is_dirty: false,
            breakpoints: HashSet::new(),
            binary_bytes: None,
        };

        if replace_active {
            let index = self.active_tab;
            if let Some(slot) = self.tabs.get_mut(index) {
                *slot = tab;
            }
            index
        } else {
            self.tabs.push(tab);
            self.active_tab = self.tabs.len() - 1;
            self.active_tab
        }
    }

    pub fn open_binary(&mut self, path: PathBuf, bytes: Vec<u8>, hexdump: String) -> usize {
        if let Some(index) = self
            .tabs
            .iter()
            .position(|tab| tab.kind == EditorTabKind::Binary && tab.path.as_ref() == Some(&path))
        {
            self.active_tab = index;
        } else {
            let id = self.alloc_tab_id();
            self.tabs.push(EditorTab {
                id,
                path: Some(path),
                code: hexdump,
                kind: EditorTabKind::Binary,
                is_dirty: false,
                breakpoints: HashSet::new(),
                binary_bytes: Some(bytes),
            });
            self.active_tab = self.tabs.len() - 1;
        }
        self.active_tab
    }

    pub fn close(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() {
            return false;
        }
        let closed_active = index == self.active_tab;
        self.tabs.remove(index);

        if self.tabs.is_empty() {
            return true;
        }
        if closed_active {
            self.active_tab = index.min(self.tabs.len() - 1);
            self.is_assembly_stale = true;
        } else if index < self.active_tab {
            self.active_tab -= 1;
        }
        false
    }

    pub fn apply_assembly(&mut self, result: AssemblyResult) {
        self.symbol_table = result.symbols;
        self.address_to_line = result.address_to_line;
        self.line_to_address = result.line_to_address;
        self.assembly_lines = result.lines;
        self.is_assembly_stale = false;
        self.program_loaded = true;
    }

    pub fn clear_assembly(&mut self) {
        self.symbol_table.clear();
        self.address_to_line.clear();
        self.line_to_address.clear();
        self.assembly_lines.clear();
        self.is_assembly_stale = false;
    }

    pub fn add_recent_file(&mut self, path: PathBuf) {
        self.recent_files.retain(|recent| recent != &path);
        self.recent_files.insert(0, path);
        self.recent_files.truncate(10);
    }

    pub fn breakpoint_addresses(&self) -> HashSet<u16> {
        let mut addresses = HashSet::new();
        if let Some(tab) = self.active() {
            for &line in &tab.breakpoints {
                if let Some(&address) = self.line_to_address.get(&line) {
                    addresses.insert(address);
                }
            }
        }
        addresses
    }
}

#[cfg(test)]
mod tests {
    use super::{EditorTab, EditorTabKind, Workspace};
    use crate::assembler::assemble_with_metadata;
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};

    fn workspace() -> Workspace {
        Workspace::with_default_tab("DEFAULT".to_string())
    }

    #[test]
    fn alloc_tab_id_starts_after_the_default_tab_and_never_returns_zero() {
        let mut workspace = workspace();
        assert_eq!(workspace.alloc_tab_id(), 2);
        assert_eq!(workspace.alloc_tab_id(), 3);

        workspace.next_tab_id = 0;
        assert_eq!(workspace.alloc_tab_id(), 1);
        assert_eq!(workspace.next_tab_id, 2);

        workspace.next_tab_id = u64::MAX;
        assert_eq!(workspace.alloc_tab_id(), u64::MAX);
        assert_eq!(workspace.next_tab_id, 0);
        assert_eq!(workspace.alloc_tab_id(), 1);
    }

    #[test]
    fn clamp_active_tab_handles_empty_in_range_and_out_of_range() {
        let mut workspace = workspace();
        workspace.tabs.clear();
        workspace.active_tab = 4;
        workspace.clamp_active_tab();
        assert_eq!(workspace.active_tab, 0);

        workspace.tabs.push(EditorTab {
            id: 1,
            path: None,
            code: String::new(),
            kind: EditorTabKind::Source,
            is_dirty: false,
            breakpoints: HashSet::new(),
            binary_bytes: None,
        });
        workspace.tabs.push(EditorTab {
            id: 2,
            path: None,
            code: String::new(),
            kind: EditorTabKind::Source,
            is_dirty: false,
            breakpoints: HashSet::new(),
            binary_bytes: None,
        });

        workspace.active_tab = 1;
        workspace.clamp_active_tab();
        assert_eq!(workspace.active_tab, 1);

        workspace.active_tab = 9;
        workspace.clamp_active_tab();
        assert_eq!(workspace.active_tab, 1);
    }

    #[test]
    fn open_source_replaces_a_disposable_tab_in_place() {
        let mut workspace = workspace();
        let original_id = workspace.tabs[0].id;

        let index = workspace.open_source(PathBuf::from("main.asm"), "NOP".to_string(), "DEFAULT");

        assert_eq!(index, 0);
        assert_eq!(workspace.tabs.len(), 1);
        assert_eq!(workspace.tabs[0].id, original_id);
        assert_eq!(workspace.tabs[0].path, Some(PathBuf::from("main.asm")));
        assert_eq!(workspace.active_tab, 0);
    }

    #[test]
    fn open_source_adds_a_tab_when_the_active_one_is_not_disposable() {
        let mut workspace = workspace();
        workspace.open_source(PathBuf::from("main.asm"), "NOP".to_string(), "DEFAULT");

        let index =
            workspace.open_source(PathBuf::from("other.asm"), "HALT".to_string(), "DEFAULT");

        assert_eq!(index, 1);
        assert_eq!(workspace.tabs.len(), 2);
        assert_ne!(workspace.tabs[0].id, workspace.tabs[1].id);
        assert_eq!(workspace.active_tab, 1);
    }

    #[test]
    fn close_fixes_up_the_active_index() {
        let mut workspace = workspace();
        workspace.tabs.clear();
        workspace.next_tab_id = 1;
        for name in ["one.asm", "two.asm", "three.asm"] {
            workspace.add_source_tab(Some(PathBuf::from(name)), String::new());
        }
        workspace.active_tab = 1;

        // Closing the active tab selects the tab that took its place.
        assert!(!workspace.close(1));
        assert_eq!(workspace.tabs.len(), 2);
        assert_eq!(workspace.active_tab, 1);
        assert!(workspace.is_assembly_stale);

        // Closing a tab before the active one keeps the same document.
        workspace.active_tab = 1;
        assert!(!workspace.close(0));
        assert_eq!(workspace.active_tab, 0);

        // Closing a tab after the active one leaves it untouched.
        workspace.add_source_tab(Some(PathBuf::from("four.asm")), String::new());
        workspace.active_tab = 0;
        assert!(!workspace.close(1));
        assert_eq!(workspace.active_tab, 0);

        // The last close reports that the caller must add a replacement.
        assert!(workspace.close(0));
        assert!(workspace.tabs.is_empty());
    }

    #[test]
    fn apply_assembly_populates_the_maps_and_clears_the_stale_flag() {
        let mut workspace = workspace();
        workspace.is_assembly_stale = true;
        let result = assemble_with_metadata("        ORG 0000h\nSTART:  NOP\n        HALT\n")
            .expect("program assembles");

        workspace.apply_assembly(result);

        assert_eq!(workspace.symbol_table.len(), 1);
        assert!(!workspace.address_to_line.is_empty());
        assert!(!workspace.line_to_address.is_empty());
        assert!(!workspace.assembly_lines.is_empty());
        assert!(!workspace.is_assembly_stale);
        assert!(workspace.program_loaded);
    }

    #[test]
    fn find_tab_by_path_matches_only_the_exact_path() {
        let mut workspace = workspace();
        workspace.tabs[0].path = Some(PathBuf::from("one.asm"));
        workspace.add_source_tab(Some(PathBuf::from("two.asm")), String::new());

        assert_eq!(workspace.find_tab_by_path(Path::new("one.asm")), Some(0));
        assert_eq!(workspace.find_tab_by_path(Path::new("two.asm")), Some(1));
        assert_eq!(workspace.find_tab_by_path(Path::new("three.asm")), None);

        // An untitled tab has no path to match.
        workspace.tabs[0].path = None;
        assert_eq!(workspace.find_tab_by_path(Path::new("one.asm")), None);
    }

    #[test]
    fn focus_tab_reports_out_of_range_and_marks_stale_only_on_change() {
        let mut workspace = workspace();
        workspace.add_source_tab(Some(PathBuf::from("two.asm")), String::new());
        assert_eq!(workspace.active_tab, 1);
        assert!(!workspace.is_assembly_stale);

        assert!(!workspace.focus_tab(9));
        assert_eq!(workspace.active_tab, 1);
        assert!(!workspace.is_assembly_stale);

        assert!(workspace.focus_tab(0));
        assert_eq!(workspace.active_tab, 0);
        assert!(workspace.is_assembly_stale);

        workspace.is_assembly_stale = false;
        assert!(workspace.focus_tab(0));
        assert_eq!(workspace.active_tab, 0);
        assert!(!workspace.is_assembly_stale);
    }

    #[test]
    fn breakpoint_addresses_skip_lines_without_an_address() {
        let mut workspace = workspace();
        let result = assemble_with_metadata("        ORG 0000h\n        NOP\n        HALT\n")
            .expect("program assembles");
        workspace.apply_assembly(result);

        let mapped_line = *workspace.line_to_address.keys().next().unwrap();
        let unmapped_line = 2_000;
        workspace.tabs[0].breakpoints.insert(mapped_line);
        workspace.tabs[0].breakpoints.insert(unmapped_line);

        let addresses = workspace.breakpoint_addresses();

        assert_eq!(addresses.len(), 1);
        assert!(addresses.contains(&workspace.line_to_address[&mapped_line]));
    }
}
