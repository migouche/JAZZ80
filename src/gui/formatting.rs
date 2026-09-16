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

#[cfg(test)]
mod tests;
