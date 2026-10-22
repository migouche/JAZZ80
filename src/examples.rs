/// Bundled example programs for the web build.
///
/// Examples are web-only: the native build opens real files from disk instead.
/// These embedded copies back the virtual DOS "upload example" picker; the
/// Examples menu itself is driven by `z80 files/examples.json` at runtime.
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
