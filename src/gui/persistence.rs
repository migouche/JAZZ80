use super::workspace::{EditorTab, Workspace};
use super::{DeviceSpec, SymbolDisplayFormat};
use std::collections::HashMap;
use std::path::PathBuf;

pub const WORKSPACE_VERSION: u32 = 1;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct PersistedWorkspace {
    pub version: u32,
    pub tabs: Vec<EditorTab>,
    pub active_tab: usize,
    pub symbol_display_prefs: HashMap<String, SymbolDisplayFormat>,
    pub attached_device_specs: Vec<DeviceSpec>,
    pub recent_files: Vec<PathBuf>,
    pub show_location_counter: bool,
}

impl Default for PersistedWorkspace {
    fn default() -> Self {
        Self {
            version: WORKSPACE_VERSION,
            tabs: Vec::new(),
            active_tab: 0,
            symbol_display_prefs: HashMap::new(),
            attached_device_specs: Vec::new(),
            recent_files: Vec::new(),
            show_location_counter: true,
        }
    }
}

impl PersistedWorkspace {
    pub fn from_parts(workspace: &Workspace, device_specs: Vec<DeviceSpec>) -> Self {
        Self {
            version: WORKSPACE_VERSION,
            tabs: workspace.tabs.clone(),
            active_tab: workspace.active_tab,
            symbol_display_prefs: workspace.symbol_display_prefs.clone(),
            attached_device_specs: device_specs,
            recent_files: workspace.recent_files.clone(),
            show_location_counter: workspace.show_location_counter,
        }
    }

    pub fn into_parts(self) -> (Workspace, Vec<DeviceSpec>) {
        let mut workspace = Workspace::with_default_tab(String::new());
        workspace.tabs = self.tabs;
        workspace.active_tab = self.active_tab;
        workspace.symbol_display_prefs = self.symbol_display_prefs;
        workspace.recent_files = self.recent_files;
        workspace.show_location_counter = self.show_location_counter;
        workspace.clamp_active_tab();
        (workspace, self.attached_device_specs)
    }
}

#[cfg(test)]
mod tests {
    use super::{PersistedWorkspace, WORKSPACE_VERSION};
    use crate::gui::DeviceSpec;
    use crate::gui::formatting::binary_hexdump;
    use crate::gui::workspace::{EditorTab, EditorTabKind, Workspace};
    use eframe::Storage;
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    #[derive(Default)]
    struct MemoryStorage {
        values: HashMap<String, String>,
    }

    impl Storage for MemoryStorage {
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

    fn source_tab(id: u64, code: &str, is_dirty: bool) -> EditorTab {
        EditorTab {
            id,
            path: None,
            code: code.to_string(),
            kind: EditorTabKind::Source,
            is_dirty,
            breakpoints: HashSet::new(),
            binary_bytes: None,
        }
    }

    #[test]
    fn round_trip_preserves_tabs_specs_and_clamps_the_active_index() {
        let bytes = vec![0x00, 0x76];
        let binary = EditorTab {
            id: 2,
            path: Some(PathBuf::from("program.bin")),
            code: binary_hexdump(&bytes),
            kind: EditorTabKind::Binary,
            is_dirty: false,
            breakpoints: HashSet::new(),
            // A restored binary tab only has its hexdump until it is reloaded.
            binary_bytes: None,
        };

        let mut workspace = Workspace::with_default_tab(String::new());
        workspace.tabs = vec![source_tab(1, "NOP", true), binary];
        workspace.active_tab = 7;
        workspace.recent_files = vec![PathBuf::from("program.asm")];
        workspace.show_location_counter = false;

        let specs = vec![DeviceSpec {
            menu_name: "Test device".to_string(),
            port: 0x20,
        }];
        let persisted = PersistedWorkspace::from_parts(&workspace, specs);

        let (restored, restored_specs) = persisted.into_parts();

        assert_eq!(restored.tabs.len(), 2);
        assert_eq!(restored.active_tab, 1);
        assert!(restored.tabs[0].is_dirty);
        assert_eq!(restored.tabs[1].kind, EditorTabKind::Binary);
        assert!(restored.tabs[1].binary_bytes.is_none());
        assert_eq!(restored.recent_files, vec![PathBuf::from("program.asm")]);
        assert!(!restored.show_location_counter);
        assert_eq!(restored_specs.len(), 1);
        assert_eq!(restored_specs[0].menu_name, "Test device");
        assert_eq!(restored_specs[0].port, 0x20);
    }

    #[test]
    fn legacy_ron_without_version_defaults_the_version_and_keeps_known_fields() {
        #[derive(serde::Serialize)]
        struct LegacyWorkspace {
            tabs: Vec<EditorTab>,
            active_tab: usize,
            recent_files: Vec<PathBuf>,
        }

        let mut storage = MemoryStorage::default();
        eframe::set_value(
            &mut storage,
            "workspace",
            &LegacyWorkspace {
                tabs: vec![source_tab(1, "NOP", false)],
                active_tab: 0,
                recent_files: vec![PathBuf::from("old.asm")],
            },
        );

        let loaded: PersistedWorkspace =
            eframe::get_value(&storage, "workspace").expect("legacy workspace deserializes");

        assert_eq!(loaded.version, WORKSPACE_VERSION);
        assert_eq!(loaded.tabs.len(), 1);
        assert_eq!(loaded.tabs[0].code, "NOP");
        assert_eq!(loaded.recent_files, vec![PathBuf::from("old.asm")]);
        assert!(loaded.show_location_counter);
    }

    #[test]
    fn an_object_missing_every_field_uses_the_defaults() {
        let mut storage = MemoryStorage::default();
        storage.set_string("workspace", "()".to_string());

        let loaded: PersistedWorkspace =
            eframe::get_value(&storage, "workspace").expect("empty workspace deserializes");

        assert_eq!(loaded.version, WORKSPACE_VERSION);
        assert!(loaded.tabs.is_empty());
        assert_eq!(loaded.active_tab, 0);
        assert!(loaded.show_location_counter);
    }
}
