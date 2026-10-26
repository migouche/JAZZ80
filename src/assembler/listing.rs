use super::Token;

/// The encoded bytes of one source line as the listing's Data column shows
/// them. Only the first eight bytes are included.
pub(super) fn data_column(tokens: &[Token], bytes: &[u8]) -> String {
    // `ORG` emits padding bytes, but the listing does not show them, and lines
    // without a mnemonic or bytes have nothing to show.
    let Some(Token::Identifier(mnemonic)) = tokens.first() else {
        return String::new();
    };
    if mnemonic == "ORG" || bytes.is_empty() {
        return String::new();
    }

    let bytes = &bytes[..bytes.len().min(8)];

    match mnemonic.as_str() {
        "DB" | "DEFB" => bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(", "),
        "DW" | "DEFW" => bytes
            .chunks(2)
            .map(|chunk| {
                if chunk.len() == 2 {
                    // Little-endian format for display.
                    format!("{:02X}{:02X}", chunk[1], chunk[0])
                } else {
                    format!("{:02X}", chunk[0])
                }
            })
            .collect::<Vec<_>>()
            .join(", "),
        _ => {
            // Keep prefixes glued to the main opcode.
            let op_len = if bytes.len() > 1 && matches!(bytes[0], 0xCB | 0xDD | 0xED | 0xFD) {
                2
            } else {
                1
            };
            let op_hex = bytes[..op_len]
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join("");

            if bytes.len() <= op_len {
                return op_hex;
            }

            let arg_hex = bytes[op_len..]
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ");

            let comma = tokens.iter().position(|t| matches!(t, Token::Comma));
            let open = tokens.iter().position(|t| matches!(t, Token::OpenParen));
            let has_parens = tokens.iter().any(|t| matches!(t, Token::OpenParen))
                && tokens.iter().any(|t| matches!(t, Token::CloseParen));

            // The `open < comma` comparison distinguishes `LD (nn), A`
            // (parenthesised first operand) from `LD A, (nn)`.
            match (comma, has_parens) {
                (Some(c), true) if open.is_some_and(|o| o < c) => {
                    format!("{} ({})", op_hex, arg_hex)
                }
                (Some(_), true) => format!("{}, ({})", op_hex, arg_hex),
                (Some(_), false) => format!("{}, {}", op_hex, arg_hex),
                (None, true) => format!("{} ({})", op_hex, arg_hex),
                (None, false) => format!("{} {}", op_hex, arg_hex),
            }
        }
    }
}
