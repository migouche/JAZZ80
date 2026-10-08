/// Returns the bundled example files as (filename, contents) pairs.
/// Only available on wasm32 where the files are embedded.
#[cfg(target_arch = "wasm32")]
pub fn get_example_files() -> Option<Vec<(String, Vec<u8>)>> {
    let files: &[(&str, &[u8])] = &[
        ("program.bin", include_bytes!("../z80 files/program.bin")),
        ("fib.bin", include_bytes!("../z80 files/fib.bin")),
        ("os.bin", include_bytes!("../z80 files/os.bin")),
    ];
    Some(
        files
            .iter()
            .map(|(name, contents)| (name.to_string(), contents.to_vec()))
            .collect(),
    )
}
