use super::{decode_opcode, flags, AddressingMode, Flag, GPR, PrefixAddressing, RegisterPair, SystemRegister, Z80A};
use super::alu::{self, rot};

pub(super) fn decode_ed(cpu: &mut Z80A, opcode: u8) {
    test_log!(cpu, "decode_ed");
    let (x, y, z, p, q) = decode_opcode(opcode);

    /*
    Lots of NONI's here. Some of these are instructions for the Z180, will not be implemented.
    (at least for now)
     */

    match x {
        0 | 3 => test_log!(cpu, "NONI"),
        1 => match z {
            0 => {
                if y == 6 {
                    test_log!(cpu, "IN (C)");
                    let c = cpu.get_register(GPR::C);
                    let b = cpu.get_register(GPR::B);
                    let port = ((b as u16) << 8) | (c as u16);
                    let val = cpu.read_io(port);
                    cpu.set_flag((val & flags::SIGN) != 0, Flag::S);
                    cpu.set_flag(val == 0, Flag::Z);
                    cpu.set_flag(false, Flag::H);
                    cpu.set_flag(false, Flag::N);
                    cpu.set_flag(val.count_ones().is_multiple_of(2), Flag::PV);
                } else if y < 8 {
                    test_log!(cpu, "IN r[y], (C)");
                    let reg = cpu.table_r(y);
                    let c = cpu.get_register(GPR::C);
                    let b = cpu.get_register(GPR::B);
                    let port = ((b as u16) << 8) | (c as u16);
                    let val = cpu.read_io(port);
                    cpu.write_8(reg.into(), val);
                    cpu.set_flag((val & flags::SIGN) != 0, Flag::S);
                    cpu.set_flag(val == 0, Flag::Z);
                    cpu.set_flag(false, Flag::H);
                    cpu.set_flag(false, Flag::N);
                    cpu.set_flag(val.count_ones().is_multiple_of(2), Flag::PV);
                } else { unreachable!("Invalid y value") }
            }
            1 => {
                if y == 6 {
                    test_log!(cpu, "OUT (C), 0");
                    let c = cpu.get_register(GPR::C);
                    let b = cpu.get_register(GPR::B);
                    cpu.write_io(((b as u16) << 8) | c as u16, 0);
                } else if y < 8 {
                    test_log!(cpu, "OUT (C), r[y]");
                    let reg = cpu.table_r(y);
                    let value = cpu.read_8(reg.into());
                    let c = cpu.get_register(GPR::C);
                    let b = cpu.get_register(GPR::B);
                    cpu.write_io(((b as u16) << 8) | c as u16, value);
                } else { unreachable!("Invalid y value") }
            }
            2 => {
                if !q {
                    test_log!(cpu, "SBC HL, rp[p]");
                    let dest = AddressingMode::RegisterPair(RegisterPair::HL);
                    let rp = cpu.table_rp(p);
                    cpu.sub_16_op(dest, AddressingMode::RegisterPair(rp), true)
                } else {
                    test_log!(cpu, "ADC HL, rp[p]");
                    let dest = AddressingMode::RegisterPair(RegisterPair::HL);
                    let rp = cpu.table_rp(p);
                    cpu.add_16_op(dest, AddressingMode::RegisterPair(rp), true)
                }
            }
            3 => {
                if !q {
                    test_log!(cpu, "LD (nn), rp[p]");
                    let addr = cpu.fetch_word();
                    let src = cpu.table_rp(p);
                    cpu.ld_16(AddressingMode::Absolute(addr), AddressingMode::RegisterPair(src))
                } else {
                    test_log!(cpu, "LD rp[p], (nn)");
                    let addr = cpu.fetch_word();
                    let dest = cpu.table_rp(p);
                    cpu.ld_16(AddressingMode::RegisterPair(dest), AddressingMode::Absolute(addr))
                }
            }
            4 => {
                if y == 0 {
                    test_log!(cpu, "NEG");
                    let a = cpu.get_register(GPR::A);
                    let (result, alu_flags) = alu::alu_op::sub(0, a, false);
                    cpu.set_register(GPR::A, result);
                    cpu.af_registers[cpu.active_af].f = alu_flags | flags::ADD_SUB;
                } else { test_log!(cpu, "NONI"); }
            }
            5 => {
                if y == 0 {
                    test_log!(cpu, "RETN");
                    cpu.iff1 = cpu.iff2;
                    cpu.pc = cpu.pop();
                } else if y == 1 {
                    test_log!(cpu, "RETI");
                    cpu.pc = cpu.pop();
                } else if y < 8 { test_log!(cpu, "NONI"); }
                else { unreachable!("Invalid y value") }
            }
            6 => match y {
                0 => { test_log!(cpu, "IM 0"); cpu.interrupt_mode = 0; }
                2 => { test_log!(cpu, "IM 1"); cpu.interrupt_mode = 1; }
                3 => { test_log!(cpu, "IM 2"); cpu.interrupt_mode = 2; }
                _ => test_log!(cpu, "NONI"),
            },
            7 => match y {
                0 => { test_log!(cpu, "LD I, A"); cpu.ld(AddressingMode::System(SystemRegister::I), AddressingMode::Register(GPR::A)); }
                1 => { test_log!(cpu, "LD R, A"); cpu.ld(AddressingMode::System(SystemRegister::R), AddressingMode::Register(GPR::A)); }
                2 | 3 => {
                    if y == 2 { test_log!(cpu, "LD A, I"); cpu.ld(AddressingMode::Register(GPR::A), AddressingMode::System(SystemRegister::I)); }
                    else { test_log!(cpu, "LD A, R"); cpu.ld(AddressingMode::Register(GPR::A), AddressingMode::System(SystemRegister::R)); }
                    let a = cpu.get_register(GPR::A);
                    cpu.set_flag((a & flags::SIGN) != 0, Flag::S);
                    cpu.set_flag(a == 0, Flag::Z);
                    cpu.set_flag(false, Flag::H);
                    cpu.set_flag(false, Flag::N);
                    cpu.set_flag(cpu.iff2, Flag::PV);
                }
                4 | 5 => {
                    if y == 4 { test_log!(cpu, "RRD"); } else { test_log!(cpu, "RLD"); }
                    let a = cpu.get_register(GPR::A);
                    let al = a & 0x0F;
                    let hl_addr = cpu.get_register_pair(RegisterPair::HL);
                    let value = cpu.memory.borrow().read(hl_addr);
                    let mh = value & 0xF0;
                    let ml = value & 0x0F;
                    let (new_a, new_hl) = if y == 4 { (a & 0xF0 | ml, al << 4 | mh >> 4) } else { (a & 0xF0 | mh >> 4, ml << 4 | al) };
                    let s = (new_a & flags::SIGN) != 0;
                    let z = new_a == 0;
                    let parity = new_a.count_ones().is_multiple_of(2);
                    let x = (new_a & flags::X) != 0;
                    let y_flag = (new_a & flags::Y) != 0;
                    cpu.set_register(GPR::A, new_a);
                    cpu.memory.borrow_mut().write(hl_addr, new_hl);
                    cpu.set_flag(s, Flag::S); cpu.set_flag(z, Flag::Z); cpu.set_flag(parity, Flag::PV);
                    cpu.set_flag(false, Flag::H); cpu.set_flag(false, Flag::N);
                    cpu.set_flag(x, Flag::X); cpu.set_flag(y_flag, Flag::Y);
                }
                6 | 7 => test_log!(cpu, "NONI"),
                _ => unreachable!("Invalid y value"),
            },
            _ => unreachable!("Invalid z value"),
        },
        2 => {
            if z <= 3 && y >= 4 {
                test_log!(cpu, "bli[y, z]");
                let inst = cpu.table_bli(y, z);
                cpu.execute_block_instruction(inst);
            } else { test_log!(cpu, "NONI"); }
        }
        _ => unreachable!("Invalid x value"),
    }
}

pub(super) fn decode_unprefixed(cpu: &mut Z80A, opcode: u8, addressing: PrefixAddressing) {
    match addressing {
        PrefixAddressing::HL => test_log!(cpu, "decode_unprefixed"),
        PrefixAddressing::IX => test_log!(cpu, "decode_dd"),
        PrefixAddressing::IY => test_log!(cpu, "decode_fd"),
    }

    let (x, y, z, p, q) = decode_opcode(opcode);
    match x {
        0 => match z {
            0 => match y {
                0 => test_log!(cpu, "NOP"),
                1 => {
                    test_log!(cpu, "EX AF, AF'");
                    cpu.ex_af_af_prime();
                }
                2 => {
                    test_log!(cpu, "DJNZ d");
                    let d = cpu.fetch_displacement();
                    let b = cpu.get_register(GPR::B).wrapping_sub(1);
                    cpu.set_register(GPR::B, b);
                    if b != 0 {
                        cpu.pc = cpu.pc.wrapping_add(d as i16 as u16);
                    }
                }
                3 => {
                    test_log!(cpu, "JR d");
                    let d = cpu.fetch_displacement();
                    cpu.pc = cpu.pc.wrapping_add(d as i16 as u16);
                }
                4..=7 => {
                    test_log!(cpu, "JR cc[y-4], d");
                    let condition = cpu.table_cc(y - 4);
                    let d = cpu.fetch_displacement();
                    if cpu.evaluate_condition(condition) {
                        cpu.pc = cpu.pc.wrapping_add(d as i16 as u16);
                    }
                }
                _ => unreachable!("Invalid y value"),
            },
            1 => {
                if q {
                    test_log!(cpu, "ADD HL/IX/IY, rp[p]");
                    let dest = cpu.transform_register(
                        AddressingMode::RegisterPair(RegisterPair::HL), addressing,
                    );
                    let rp = cpu.table_rp(p);
                    let src = cpu.transform_register(AddressingMode::RegisterPair(rp), addressing);
                    cpu.add_16_op(dest, src, false);
                } else {
                    test_log!(cpu, "LD rp[p], nn");
                    let nn = cpu.fetch_word();
                    let rp = cpu.table_rp(p);
                    let src = cpu.transform_register(AddressingMode::RegisterPair(rp), addressing);
                    cpu.ld_16(src, AddressingMode::ImmediateExtended(nn));
                }
            }
            2 => match (q, p) {
                (false, 0) => {
                    test_log!(cpu, "LD (BC), A");
                    cpu.ld(AddressingMode::RegisterIndirect(RegisterPair::BC), AddressingMode::Register(GPR::A))
                }
                (false, 1) => {
                    test_log!(cpu, "LD (DE), A");
                    cpu.ld(AddressingMode::RegisterIndirect(RegisterPair::DE), AddressingMode::Register(GPR::A))
                }
                (false, 2) => {
                    test_log!(cpu, "LD (nn), HL/IX/IY");
                    let addr = cpu.fetch_word();
                    let transformed = cpu.transform_register(AddressingMode::RegisterPair(RegisterPair::HL), addressing);
                    cpu.ld_16(AddressingMode::Absolute(addr), transformed)
                }
                (false, 3) => {
                    test_log!(cpu, "LD (nn), A");
                    let addr = cpu.fetch_word();
                    cpu.ld(AddressingMode::Absolute(addr), AddressingMode::Register(GPR::A))
                }
                (true, 0) => {
                    test_log!(cpu, "LD A, (BC)");
                    cpu.ld(AddressingMode::Register(GPR::A), AddressingMode::RegisterIndirect(RegisterPair::BC))
                }
                (true, 1) => {
                    test_log!(cpu, "LD A, (DE)");
                    cpu.ld(AddressingMode::Register(GPR::A), AddressingMode::RegisterIndirect(RegisterPair::DE))
                }
                (true, 2) => {
                    test_log!(cpu, "LD HL/IX/IY, (nn)");
                    let addr = cpu.fetch_word();
                    let transformed = cpu.transform_register(AddressingMode::RegisterPair(RegisterPair::HL), addressing);
                    cpu.ld_16(transformed, AddressingMode::Absolute(addr))
                }
                (true, 3) => {
                    test_log!(cpu, "LD A, (nn)");
                    let addr = cpu.fetch_word();
                    cpu.ld(AddressingMode::Register(GPR::A), AddressingMode::Absolute(addr))
                }
                _ => unreachable!("Invalid q, p values"),
            },
            3 => {
                if !q {
                    test_log!(cpu, "INC rp[p]");
                    let rp = cpu.table_rp(p);
                    let dest = cpu.transform_register(AddressingMode::RegisterPair(rp), addressing);
                    cpu.inc_16_op(dest);
                } else {
                    test_log!(cpu, "DEC rp[p]");
                    let rp = cpu.table_rp(p);
                    let dest = cpu.transform_register(AddressingMode::RegisterPair(rp), addressing);
                    cpu.dec_16_op(dest);
                }
            }
            4 => {
                test_log!(cpu, "INC r[y]");
                let reg = cpu.table_r(y);
                let dest = cpu.transform_register(reg.into(), addressing);
                cpu.inc_op(dest);
            }
            5 => {
                test_log!(cpu, "DEC r[y]");
                let reg = cpu.table_r(y);
                let dest = cpu.transform_register(reg.into(), addressing);
                cpu.dec_op(dest);
            }
            6 => {
                test_log!(cpu, "LD r[y], n");
                let reg = cpu.table_r(y);
                let dest = cpu.transform_register(reg.into(), addressing);
                let n = cpu.fetch();
                cpu.ld(dest, AddressingMode::Immediate(n))
            }
            7 => match y {
                0 => { test_log!(cpu, "RLCA"); let a = cpu.get_register(GPR::A); let (res, f) = rot::rlc(a); cpu.set_register(GPR::A, res); cpu.set_flag((f & flags::CARRY) != 0, Flag::C); cpu.set_flag(false, Flag::N); cpu.set_flag(false, Flag::H); }
                1 => { test_log!(cpu, "RRCA"); let a = cpu.get_register(GPR::A); let (res, f) = rot::rrc(a); cpu.set_register(GPR::A, res); cpu.set_flag((f & flags::CARRY) != 0, Flag::C); cpu.set_flag(false, Flag::N); cpu.set_flag(false, Flag::H); }
                2 => { test_log!(cpu, "RLA"); let a = cpu.get_register(GPR::A); let carry = cpu.get_flag(Flag::C); let (res, f) = rot::rl(a, carry); cpu.set_register(GPR::A, res); cpu.set_flag((f & flags::CARRY) != 0, Flag::C); cpu.set_flag(false, Flag::N); cpu.set_flag(false, Flag::H); }
                3 => { test_log!(cpu, "RRA"); let a = cpu.get_register(GPR::A); let carry = cpu.get_flag(Flag::C); let (res, f) = rot::rr(a, carry); cpu.set_register(GPR::A, res); cpu.set_flag((f & flags::CARRY) != 0, Flag::C); cpu.set_flag(false, Flag::N); cpu.set_flag(false, Flag::H); }
                4 => { test_log!(cpu, "DAA"); cpu.daa(); }
                5 => { test_log!(cpu, "CPL"); let result = cpu.get_register(GPR::A) ^ 0xFF; cpu.set_register(GPR::A, result); cpu.set_flag(true, Flag::N); cpu.set_flag(true, Flag::H); cpu.set_flag((result & flags::X) != 0, Flag::X); cpu.set_flag((result & flags::Y) != 0, Flag::Y); }
                6 => { test_log!(cpu, "SCF"); cpu.set_flag(true, Flag::C); cpu.set_flag(false, Flag::N); cpu.set_flag(false, Flag::H); let a = cpu.get_register(GPR::A); cpu.set_flag((a & flags::X) != 0, Flag::X); cpu.set_flag((a & flags::Y) != 0, Flag::Y); }
                7 => { test_log!(cpu, "CCF"); let current_carry = cpu.get_flag(Flag::C); cpu.set_flag(!current_carry, Flag::C); cpu.set_flag(false, Flag::N); cpu.set_flag(current_carry, Flag::H); let a = cpu.get_register(GPR::A); cpu.set_flag((a & flags::X) != 0, Flag::X); cpu.set_flag((a & flags::Y) != 0, Flag::Y); }
                _ => unreachable!("Invalid y value"),
            },
            _ => unreachable!("Invalid z value"),
        },
        1 => {
            if z == 6 && y == 6 {
                test_log!(cpu, "HALT");
                cpu.halted = true;
            } else {
                test_log!(cpu, "LD r[y], r[z]");
                let reg = cpu.table_r(y);
                let dest = cpu.transform_register(reg.into(), addressing);
                let reg = cpu.table_r(z);
                let src = cpu.transform_register(reg.into(), addressing);
                cpu.ld(dest, src);
            }
        }
        2 => {
            test_log!(cpu, "ALU[y] r[z]");
            let operation = cpu.table_alu(y);
            let reg = cpu.table_r(z);
            let src = cpu.transform_register(reg.into(), addressing);
            let value = cpu.read_8(src);
            cpu.alu_op(operation, value);
        }
        3 => match z {
            0 => { test_log!(cpu, "RET cc[y]"); let condition = cpu.table_cc(y); if cpu.evaluate_condition(condition) { cpu.pc = cpu.pop(); } }
            1 => match (q, p) {
                (false, _) => { test_log!(cpu, "POP rp2[p]"); let rp = cpu.table_rp2(p); let dest = cpu.transform_register(AddressingMode::RegisterPair(rp), addressing); let value = cpu.pop(); cpu.ld_16(dest, AddressingMode::ImmediateExtended(value)); }
                (true, 0) => { test_log!(cpu, "RET"); cpu.pc = cpu.pop(); }
                (true, 1) => { test_log!(cpu, "EXX"); cpu.exx(); }
                (true, 2) => { test_log!(cpu, "JP HL"); let src = cpu.transform_register(AddressingMode::RegisterPair(RegisterPair::HL), addressing); cpu.pc = cpu.read_16(src); }
                (true, 3) => { test_log!(cpu, "LD SP, HL"); let src = cpu.transform_register(AddressingMode::RegisterPair(RegisterPair::HL), addressing); cpu.ld_16(AddressingMode::RegisterPair(RegisterPair::SP), src); }
                _ => unreachable!("Invalid q, p values"),
            },
            2 => { test_log!(cpu, "JP cc[y], nn"); let condition = cpu.table_cc(y); let addr = cpu.fetch_word(); if cpu.evaluate_condition(condition) { cpu.pc = addr; } }
            3 => match y {
                0 => { test_log!(cpu, "JP nn"); cpu.pc = cpu.fetch_word(); }
                1 => unreachable!("CB prefix, should be handled separately"),
                2 => { test_log!(cpu, "OUT (n), A"); let port = cpu.fetch() as u16; let a = cpu.get_register(GPR::A); cpu.write_io(port, a); }
                3 => { test_log!(cpu, "IN A, (n)"); let port = cpu.fetch() as u16; let val = cpu.read_io(port); cpu.set_register(GPR::A, val); }
                4 => { test_log!(cpu, "EX (SP), HL/IX/IY"); let temp_l = cpu.memory.borrow().read(cpu.sp); let temp_h = cpu.memory.borrow().read(cpu.sp.wrapping_add(1)); let register_pair = cpu.transform_register(AddressingMode::RegisterPair(RegisterPair::HL), addressing); let rp = cpu.read_16(register_pair); cpu.memory.borrow_mut().write(cpu.sp, rp as u8); cpu.memory.borrow_mut().write(cpu.sp.wrapping_add(1), (rp >> 8) as u8); cpu.write_16(register_pair, ((temp_h as u16) << 8) | temp_l as u16); }
                5 => { test_log!(cpu, "EX DE, HL"); let de = cpu.get_register_pair(RegisterPair::DE); let hl = cpu.get_register_pair(RegisterPair::HL); cpu.set_register_pair(RegisterPair::DE, hl); cpu.set_register_pair(RegisterPair::HL, de); }
                6 => { test_log!(cpu, "DI"); cpu.iff1 = false; cpu.iff2 = false; cpu.iff_delay_count = 0; }
                7 => { test_log!(cpu, "EI"); cpu.iff_delay_count = 2; }
                _ => unreachable!("Invalid y value"),
            },
            4 => { test_log!(cpu, "CALL cc[y], nn"); let condition = cpu.table_cc(y); let addr = cpu.fetch_word(); if cpu.evaluate_condition(condition) { cpu.push(cpu.pc); cpu.pc = addr; } }
            5 => match (q, p) {
                (false, _) => { test_log!(cpu, "PUSH rp2[p]"); let rp = cpu.table_rp2(p); let src = cpu.transform_register(AddressingMode::RegisterPair(rp), addressing); let value = cpu.read_16(src); cpu.push(value); }
                (true, 0) => { test_log!(cpu, "CALL nn"); let addr = cpu.fetch_word(); cpu.push(cpu.pc); cpu.pc = addr; }
                (true, 1) => unreachable!("Shouldn't reach ED prefix here"),
                (true, 2) => unreachable!("Shouldn't reach DD prefix here"),
                (true, 3) => unreachable!("Shouldn't reach FD prefix here"),
                _ => unreachable!("Invalid q, p values"),
            },
            6 => { test_log!(cpu, "ALU[y] n"); let operation = cpu.table_alu(y); let n = cpu.fetch(); cpu.alu_op(operation, n); }
            7 => { test_log!(cpu, "RST y*8"); let rst_addr = y as u16 * 8; test_log!(cpu, "{:02X}h", rst_addr); cpu.push(cpu.pc); cpu.pc = rst_addr; }
            _ => unreachable!("Invalid z value"),
        },
        _ => unreachable!("Invalid x value"),
    }
}