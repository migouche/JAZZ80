/// Bundled example programs, embedded in the binary for both native and web.
///
/// Keeping this available on every target means the Examples menu is not a
/// web-only feature: a fresh native install has something to run without
/// hunting for a folder first.
pub fn get_example_files() -> Vec<(String, Vec<u8>)> {
    let asm_files: &[(&str, &[u8])] = &[
        ("fib.asm", include_bytes!("../z80 files/fib.asm")),
        ("im0.asm", include_bytes!("../z80 files/im0.asm")),
        ("im1.asm", include_bytes!("../z80 files/im1.asm")),
        ("im2.asm", include_bytes!("../z80 files/im2.asm")),
        ("lcd_test.asm", include_bytes!("../z80 files/lcd_test.asm")),
        ("nmi.asm", include_bytes!("../z80 files/nmi.asm")),
        ("os.asm", include_bytes!("../z80 files/os.asm")),
        ("program.asm", include_bytes!("../z80 files/program.asm")),
    ];

    let bin_files: &[(&str, &[u8])] = &[
        ("fib.bin", include_bytes!("../z80 files/fib.bin")),
        ("os.bin", include_bytes!("../z80 files/os.bin")),
        ("program.bin", include_bytes!("../z80 files/program.bin")),
    ];

    asm_files
        .iter()
        .chain(bin_files.iter())
        .map(|(name, contents)| (name.to_string(), contents.to_vec()))
        .collect()
}

/// The bundled examples offered in the Examples menu.
pub fn example_names() -> Vec<String> {
    get_example_files()
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}
