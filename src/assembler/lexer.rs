use super::Token;

pub(super) fn tokenize(text: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = text.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ':' => {
                tokens.push(Token::Colon);
                chars.next();
            }
            '"' => {
                chars.next();
                let mut s = String::new();
                let mut terminated = false;
                while let Some(&c) = chars.peek() {
                    if c == '"' {
                        chars.next();
                        terminated = true;
                        break;
                    }
                    s.push(c);
                    chars.next();
                }
                if !terminated {
                    return Err("Unterminated string literal".to_string());
                }
                tokens.push(Token::String(s));
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            '(' => {
                tokens.push(Token::OpenParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::CloseParen);
                chars.next();
            }
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            c if c.is_whitespace() => {
                chars.next();
            }
            c if c.is_alphabetic() || c == '_' || c == '.' => {
                let mut ident = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' || c == '\'' || c == '.' {
                        ident.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let upper = ident.to_ascii_uppercase();

                let is_hex_candidate = upper.ends_with('H')
                    && upper.len() > 1
                    && upper[..upper.len() - 1]
                        .chars()
                        .all(|c| c.is_ascii_hexdigit());

                if is_hex_candidate {
                    if let Ok(val) = u16::from_str_radix(&upper[..upper.len() - 1], 16) {
                        tokens.push(Token::Number(val));
                    } else {
                        tokens.push(Token::Identifier(upper));
                    }
                } else {
                    tokens.push(Token::Identifier(upper));
                }
            }
            c if c.is_ascii_digit() => {
                let mut num_str = String::new();
                let mut is_hex = false;

                if c == '0' {
                    chars.next();
                    if let Some(&nc) = chars.peek() {
                        if nc == 'x' || nc == 'X' {
                            chars.next();
                            is_hex = true;
                        } else {
                            num_str.push('0');
                        }
                    } else {
                        num_str.push('0');
                    }
                }

                while let Some(&c) = chars.peek() {
                    if c.is_ascii_hexdigit() {
                        num_str.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }

                if !is_hex
                    && !num_str.is_empty()
                    && let Some(&nc) = chars.peek()
                    && (nc == 'H' || nc == 'h')
                {
                    is_hex = true;
                    chars.next();
                }

                let val = if is_hex {
                    u16::from_str_radix(&num_str, 16).map_err(|_| "Invalid hex number")?
                } else {
                    num_str.parse::<u16>().map_err(|_| "Invalid number")?
                };
                tokens.push(Token::Number(val));
            }
            '$' => {
                chars.next();
                let mut num_str = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_hexdigit() {
                        num_str.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let val = u16::from_str_radix(&num_str, 16).map_err(|_| "Invalid hex number")?;
                tokens.push(Token::Number(val));
            }
            _ => return Err(format!("Unexpected character: {}", c)),
        }
    }
    Ok(tokens)
}
