use super::{Operand, Token, keywords};

pub(super) fn parse_operands(tokens: &[Token]) -> Result<Vec<Operand>, String> {
    let mut operands = Vec::new();
    if tokens.is_empty() {
        return Ok(operands);
    }

    let mut i = 0;
    while i < tokens.len() {
        if i > 0 {
            if let Token::Comma = tokens[i] {
                i += 1;
            } else {
                return Err("Expected comma".to_string());
            }
        }
        if i >= tokens.len() {
            break;
        }

        match &tokens[i] {
            Token::Minus => {
                i += 1;
                if i >= tokens.len() {
                    return Err("Expected number after minus".to_string());
                }
                if let Token::Number(n) = tokens[i] {
                    let val = -(n as i16) as u16;
                    operands.push(Operand::Immediate(val));
                    i += 1;
                } else {
                    return Err("Expected number after minus".to_string());
                }
            }
            Token::Identifier(r) => {
                if is_condition(r) {
                    operands.push(Operand::Condition(r.clone()));
                } else if is_register(r) {
                    operands.push(Operand::Register(r.clone()));
                } else {
                    operands.push(Operand::Label(r.clone()));
                }
                i += 1;
            }
            Token::String(s) => {
                operands.push(Operand::StringLiteral(s.clone()));
                i += 1;
            }
            Token::Number(n) => {
                operands.push(Operand::Immediate(*n));
                i += 1;
            }
            Token::OpenParen => {
                i += 1;
                if i >= tokens.len() {
                    return Err("Unexpected end in parens".to_string());
                }

                match &tokens[i] {
                    Token::Number(n) => {
                        operands.push(Operand::IndirectImmediate(*n));
                        i += 1;
                    }
                    Token::Identifier(r) => {
                        if is_register(r) {
                            let reg = r.clone();
                            i += 1;
                            if i < tokens.len() && matches!(tokens[i], Token::Plus | Token::Minus) {
                                let sign = match tokens[i] {
                                    Token::Plus => 1,
                                    Token::Minus => -1,
                                    _ => 1,
                                };
                                i += 1;
                                if i >= tokens.len() {
                                    return Err("Expected offset".to_string());
                                }
                                if let Token::Number(offset) = tokens[i] {
                                    let final_offset = (offset as i16 * sign as i16) as i8;
                                    operands.push(Operand::IndirectIndex(reg, final_offset));
                                    i += 1;
                                } else {
                                    return Err("Expected number offset".to_string());
                                }
                            } else {
                                operands.push(Operand::IndirectRegister(reg));
                            }
                        } else {
                            operands.push(Operand::IndirectLabel(r.clone()));
                            i += 1;
                        }
                    }
                    _ => return Err("Invalid start of indirect operand".to_string()),
                }

                if i >= tokens.len() || tokens[i] != Token::CloseParen {
                    return Err("Expected )".to_string());
                }
                i += 1;
            }
            _ => return Err("Invalid operand".to_string()),
        }
    }
    Ok(operands)
}

pub(super) fn is_condition(s: &str) -> bool {
    matches!(s, "NZ" | "Z" | "NC" | "PO" | "PE" | "P" | "M")
}

pub(super) fn is_register(s: &str) -> bool {
    keywords::REGISTERS.contains(&s)
}
