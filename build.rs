use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=z80 files/examples.json");

    // Bundled examples are a web-only feature: the native build opens real
    // files from disk instead.
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() != Ok("wasm32") {
        return;
    }

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("bundled_examples.inc");
    let examples_json = Path::new("z80 files/examples.json");

    if !examples_json.exists() {
        fs::write(
            &dest_path,
            "pub(crate) const BUNDLED_EXAMPLES: &[(&str, &[u8])] = &[];\n",
        )
        .unwrap();
        return;
    }

    let json = fs::read_to_string(examples_json).expect("Failed to read examples.json");
    let names: Vec<String> = serde_json::from_str(&json).expect("Invalid examples.json");

    let mut out = String::new();
    out.push_str("pub(crate) const BUNDLED_EXAMPLES: &[(&str, &[u8])] = &[\n");
    for n in &names {
        out.push_str(&format!(
            "    (\"{}\", include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/z80 files/{}\"))),\n",
            n, n
        ));
    }
    out.push_str("];\n");

    fs::write(&dest_path, out).expect("Failed to write generated examples");
}
