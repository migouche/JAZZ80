use super::{BlockInstruction, Flag, GPR, RegisterPair, Z80A};

pub(super) fn execute(cpu: &mut Z80A, instruction: BlockInstruction) {
    cpu.execute_block_instruction_impl(instruction);
}

impl Z80A {
    fn execute_block_instruction_impl(&mut self, instruction: BlockInstruction) {
        match instruction {
            BlockInstruction::LDI => {
                let hl = self.get_register_pair(RegisterPair::HL);
                let de = self.get_register_pair(RegisterPair::DE);
                let bc = self.get_register_pair(RegisterPair::BC);
                let val = self.memory.borrow().read(hl);
                self.memory.borrow_mut().write(de, val);

                self.set_register_pair(RegisterPair::HL, hl.wrapping_add(1));
                self.set_register_pair(RegisterPair::DE, de.wrapping_add(1));
                let next_bc = bc.wrapping_sub(1);
                self.set_register_pair(RegisterPair::BC, next_bc);

                self.set_flag(false, Flag::H);
                self.set_flag(false, Flag::N);
                self.set_flag(next_bc != 0, Flag::PV);
                // X (3) and Y (5) from (A + (HL))? Manual says:
                // Bit 5 is bit 1 of (A + (HL)), Bit 3 is bit 3 of (A + (HL)) -- wait
                // "The contents of DE, HL, and BC are incremented/decremented... P/V is set if BC-1 is not 0..."
                // Z80 User Manual p. 195 (LDI):
                // S, Z, C not affected.
                // H, N reset.
                // P/V set if BC not 0.
            }
            BlockInstruction::LDIR => {
                self.execute_block_instruction_impl(BlockInstruction::LDI);
                if self.get_register_pair(RegisterPair::BC) != 0 {
                    self.pc = self.pc.wrapping_sub(2);
                }
            }
            BlockInstruction::LDD => {
                let hl = self.get_register_pair(RegisterPair::HL);
                let de = self.get_register_pair(RegisterPair::DE);
                let bc = self.get_register_pair(RegisterPair::BC);
                let val = self.memory.borrow().read(hl);
                self.memory.borrow_mut().write(de, val);

                self.set_register_pair(RegisterPair::HL, hl.wrapping_sub(1));
                self.set_register_pair(RegisterPair::DE, de.wrapping_sub(1));
                let next_bc = bc.wrapping_sub(1);
                self.set_register_pair(RegisterPair::BC, next_bc);

                self.set_flag(false, Flag::H);
                self.set_flag(false, Flag::N);
                self.set_flag(next_bc != 0, Flag::PV);
            }
            BlockInstruction::LDDR => {
                self.execute_block_instruction_impl(BlockInstruction::LDD);
                if self.get_register_pair(RegisterPair::BC) != 0 {
                    self.pc = self.pc.wrapping_sub(2);
                }
            }
            BlockInstruction::CPI => {
                let a = self.get_register(GPR::A);
                let hl = self.get_register_pair(RegisterPair::HL);
                let bc = self.get_register_pair(RegisterPair::BC);
                let val = self.memory.borrow().read(hl);

                let res = a.wrapping_sub(val);

                let next_bc = bc.wrapping_sub(1);
                self.set_register_pair(RegisterPair::HL, hl.wrapping_add(1));
                self.set_register_pair(RegisterPair::BC, next_bc);

                self.set_flag(res == 0, Flag::Z);
                self.set_flag((res & 0x80) != 0, Flag::S);
                // H is set if borrow from bit 4. (a & 0xF) < (val & 0xF)
                let h_borrow = (a & 0x0F) < (val & 0x0F);
                self.set_flag(h_borrow, Flag::H);
                self.set_flag(next_bc != 0, Flag::PV);
                self.set_flag(true, Flag::N);
            }
            BlockInstruction::CPIR => {
                self.execute_block_instruction_impl(BlockInstruction::CPI);
                let bc = self.get_register_pair(RegisterPair::BC);
                let z = self.get_flag(Flag::Z);
                if bc != 0 && !z {
                    self.pc = self.pc.wrapping_sub(2);
                }
            }
            BlockInstruction::CPD => {
                let a = self.get_register(GPR::A);
                let hl = self.get_register_pair(RegisterPair::HL);
                let bc = self.get_register_pair(RegisterPair::BC);
                let val = self.memory.borrow().read(hl);

                let res = a.wrapping_sub(val);

                let next_bc = bc.wrapping_sub(1);
                self.set_register_pair(RegisterPair::HL, hl.wrapping_sub(1));
                self.set_register_pair(RegisterPair::BC, next_bc);

                self.set_flag(res == 0, Flag::Z);
                self.set_flag((res & 0x80) != 0, Flag::S);
                let h_borrow = (a & 0x0F) < (val & 0x0F);
                self.set_flag(h_borrow, Flag::H);
                self.set_flag(next_bc != 0, Flag::PV);
                self.set_flag(true, Flag::N);
            }
            BlockInstruction::CPDR => {
                self.execute_block_instruction_impl(BlockInstruction::CPD);
                let bc = self.get_register_pair(RegisterPair::BC);
                let z = self.get_flag(Flag::Z);
                if bc != 0 && !z {
                    self.pc = self.pc.wrapping_sub(2);
                }
            }
            BlockInstruction::INI => {
                let b = self.get_register(GPR::B);
                let c = self.get_register(GPR::C);
                let hl = self.get_register_pair(RegisterPair::HL);

                let port = ((b as u16) << 8) | (c as u16);
                let val = self.read_io(port);
                self.memory.borrow_mut().write(hl, val);

                self.set_register_pair(RegisterPair::HL, hl.wrapping_add(1));
                let next_b = b.wrapping_sub(1);
                self.set_register(GPR::B, next_b);

                self.set_flag(next_b == 0, Flag::Z);
                self.set_flag(true, Flag::N);
            }
            BlockInstruction::INIR => {
                self.execute_block_instruction_impl(BlockInstruction::INI);
                if self.get_register(GPR::B) != 0 {
                    self.pc = self.pc.wrapping_sub(2);
                }
            }
            BlockInstruction::IND => {
                let b = self.get_register(GPR::B);
                let c = self.get_register(GPR::C);
                let hl = self.get_register_pair(RegisterPair::HL);

                let port = ((b as u16) << 8) | (c as u16);
                let val = self.read_io(port);
                self.memory.borrow_mut().write(hl, val);

                self.set_register_pair(RegisterPair::HL, hl.wrapping_sub(1));
                let next_b = b.wrapping_sub(1);
                self.set_register(GPR::B, next_b);

                self.set_flag(next_b == 0, Flag::Z);
                self.set_flag(true, Flag::N);
            }
            BlockInstruction::INDR => {
                self.execute_block_instruction_impl(BlockInstruction::IND);
                if self.get_register(GPR::B) != 0 {
                    self.pc = self.pc.wrapping_sub(2);
                }
            }
            BlockInstruction::OUTI => {
                let b = self.get_register(GPR::B);
                let c = self.get_register(GPR::C);
                let hl = self.get_register_pair(RegisterPair::HL);

                let val = self.memory.borrow().read(hl);

                let next_b = b.wrapping_sub(1);
                self.set_register(GPR::B, next_b);
                let port = ((next_b as u16) << 8) | (c as u16);
                self.write_io(port, val);

                self.set_register_pair(RegisterPair::HL, hl.wrapping_add(1));

                self.set_flag(next_b == 0, Flag::Z);
                self.set_flag(true, Flag::N);
            }
            BlockInstruction::OTIR => {
                self.execute_block_instruction_impl(BlockInstruction::OUTI);
                if self.get_register(GPR::B) != 0 {
                    self.pc = self.pc.wrapping_sub(2);
                }
            }
            BlockInstruction::OUTD => {
                let b = self.get_register(GPR::B);
                let c = self.get_register(GPR::C);
                let hl = self.get_register_pair(RegisterPair::HL);

                let val = self.memory.borrow().read(hl);

                let next_b = b.wrapping_sub(1);
                self.set_register(GPR::B, next_b);
                let port = ((next_b as u16) << 8) | (c as u16);
                self.write_io(port, val);

                self.set_register_pair(RegisterPair::HL, hl.wrapping_sub(1));

                self.set_flag(next_b == 0, Flag::Z);
                self.set_flag(true, Flag::N);
            }
            BlockInstruction::OTDR => {
                self.execute_block_instruction_impl(BlockInstruction::OUTD);
                if self.get_register(GPR::B) != 0 {
                    self.pc = self.pc.wrapping_sub(2);
                }
            }
        }
    }
}