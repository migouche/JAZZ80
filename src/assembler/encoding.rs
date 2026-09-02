use std::collections::HashMap;

use crate::cpu::ALUOperation;
use crate::cpu::alu::rot::RotOperation;

use super::{
    Operand, get_condition_code, get_r_code, get_rp_code, resolve_immediate, resolve_indirect,
};

pub(super) fn encode_alu_uni(base: u8, ops: &[Operand]) -> Result<Vec<u8>, String> {
    if ops.len() != 1 {
        return Err("Needs 1 Op".to_string());
    }
    match &ops[0] {
        Operand::Register(r) => {
            if let Some(rc) = get_r_code(r) {
                return Ok(vec![base | (rc << 3)]);
            }
            if let Some(rpc) = get_rp_code(r) {
                let val = if base == 0x04 { 0x03 } else { 0x0B };
                return Ok(vec![val | (rpc << 4)]);
            }
            if r == "IX" {
                let val = if base == 0x04 { 0x23 } else { 0x2B };
                return Ok(vec![0xDD, val]);
            }
            if r == "IY" {
                let val = if base == 0x04 { 0x23 } else { 0x2B };
                return Ok(vec![0xFD, val]);
            }
            Err("Invalid register for INC/DEC".to_string())
        }
        Operand::IndirectRegister(r) if r == "HL" => Ok(vec![base | (6 << 3)]),
        Operand::IndirectIndex(idx, d) => {
            let prefix = if idx == "IX" { 0xDD } else { 0xFD };
            Ok(vec![prefix, base | (6 << 3), *d as u8])
        }
        _ => Err("Invalid INC/DEC".to_string()),
    }
}

pub(super) fn encode_add(
    ops: &[Operand],
    labels: &HashMap<String, u16>,
    dry: bool,
) -> Result<Vec<u8>, String> {
    if ops.len() == 2
        && let Operand::Register(dest) = &ops[0]
    {
        if dest == "HL"
            && let Operand::Register(src) = &ops[1]
            && let Some(rpc) = get_rp_code(src)
        {
            return Ok(vec![0x09 | (rpc << 4)]);
        }
        if dest == "IX" || dest == "IY" {
            let prefix = if dest == "IX" { 0xDD } else { 0xFD };
            if let Operand::Register(src) = &ops[1] {
                let pp = match src.as_str() {
                    "BC" => 0,
                    "DE" => 1,
                    "IX" | "IY" => 2,
                    "SP" => 3,
                    _ => return Err("Invalid operand for ADD index".to_string()),
                };
                return Ok(vec![prefix, 0x09 | (pp << 4)]);
            }
        }
    }
    encode_alu_op(ALUOperation::ADD, ops, labels, dry)
}

pub(super) fn encode_alu_op(
    op: ALUOperation,
    ops: &[Operand],
    labels: &HashMap<String, u16>,
    dry: bool,
) -> Result<Vec<u8>, String> {
    let op_code = op as u8;
    let base_r = 0x80 | (op_code << 3);
    let base_n = 0xC6 | (op_code << 3);
    encode_alu_bin(base_r, base_n, ops, labels, dry)
}

fn encode_alu_bin(
    base_r: u8,
    base_n: u8,
    ops: &[Operand],
    labels: &HashMap<String, u16>,
    dry: bool,
) -> Result<Vec<u8>, String> {
    if ops.len() != 1 {
        if ops.len() == 2
            && let Operand::Register(r) = &ops[0]
            && r == "A"
        {
            return encode_alu_bin(base_r, base_n, &ops[1..], labels, dry);
        }
        return Err("ALU ops count error".to_string());
    }
    match &ops[0] {
        Operand::Register(r) => {
            if let Some(rc) = get_r_code(r) {
                Ok(vec![base_r | rc])
            } else if r == "IXH" {
                Ok(vec![0xDD, base_r | 4])
            } else if r == "IXL" {
                Ok(vec![0xDD, base_r | 5])
            } else {
                Err("Invalid ALU register".to_string())
            }
        }
        op if matches!(op, Operand::Immediate(_) | Operand::Label(_)) => {
            let n = resolve_immediate(op, labels, dry)?;
            Ok(vec![base_n, n as u8])
        }
        Operand::IndirectRegister(r) if r == "HL" => Ok(vec![base_r | 6]),
        Operand::IndirectIndex(idx, d) => {
            let prefix = if idx == "IX" { 0xDD } else { 0xFD };
            Ok(vec![prefix, base_r | 6, *d as u8])
        }
        _ => Err("Invalid ALU operand".to_string()),
    }
}

pub(super) fn encode_rot_op(op: RotOperation, ops: &[Operand]) -> Result<Vec<u8>, String> {
    let base = (op as u8) << 3;
    encode_rot_shift(base, ops)
}

fn encode_rot_shift(base: u8, ops: &[Operand]) -> Result<Vec<u8>, String> {
    if ops.len() == 1 {
        match &ops[0] {
            Operand::Register(r) => {
                if let Some(rc) = get_r_code(r) {
                    Ok(vec![0xCB, base | rc])
                } else {
                    Err("Invalid register for Rotate/Shift".to_string())
                }
            }
            Operand::IndirectRegister(r) if r == "HL" => Ok(vec![0xCB, base | 6]),
            Operand::IndirectIndex(idx, d) => {
                let prefix = if idx == "IX" { 0xDD } else { 0xFD };
                Ok(vec![prefix, 0xCB, *d as u8, base | 6])
            }
            _ => Err("Invalid operand for Rotate/Shift".to_string()),
        }
    } else if ops.len() == 2 {
        if let (Operand::IndirectIndex(idx, d), Operand::Register(r)) = (&ops[0], &ops[1]) {
            if let Some(rc) = get_r_code(r) {
                let prefix = if idx == "IX" { 0xDD } else { 0xFD };
                Ok(vec![prefix, 0xCB, *d as u8, base | rc])
            } else {
                Err("Invalid register2 for Rotate/Shift".to_string())
            }
        } else {
            Err("Invalid operands for Rotate/Shift (2 ops)".to_string())
        }
    } else {
        Err("Rotate/Shift expects 1 or 2 operands".to_string())
    }
}

pub(super) fn encode_ex(ops: &[Operand]) -> Result<Vec<u8>, String> {
    if ops.len() != 2 {
        return Err("EX 2 ops".to_string());
    }
    match (&ops[0], &ops[1]) {
        (Operand::Register(r1), Operand::Register(r2)) => {
            if r1 == "DE" && r2 == "HL" {
                return Ok(vec![0xEB]);
            }
            if r1 == "AF" && (r2 == "AF'" || r2 == "AF") {
                return Ok(vec![0x08]);
            }
            Err("Invalid EX".to_string())
        }
        (Operand::IndirectRegister(r1), Operand::Register(r2)) if r1 == "SP" => {
            if r2 == "HL" {
                return Ok(vec![0xE3]);
            }
            if r2 == "IX" {
                return Ok(vec![0xDD, 0xE3]);
            }
            if r2 == "IY" {
                return Ok(vec![0xFD, 0xE3]);
            }
            Err("Invalid EX (SP)".to_string())
        }
        _ => Err("Invalid EX combo".to_string()),
    }
}

pub(super) fn encode_bit_op(base: u8, ops: &[Operand]) -> Result<Vec<u8>, String> {
    if ops.len() < 2 || ops.len() > 3 {
        return Err("Bit op expects 2 or 3 operands".to_string());
    }

    let b = match &ops[0] {
        Operand::Immediate(n) => *n,
        _ => return Err("Bit index must be a number".to_string()),
    };

    if b > 7 {
        return Err("Bit index must be 0-7".to_string());
    }

    let opcode_base = base + ((b as u8) << 3);

    match &ops[1] {
        Operand::Register(r) => {
            if ops.len() != 2 {
                return Err("Too many operands for Register target".to_string());
            }
            if let Some(rc) = get_r_code(r) {
                Ok(vec![0xCB, opcode_base | rc])
            } else {
                Err("Invalid register".to_string())
            }
        }
        Operand::IndirectRegister(reg) if reg == "HL" => {
            if ops.len() != 2 {
                return Err("Too many operands for (HL)".to_string());
            }
            Ok(vec![0xCB, opcode_base | 6])
        }
        Operand::IndirectIndex(idx, d) => {
            let prefix = if idx == "IX" { 0xDD } else { 0xFD };
            if ops.len() == 2 {
                Ok(vec![prefix, 0xCB, *d as u8, opcode_base | 6])
            } else {
                if base == 0x40 {
                    return Err("BIT does not support 3 operands".to_string());
                }
                if let Operand::Register(r) = &ops[2] {
                    if let Some(rc) = get_r_code(r) {
                        Ok(vec![prefix, 0xCB, *d as u8, opcode_base | rc])
                    } else {
                        Err("Invalid register 2".to_string())
                    }
                } else {
                    Err("Third operand must be register".to_string())
                }
            }
        }
        _ => Err("Invalid target operand".to_string()),
    }
}

pub(super) fn encode_jp(
    ops: &[Operand],
    labels: &HashMap<String, u16>,
    dry: bool,
) -> Result<Vec<u8>, String> {
    match ops.len() {
        1 => match &ops[0] {
            op if matches!(
                op,
                Operand::Immediate(_)
                    | Operand::Label(_)
                    | Operand::IndirectImmediate(_)
                    | Operand::IndirectLabel(_)
            ) =>
            {
                let nn = if matches!(
                    op,
                    Operand::IndirectImmediate(_) | Operand::IndirectLabel(_)
                ) {
                    resolve_indirect(op, labels, dry)?
                } else {
                    resolve_immediate(op, labels, dry)?
                };
                Ok(vec![0xC3, (nn & 0xFF) as u8, (nn >> 8) as u8])
            }
            Operand::IndirectRegister(r) | Operand::Register(r) if r == "HL" => Ok(vec![0xE9]),
            Operand::IndirectRegister(r) | Operand::Register(r) if r == "IX" => {
                Ok(vec![0xDD, 0xE9])
            }
            Operand::IndirectRegister(r) | Operand::Register(r) if r == "IY" => {
                Ok(vec![0xFD, 0xE9])
            }
            _ => Err("Invalid JP target".to_string()),
        },
        2 => {
            let cond = match &ops[0] {
                Operand::Condition(c) => Some(c.as_str()),
                Operand::Register(r) if r == "C" => Some("C"),
                _ => None,
            };
            if let Some(c) = cond
                && let Some(cc) = get_condition_code(c)
            {
                let nn = if matches!(
                    &ops[1],
                    Operand::IndirectImmediate(_) | Operand::IndirectLabel(_)
                ) {
                    resolve_indirect(&ops[1], labels, dry)?
                } else {
                    resolve_immediate(&ops[1], labels, dry)?
                };
                return Ok(vec![0xC2 | (cc << 3), (nn & 0xFF) as u8, (nn >> 8) as u8]);
            }
            Err("Invalid JP condition/target".to_string())
        }
        _ => Err("Invalid JP args".to_string()),
    }
}

pub(super) fn encode_jr(
    ops: &[Operand],
    pc: u16,
    labels: &HashMap<String, u16>,
    dry: bool,
) -> Result<Vec<u8>, String> {
    // JR d / JR C, d
    // Opcode size is 2 bytes. Offset is relative to PC+2.
    // Target = (PC + 2) + offset (signed i8)
    // Offset = Target - (PC + 2)
    match ops.len() {
        1 => {
            // JR d
            let target = resolve_immediate(&ops[0], labels, dry)?;
            let offset_val = (target as i32) - ((pc as i32) + 2);
            if !dry && !(-128..=127).contains(&offset_val) {
                return Err("JR offset out of range".to_string());
            }
            Ok(vec![0x18, offset_val as i8 as u8])
        }
        2 => {
            let cond = match &ops[0] {
                Operand::Condition(c) => Some(c.as_str()),
                Operand::Register(r) if r == "C" => Some("C"),
                _ => None,
            };
            if let Some(c) = cond
                && let Some(cc) = get_condition_code(c)
            {
                if cc > 3 {
                    return Err("Invalid JR condition".to_string());
                }
                let target = resolve_immediate(&ops[1], labels, dry)?;
                let offset_val = (target as i32) - ((pc as i32) + 2);
                if !dry && !(-128..=127).contains(&offset_val) {
                    return Err("JR offset out of range".to_string());
                }
                return Ok(vec![0x20 | (cc << 3), offset_val as i8 as u8]);
            }
            Err("Invalid JR args".to_string())
        }
        _ => Err("Invalid JR args".to_string()),
    }
}

pub(super) fn encode_call(
    ops: &[Operand],
    labels: &HashMap<String, u16>,
    dry: bool,
) -> Result<Vec<u8>, String> {
    match ops.len() {
        1 => {
            let nn = match &ops[0] {
                op if matches!(
                    op,
                    Operand::Immediate(_)
                        | Operand::IndirectImmediate(_)
                        | Operand::Label(_)
                        | Operand::IndirectLabel(_)
                ) =>
                {
                    if matches!(
                        op,
                        Operand::IndirectImmediate(_) | Operand::IndirectLabel(_)
                    ) {
                        resolve_indirect(op, labels, dry)?
                    } else {
                        resolve_immediate(op, labels, dry)?
                    }
                }
                _ => return Err("CALL needs address".to_string()),
            };
            Ok(vec![0xCD, (nn & 0xFF) as u8, (nn >> 8) as u8])
        }
        2 => {
            let cond = match &ops[0] {
                Operand::Condition(c) => Some(c.as_str()),
                Operand::Register(r) if r == "C" => Some("C"),
                _ => None,
            };
            if let Some(c) = cond {
                if let Some(cc) = get_condition_code(c) {
                    let nn = match &ops[1] {
                        op if matches!(
                            op,
                            Operand::Immediate(_)
                                | Operand::IndirectImmediate(_)
                                | Operand::Label(_)
                                | Operand::IndirectLabel(_)
                        ) =>
                        {
                            if matches!(
                                op,
                                Operand::IndirectImmediate(_) | Operand::IndirectLabel(_)
                            ) {
                                resolve_indirect(op, labels, dry)?
                            } else {
                                resolve_immediate(op, labels, dry)?
                            }
                        }
                        _ => return Err("CALL needs address".to_string()),
                    };
                    Ok(vec![0xC4 | (cc << 3), (nn & 0xFF) as u8, (nn >> 8) as u8])
                } else {
                    Err("Bad cond".to_string())
                }
            } else {
                Err("Call format error".to_string())
            }
        }
        _ => Err("CALL args".to_string()),
    }
}
