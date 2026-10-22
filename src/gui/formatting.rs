pub(super) fn binary_hexdump(bytes: &[u8]) -> String {
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

/// Recover the raw bytes from a hexdump produced by [`binary_hexdump`].
///
/// Binary tabs are persisted as their hexdump text (the raw bytes are
/// `#[serde(skip)]`), so this is what makes a binary tab survive a restart.
pub(super) fn parse_hexdump(text: &str) -> Option<Vec<u8>> {
    // An empty dump is a valid zero-byte image, not a parse failure.
    if text.trim().is_empty() {
        return Some(Vec::new());
    }

    let mut bytes = Vec::new();
    let mut saw_line = false;
    for line in text.lines() {
        let Some(colon) = line.find(':') else {
            continue;
        };
        let rest = &line[colon + 1..];
        // Each dump line is ` ` + the hex field padded to 47 columns + ` ` +
        // the ASCII preview. Only the hex field counts.
        let hex_field = rest.get(1..48).unwrap_or(rest);
        for token in hex_field.split_whitespace() {
            if token.len() != 2 || !token.chars().all(|c| c.is_ascii_hexdigit()) {
                return None;
            }
            bytes.push(u8::from_str_radix(token, 16).ok()?);
        }
        saw_line = true;
    }
    saw_line.then_some(bytes)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;
