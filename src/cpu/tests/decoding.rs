use rstest::rstest;

use crate::cpu::SyncronousComponent;
use crate::cpu::tests::setup_cpu;

/*
 * Note: these tests are incomplete, for example  "DEC r[y]" is used for multiple opcodes.
 * It's better than nothing for now, but more logging will be added later on the helper functions.
*/

#[rstest]
// UNPREFIXED TABLE (MAIN INSTRUCTIONS)
#[case::nop(0x29f9, &[0x00], &["decode_unprefixed", "NOP"])] // NOP
#[case::ld_bc_nn(0x29f9, &[0x01, 0x34, 0x12], &["decode_unprefixed", "LD rp[p], nn", "BC"])] // LD BC, nn
#[case::ld_bci_a(0x29f9, &[0x02], &["decode_unprefixed", "LD (BC), A"])] // LD (BC), A
#[case::inc_bc(0x29f9, &[0x03], &["decode_unprefixed", "INC rp[p]", "BC"])] // INC BC
#[case::inc_b(0x29f9, &[0x04], &["decode_unprefixed", "INC r[y]", "B"])] // INC B
#[case::dec_b(0x29f9, &[0x05], &["decode_unprefixed", "DEC r[y]", "B"])] // DEC B
#[case::ld_b_n(0x29f9, &[0x06, 0x56], &["decode_unprefixed", "LD r[y], n", "B"])] // LD B, n
#[case::rlca(0x29f9, &[0x07], &["decode_unprefixed", "RLCA"])] // RLCA
#[case::ex_af_afp(0x29f9, &[0x08], &["decode_unprefixed", "EX AF, AF'"])] // EX AF, AF'
#[case::add_hl_bc(0x29f9, &[0x09], &["decode_unprefixed", "ADD HL/IX/IY, rp[p]", "BC"])] // ADD HL, BC
#[case::ld_a_bci(0x29f9, &[0x0a], &["decode_unprefixed", "LD A, (BC)"])] // LD A, (BC)
#[case::dec_bc(0x29f9, &[0x0b], &["decode_unprefixed", "DEC rp[p]", "BC"])] // DEC BC
#[case::inc_c(0x29f9, &[0x0c], &["decode_unprefixed", "INC r[y]", "C"])] // INC C
#[case::dec_c(0x29f9, &[0x0d], &["decode_unprefixed", "DEC r[y]", "C"])] // DEC C
#[case::ld_c_n(0x29f9, &[0x0e, 0x78], &["decode_unprefixed", "LD r[y], n", "C"])] // LD C, n
#[case::rrca(0x29f9, &[0x0f], &["decode_unprefixed", "RRCA"])] // RRCA
#[case::djnz_d(0x29f9, &[0x10, 0xfe], &["decode_unprefixed", "DJNZ d"])] // DJNZ d
#[case::ld_de_n(0x29f9, &[0x11, 0x34, 0x12], &["decode_unprefixed", "LD rp[p], nn", "DE"])] // LD DE, nn
#[case::ld_dei_a(0x29f9, &[0x12], &["decode_unprefixed", "LD (DE), A"])] // LD (DE), A
#[case::inc_de(0x29f9, &[0x13], &["decode_unprefixed", "INC rp[p]", "DE"])] // INC DE
#[case::inc_d(0x29f9, &[0x14], &["decode_unprefixed", "INC r[y]", "D"])] // INC D
#[case::dec_d(0x29f9, &[0x15], &["decode_unprefixed", "DEC r[y]", "D"])] // DEC D
#[case::ld_d_n(0x29f9, &[0x16, 0x9a], &["decode_unprefixed", "LD r[y], n", "D"])] // LD D, n
#[case::rla(0x29f9, &[0x17], &["decode_unprefixed", "RLA"])] // RLA
#[case::jr_d(0x29f9, &[0x18, 0xfe], &["decode_unprefixed", "JR d"])] // JR d
#[case::add_hl_de(0x29f9, &[0x19], &["decode_unprefixed", "ADD HL/IX/IY, rp[p]", "DE"])] // ADD HL, DE
#[case::ld_a_dei(0x29f9, &[0x1a], &["decode_unprefixed", "LD A, (DE)"])] // LD A, (DE)
#[case::dec_de(0x29f9, &[0x1b], &["decode_unprefixed", "DEC rp[p]", "DE"])] // DEC DE
#[case::inc_e(0x29f9, &[0x1c], &["decode_unprefixed", "INC r[y]", "E"])] // INC E
#[case::dec_e(0x29f9, &[0x1d], &["decode_unprefixed", "DEC r[y]", "E"])] // DEC E
#[case::ld_e_n(0x29f9, &[0x1e, 0xbc], &["decode_unprefixed", "LD r[y], n", "E"])] // LD E, n
#[case::rra(0x29f9, &[0x1f], &["decode_unprefixed", "RRA"])] // RRA
#[case::jr_nz_d(0x29f9, &[0x20, 0xfe], &["decode_unprefixed", "JR cc[y-4], d", "NZ"])] // JR NZ, d
#[case::ld_hl_nn(0x29f9, &[0x21, 0x34, 0x12], &["decode_unprefixed", "LD rp[p], nn", "HL"])] // LD HL, nn
#[case::ld_nni_hl(0x29f9, &[0x22, 0x00, 0x80], &["decode_unprefixed", "LD (nn), HL/IX/IY"])] // LD (nn), HL
#[case::inc_hl(0x29f9, &[0x23], &["decode_unprefixed", "INC rp[p]", "HL"])] // INC HL
#[case::inc_h(0x29f9, &[0x24], &["decode_unprefixed", "INC r[y]", "H"])] // INC H
#[case::dec_h(0x29f9, &[0x25], &["decode_unprefixed", "DEC r[y]", "H"])] // DEC H
#[case::ld_h_n(0x29f9, &[0x26, 0x56], &["decode_unprefixed", "LD r[y], n", "H"])] // LD H, n
#[case::daa(0x29f9, &[0x27], &["decode_unprefixed", "DAA"])] // DAA
#[case::jr_z_d(0x29f9, &[0x28, 0xfe], &["decode_unprefixed", "JR cc[y-4], d", "Z"])] // JR Z, d
#[case::add_hl_hl(0x29f9, &[0x29], &["decode_unprefixed", "ADD HL/IX/IY, rp[p]", "HL"])] // ADD HL, HL
#[case::ld_hl_nni(0x29f9, &[0x2a, 0x00, 0x80], &["decode_unprefixed", "LD HL/IX/IY, (nn)"])] // LD HL, (nn)
#[case::dec_hl(0x29f9, &[0x2b], &["decode_unprefixed", "DEC rp[p]", "HL"])] // DEC HL
#[case::inc_l(0x29f9, &[0x2c], &["decode_unprefixed", "INC r[y]", "L"])] // INC L
#[case::dec_l(0x29f9, &[0x2d], &["decode_unprefixed", "DEC r[y]", "L"])] // DEC L
#[case::ld_l_n(0x29f9, &[0x2e, 0x78], &["decode_unprefixed", "LD r[y], n", "L"])] // LD L, n
#[case::cpl(0x29f9, &[0x2f], &["decode_unprefixed", "CPL"])] // CPL
#[case::jr_nc_d(0x29f9, &[0x30, 0xfe], &["decode_unprefixed", "JR cc[y-4], d", "NC"])] // JR NC, d
#[case::ld_sp_nn(0x29f9, &[0x31, 0x34, 0x12], &["decode_unprefixed", "LD rp[p], nn", "SP"])] // LD SP, nn
#[case::ld_nni_a(0x29f9, &[0x32, 0x00, 0x80], &["decode_unprefixed", "LD (nn), A"])] // LD (nn), A
#[case::inc_sp(0x29f9, &[0x33], &["decode_unprefixed", "INC rp[p]", "SP"])] // INC SP
#[case::inc_hli(0x29f9, &[0x34], &["decode_unprefixed", "INC r[y]", "(HL)"])] // INC (HL)
#[case::dec_hli(0x29f9, &[0x35], &["decode_unprefixed", "DEC r[y]", "(HL)"])] // DEC (HL)
#[case::ld_hli_n(0x29f9, &[0x36, 0x56], &["decode_unprefixed", "LD r[y], n", "(HL)"])] // LD (HL), n
#[case::scf(0x29f9, &[0x37], &["decode_unprefixed", "SCF"])] // SCF
#[case::jr_c_d(0x29f9, &[0x38, 0xfe], &["decode_unprefixed", "JR cc[y-4], d", "C"])] // JR C, d
#[case::add_hl_sp(0x29f9, &[0x39], &["decode_unprefixed", "ADD HL/IX/IY, rp[p]", "SP"])] // ADD HL, SP
#[case::ld_a_nni(0x29f9, &[0x3a, 0x00, 0x80], &["decode_unprefixed", "LD A, (nn)"])] // LD A, (nn)
#[case::dec_sp(0x29f9, &[0x3b], &["decode_unprefixed", "DEC rp[p]", "SP"])] // DEC SP
#[case::inc_lli(0x29f9, &[0x3c], &["decode_unprefixed", "INC r[y]", "A"])] // INC A
#[case::dec_lli(0x29f9, &[0x3d], &["decode_unprefixed", "DEC r[y]", "A"])] // DEC A
#[case::ld_a_n(0x29f9, &[0x3e, 0x78], &["decode_unprefixed", "LD r[y], n", "A"])] // LD A, n
#[case::ccf(0x29f9, &[0x3f], &["decode_unprefixed", "CCF"])] // CCF
#[case::ld_b_b(0x29f9, &[0x40], &["decode_unprefixed", "LD r[y], r[z]", "B", "B"])] // LD B, B
#[case::ld_b_c(0x29f9, &[0x41], &["decode_unprefixed", "LD r[y], r[z]", "B", "C"])] // LD B, C
#[case::ld_b_d(0x29f9, &[0x42], &["decode_unprefixed", "LD r[y], r[z]", "B", "D"])] // LD B, D
#[case::ld_b_e(0x29f9, &[0x43], &["decode_unprefixed", "LD r[y], r[z]", "B", "E"])] // LD B, E
#[case::ld_b_h(0x29f9, &[0x44], &["decode_unprefixed", "LD r[y], r[z]", "B", "H"])] // LD B, H
#[case::ld_b_l(0x29f9, &[0x45], &["decode_unprefixed", "LD r[y], r[z]", "B", "L"])] // LD B, L
#[case::ld_b_hli(0x29f9, &[0x46], &["decode_unprefixed", "LD r[y], r[z]", "B", "(HL)"])] // LD B, (HL)
#[case::ld_b_a(0x29f9, &[0x47], &["decode_unprefixed", "LD r[y], r[z]", "B", "A"])] // LD B, A
#[case::ld_c_b(0x29f9, &[0x48], &["decode_unprefixed", "LD r[y], r[z]", "C", "B"])] // LD C, B
#[case::ld_c_c(0x29f9, &[0x49], &["decode_unprefixed", "LD r[y], r[z]", "C", "C"])] // LD C, C
#[case::ld_c_d(0x29f9, &[0x4a], &["decode_unprefixed", "LD r[y], r[z]", "C", "D"])] // LD C, D
#[case::ld_c_e(0x29f9, &[0x4b], &["decode_unprefixed", "LD r[y], r[z]", "C", "E"])] // LD C, E
#[case::ld_c_h(0x29f9, &[0x4c], &["decode_unprefixed", "LD r[y], r[z]", "C", "H"])] // LD C, H
#[case::ld_c_l(0x29f9, &[0x4d], &["decode_unprefixed", "LD r[y], r[z]", "C", "L"])] // LD C, L
#[case::ld_c_hli(0x29f9, &[0x4e], &["decode_unprefixed", "LD r[y], r[z]", "C", "(HL)"])] // LD C, (HL)
#[case::ld_c_a(0x29f9, &[0x4f], &["decode_unprefixed", "LD r[y], r[z]", "C", "A"])] // LD C, A
#[case::ld_d_b(0x29f9, &[0x50], &["decode_unprefixed", "LD r[y], r[z]", "D", "B"])] // LD D, B
#[case::ld_d_c(0x29f9, &[0x51], &["decode_unprefixed", "LD r[y], r[z]", "D", "C"])] // LD D, C
#[case::ld_d_d(0x29f9, &[0x52], &["decode_unprefixed", "LD r[y], r[z]", "D", "D"])] // LD D, D
#[case::ld_d_e(0x29f9, &[0x53], &["decode_unprefixed", "LD r[y], r[z]", "D", "E"])] // LD D, E
#[case::ld_d_h(0x29f9, &[0x54], &["decode_unprefixed", "LD r[y], r[z]", "D", "H"])] // LD D, H
#[case::ld_d_l(0x29f9, &[0x55], &["decode_unprefixed", "LD r[y], r[z]", "D", "L"])] // LD D, L
#[case::ld_d_hli(0x29f9, &[0x56], &["decode_unprefixed", "LD r[y], r[z]", "D", "(HL)"])] // LD D, (HL)
#[case::ld_d_a(0x29f9, &[0x57], &["decode_unprefixed", "LD r[y], r[z]", "D", "A"])] // LD D, A
#[case::ld_e_b(0x29f9, &[0x58], &["decode_unprefixed", "LD r[y], r[z]", "E", "B"])] // LD E, B
#[case::ld_e_c(0x29f9, &[0x59], &["decode_unprefixed", "LD r[y], r[z]", "E", "C"])] // LD E, C
#[case::ld_e_d(0x29f9, &[0x5a], &["decode_unprefixed", "LD r[y], r[z]", "E", "D"])] // LD E, D
#[case::ld_e_e(0x29f9, &[0x5b], &["decode_unprefixed", "LD r[y], r[z]", "E", "E"])] // LD E, E
#[case::ld_e_h(0x29f9, &[0x5c], &["decode_unprefixed", "LD r[y], r[z]", "E", "H"])] // LD E, H
#[case::ld_e_l(0x29f9, &[0x5d], &["decode_unprefixed", "LD r[y], r[z]", "E", "L"])] // LD E, L
#[case::ld_e_hli(0x29f9, &[0x5e], &["decode_unprefixed", "LD r[y], r[z]", "E", "(HL)"])] // LD E, (HL)
#[case::ld_e_a(0x29f9, &[0x5f], &["decode_unprefixed", "LD r[y], r[z]", "E", "A"])] // LD E, A
#[case::ld_h_b(0x29f9, &[0x60], &["decode_unprefixed", "LD r[y], r[z]", "H", "B"])] // LD H, B
#[case::ld_h_c(0x29f9, &[0x61], &["decode_unprefixed", "LD r[y], r[z]", "H", "C"])] // LD H, C
#[case::ld_h_d(0x29f9, &[0x62], &["decode_unprefixed", "LD r[y], r[z]", "H", "D"])] // LD H, D
#[case::ld_h_e(0x29f9, &[0x63], &["decode_unprefixed", "LD r[y], r[z]", "H", "E"])] // LD H, E
#[case::ld_h_h(0x29f9, &[0x64], &["decode_unprefixed", "LD r[y], r[z]", "H", "H"])] // LD H, H
#[case::ld_h_l(0x29f9, &[0x65], &["decode_unprefixed", "LD r[y], r[z]", "H", "L"])] // LD H, L
#[case::ld_h_hli(0x29f9, &[0x66], &["decode_unprefixed", "LD r[y], r[z]", "H", "(HL)"])] // LD H, (HL)
#[case::ld_h_a(0x29f9, &[0x67], &["decode_unprefixed", "LD r[y], r[z]", "H", "A"])] // LD H, A
#[case::ld_l_b(0x29f9, &[0x68], &["decode_unprefixed", "LD r[y], r[z]", "L", "B"])] // LD L, B
#[case::ld_l_c(0x29f9, &[0x69], &["decode_unprefixed", "LD r[y], r[z]", "L", "C"])] // LD L, C
#[case::ld_l_d(0x29f9, &[0x6a], &["decode_unprefixed", "LD r[y], r[z]", "L", "D"])] // LD L, D
#[case::ld_l_e(0x29f9, &[0x6b], &["decode_unprefixed", "LD r[y], r[z]", "L", "E"])] // LD L, E
#[case::ld_l_h(0x29f9, &[0x6c], &["decode_unprefixed", "LD r[y], r[z]", "L", "H"])] // LD L, H
#[case::ld_l_l(0x29f9, &[0x6d], &["decode_unprefixed", "LD r[y], r[z]", "L", "L"])] // LD L, L
#[case::ld_l_hli(0x29f9, &[0x6e], &["decode_unprefixed", "LD r[y], r[z]", "L", "(HL)"])] // LD L, (HL)
#[case::ld_l_a(0x29f9, &[0x6f], &["decode_unprefixed", "LD r[y], r[z]", "L", "A"])] // LD L, A
#[case::ld_hli_b(0x29f9, &[0x70], &["decode_unprefixed", "LD r[y], r[z]", "(HL)", "B"])] // LD (HL), B
#[case::ld_hli_c(0x29f9, &[0x71], &["decode_unprefixed", "LD r[y], r[z]", "(HL)", "C"])] // LD (HL), C
#[case::ld_hli_d(0x29f9, &[0x72], &["decode_unprefixed", "LD r[y], r[z]", "(HL)", "D"])] // LD (HL), D
#[case::ld_hli_e(0x29f9, &[0x73], &["decode_unprefixed", "LD r[y], r[z]", "(HL)", "E"])] // LD (HL), E
#[case::ld_hli_h(0x29f9, &[0x74], &["decode_unprefixed", "LD r[y], r[z]", "(HL)", "H"])] // LD (HL), H
#[case::ld_hli_l(0x29f9, &[0x75], &["decode_unprefixed", "LD r[y], r[z]", "(HL)", "L"])] // LD (HL), L
#[case::halt(0x29f9, &[0x76], &["decode_unprefixed", "HALT"])] // HALT
#[case::ld_hli_a(0x29f9, &[0x77], &["decode_unprefixed", "LD r[y], r[z]", "(HL)", "A"])] // LD (HL), A
#[case::ld_a_b(0x29f9, &[0x78], &["decode_unprefixed", "LD r[y], r[z]", "A", "B"])] // LD A, B
#[case::ld_a_c(0x29f9, &[0x79], &["decode_unprefixed", "LD r[y], r[z]", "A", "C"])] // LD A, C
#[case::ld_a_d(0x29f9, &[0x7a], &["decode_unprefixed", "LD r[y], r[z]", "A", "D"])] // LD A, D
#[case::ld_a_e(0x29f9, &[0x7b], &["decode_unprefixed", "LD r[y], r[z]", "A", "E"])] // LD A, E
#[case::ld_a_h(0x29f9, &[0x7c], &["decode_unprefixed", "LD r[y], r[z]", "A", "H"])] // LD A, H
#[case::ld_a_l(0x29f9, &[0x7d], &["decode_unprefixed", "LD r[y], r[z]", "A", "L"])] // LD A, L
#[case::ld_a_hli(0x29f9, &[0x7e], &["decode_unprefixed", "LD r[y], r[z]", "A", "(HL)"])] // LD A, (HL)
#[case::ld_a_a(0x29f9, &[0x7f], &["decode_unprefixed", "LD r[y], r[z]", "A", "A"])] // LD A, A
#[case::add_a_b(0x29f9, &[0x80], &["decode_unprefixed", "ALU[y] r[z]", "ADD A", "B"])] // ADD A, B
#[case::add_a_c(0x29f9, &[0x81], &["decode_unprefixed", "ALU[y] r[z]", "ADD A", "C"])] // ADD A, C
#[case::add_a_d(0x29f9, &[0x82], &["decode_unprefixed", "ALU[y] r[z]", "ADD A", "D"])] // ADD A, D
#[case::add_a_e(0x29f9, &[0x83], &["decode_unprefixed", "ALU[y] r[z]", "ADD A", "E"])] // ADD A, E
#[case::add_a_h(0x29f9, &[0x84], &["decode_unprefixed", "ALU[y] r[z]", "ADD A", "H"])] // ADD A, H
#[case::add_a_l(0x29f9, &[0x85], &["decode_unprefixed", "ALU[y] r[z]", "ADD A", "L"])] // ADD A, L
#[case::add_a_hli(0x29f9, &[0x86], &["decode_unprefixed", "ALU[y] r[z]", "ADD A", "(HL)"])] // ADD A, (HL)
#[case::add_a_a(0x29f9, &[0x87], &["decode_unprefixed", "ALU[y] r[z]", "ADD A", "A"])] // ADD A, A
#[case::adc_a_b(0x29f9, &[0x88], &["decode_unprefixed", "ALU[y] r[z]", "ADC A", "B"])] // ADC A, B
#[case::adc_a_c(0x29f9, &[0x89], &["decode_unprefixed", "ALU[y] r[z]", "ADC A", "C"])] // ADC A, C
#[case::adc_a_d(0x29f9, &[0x8a], &["decode_unprefixed", "ALU[y] r[z]", "ADC A", "D"])] // ADC A, D
#[case::adc_a_e(0x29f9, &[0x8b], &["decode_unprefixed", "ALU[y] r[z]", "ADC A", "E"])] // ADC A, E
#[case::adc_a_h(0x29f9, &[0x8c], &["decode_unprefixed", "ALU[y] r[z]", "ADC A", "H"])] // ADC A, H
#[case::adc_a_l(0x29f9, &[0x8d], &["decode_unprefixed", "ALU[y] r[z]", "ADC A", "L"])] // ADC A, L
#[case::adc_a_hli(0x29f9, &[0x8e], &["decode_unprefixed", "ALU[y] r[z]", "ADC A", "(HL)"])] // ADC A, (HL)
#[case::adc_a_a(0x29f9, &[0x8f], &["decode_unprefixed", "ALU[y] r[z]", "ADC A", "A"])] // ADC A, A
#[case::sub_a_b(0x29f9, &[0x90], &["decode_unprefixed", "ALU[y] r[z]", "SUB A", "B"])] // SUB A, B
#[case::sub_a_c(0x29f9, &[0x91], &["decode_unprefixed", "ALU[y] r[z]", "SUB A", "C"])] // SUB A, C
#[case::sub_a_d(0x29f9, &[0x92], &["decode_unprefixed", "ALU[y] r[z]", "SUB A", "D"])] // SUB A, D
#[case::sub_a_e(0x29f9, &[0x93], &["decode_unprefixed", "ALU[y] r[z]", "SUB A", "E"])] // SUB A, E
#[case::sub_a_h(0x29f9, &[0x94], &["decode_unprefixed", "ALU[y] r[z]", "SUB A", "H"])] // SUB A, H
#[case::sub_a_l(0x29f9, &[0x95], &["decode_unprefixed", "ALU[y] r[z]", "SUB A", "L"])] // SUB A, L
#[case::sub_a_hli(0x29f9, &[0x96], &["decode_unprefixed", "ALU[y] r[z]", "SUB A", "(HL)"])] // SUB A, (HL)
#[case::sub_a_a(0x29f9, &[0x97], &["decode_unprefixed", "ALU[y] r[z]", "SUB A", "A"])] // SUB A, A
#[case::sbc_a_b(0x29f9, &[0x98], &["decode_unprefixed", "ALU[y] r[z]", "SBC A", "B"])] // SBC A, B
#[case::sbc_a_c(0x29f9, &[0x99], &["decode_unprefixed", "ALU[y] r[z]", "SBC A", "C"])] // SBC A, C
#[case::sbc_a_d(0x29f9, &[0x9a], &["decode_unprefixed", "ALU[y] r[z]", "SBC A", "D"])] // SBC A, D
#[case::sbc_a_e(0x29f9, &[0x9b], &["decode_unprefixed", "ALU[y] r[z]", "SBC A", "E"])] // SBC A, E
#[case::sbc_a_h(0x29f9, &[0x9c], &["decode_unprefixed", "ALU[y] r[z]", "SBC A", "H"])] // SBC A, H
#[case::sbc_a_l(0x29f9, &[0x9d], &["decode_unprefixed", "ALU[y] r[z]", "SBC A", "L"])] // SBC A, L
#[case::sbc_a_hli(0x29f9, &[0x9e], &["decode_unprefixed", "ALU[y] r[z]", "SBC A", "(HL)"])] // SBC A, (HL)
#[case::sbc_a_a(0x29f9, &[0x9f], &["decode_unprefixed", "ALU[y] r[z]", "SBC A", "A"])] // SBC A, A
#[case::and_a_b(0x29f9, &[0xa0], &["decode_unprefixed", "ALU[y] r[z]", "AND A", "B"])] // AND A, B
#[case::and_a_c(0x29f9, &[0xa1], &["decode_unprefixed", "ALU[y] r[z]", "AND A", "C"])] // AND A, C
#[case::and_a_d(0x29f9, &[0xa2], &["decode_unprefixed", "ALU[y] r[z]", "AND A", "D"])] // AND A, D
#[case::and_a_e(0x29f9, &[0xa3], &["decode_unprefixed", "ALU[y] r[z]", "AND A", "E"])] // AND A, E
#[case::and_a_h(0x29f9, &[0xa4], &["decode_unprefixed", "ALU[y] r[z]", "AND A", "H"])] // AND A, H
#[case::and_a_l(0x29f9, &[0xa5], &["decode_unprefixed", "ALU[y] r[z]", "AND A", "L"])] // AND A, L
#[case::and_a_hli(0x29f9, &[0xa6], &["decode_unprefixed", "ALU[y] r[z]", "AND A", "(HL)"])] // AND A, (HL)
#[case::and_a_a(0x29f9, &[0xa7], &["decode_unprefixed", "ALU[y] r[z]", "AND A", "A"])] // AND A, A
#[case::xor_a_b(0x29f9, &[0xa8], &["decode_unprefixed", "ALU[y] r[z]", "XOR A", "B"])] // XOR A, B
#[case::xor_a_c(0x29f9, &[0xa9], &["decode_unprefixed", "ALU[y] r[z]", "XOR A", "C"])] // XOR A, C
#[case::xor_a_d(0x29f9, &[0xaa], &["decode_unprefixed", "ALU[y] r[z]", "XOR A", "D"])] // XOR A, D
#[case::xor_a_e(0x29f9, &[0xab], &["decode_unprefixed", "ALU[y] r[z]", "XOR A", "E"])] // XOR A, E
#[case::xor_a_h(0x29f9, &[0xac], &["decode_unprefixed", "ALU[y] r[z]", "XOR A", "H"])] // XOR A, H
#[case::xor_a_l(0x29f9, &[0xad], &["decode_unprefixed", "ALU[y] r[z]", "XOR A", "L"])] // XOR A, L
#[case::xor_a_hli(0x29f9, &[0xae], &["decode_unprefixed", "ALU[y] r[z]", "XOR A", "(HL)"])] // XOR A, (HL)
#[case::xor_a_a(0x29f9, &[0xaf], &["decode_unprefixed", "ALU[y] r[z]", "XOR A", "A"])] // XOR A, A
#[case::or_a_b(0x29f9, &[0xb0], &["decode_unprefixed", "ALU[y] r[z]", "OR A", "B"])] // OR A, B
#[case::or_a_c(0x29f9, &[0xb1], &["decode_unprefixed", "ALU[y] r[z]", "OR A", "C"])] // OR A, C
#[case::or_a_d(0x29f9, &[0xb2], &["decode_unprefixed", "ALU[y] r[z]", "OR A", "D"])] // OR A, D
#[case::or_a_e(0x29f9, &[0xb3], &["decode_unprefixed", "ALU[y] r[z]", "OR A", "E"])] // OR A, E
#[case::or_a_h(0x29f9, &[0xb4], &["decode_unprefixed", "ALU[y] r[z]", "OR A", "H"])] // OR A, H
#[case::or_a_l(0x29f9, &[0xb5], &["decode_unprefixed", "ALU[y] r[z]", "OR A", "L"])] // OR A, L
#[case::or_a_hli(0x29f9, &[0xb6], &["decode_unprefixed", "ALU[y] r[z]", "OR A", "(HL)"])] // OR A, (HL)
#[case::or_a_a(0x29f9, &[0xb7], &["decode_unprefixed", "ALU[y] r[z]", "OR A", "A"])] // OR A, A
#[case::cp_a_b(0x29f9, &[0xb8], &["decode_unprefixed", "ALU[y] r[z]", "CP A", "B"])] // CP A, B
#[case::cp_a_c(0x29f9, &[0xb9], &["decode_unprefixed", "ALU[y] r[z]", "CP A", "C"])] // CP A, C
#[case::cp_a_d(0x29f9, &[0xba], &["decode_unprefixed", "ALU[y] r[z]", "CP A", "D"])] // CP A, D
#[case::cp_a_e(0x29f9, &[0xbb], &["decode_unprefixed", "ALU[y] r[z]", "CP A", "E"])] // CP A, E
#[case::cp_a_h(0x29f9, &[0xbc], &["decode_unprefixed", "ALU[y] r[z]", "CP A", "H"])] // CP A, H
#[case::cp_a_l(0x29f9, &[0xbd], &["decode_unprefixed", "ALU[y] r[z]", "CP A", "L"])] // CP A, L
#[case::cp_a_hli(0x29f9, &[0xbe], &["decode_unprefixed", "ALU[y] r[z]", "CP A", "(HL)"])] // CP A, (HL)
#[case::cp_a_a(0x29f9, &[0xbf], &["decode_unprefixed", "ALU[y] r[z]", "CP A", "A"])] // CP A, A
#[case::ret_nz(0x29f9, &[0xc0], &["decode_unprefixed", "RET cc[y]", "NZ"])] // RET NZ
#[case::pop_bc(0x29f9, &[0xc1], &["decode_unprefixed", "POP rp2[p]", "BC"])] // POP BC
#[case::jp_nz_nn(0x29f9, &[0xc2, 0x34, 0x12], &["decode_unprefixed", "JP cc[y], nn", "NZ"])] // JP NZ, nn
#[case::jp_nn(0x29f9, &[0xc3, 0x34, 0x12], &["decode_unprefixed", "JP nn"])] // JP nn
#[case::call_nz_nn(0x29f9, &[0xc4, 0x34, 0x12], &["decode_unprefixed", "CALL cc[y], nn", "NZ"])] // CALL NZ, nn
#[case::push_bc(0x29f9, &[0xc5], &["decode_unprefixed", "PUSH rp2[p]", "BC"])] // PUSH BC
#[case::add_a_n(0x29f9, &[0xc6, 0x56], &["decode_unprefixed", "ALU[y] n", "ADD A"])] // ADD A, n
#[case::rst_00h(0x29f9, &[0xc7], &["decode_unprefixed", "RST y*8", "00h"])] // RST 00h
#[case::ret_z(0x29f9, &[0xc8], &["decode_unprefixed", "RET cc[y]", "Z"])] // RET Z
#[case::ret(0x29f9, &[0xc9], &["decode_unprefixed", "RET"])] // RET
#[case::jp_z_nn(0x29f9, &[0xca, 0x34, 0x12], &["decode_unprefixed", "JP cc[y], nn", "Z"])] // JP Z, nn
#[case::cb_prefix(0x29f9, &[0xcb, 0x00], &["decode_cb", "rot[y] r[z]", "RLC", "B"])] // CB Prefix (RLC B)
#[case::call_z_nn(0x29f9, &[0xcc, 0x34, 0x12], &["decode_unprefixed", "CALL cc[y], nn", "Z"])] // CALL Z, nn
#[case::call_nn(0x29f9, &[0xcd, 0x34, 0x12], &["decode_unprefixed", "CALL nn"])] // CALL nn
#[case::adc_a_n(0x29f9, &[0xce, 0x56], &["decode_unprefixed", "ALU[y] n", "ADC A"])] // ADC A, n
#[case::rst_08h(0x29f9, &[0xcf], &["decode_unprefixed", "RST y*8", "08h"])] // RST 08h
#[case::ret_nc(0x29f9, &[0xd0], &["decode_unprefixed", "RET cc[y]", "NC"])] // RET NC
#[case::pop_de(0x29f9, &[0xd1], &["decode_unprefixed", "POP rp2[p]", "DE"])] // POP DE
#[case::jp_nc_nn(0x29f9, &[0xd2, 0x34, 0x12], &["decode_unprefixed", "JP cc[y], nn", "NC"])] // JP NC, nn
#[case::out_n_a(0x29f9, &[0xd3, 0x56], &["decode_unprefixed", "OUT (n), A"])] // OUT (n), A
#[case::call_nc_nn(0x29f9, &[0xd4, 0x34, 0x12], &["decode_unprefixed", "CALL cc[y], nn", "NC"])] // CALL NC, nn
#[case::push_de(0x29f9, &[0xd5], &["decode_unprefixed", "PUSH rp2[p]", "DE"])] // PUSH DE
#[case::sub_a_n(0x29f9, &[0xd6, 0x56], &["decode_unprefixed", "ALU[y] n", "SUB A"])] // SUB A, n
#[case::rst_10h(0x29f9, &[0xd7], &["decode_unprefixed", "RST y*8", "10h"])] // RST 10h
#[case::ret_c(0x29f9, &[0xd8], &["decode_unprefixed", "RET cc[y]", "C"])] // RET C
#[case::exx(0x29f9, &[0xd9], &["decode_unprefixed", "EXX"])] // EXX
#[case::jp_c_nn(0x29f9, &[0xda, 0x34, 0x12], &["decode_unprefixed", "JP cc[y], nn", "C"])] // JP C, nn
#[case::in_a_n(0x29f9, &[0xdb, 0x56], &["decode_unprefixed", "IN A, (n)"])] // IN A, (n)
#[case::call_c_nn(0x29f9, &[0xdc, 0x34, 0x12], &["decode_unprefixed", "CALL cc[y], nn", "C"])] // CALL C, nn
#[case::dd_prefix(0x29f9, &[0xdd, 0x00], &["decode_dd", "NOP"])] // DD Prefix (NOP)
#[case::sbc_a_n(0x29f9, &[0xde, 0x56], &["decode_unprefixed", "ALU[y] n", "SBC A"])] // SBC A, n
#[case::rst_18h(0x29f9, &[0xdf], &["decode_unprefixed", "RST y*8", "18h"])] // RST 18h
#[case::ret_po(0x29f9, &[0xe0], &["decode_unprefixed", "RET cc[y]", "PO"])] // RET PO
#[case::pop_hl(0x29f9, &[0xe1], &["decode_unprefixed", "POP rp2[p]", "HL"])] // POP HL
#[case::jp_po_nn(0x29f9, &[0xe2, 0x34, 0x12], &["decode_unprefixed", "JP cc[y], nn", "PO"])] // JP PO, nn
#[case::ex_sp_hl(0x29f9, &[0xe3], &["decode_unprefixed", "EX (SP), HL/IX/IY"])] // EX (SP), HL/IX/IY
#[case::call_po_nn(0x29f9, &[0xe4, 0x34, 0x12], &["decode_unprefixed", "CALL cc[y], nn", "PO"])] // CALL PO, nn
#[case::push_hl(0x29f9, &[0xe5], &["decode_unprefixed", "PUSH rp2[p]", "HL"])] // PUSH HL
#[case::and_a_n(0x29f9, &[0xe6, 0x56], &["decode_unprefixed", "ALU[y] n", "AND A"])] // AND A, n
#[case::rst_20h(0x29f9, &[0xe7], &["decode_unprefixed", "RST y*8", "20h"])] // RST 20h
#[case::ret_pe(0x29f9, &[0xe8], &["decode_unprefixed", "RET cc[y]", "PE"])] // RET PE
#[case::jp_hl(0x29f9, &[0xe9], &["decode_unprefixed", "JP HL"])] // JP HL
#[case::jp_pe_nn(0x29f9, &[0xea, 0x34, 0x12], &["decode_unprefixed", "JP cc[y], nn", "PE"])] // JP PE, nn
#[case::ex_de_hl(0x29f9, &[0xeb], &["decode_unprefixed", "EX DE, HL"])] // EX DE, HL NOTE: this is the unaffected by prefixes
#[case::call_pe_nn(0x29f9, &[0xec, 0x34, 0x12], &["decode_unprefixed", "CALL cc[y], nn", "PE"])] // CALL PE, nn
#[case::ed_prefix(0x29f9, &[0xed, 0x00], &["decode_ed", "NONI"])] // ED Prefix (NONI)
#[case::xor_a_n(0x29f9, &[0xee, 0x56], &["decode_unprefixed", "ALU[y] n", "XOR A"])] // XOR A, n
#[case::rst_28h(0x29f9, &[0xef], &["decode_unprefixed", "RST y*8", "28h"])] // RST 28h
#[case::ret_p(0x29f9, &[0xf0], &["decode_unprefixed", "RET cc[y]", "P"])] // RET P
#[case::pop_af(0x29f9, &[0xf1], &["decode_unprefixed", "POP rp2[p]", "AF"])] // POP AF
#[case::jp_p_nn(0x29f9, &[0xf2, 0x34, 0x12], &["decode_unprefixed", "JP cc[y], nn", "P"])] // JP P, nn
#[case::di(0x29f9, &[0xf3], &["decode_unprefixed", "DI"])] // DI
#[case::call_p_nn(0x29f9, &[0xf4, 0x34, 0x12], &["decode_unprefixed", "CALL cc[y], nn", "P"])] // CALL P, nn
#[case::push_af(0x29f9, &[0xf5], &["decode_unprefixed", "PUSH rp2[p]", "AF"])] // PUSH AF
#[case::or_a_n(0x29f9, &[0xf6, 0x56], &["decode_unprefixed", "ALU[y] n", "OR A"])] // OR A, n
#[case::rst_30h(0x29f9, &[0xf7], &["decode_unprefixed", "RST y*8", "30h"])] // RST 30h
#[case::ret_m(0x29f9, &[0xf8], &["decode_unprefixed", "RET cc[y]", "M"])] // RET M
#[case::ld_sp_hl(0x29f9, &[0xf9], &["decode_unprefixed", "LD SP, HL"])] // LD SP, HL
#[case::jp_m_nn(0x29f9, &[0xfa, 0x34, 0x12], &["decode_unprefixed", "JP cc[y], nn", "M"])] // JP M, nn
#[case::ei(0x29f9, &[0xfb], &["decode_unprefixed", "EI"])] // EI
#[case::call_m_nn(0x29f9, &[0xfc, 0x34, 0x12], &["decode_unprefixed", "CALL cc[y], nn", "M"])] // CALL M, nn
#[case::fd_prefix(0x29f9, &[0xfd, 0x00], &["decode_fd", "NOP"])] // DD Prefix (NOP)
#[case::cp_a_n(0x29f9, &[0xfe, 0x56], &["decode_unprefixed", "ALU[y] n", "CP A"])] // CP A, n
#[case::rst_38h(0x29f9, &[0xff], &["decode_unprefixed", "RST y*8", "38h"])]
// RST 38h

// ------------------------------------------------------------------------------
// ED TABLE (MISC INSTRUCTIONS)
// ------------------------------------------------------------------------------
#[case::in_b_c(0x29f9, &[0xed, 0x40], &["decode_ed", "IN r[y], (C)", "B"])] // IN B, (C)
#[case::out_c_b(0x29f9, &[0xed, 0x41], &["decode_ed", "OUT (C), r[y]", "B"])] // OUT (C), B
#[case::sbc_hl_bc(0x29f9, &[0xed, 0x42], &["decode_ed", "SBC HL, rp[p]", "BC"])] // SBC HL, BC
#[case::ld_nni_bc_a(0x29f9, &[0xed, 0x43], &["decode_ed", "LD (nn), rp[p]", "BC"])] // LD (nn), BC
#[case::neg(0x29f9, &[0xed, 0x44], &["decode_ed", "NEG"])] // NEG
#[case::retn(0x29f9, &[0xed, 0x45], &["decode_ed", "RETN"])] // RETN
#[case::im_0(0x29f9, &[0xed, 0x46], &["decode_ed", "IM 0"])] // IM 0
#[case::ld_i_a(0x29f9, &[0xed, 0x47], &["decode_ed", "LD I, A"])] // LD I, A
#[case::in_c_c(0x29f9, &[0xed, 0x48], &["decode_ed", "IN r[y], (C)", "C"])] // IN C, (C)
#[case::out_c_c(0x29f9, &[0xed, 0x49], &["decode_ed", "OUT (C), r[y]", "C"])] // OUT (C), C
#[case::adc_hl_bc(0x29f9, &[0xed, 0x4a], &["decode_ed", "ADC HL, rp[p]", "BC"])] // ADC HL, BC
#[case::ld_bc_nni(0x29f9, &[0xed, 0x4b], &["decode_ed", "LD rp[p], (nn)", "BC"])] // LD BC, (nn)
#[case::ed_4c(0x29f9, &[0xed, 0x4c], &["decode_ed", "NONI"])] // NONI
#[case::reti(0x29f9, &[0xed, 0x4d], &["decode_ed", "RETI"])] // RETI
#[case::ed_4e(0x29f9, &[0xed, 0x4e], &["decode_ed", "NONI"])] // NONI
#[case::ld_r_a(0x29f9, &[0xed, 0x4f], &["decode_ed", "LD R, A"])] // LD R, A
#[case::in_d_c(0x29f9, &[0xed, 0x50], &["decode_ed", "IN r[y], (C)", "D"])] // IN D, (C)
#[case::out_c_d(0x29f9, &[0xed, 0x51], &["decode_ed", "OUT (C), r[y]", "D"])] // OUT (C), D
#[case::sbc_hl_de(0x29f9, &[0xed, 0x52], &["decode_ed", "SBC HL, rp[p]", "DE"])] // SBC HL, DE
#[case::ld_nni_de_a(0x29f9, &[0xed, 0x53], &["decode_ed", "LD (nn), rp[p]", "DE"])] // LD (nn), DE
#[case::ed_54(0x29f9, &[0xed, 0x54], &["decode_ed", "NONI"])] // NONI
#[case::ed_55(0x29f9, &[0xed, 0x55], &["decode_ed", "NONI"])] // NONI
#[case::im_1(0x29f9, &[0xed, 0x56], &["decode_ed", "IM 1"])] // IM 1
#[case::ld_a_i(0x29f9, &[0xed, 0x57], &["decode_ed", "LD A, I"])] // LD A, I
#[case::in_e_c(0x29f9, &[0xed, 0x58], &["decode_ed", "IN r[y], (C)", "E"])] // IN E, (C)
#[case::out_c_e(0x29f9, &[0xed, 0x59], &["decode_ed", "OUT (C), r[y]", "E"])] // OUT (C), E
#[case::adc_hl_de(0x29f9, &[0xed, 0x5a], &["decode_ed", "ADC HL, rp[p]", "DE"])] // ADC HL, DE
#[case::ld_de_nni(0x29f9, &[0xed, 0x5b], &["decode_ed", "LD rp[p], (nn)", "DE"])] // LD DE, (nn)
#[case::ed_5c(0x29f9, &[0xed, 0x5c], &["decode_ed", "NONI"])] // NONI
#[case::ed_5d(0x29f9, &[0xed, 0x5d], &["decode_ed", "NONI"])] // NONI
#[case::im_2(0x29f9, &[0xed, 0x5e], &["decode_ed", "IM 2"])] // IM 2
#[case::ld_a_r(0x29f9, &[0xed, 0x5f], &["decode_ed", "LD A, R"])] // LD A, R
#[case::in_h_c(0x29f9, &[0xed, 0x60], &["decode_ed", "IN r[y], (C)", "H"])] // IN H, (C)
#[case::out_c_h(0x29f9, &[0xed, 0x61], &["decode_ed", "OUT (C), r[y]", "H"])] // OUT (C), H
#[case::sbc_hl_hl(0x29f9, &[0xed, 0x62], &["decode_ed", "SBC HL, rp[p]", "HL"])] // SBC HL, HL
#[case::ld_nni_hl_a(0x29f9, &[0xed, 0x63], &["decode_ed", "LD (nn), rp[p]", "HL"])] // LD (nn), HL
#[case::ed_64(0x29f9, &[0xed, 0x64], &["decode_ed", "NONI"])] // NONI
#[case::ed_65(0x29f9, &[0xed, 0x65], &["decode_ed", "NONI"])] // NONI
#[case::ed_66(0x29f9, &[0xed, 0x66], &["decode_ed", "NONI"])] // NONI
#[case::rrd(0x29f9, &[0xed, 0x67], &["decode_ed", "RRD"])] // RRD
#[case::in_l_c(0x29f9, &[0xed, 0x68], &["decode_ed", "IN r[y], (C)", "L"])] // IN L, (C)
#[case::out_c_l(0x29f9, &[0xed, 0x69], &["decode_ed", "OUT (C), r[y]", "L"])] // OUT (C), L
#[case::adc_hl_hl(0x29f9, &[0xed, 0x6a], &["decode_ed", "ADC HL, rp[p]", "HL"])] // ADC HL, HL
#[case::ld_hl_nni(0x29f9, &[0xed, 0x6b], &["decode_ed", "LD rp[p], (nn)", "HL"])] // LD HL, (nn)
#[case::ed_6c(0x29f9, &[0xed, 0x6c], &["decode_ed", "NONI"])] // NONI
#[case::ed_6d(0x29f9, &[0xed, 0x6d], &["decode_ed", "NONI"])] // NONI
#[case::ed_6e(0x29f9, &[0xed, 0x6e], &["decode_ed", "NONI"])] // NONI
#[case::rld(0x29f9, &[0xed, 0x6f], &["decode_ed", "RLD"])] // RLD
#[case::in_c(0x29f9, &[0xed, 0x70], &["decode_ed", "IN (C)"])] // IN (C)
#[case::out_c_0(0x29f9, &[0xed, 0x71], &["decode_ed", "OUT (C), 0"])] // OUT (C), 0
#[case::sbc_hl_sp(0x29f9, &[0xed, 0x72], &["decode_ed", "SBC HL, rp[p]", "SP"])] // SBC HL, SP
#[case::ld_nni_sp_a(0x29f9, &[0xed, 0x73], &["decode_ed", "LD (nn), rp[p]", "SP"])] // LD (nn), SP
#[case::ed_74(0x29f9, &[0xed, 0x74], &["decode_ed", "NONI"])] // NONI
#[case::ed_75(0x29f9, &[0xed, 0x75], &["decode_ed", "NONI"])] // NONI
#[case::ed_76(0x29f9, &[0xed, 0x76], &["decode_ed", "NONI"])] // NONI
#[case::ed_77(0x29f9, &[0xed, 0x77], &["decode_ed", "NONI"])] // NONI
#[case::in_a_c(0x29f9, &[0xed, 0x78], &["decode_ed", "IN r[y], (C)", "A"])] // IN A, (C)
#[case::out_c_a(0x29f9, &[0xed, 0x79], &["decode_ed", "OUT (C), r[y]", "A"])] // OUT (C), A
#[case::adc_hl_sp(0x29f9, &[0xed, 0x7a], &["decode_ed", "ADC HL, rp[p]", "SP"])] // ADC HL, SP
#[case::ld_sp_nni(0x29f9, &[0xed, 0x7b], &["decode_ed", "LD rp[p], (nn)", "SP"])] // LD SP, (nn)
#[case::ldi(0x29f9, &[0xed, 0xa0], &["decode_ed", "bli[y, z]", "LDI"])] // LDI
#[case::cpi(0x29f9, &[0xed, 0xa1], &["decode_ed", "bli[y, z]", "CPI"])] // CPI
#[case::ini(0x29f9, &[0xed, 0xa2], &["decode_ed", "bli[y, z]", "INI"])] // INI
#[case::outi(0x29f9, &[0xed, 0xa3], &["decode_ed", "bli[y, z]", "OUTI"])] // OUTI
#[case::ldd(0x29f9, &[0xed, 0xa8], &["decode_ed", "bli[y, z]", "LDD"])] // LDD
#[case::cpd(0x29f9, &[0xed, 0xa9], &["decode_ed", "bli[y, z]", "CPD"])] // CPD
#[case::ind(0x29f9, &[0xed, 0xaa], &["decode_ed", "bli[y, z]", "IND"])] // IND
#[case::outd(0x29f9, &[0xed, 0xab], &["decode_ed", "bli[y, z]", "OUTD"])] // OUTD
#[case::ldir(0x29f9, &[0xed, 0xb0], &["decode_ed", "bli[y, z]", "LDIR"])] // LDIR
#[case::cpir(0x29f9, &[0xed, 0xb1], &["decode_ed", "bli[y, z]", "CPIR"])] // CPIR
#[case::inir(0x29f9, &[0xed, 0xb2], &["decode_ed", "bli[y, z]", "INIR"])] // INIR
#[case::outir(0x29f9, &[0xed, 0xb3], &["decode_ed", "bli[y, z]", "OTIR"])]
// OUTIR

// ------------------------------------------------------------------------
// CB TABLE (BIT INSTRUCTIONS)
// ------------------------------------------------------------------------
#[case::rlc_b(0x29f9, &[0xcb, 0x00], &["decode_cb", "rot[y] r[z]", "RLC", "B"])] // RLC B
#[case::rlc_c(0x29f9, &[0xcb, 0x01], &["decode_cb", "rot[y] r[z]", "RLC", "C"])] // RLC C
#[case::rlc_d(0x29f9, &[0xcb, 0x02], &["decode_cb", "rot[y] r[z]", "RLC", "D"])] // RLC D
#[case::rlc_e(0x29f9, &[0xcb, 0x03], &["decode_cb", "rot[y] r[z]", "RLC", "E"])] // RLC E
#[case::rlc_h(0x29f9, &[0xcb, 0x04], &["decode_cb", "rot[y] r[z]", "RLC", "H"])] // RLC H
#[case::rlc_l(0x29f9, &[0xcb, 0x05], &["decode_cb", "rot[y] r[z]", "RLC", "L"])] // RLC L
#[case::rlc_hli(0x29f9, &[0xcb, 0x06], &["decode_cb", "rot[y] r[z]", "RLC", "(HL)"])] // RLC (HL)
#[case::rlc_a(0x29f9, &[0xcb, 0x07], &["decode_cb", "rot[y] r[z]", "RLC", "A"])] // RLC A
#[case::rrc_b(0x29f9, &[0xcb, 0x08], &["decode_cb", "rot[y] r[z]", "RRC", "B"])] // RRC B
#[case::rrc_c(0x29f9, &[0xcb, 0x09], &["decode_cb", "rot[y] r[z]", "RRC", "C"])] // RRC C
#[case::rrc_d(0x29f9, &[0xcb, 0x0A], &["decode_cb", "rot[y] r[z]", "RRC", "D"])] // RRC D
#[case::rrc_e(0x29f9, &[0xcb, 0x0B], &["decode_cb", "rot[y] r[z]", "RRC", "E"])] // RRC E
#[case::rrc_h(0x29f9, &[0xcb, 0x0C], &["decode_cb", "rot[y] r[z]", "RRC", "H"])] // RRC H
#[case::rrc_l(0x29f9, &[0xcb, 0x0D], &["decode_cb", "rot[y] r[z]", "RRC", "L"])] // RRC L
#[case::rrc_hli(0x29f9, &[0xcb, 0x0E], &["decode_cb", "rot[y] r[z]", "RRC", "(HL)"])] // RRC (HL)
#[case::rrc_a(0x29f9, &[0xcb, 0x0F], &["decode_cb", "rot[y] r[z]", "RRC", "A"])] // RRC A
#[case::rl_b(0x29f9, &[0xcb, 0x10], &["decode_cb", "rot[y] r[z]", "RL", "B"])] // RL B
#[case::rl_c(0x29f9, &[0xcb, 0x11], &["decode_cb", "rot[y] r[z]", "RL", "C"])] // RL C
#[case::rl_d(0x29f9, &[0xcb, 0x12], &["decode_cb", "rot[y] r[z]", "RL", "D"])] // RL D
#[case::rl_e(0x29f9, &[0xcb, 0x13], &["decode_cb", "rot[y] r[z]", "RL", "E"])] // RL E
#[case::rl_h(0x29f9, &[0xcb, 0x14], &["decode_cb", "rot[y] r[z]", "RL", "H"])] // RL H
#[case::rl_l(0x29f9, &[0xcb, 0x15], &["decode_cb", "rot[y] r[z]", "RL", "L"])] // RL L
#[case::rl_hli(0x29f9, &[0xcb, 0x16], &["decode_cb", "rot[y] r[z]", "RL", "(HL)"])] // RL (HL)
#[case::rl_a(0x29f9, &[0xcb, 0x17], &["decode_cb", "rot[y] r[z]", "RL", "A"])] // RL A
#[case::rr_b(0x29f9, &[0xcb, 0x18], &["decode_cb", "rot[y] r[z]", "RR", "B"])] // RR B
#[case::rr_c(0x29f9, &[0xcb, 0x19], &["decode_cb", "rot[y] r[z]", "RR", "C"])] // RR C
#[case::rr_d(0x29f9, &[0xcb, 0x1A], &["decode_cb", "rot[y] r[z]", "RR", "D"])] // RR D
#[case::rr_e(0x29f9, &[0xcb, 0x1B], &["decode_cb", "rot[y] r[z]", "RR", "E"])] // RR E
#[case::rr_h(0x29f9, &[0xcb, 0x1C], &["decode_cb", "rot[y] r[z]", "RR", "H"])] // RR H
#[case::rr_l(0x29f9, &[0xcb, 0x1D], &["decode_cb", "rot[y] r[z]", "RR", "L"])] // RR L
#[case::rr_hli(0x29f9, &[0xcb, 0x1E], &["decode_cb", "rot[y] r[z]", "RR", "(HL)"])] // RR (HL)
#[case::rr_a(0x29f9, &[0xcb, 0x1F], &["decode_cb", "rot[y] r[z]", "RR", "A"])] // RR A
#[case::sla_b(0x29f9, &[0xcb, 0x20], &["decode_cb", "rot[y] r[z]", "SLA", "B"])] // SLA B
#[case::sla_c(0x29f9, &[0xcb, 0x21], &["decode_cb", "rot[y] r[z]", "SLA", "C"])] // SLA C
#[case::sla_d(0x29f9, &[0xcb, 0x22], &["decode_cb", "rot[y] r[z]", "SLA", "D"])] // SLA D
#[case::sla_e(0x29f9, &[0xcb, 0x23], &["decode_cb", "rot[y] r[z]", "SLA", "E"])] // SLA E
#[case::sla_h(0x29f9, &[0xcb, 0x24], &["decode_cb", "rot[y] r[z]", "SLA", "H"])] // SLA H
#[case::sla_l(0x29f9, &[0xcb, 0x25], &["decode_cb", "rot[y] r[z]", "SLA", "L"])] // SLA L
#[case::sla_hli(0x29f9, &[0xcb, 0x26], &["decode_cb", "rot[y] r[z]", "SLA", "(HL)"])] // SLA (HL)
#[case::sla_a(0x29f9, &[0xcb, 0x27], &["decode_cb", "rot[y] r[z]", "SLA", "A"])] // SLA A
#[case::sra_b(0x29f9, &[0xcb, 0x28], &["decode_cb", "rot[y] r[z]", "SRA", "B"])] // SRA B
#[case::sra_c(0x29f9, &[0xcb, 0x29], &["decode_cb", "rot[y] r[z]", "SRA", "C"])] // SRA C
#[case::sra_d(0x29f9, &[0xcb, 0x2A], &["decode_cb", "rot[y] r[z]", "SRA", "D"])] // SRA D
#[case::sra_e(0x29f9, &[0xcb, 0x2B], &["decode_cb", "rot[y] r[z]", "SRA", "E"])] // SRA E
#[case::sra_h(0x29f9, &[0xcb, 0x2C], &["decode_cb", "rot[y] r[z]", "SRA", "H"])] // SRA H
#[case::sra_l(0x29f9, &[0xcb, 0x2D], &["decode_cb", "rot[y] r[z]", "SRA", "L"])] // SRA L
#[case::sra_hli(0x29f9, &[0xcb, 0x2E], &["decode_cb", "rot[y] r[z]", "SRA", "(HL)"])] // SRA (HL)
#[case::sra_a(0x29f9, &[0xcb, 0x2F], &["decode_cb", "rot[y] r[z]", "SRA", "A"])] // SRA A
#[case::sll_b(0x29f9, &[0xcb, 0x30], &["decode_cb", "rot[y] r[z]", "SLL", "B"])] // SLL B
#[case::sll_c(0x29f9, &[0xcb, 0x31], &["decode_cb", "rot[y] r[z]", "SLL", "C"])] // SLL C
#[case::sll_d(0x29f9, &[0xcb, 0x32], &["decode_cb", "rot[y] r[z]", "SLL", "D"])] // SLL D
#[case::sll_e(0x29f9, &[0xcb, 0x33], &["decode_cb", "rot[y] r[z]", "SLL", "E"])] // SLL E
#[case::sll_h(0x29f9, &[0xcb, 0x34], &["decode_cb", "rot[y] r[z]", "SLL", "H"])] // SLL H
#[case::sll_l(0x29f9, &[0xcb, 0x35], &["decode_cb", "rot[y] r[z]", "SLL", "L"])] // SLL L
#[case::sll_hli(0x29f9, &[0xcb, 0x36], &["decode_cb", "rot[y] r[z]", "SLL", "(HL)"])] // SLL (HL)
#[case::sll_a(0x29f9, &[0xcb, 0x37], &["decode_cb", "rot[y] r[z]", "SLL", "A"])] // SLL A
#[case::srl_b(0x29f9, &[0xcb, 0x38], &["decode_cb", "rot[y] r[z]", "SRL", "B"])] // SRL B
#[case::srl_c(0x29f9, &[0xcb, 0x39], &["decode_cb", "rot[y] r[z]", "SRL", "C"])] // SRL C
#[case::srl_d(0x29f9, &[0xcb, 0x3A], &["decode_cb", "rot[y] r[z]", "SRL", "D"])] // SRL D
#[case::srl_e(0x29f9, &[0xcb, 0x3B], &["decode_cb", "rot[y] r[z]", "SRL", "E"])] // SRL E
#[case::srl_h(0x29f9, &[0xcb, 0x3C], &["decode_cb", "rot[y] r[z]", "SRL", "H"])] // SRL H
#[case::srl_l(0x29f9, &[0xcb, 0x3D], &["decode_cb", "rot[y] r[z]", "SRL", "L"])] // SRL L
#[case::srl_hli(0x29f9, &[0xcb, 0x3E], &["decode_cb", "rot[y] r[z]", "SRL", "(HL)"])] // SRL (HL)
#[case::srl_a(0x29f9, &[0xcb, 0x3F], &["decode_cb", "rot[y] r[z]", "SRL", "A"])] // SRL A
#[case::bit_0_b(0x29f9, &[0xcb, 0x40], &["decode_cb", "BIT y, r[z]", "0", "B"])] // BIT 0, B
#[case::bit_0_c(0x29f9, &[0xcb, 0x41], &["decode_cb", "BIT y, r[z]", "0", "C"])] // BIT 0, C
#[case::bit_0_d(0x29f9, &[0xcb, 0x42], &["decode_cb", "BIT y, r[z]", "0", "D"])] // BIT 0, D
#[case::bit_0_e(0x29f9, &[0xcb, 0x43], &["decode_cb", "BIT y, r[z]", "0", "E"])] // BIT 0, E
#[case::bit_0_h(0x29f9, &[0xcb, 0x44], &["decode_cb", "BIT y, r[z]", "0", "H"])] // BIT 0, H
#[case::bit_0_l(0x29f9, &[0xcb, 0x45], &["decode_cb", "BIT y, r[z]", "0", "L"])] // BIT 0, L
#[case::bit_0_hli(0x29f9, &[0xcb, 0x46], &["decode_cb", "BIT y, r[z]", "0", "(HL)"])] // BIT 0, (HL)
#[case::bit_0_a(0x29f9, &[0xcb, 0x47], &["decode_cb", "BIT y, r[z]", "0", "A"])] // BIT 0, A
#[case::bit_1_b(0x29f9, &[0xcb, 0x48], &["decode_cb", "BIT y, r[z]", "1", "B"])] // BIT 1, B
#[case::bit_1_c(0x29f9, &[0xcb, 0x49], &["decode_cb", "BIT y, r[z]", "1", "C"])] // BIT 1, C
#[case::bit_1_d(0x29f9, &[0xcb, 0x4A], &["decode_cb", "BIT y, r[z]", "1", "D"])] // BIT 1, D
#[case::bit_1_e(0x29f9, &[0xcb, 0x4B], &["decode_cb", "BIT y, r[z]", "1", "E"])] // BIT 1, E
#[case::bit_1_h(0x29f9, &[0xcb, 0x4C], &["decode_cb", "BIT y, r[z]", "1", "H"])] // BIT 1, H
#[case::bit_1_l(0x29f9, &[0xcb, 0x4D], &["decode_cb", "BIT y, r[z]", "1", "L"])] // BIT 1, L
#[case::bit_1_hli(0x29f9, &[0xcb, 0x4E], &["decode_cb", "BIT y, r[z]", "1", "(HL)"])] // BIT 1, (HL)
#[case::bit_1_a(0x29f9, &[0xcb, 0x4F], &["decode_cb", "BIT y, r[z]", "1", "A"])] // BIT 1, A
#[case::bit_2_b(0x29f9, &[0xcb, 0x50], &["decode_cb", "BIT y, r[z]", "2", "B"])] // BIT 2, B
#[case::bit_2_c(0x29f9, &[0xcb, 0x51], &["decode_cb", "BIT y, r[z]", "2", "C"])] // BIT 2, C
#[case::bit_2_d(0x29f9, &[0xcb, 0x52], &["decode_cb", "BIT y, r[z]", "2", "D"])] // BIT 2, D
#[case::bit_2_e(0x29f9, &[0xcb, 0x53], &["decode_cb", "BIT y, r[z]", "2", "E"])] // BIT 2, E
#[case::bit_2_h(0x29f9, &[0xcb, 0x54], &["decode_cb", "BIT y, r[z]", "2", "H"])] // BIT 2, H
#[case::bit_2_l(0x29f9, &[0xcb, 0x55], &["decode_cb", "BIT y, r[z]", "2", "L"])] // BIT 2, L
#[case::bit_2_hli(0x29f9, &[0xcb, 0x56], &["decode_cb", "BIT y, r[z]", "2", "(HL)"])] // BIT 2, (HL)
#[case::bit_2_a(0x29f9, &[0xcb, 0x57], &["decode_cb", "BIT y, r[z]", "2", "A"])] // BIT 2, A
#[case::bit_3_b(0x29f9, &[0xcb, 0x58], &["decode_cb", "BIT y, r[z]", "3", "B"])] // BIT 3, B
#[case::bit_3_c(0x29f9, &[0xcb, 0x59], &["decode_cb", "BIT y, r[z]", "3", "C"])] // BIT 3, C
#[case::bit_3_d(0x29f9, &[0xcb, 0x5A], &["decode_cb", "BIT y, r[z]", "3", "D"])] // BIT 3, D
#[case::bit_3_e(0x29f9, &[0xcb, 0x5B], &["decode_cb", "BIT y, r[z]", "3", "E"])] // BIT 3, E
#[case::bit_3_h(0x29f9, &[0xcb, 0x5C], &["decode_cb", "BIT y, r[z]", "3", "H"])] // BIT 3, H
#[case::bit_3_l(0x29f9, &[0xcb, 0x5D], &["decode_cb", "BIT y, r[z]", "3", "L"])] // BIT 3, L
#[case::bit_3_hli(0x29f9, &[0xcb, 0x5E], &["decode_cb", "BIT y, r[z]", "3", "(HL)"])] // BIT 3, (HL)
#[case::bit_3_a(0x29f9, &[0xcb, 0x5F], &["decode_cb", "BIT y, r[z]", "3", "A"])] // BIT 3, A
#[case::bit_4_b(0x29f9, &[0xcb, 0x60], &["decode_cb", "BIT y, r[z]", "4", "B"])] // BIT 4, B
#[case::bit_4_c(0x29f9, &[0xcb, 0x61], &["decode_cb", "BIT y, r[z]", "4", "C"])] // BIT 4, C
#[case::bit_4_d(0x29f9, &[0xcb, 0x62], &["decode_cb", "BIT y, r[z]", "4", "D"])] // BIT 4, D
#[case::bit_4_e(0x29f9, &[0xcb, 0x63], &["decode_cb", "BIT y, r[z]", "4", "E"])] // BIT 4, E
#[case::bit_4_h(0x29f9, &[0xcb, 0x64], &["decode_cb", "BIT y, r[z]", "4", "H"])] // BIT 4, H
#[case::bit_4_l(0x29f9, &[0xcb, 0x65], &["decode_cb", "BIT y, r[z]", "4", "L"])] // BIT 4, L
#[case::bit_4_hli(0x29f9, &[0xcb, 0x66], &["decode_cb", "BIT y, r[z]", "4", "(HL)"])] // BIT 4, (HL)
#[case::bit_4_a(0x29f9, &[0xcb, 0x67], &["decode_cb", "BIT y, r[z]", "4", "A"])] // BIT 4, A
#[case::bit_5_b(0x29f9, &[0xcb, 0x68], &["decode_cb", "BIT y, r[z]", "5", "B"])] // BIT 5, B
#[case::bit_5_c(0x29f9, &[0xcb, 0x69], &["decode_cb", "BIT y, r[z]", "5", "C"])] // BIT 5, C
#[case::bit_5_d(0x29f9, &[0xcb, 0x6A], &["decode_cb", "BIT y, r[z]", "5", "D"])] // BIT 5, D
#[case::bit_5_e(0x29f9, &[0xcb, 0x6B], &["decode_cb", "BIT y, r[z]", "5", "E"])] // BIT 5, E
#[case::bit_5_h(0x29f9, &[0xcb, 0x6C], &["decode_cb", "BIT y, r[z]", "5", "H"])] // BIT 5, H
#[case::bit_5_l(0x29f9, &[0xcb, 0x6D], &["decode_cb", "BIT y, r[z]", "5", "L"])] // BIT 5, L
#[case::bit_5_hli(0x29f9, &[0xcb, 0x6E], &["decode_cb", "BIT y, r[z]", "5", "(HL)"])] // BIT 5, (HL)
#[case::bit_5_a(0x29f9, &[0xcb, 0x6F], &["decode_cb", "BIT y, r[z]", "5", "A"])] // BIT 5, A
#[case::bit_6_b(0x29f9, &[0xcb, 0x70], &["decode_cb", "BIT y, r[z]", "6", "B"])] // BIT 6, B
#[case::bit_6_c(0x29f9, &[0xcb, 0x71], &["decode_cb", "BIT y, r[z]", "6", "C"])] // BIT 6, C
#[case::bit_6_d(0x29f9, &[0xcb, 0x72], &["decode_cb", "BIT y, r[z]", "6", "D"])] // BIT 6, D
#[case::bit_6_e(0x29f9, &[0xcb, 0x73], &["decode_cb", "BIT y, r[z]", "6", "E"])] // BIT 6, E
#[case::bit_6_h(0x29f9, &[0xcb, 0x74], &["decode_cb", "BIT y, r[z]", "6", "H"])] // BIT 6, H
#[case::bit_6_l(0x29f9, &[0xcb, 0x75], &["decode_cb", "BIT y, r[z]", "6", "L"])] // BIT 6, L
#[case::bit_6_hli(0x29f9, &[0xcb, 0x76], &["decode_cb", "BIT y, r[z]", "6", "(HL)"])] // BIT 6, (HL)
#[case::bit_6_a(0x29f9, &[0xcb, 0x77], &["decode_cb", "BIT y, r[z]", "6", "A"])] // BIT 6, A
#[case::bit_7_b(0x29f9, &[0xcb, 0x78], &["decode_cb", "BIT y, r[z]", "7", "B"])] // BIT 7, B
#[case::bit_7_c(0x29f9, &[0xcb, 0x79], &["decode_cb", "BIT y, r[z]", "7", "C"])] // BIT 7, C
#[case::bit_7_d(0x29f9, &[0xcb, 0x7A], &["decode_cb", "BIT y, r[z]", "7", "D"])] // BIT 7, D
#[case::bit_7_e(0x29f9, &[0xcb, 0x7B], &["decode_cb", "BIT y, r[z]", "7", "E"])] // BIT 7, E
#[case::bit_7_h(0x29f9, &[0xcb, 0x7C], &["decode_cb", "BIT y, r[z]", "7", "H"])] // BIT 7, H
#[case::bit_7_l(0x29f9, &[0xcb, 0x7D], &["decode_cb", "BIT y, r[z]", "7", "L"])] // BIT 7, L
#[case::bit_7_hli(0x29f9, &[0xcb, 0x7E], &["decode_cb", "BIT y, r[z]", "7", "(HL)"])] // BIT 7, (HL)
#[case::bit_7_a(0x29f9, &[0xcb, 0x7F], &["decode_cb", "BIT y, r[z]", "7", "A"])] // BIT 7, A
#[case::res_0_b(0x29f9, &[0xcb, 0x80], &["decode_cb", "RES y, r[z]", "0", "B"])] // RES 0, B
#[case::res_0_c(0x29f9, &[0xcb, 0x81], &["decode_cb", "RES y, r[z]", "0", "C"])] // RES 0, C
#[case::res_0_d(0x29f9, &[0xcb, 0x82], &["decode_cb", "RES y, r[z]", "0", "D"])] // RES 0, D
#[case::res_0_e(0x29f9, &[0xcb, 0x83], &["decode_cb", "RES y, r[z]", "0", "E"])] // RES 0, E
#[case::res_0_h(0x29f9, &[0xcb, 0x84], &["decode_cb", "RES y, r[z]", "0", "H"])] // RES 0, H
#[case::res_0_l(0x29f9, &[0xcb, 0x85], &["decode_cb", "RES y, r[z]", "0", "L"])] // RES 0, L
#[case::res_0_hli(0x29f9, &[0xcb, 0x86], &["decode_cb", "RES y, r[z]", "0", "(HL)"])] // RES 0, (HL)
#[case::res_0_a(0x29f9, &[0xcb, 0x87], &["decode_cb", "RES y, r[z]", "0", "A"])] // RES 0, A
#[case::res_1_b(0x29f9, &[0xcb, 0x88], &["decode_cb", "RES y, r[z]", "1", "B"])] // RES 1, B
#[case::res_1_c(0x29f9, &[0xcb, 0x89], &["decode_cb", "RES y, r[z]", "1", "C"])] // RES 1, C
#[case::res_1_d(0x29f9, &[0xcb, 0x8A], &["decode_cb", "RES y, r[z]", "1", "D"])] // RES 1, D
#[case::res_1_e(0x29f9, &[0xcb, 0x8B], &["decode_cb", "RES y, r[z]", "1", "E"])] // RES 1, E
#[case::res_1_h(0x29f9, &[0xcb, 0x8C], &["decode_cb", "RES y, r[z]", "1", "H"])] // RES 1, H
#[case::res_1_l(0x29f9, &[0xcb, 0x8D], &["decode_cb", "RES y, r[z]", "1", "L"])] // RES 1, L
#[case::res_1_hli(0x29f9, &[0xcb, 0x8E], &["decode_cb", "RES y, r[z]", "1", "(HL)"])] // RES 1, (HL)
#[case::res_1_a(0x29f9, &[0xcb, 0x8F], &["decode_cb", "RES y, r[z]", "1", "A"])] // RES 1, A
#[case::res_2_b(0x29f9, &[0xcb, 0x90], &["decode_cb", "RES y, r[z]", "2", "B"])] // RES 2, B
#[case::res_2_c(0x29f9, &[0xcb, 0x91], &["decode_cb", "RES y, r[z]", "2", "C"])] // RES 2, C
#[case::res_2_d(0x29f9, &[0xcb, 0x92], &["decode_cb", "RES y, r[z]", "2", "D"])] // RES 2, D
#[case::res_2_e(0x29f9, &[0xcb, 0x93], &["decode_cb", "RES y, r[z]", "2", "E"])] // RES 2, E
#[case::res_2_h(0x29f9, &[0xcb, 0x94], &["decode_cb", "RES y, r[z]", "2", "H"])] // RES 2, H
#[case::res_2_l(0x29f9, &[0xcb, 0x95], &["decode_cb", "RES y, r[z]", "2", "L"])] // RES 2, L
#[case::res_2_hli(0x29f9, &[0xcb, 0x96], &["decode_cb", "RES y, r[z]", "2", "(HL)"])] // RES 2, (HL)
#[case::res_2_a(0x29f9, &[0xcb, 0x97], &["decode_cb", "RES y, r[z]", "2", "A"])] // RES 2, A
#[case::res_3_b(0x29f9, &[0xcb, 0x98], &["decode_cb", "RES y, r[z]", "3", "B"])] // RES 3, B
#[case::res_3_c(0x29f9, &[0xcb, 0x99], &["decode_cb", "RES y, r[z]", "3", "C"])] // RES 3, C
#[case::res_3_d(0x29f9, &[0xcb, 0x9A], &["decode_cb", "RES y, r[z]", "3", "D"])] // RES 3, D
#[case::res_3_e(0x29f9, &[0xcb, 0x9B], &["decode_cb", "RES y, r[z]", "3", "E"])] // RES 3, E
#[case::res_3_h(0x29f9, &[0xcb, 0x9C], &["decode_cb", "RES y, r[z]", "3", "H"])] // RES 3, H
#[case::res_3_l(0x29f9, &[0xcb, 0x9D], &["decode_cb", "RES y, r[z]", "3", "L"])] // RES 3, L
#[case::res_3_hli(0x29f9, &[0xcb, 0x9E], &["decode_cb", "RES y, r[z]", "3", "(HL)"])] // RES 3, (HL)
#[case::res_3_a(0x29f9, &[0xcb, 0x9F], &["decode_cb", "RES y, r[z]", "3", "A"])] // RES 3, A
#[case::res_4_b(0x29f9, &[0xcb, 0xA0], &["decode_cb", "RES y, r[z]", "4", "B"])] // RES 4, B
#[case::res_4_c(0x29f9, &[0xcb, 0xA1], &["decode_cb", "RES y, r[z]", "4", "C"])] // RES 4, C
#[case::res_4_d(0x29f9, &[0xcb, 0xA2], &["decode_cb", "RES y, r[z]", "4", "D"])] // RES 4, D
#[case::res_4_e(0x29f9, &[0xcb, 0xA3], &["decode_cb", "RES y, r[z]", "4", "E"])] // RES 4, E
#[case::res_4_h(0x29f9, &[0xcb, 0xA4], &["decode_cb", "RES y, r[z]", "4", "H"])] // RES 4, H
#[case::res_4_l(0x29f9, &[0xcb, 0xA5], &["decode_cb", "RES y, r[z]", "4", "L"])] // RES 4, L
#[case::res_4_hli(0x29f9, &[0xcb, 0xA6], &["decode_cb", "RES y, r[z]", "4", "(HL)"])] // RES 4, (HL)
#[case::res_4_a(0x29f9, &[0xcb, 0xA7], &["decode_cb", "RES y, r[z]", "4", "A"])] // RES 4, A
#[case::res_5_b(0x29f9, &[0xcb, 0xA8], &["decode_cb", "RES y, r[z]", "5", "B"])] // RES 5, B
#[case::res_5_c(0x29f9, &[0xcb, 0xA9], &["decode_cb", "RES y, r[z]", "5", "C"])] // RES 5, C
#[case::res_5_d(0x29f9, &[0xcb, 0xAA], &["decode_cb", "RES y, r[z]", "5", "D"])] // RES 5, D
#[case::res_5_e(0x29f9, &[0xcb, 0xAB], &["decode_cb", "RES y, r[z]", "5", "E"])] // RES 5, E
#[case::res_5_h(0x29f9, &[0xcb, 0xAC], &["decode_cb", "RES y, r[z]", "5", "H"])] // RES 5, H
#[case::res_5_l(0x29f9, &[0xcb, 0xAD], &["decode_cb", "RES y, r[z]", "5", "L"])] // RES 5, L
#[case::res_5_hli(0x29f9, &[0xcb, 0xAE], &["decode_cb", "RES y, r[z]", "5", "(HL)"])] // RES 5, (HL)
#[case::res_5_a(0x29f9, &[0xcb, 0xAF], &["decode_cb", "RES y, r[z]", "5", "A"])] // RES 5, A
#[case::res_6_b(0x29f9, &[0xcb, 0xB0], &["decode_cb", "RES y, r[z]", "6", "B"])] // RES 6, B
#[case::res_6_c(0x29f9, &[0xcb, 0xB1], &["decode_cb", "RES y, r[z]", "6", "C"])] // RES 6, C
#[case::res_6_d(0x29f9, &[0xcb, 0xB2], &["decode_cb", "RES y, r[z]", "6", "D"])] // RES 6, D
#[case::res_6_e(0x29f9, &[0xcb, 0xB3], &["decode_cb", "RES y, r[z]", "6", "E"])] // RES 6, E
#[case::res_6_h(0x29f9, &[0xcb, 0xB4], &["decode_cb", "RES y, r[z]", "6", "H"])] // RES 6, H
#[case::res_6_l(0x29f9, &[0xcb, 0xB5], &["decode_cb", "RES y, r[z]", "6", "L"])] // RES 6, L
#[case::res_6_hli(0x29f9, &[0xcb, 0xB6], &["decode_cb", "RES y, r[z]", "6", "(HL)"])] // RES 6, (HL)
#[case::res_6_a(0x29f9, &[0xcb, 0xB7], &["decode_cb", "RES y, r[z]", "6", "A"])] // RES 6, A
#[case::res_7_b(0x29f9, &[0xcb, 0xB8], &["decode_cb", "RES y, r[z]", "7", "B"])] // RES 7, B
#[case::res_7_c(0x29f9, &[0xcb, 0xB9], &["decode_cb", "RES y, r[z]", "7", "C"])] // RES 7, C
#[case::res_7_d(0x29f9, &[0xcb, 0xBA], &["decode_cb", "RES y, r[z]", "7", "D"])] // RES 7, D
#[case::res_7_e(0x29f9, &[0xcb, 0xBB], &["decode_cb", "RES y, r[z]", "7", "E"])] // RES 7, E
#[case::res_7_h(0x29f9, &[0xcb, 0xBC], &["decode_cb", "RES y, r[z]", "7", "H"])] // RES 7, H
#[case::res_7_l(0x29f9, &[0xcb, 0xBD], &["decode_cb", "RES y, r[z]", "7", "L"])] // RES 7, L
#[case::res_7_hli(0x29f9, &[0xcb, 0xBE], &["decode_cb", "RES y, r[z]", "7", "(HL)"])] // RES 7, (HL)
#[case::res_7_a(0x29f9, &[0xcb, 0xBF], &["decode_cb", "RES y, r[z]", "7", "A"])] // RES 7, A
#[case::set_0_b(0x29f9, &[0xcb, 0xC0], &["decode_cb", "SET y, r[z]", "0", "B"])] // SET 0, B
#[case::set_0_c(0x29f9, &[0xcb, 0xC1], &["decode_cb", "SET y, r[z]", "0", "C"])] // SET 0, C
#[case::set_0_d(0x29f9, &[0xcb, 0xC2], &["decode_cb", "SET y, r[z]", "0", "D"])] // SET 0, D
#[case::set_0_e(0x29f9, &[0xcb, 0xC3], &["decode_cb", "SET y, r[z]", "0", "E"])] // SET 0, E
#[case::set_0_h(0x29f9, &[0xcb, 0xC4], &["decode_cb", "SET y, r[z]", "0", "H"])] // SET 0, H
#[case::set_0_l(0x29f9, &[0xcb, 0xC5], &["decode_cb", "SET y, r[z]", "0", "L"])] // SET 0, L
#[case::set_0_hli(0x29f9, &[0xcb, 0xC6], &["decode_cb", "SET y, r[z]", "0", "(HL)"])] // SET 0, (HL)
#[case::set_0_a(0x29f9, &[0xcb, 0xC7], &["decode_cb", "SET y, r[z]", "0", "A"])] // SET 0, A
#[case::set_1_b(0x29f9, &[0xcb, 0xC8], &["decode_cb", "SET y, r[z]", "1", "B"])] // SET 1, B
#[case::set_1_c(0x29f9, &[0xcb, 0xC9], &["decode_cb", "SET y, r[z]", "1", "C"])] // SET 1, C
#[case::set_1_d(0x29f9, &[0xcb, 0xCA], &["decode_cb", "SET y, r[z]", "1", "D"])] // SET 1, D
#[case::set_1_e(0x29f9, &[0xcb, 0xCB], &["decode_cb", "SET y, r[z]", "1", "E"])] // SET 1, E
#[case::set_1_h(0x29f9, &[0xcb, 0xCC], &["decode_cb", "SET y, r[z]", "1", "H"])] // SET 1, H
#[case::set_1_l(0x29f9, &[0xcb, 0xCD], &["decode_cb", "SET y, r[z]", "1", "L"])] // SET 1, L
#[case::set_1_hli(0x29f9, &[0xcb, 0xCE], &["decode_cb", "SET y, r[z]", "1", "(HL)"])] // SET 1, (HL)
#[case::set_1_a(0x29f9, &[0xcb, 0xCF], &["decode_cb", "SET y, r[z]", "1", "A"])] // SET 1, A
#[case::set_2_b(0x29f9, &[0xcb, 0xD0], &["decode_cb", "SET y, r[z]", "2", "B"])] // SET 2, B
#[case::set_2_c(0x29f9, &[0xcb, 0xD1], &["decode_cb", "SET y, r[z]", "2", "C"])] // SET 2, C
#[case::set_2_d(0x29f9, &[0xcb, 0xD2], &["decode_cb", "SET y, r[z]", "2", "D"])] // SET 2, D
#[case::set_2_e(0x29f9, &[0xcb, 0xD3], &["decode_cb", "SET y, r[z]", "2", "E"])] // SET 2, E
#[case::set_2_h(0x29f9, &[0xcb, 0xD4], &["decode_cb", "SET y, r[z]", "2", "H"])] // SET 2, H
#[case::set_2_l(0x29f9, &[0xcb, 0xD5], &["decode_cb", "SET y, r[z]", "2", "L"])] // SET 2, L
#[case::set_2_hli(0x29f9, &[0xcb, 0xD6], &["decode_cb", "SET y, r[z]", "2", "(HL)"])] // SET 2, (HL)
#[case::set_2_a(0x29f9, &[0xcb, 0xD7], &["decode_cb", "SET y, r[z]", "2", "A"])] // SET 2, A
#[case::set_3_b(0x29f9, &[0xcb, 0xD8], &["decode_cb", "SET y, r[z]", "3", "B"])] // SET 3, B
#[case::set_3_c(0x29f9, &[0xcb, 0xD9], &["decode_cb", "SET y, r[z]", "3", "C"])] // SET 3, C
#[case::set_3_d(0x29f9, &[0xcb, 0xDA], &["decode_cb", "SET y, r[z]", "3", "D"])] // SET 3, D
#[case::set_3_e(0x29f9, &[0xcb, 0xDB], &["decode_cb", "SET y, r[z]", "3", "E"])] // SET 3, E
#[case::set_3_h(0x29f9, &[0xcb, 0xDC], &["decode_cb", "SET y, r[z]", "3", "H"])] // SET 3, H
#[case::set_3_l(0x29f9, &[0xcb, 0xDD], &["decode_cb", "SET y, r[z]", "3", "L"])] // SET 3, L
#[case::set_3_hli(0x29f9, &[0xcb, 0xDE], &["decode_cb", "SET y, r[z]", "3", "(HL)"])] // SET 3, (HL)
#[case::set_3_a(0x29f9, &[0xcb, 0xDF], &["decode_cb", "SET y, r[z]", "3", "A"])] // SET 3, A
#[case::set_4_b(0x29f9, &[0xcb, 0xE0], &["decode_cb", "SET y, r[z]", "4", "B"])] // SET 4, B
#[case::set_4_c(0x29f9, &[0xcb, 0xE1], &["decode_cb", "SET y, r[z]", "4", "C"])] // SET 4, C
#[case::set_4_d(0x29f9, &[0xcb, 0xE2], &["decode_cb", "SET y, r[z]", "4", "D"])] // SET 4, D
#[case::set_4_e(0x29f9, &[0xcb, 0xE3], &["decode_cb", "SET y, r[z]", "4", "E"])] // SET 4, E
#[case::set_4_h(0x29f9, &[0xcb, 0xE4], &["decode_cb", "SET y, r[z]", "4", "H"])] // SET 4, H
#[case::set_4_l(0x29f9, &[0xcb, 0xE5], &["decode_cb", "SET y, r[z]", "4", "L"])] // SET 4, L
#[case::set_4_hli(0x29f9, &[0xcb, 0xE6], &["decode_cb", "SET y, r[z]", "4", "(HL)"])] // SET 4, (HL)
#[case::set_4_a(0x29f9, &[0xcb, 0xE7], &["decode_cb", "SET y, r[z]", "4", "A"])] // SET 4, A
#[case::set_5_b(0x29f9, &[0xcb, 0xE8], &["decode_cb", "SET y, r[z]", "5", "B"])] // SET 5, B
#[case::set_5_c(0x29f9, &[0xcb, 0xE9], &["decode_cb", "SET y, r[z]", "5", "C"])] // SET 5, C
#[case::set_5_d(0x29f9, &[0xcb, 0xEA], &["decode_cb", "SET y, r[z]", "5", "D"])] // SET 5, D
#[case::set_5_e(0x29f9, &[0xcb, 0xEB], &["decode_cb", "SET y, r[z]", "5", "E"])] // SET 5, E
#[case::set_5_h(0x29f9, &[0xcb, 0xEC], &["decode_cb", "SET y, r[z]", "5", "H"])] // SET 5, H
#[case::set_5_l(0x29f9, &[0xcb, 0xED], &["decode_cb", "SET y, r[z]", "5", "L"])] // SET 5, L
#[case::set_5_hli(0x29f9, &[0xcb, 0xEE], &["decode_cb", "SET y, r[z]", "5", "(HL)"])] // SET 5, (HL)
#[case::set_5_a(0x29f9, &[0xcb, 0xEF], &["decode_cb", "SET y, r[z]", "5", "A"])] // SET 5, A
#[case::set_6_b(0x29f9, &[0xcb, 0xF0], &["decode_cb", "SET y, r[z]", "6", "B"])] // SET 6, B
#[case::set_6_c(0x29f9, &[0xcb, 0xF1], &["decode_cb", "SET y, r[z]", "6", "C"])] // SET 6, C
#[case::set_6_d(0x29f9, &[0xcb, 0xF2], &["decode_cb", "SET y, r[z]", "6", "D"])] // SET 6, D
#[case::set_6_e(0x29f9, &[0xcb, 0xF3], &["decode_cb", "SET y, r[z]", "6", "E"])] // SET 6, E
#[case::set_6_h(0x29f9, &[0xcb, 0xF4], &["decode_cb", "SET y, r[z]", "6", "H"])] // SET 6, H
#[case::set_6_l(0x29f9, &[0xcb, 0xF5], &["decode_cb", "SET y, r[z]", "6", "L"])] // SET 6, L
#[case::set_6_hli(0x29f9, &[0xcb, 0xF6], &["decode_cb", "SET y, r[z]", "6", "(HL)"])] // SET 6, (HL)
#[case::set_6_a(0x29f9, &[0xcb, 0xF7], &["decode_cb", "SET y, r[z]", "6", "A"])] // SET 6, A
#[case::set_7_b(0x29f9, &[0xcb, 0xF8], &["decode_cb", "SET y, r[z]", "7", "B"])] // SET 7, B
#[case::set_7_c(0x29f9, &[0xcb, 0xF9], &["decode_cb", "SET y, r[z]", "7", "C"])] // SET 7, C
#[case::set_7_d(0x29f9, &[0xcb, 0xFA], &["decode_cb", "SET y, r[z]", "7", "D"])] // SET 7, D
#[case::set_7_e(0x29f9, &[0xcb, 0xFB], &["decode_cb", "SET y, r[z]", "7", "E"])] // SET 7, E
#[case::set_7_h(0x29f9, &[0xcb, 0xFC], &["decode_cb", "SET y, r[z]", "7", "H"])] // SET 7, H
#[case::set_7_l(0x29f9, &[0xcb, 0xFD], &["decode_cb", "SET y, r[z]", "7", "L"])] // SET 7, L
#[case::set_7_hli(0x29f9, &[0xcb, 0xFE], &["decode_cb", "SET y, r[z]", "7", "(HL)"])] // SET 7, (HL)
#[case::set_7_a(0x29f9, &[0xcb, 0xFF], &["decode_cb", "SET y, r[z]", "7", "A"])]
// SET 7, A

// ----------------------------------------------------------------------------------------------------
// DD TABLE (IX INSTRUCTIONS)
// ----------------------------------------------------------------------------------------------------
// TODO: there are some instructions (undocumented obviously) that are stated as NOP in some places and as the "original" in others
// TODO: For now i wont test those cases.
#[case::dd_inc_b(0x29f9, &[0xdd, 0x04], &["decode_dd", "INC r[y]", "B"])] // INC B
#[case::dd_dec_b(0x29f9, &[0xdd, 0x05], &["decode_dd", "DEC r[y]", "B"])] // DEC B
#[case::dd_ld_b_n(0x29f9, &[0xdd, 0x06, 0x42], &["decode_dd", "LD r[y], n", "B"])] // LD B, n
#[case::dd_add_ix_bc(0x29f9, &[0xdd, 0x09], &["decode_dd", "ADD HL/IX/IY, rp[p]", "IX", "BC"])] // ADD IX, BC
#[case::dd_inc_c(0x29f9, &[0xdd, 0x0C], &["decode_dd", "INC r[y]", "C"])] // INC C
#[case::dd_dec_c(0x29f9, &[0xdd, 0x0D], &["decode_dd", "DEC r[y]", "C"])] // DEC C
#[case::dd_ld_c_n(0x29f9, &[0xdd, 0x0E, 0x42], &["decode_dd", "LD r[y], n", "C"])] // LD C, n
#[case::dd_inc_d(0x29f9, &[0xdd, 0x14], &["decode_dd", "INC r[y]", "D"])] // INC D
#[case::dd_dec_d(0x29f9, &[0xdd, 0x15], &["decode_dd", "DEC r[y]", "D"])] // DEC D
#[case::dd_ld_d_n(0x29f9, &[0xdd, 0x16, 0x42], &["decode_dd", "LD r[y], n", "D"])] // LD D, n
#[case::dd_add_ix_de(0x29f9, &[0xdd, 0x19], &["decode_dd", "ADD HL/IX/IY, rp[p]", "IX", "DE"])] // ADD IX, DE
#[case::dd_inc_e(0x29f9, &[0xdd, 0x1C], &["decode_dd", "INC r[y]", "E"])] // INC E
#[case::dd_dec_e(0x29f9, &[0xdd, 0x1D], &["decode_dd", "DEC r[y]", "E"])] // DEC E
#[case::dd_ld_e_n(0x29f9, &[0xdd, 0x1E, 0x42], &["decode_dd", "LD r[y], n", "E"])] // LD E, n
#[case::dd_ld_ix_nn(0x29f9, &[0xdd, 0x21, 0x34, 0x12], &["decode_dd", "LD rp[p], nn", "HL", "IX"])] // LD IX, nn
#[case::dd_ld_nni_ix(0x29f9, &[0xdd, 0x22, 0x34, 0x12], &["decode_dd", "LD (nn), HL/IX/IY", "IX"])] // LD (nn), IX
#[case::dd_inc_ix(0x29f9, &[0xdd, 0x23], &["decode_dd", "INC rp[p]", "HL", "IX"])] // INC IX
#[case::dd_inc_ixh(0x29f9, &[0xdd, 0x24], &["decode_dd", "INC r[y]", "H", "IXH"])] // INC IXH
#[case::dd_dec_ixh(0x29f9, &[0xdd, 0x25], &["decode_dd", "DEC r[y]", "H", "IXH"])] // DEC IXH
#[case::dd_ld_ixh_n(0x29f9, &[0xdd, 0x26, 0x42], &["decode_dd", "LD r[y], n", "H", "IXH"])] // LD IXH, n
#[case::dd_add_ix_ix(0x29f9, &[0xdd, 0x29], &["decode_dd", "ADD HL/IX/IY, rp[p]", "IX", "HL", "IX"])] // ADD IX, IX
#[case::dd_ld_ix_nni(0x29f9, &[0xdd, 0x2A, 0x34, 0x12], &["decode_dd", "LD HL/IX/IY, (nn)", "IX"])] // LD IX, (nn)
#[case::dd_dec_ix(0x29f9, &[0xdd, 0x2B], &["decode_dd", "DEC rp[p]", "HL", "IX"])] // DEC IX
#[case::dd_inc_ixl(0x29f9, &[0xdd, 0x2C], &["decode_dd", "INC r[y]", "L", "IXL"])] // INC IXL
#[case::dd_dec_ixl(0x29f9, &[0xdd, 0x2D], &["decode_dd", "DEC r[y]", "L", "IXL"])] // DEC IXL
#[case::dd_ld_ixl_n(0x29f9, &[0xdd, 0x2E, 0x42], &["decode_dd", "LD r[y], n", "L", "IXL"])] // LD IXL, n
#[case::dd_inc_ixd(0x29f9, &[0xdd, 0x34], &["decode_dd", "INC r[y]", "(HL)", "(IX + d)"])] // INC (IX + d)
#[case::dd_dec_ixd(0x29f9, &[0xdd, 0x35], &["decode_dd", "DEC r[y]", "(HL)", "(IX + d)"])] // DEC (IX + d)
#[case::dd_ld_ixd_n(0x29f9, &[0xdd, 0x36, 0x42], &["decode_dd", "LD r[y], n", "(HL)", "(IX + d)"])] // LD (IX + d), n
#[case::dd_add_ix_sp(0x29f9, &[0xdd, 0x39], &["decode_dd", "ADD HL/IX/IY, rp[p]", "IX", "SP"])] // ADD IX, SP
#[case::dd_inc_a(0x29f9, &[0xdd, 0x3C], &["decode_dd", "INC r[y]", "A"])] // INC A
#[case::dd_dec_a(0x29f9, &[0xdd, 0x3D], &["decode_dd", "DEC r[y]", "A"])] // DEC A
#[case::dd_ld_a_n(0x29f9, &[0xdd, 0x3E, 0x42], &["decode_dd", "LD r[y], n", "A"])] // LD A, n
#[case::dd_ld_b_b(0x29f9, &[0xdd, 0x40], &["decode_dd", "LD r[y], r[z]", "B", "B"])] // LD B, B
#[case::dd_ld_b_c(0x29f9, &[0xdd, 0x41], &["decode_dd", "LD r[y], r[z]", "B", "C"])] // LD B, C
#[case::dd_ld_b_d(0x29f9, &[0xdd, 0x42], &["decode_dd", "LD r[y], r[z]", "B", "D"])] // LD B, D
#[case::dd_ld_b_e(0x29f9, &[0xdd, 0x43], &["decode_dd", "LD r[y], r[z]", "B", "E"])] // LD B, E
#[case::dd_ld_b_ixh(0x29f9, &[0xdd, 0x44], &["decode_dd", "LD r[y], r[z]", "B", "H", "IXH"])] // LD B, IXH
#[case::dd_ld_b_ixl(0x29f9, &[0xdd, 0x45], &["decode_dd", "LD r[y], r[z]", "B", "L", "IXL"])] // LD B, IXL
#[case::dd_ld_b_ixd(0x29f9, &[0xdd, 0x46, 0x05], &["decode_dd", "LD r[y], r[z]", "B", "(HL)", "(IX + d)"])] // LD B, (IX + d)
#[case::dd_ld_b_a(0x29f9, &[0xdd, 0x47], &["decode_dd", "LD r[y], r[z]", "B", "A"])] // LD B, A
#[case::dd_ld_c_b(0x29f9, &[0xdd, 0x48], &["decode_dd", "LD r[y], r[z]", "C", "B"])] // LD C, B
#[case::dd_ld_c_c(0x29f9, &[0xdd, 0x49], &["decode_dd", "LD r[y], r[z]", "C", "C"])] // LD C, C
#[case::dd_ld_c_d(0x29f9, &[0xdd, 0x4A], &["decode_dd", "LD r[y], r[z]", "C", "D"])] // LD C, D
#[case::dd_ld_c_e(0x29f9, &[0xdd, 0x4B], &["decode_dd", "LD r[y], r[z]", "C", "E"])] // LD C, E
#[case::dd_ld_c_ixh(0x29f9, &[0xdd, 0x4C], &["decode_dd", "LD r[y], r[z]", "C", "H", "IXH"])] // LD C, IXH
#[case::dd_ld_c_ixl(0x29f9, &[0xdd, 0x4D], &["decode_dd", "LD r[y], r[z]", "C", "L", "IXL"])] // LD C, IXL
#[case::dd_ld_c_ixd(0x29f9, &[0xdd, 0x4E, 0x05], &["decode_dd", "LD r[y], r[z]", "C", "(HL)", "(IX + d)"])] // LD C, (IX + d)
#[case::dd_ld_c_a(0x29f9, &[0xdd, 0x4F], &["decode_dd", "LD r[y], r[z]", "C", "A"])] // LD C, A
#[case::dd_ld_d_b(0x29f9, &[0xdd, 0x50], &["decode_dd", "LD r[y], r[z]", "D", "B"])] // LD D, B
#[case::dd_ld_d_c(0x29f9, &[0xdd, 0x51], &["decode_dd", "LD r[y], r[z]", "D", "C"])] // LD D, C
#[case::dd_ld_d_d(0x29f9, &[0xdd, 0x52], &["decode_dd", "LD r[y], r[z]", "D", "D"])] // LD D, D
#[case::dd_ld_d_e(0x29f9, &[0xdd, 0x53], &["decode_dd", "LD r[y], r[z]", "D", "E"])] // LD D, E
#[case::dd_ld_d_ixh(0x29f9, &[0xdd, 0x54], &["decode_dd", "LD r[y], r[z]", "D", "H", "IXH"])] // LD D, IXH
#[case::dd_ld_d_ixl(0x29f9, &[0xdd, 0x55], &["decode_dd", "LD r[y], r[z]", "D", "L", "IXL"])] // LD D, IXL
#[case::dd_ld_d_ixd(0x29f9, &[0xdd, 0x56, 0x05], &["decode_dd", "LD r[y], r[z]", "D", "(HL)", "(IX + d)"])] // LD D, (IX + d)
#[case::dd_ld_d_a(0x29f9, &[0xdd, 0x57], &["decode_dd", "LD r[y], r[z]", "D", "A"])] // LD D, A
#[case::dd_ld_e_b(0x29f9, &[0xdd, 0x58], &["decode_dd", "LD r[y], r[z]", "E", "B"])] // LD E, B
#[case::dd_ld_e_c(0x29f9, &[0xdd, 0x59], &["decode_dd", "LD r[y], r[z]", "E", "C"])] // LD E, C
#[case::dd_ld_e_d(0x29f9, &[0xdd, 0x5A], &["decode_dd", "LD r[y], r[z]", "E", "D"])] // LD E, D
#[case::dd_ld_e_e(0x29f9, &[0xdd, 0x5B], &["decode_dd", "LD r[y], r[z]", "E", "E"])] // LD E, E
#[case::dd_ld_e_ixh(0x29f9, &[0xdd, 0x5C], &["decode_dd", "LD r[y], r[z]", "E", "H", "IXH"])] // LD E, IXH
#[case::dd_ld_e_ixl(0x29f9, &[0xdd, 0x5D], &["decode_dd", "LD r[y], r[z]", "E", "L", "IXL"])] // LD E, IXL
#[case::dd_ld_e_ixd(0x29f9, &[0xdd, 0x5E, 0x05], &["decode_dd", "LD r[y], r[z]", "E", "(HL)", "(IX + d)"])] // LD E, (IX + d)
#[case::dd_ld_e_a(0x29f9, &[0xdd, 0x5F], &["decode_dd", "LD r[y], r[z]", "E", "A"])] // LD E, A
#[case::dd_ld_ixh_b(0x29f9, &[0xdd, 0x60], &["decode_dd", "LD r[y], r[z]", "H", "IXH", "B"])] // LD IXH, B
#[case::dd_ld_ixh_c(0x29f9, &[0xdd, 0x61], &["decode_dd", "LD r[y], r[z]", "H", "IXH", "C"])] // LD IXH, C
#[case::dd_ld_ixh_d(0x29f9, &[0xdd, 0x62], &["decode_dd", "LD r[y], r[z]", "H", "IXH", "D"])] // LD IXH, D
#[case::dd_ld_ixh_e(0x29f9, &[0xdd, 0x63], &["decode_dd", "LD r[y], r[z]", "H", "IXH", "E"])] // LD IXH, E
#[case::dd_ld_ixh_ixh (0x29f9, &[0xdd, 0x64], &["decode_dd", "LD r[y], r[z]", "H", "IXH", "H", "IXH"])] // LD IXH, IXH
#[case::dd_ld_ixh_ixl (0x29f9, &[0xdd, 0x65], &["decode_dd", "LD r[y], r[z]", "H", "IXH", "L", "IXL"])] // LD IXH, IXL
#[case::dd_ld_ixh_ixd (0x29f9, &[0xdd, 0x66, 0x05], &["decode_dd", "LD r[y], r[z]", "H", "IXH", "(HL)", "(IX + d)"])] // LD IXH, (IX + d)
#[case::dd_ld_ixh_a(0x29f9, &[0xdd, 0x67], &["decode_dd", "LD r[y], r[z]", "H", "IXH", "A"])] // LD IXH, A
#[case::dd_ld_ixl_b(0x29f9, &[0xdd, 0x68], &["decode_dd", "LD r[y], r[z]", "L", "IXL", "B"])] // LD IXL, B
#[case::dd_ld_ixl_c(0x29f9, &[0xdd, 0x69], &["decode_dd", "LD r[y], r[z]", "L", "IXL", "C"])] // LD IXL, C
#[case::dd_ld_ixl_d(0x29f9, &[0xdd, 0x6A], &["decode_dd", "LD r[y], r[z]", "L", "IXL", "D"])] // LD IXL, D
#[case::dd_ld_ixl_e(0x29f9, &[0xdd, 0x6B], &["decode_dd", "LD r[y], r[z]", "L", "IXL", "E"])] // LD IXL, E
#[case::dd_ld_ixl_ixh (0x29f9, &[0xdd, 0x6C], &["decode_dd", "LD r[y], r[z]", "L", "IXL", "H", "IXH"])] // LD IXL, IXH
#[case::dd_ld_ixl_ixl (0x29f9, &[0xdd, 0x6D], &["decode_dd", "LD r[y], r[z]", "L", "IXL", "L", "IXL"])] // LD IXL, IXL
#[case::dd_ld_ixl_ixd (0x29f9, &[0xdd, 0x6E, 0x05], &["decode_dd", "LD r[y], r[z]", "L", "IXL", "(HL)", "(IX + d)"])] // LD IXL, (IX + d)
#[case::dd_ld_ixl_a(0x29f9, &[0xdd, 0x6F], &["decode_dd", "LD r[y], r[z]", "L", "IXL", "A"])] // LD IXL, A
#[case::dd_ld_ixd_b(0x29f9, &[0xdd, 0x70, 0x05], &["decode_dd", "LD r[y], r[z]", "(HL)", "(IX + d)", "B"])] // LD (IX + d), B
#[case::dd_ld_ixd_c(0x29f9, &[0xdd, 0x71, 0x05], &["decode_dd", "LD r[y], r[z]", "(HL)", "(IX + d)", "C"])] // LD (IX + d), C
#[case::dd_ld_ixd_d(0x29f9, &[0xdd, 0x72, 0x05], &["decode_dd", "LD r[y], r[z]", "(HL)", "(IX + d)", "D"])] // LD (IX + d), D
#[case::dd_ld_ixd_e(0x29f9, &[0xdd, 0x73, 0x05], &["decode_dd", "LD r[y], r[z]", "(HL)", "(IX + d)", "E"])] // LD (IX + d), E
#[case::dd_ld_ixd_ixh (0x29f9, &[0xdd, 0x74, 0x05], &["decode_dd", "LD r[y], r[z]", "(HL)", "(IX + d)", "H", "IXH"])] // LD (IX + d), IXH
#[case::dd_ld_ixd_ixl (0x29f9, &[0xdd, 0x75, 0x05], &["decode_dd", "LD r[y], r[z]", "(HL)", "(IX + d)", "L", "IXL"])] // LD (IX + d), IXL
#[case::dd_halt(0x29f9, &[0xdd, 0x76], &["decode_dd", "HALT"])] // HALT (IX)
#[case::dd_ld_ixd_a(0x29f9, &[0xdd, 0x77, 0x05], &["decode_dd", "LD r[y], r[z]", "(HL)", "(IX + d)", "A"])] // LD (IX + d), A
#[case::dd_ld_a_b(0x29f9, &[0xdd, 0x78], &["decode_dd", "LD r[y], r[z]", "A", "B"])] // LD A, B
#[case::dd_ld_a_c(0x29f9, &[0xdd, 0x79], &["decode_dd", "LD r[y], r[z]", "A", "C"])] // LD A, C
#[case::dd_ld_a_d(0x29f9, &[0xdd, 0x7A], &["decode_dd", "LD r[y], r[z]", "A", "D"])] // LD A, D
#[case::dd_ld_a_e(0x29f9, &[0xdd, 0x7B], &["decode_dd", "LD r[y], r[z]", "A", "E"])] // LD A, E
#[case::dd_ld_a_ixh(0x29f9, &[0xdd, 0x7C], &["decode_dd", "LD r[y], r[z]", "A", "H", "IXH"])] // LD A, IXH
#[case::dd_ld_a_ixl(0x29f9, &[0xdd, 0x7D], &["decode_dd", "LD r[y], r[z]", "A", "L", "IXL"])] // LD A, IXL
#[case::dd_ld_a_ixd(0x29f9, &[0xdd, 0x7E, 0x05], &["decode_dd", "LD r[y], r[z]", "A", "(HL)", "(IX + d)"])] // LD A, (IX + d)
#[case::dd_ld_a_a(0x29f9, &[0xdd, 0x7F], &["decode_dd", "LD r[y], r[z]", "A", "A"])] // LD A, A
#[case::dd_add_a_b(0x29f9, &[0xdd, 0x80], & ["decode_dd", "ALU[y] r[z]", "ADD A", "B"])] // ADD A, B
#[case::dd_add_a_c(0x29f9, &[0xdd, 0x81], & ["decode_dd", "ALU[y] r[z]", "ADD A", "C"])] // ADD A, C
#[case::dd_add_a_d(0x29f9, &[0xdd, 0x82], & ["decode_dd", "ALU[y] r[z]", "ADD A", "D"])] // ADD A, D
#[case::dd_add_a_e(0x29f9, &[0xdd, 0x83], & ["decode_dd", "ALU[y] r[z]", "ADD A", "E"])] // ADD A, E
#[case::dd_add_a_ixh(0x29f9, &[0xdd, 0x84], & ["decode_dd", "ALU[y] r[z]", "ADD A", "H", "IXH"])] // ADD A, IXH
#[case::dd_add_a_ixl(0x29f9, &[0xdd, 0x85], & ["decode_dd", "ALU[y] r[z]", "ADD A", "L", "IXL"])] // ADD A, IXL
#[case::dd_add_a_ixd(0x29f9, &[0xdd, 0x86, 0x05], & ["decode_dd", "ALU[y] r[z]", "ADD A", "(HL)", "(IX + d)"])] // ADD A, (IX + d)
#[case::dd_add_a_a(0x29f9, &[0xdd, 0x87], & ["decode_dd", "ALU[y] r[z]", "ADD A", "A"])] // ADD A, A
#[case::dd_adc_a_b(0x29f9, &[0xdd, 0x88], & ["decode_dd", "ALU[y] r[z]", "ADC A", "B"])] // ADC A, B
#[case::dd_adc_a_c(0x29f9, &[0xdd, 0x89], & ["decode_dd", "ALU[y] r[z]", "ADC A", "C"])] // ADC A, C
#[case::dd_adc_a_d(0x29f9, &[0xdd, 0x8A], & ["decode_dd", "ALU[y] r[z]", "ADC A", "D"])] // ADC A, D
#[case::dd_adc_a_e(0x29f9, &[0xdd, 0x8B], & ["decode_dd", "ALU[y] r[z]", "ADC A", "E"])] // ADC A, E
#[case::dd_adc_a_ixh(0x29f9, &[0xdd, 0x8C], & ["decode_dd", "ALU[y] r[z]", "ADC A", "H", "IXH"])] // ADC A, IXH
#[case::dd_adc_a_ixl(0x29f9, &[0xdd, 0x8D], & ["decode_dd", "ALU[y] r[z]", "ADC A", "L", "IXL"])] // ADC A, IXL
#[case::dd_adc_a_ixd(0x29f9, &[0xdd, 0x8E, 0x05], & ["decode_dd", "ALU[y] r[z]", "ADC A", "(HL)", "(IX + d)"])] // ADC A, (IX + d)
#[case::dd_adc_a_a(0x29f9, &[0xdd, 0x8F], & ["decode_dd", "ALU[y] r[z]", "ADC A", "A"])] // ADC A, A
#[case::dd_sub_a_b(0x29f9, &[0xdd, 0x90], & ["decode_dd", "ALU[y] r[z]", "SUB A", "B"])] // SUB A, B
#[case::dd_sub_a_c(0x29f9, &[0xdd, 0x91], & ["decode_dd", "ALU[y] r[z]", "SUB A", "C"])] // SUB A, C
#[case::dd_sub_a_d(0x29f9, &[0xdd, 0x92], & ["decode_dd", "ALU[y] r[z]", "SUB A", "D"])] // SUB A, D
#[case::dd_sub_a_e(0x29f9, &[0xdd, 0x93], & ["decode_dd", "ALU[y] r[z]", "SUB A", "E"])] // SUB A, E
#[case::dd_sub_a_ixh(0x29f9, &[0xdd, 0x94], & ["decode_dd", "ALU[y] r[z]", "SUB A", "H", "IXH"])] // SUB A, IXH
#[case::dd_sub_a_ixl(0x29f9, &[0xdd, 0x95], & ["decode_dd", "ALU[y] r[z]", "SUB A", "L", "IXL"])] // SUB A, IXL
#[case::dd_sub_a_ixd(0x29f9, &[0xdd, 0x96, 0x05], & ["decode_dd", "ALU[y] r[z]", "SUB A", "(HL)", "(IX + d)"])] // SUB A, (IX + d)
#[case::dd_sub_a_a(0x29f9, &[0xdd, 0x97], & ["decode_dd", "ALU[y] r[z]", "SUB A", "A"])] // SUB A, A
#[case::dd_sbc_a_b(0x29f9, &[0xdd, 0x98], & ["decode_dd", "ALU[y] r[z]", "SBC A", "B"])] // SBC A, B
#[case::dd_sbc_a_c(0x29f9, &[0xdd, 0x99], & ["decode_dd", "ALU[y] r[z]", "SBC A", "C"])] // SBC A, C
#[case::dd_sbc_a_d(0x29f9, &[0xdd, 0x9A], & ["decode_dd", "ALU[y] r[z]", "SBC A", "D"])] // SBC A, D
#[case::dd_sbc_a_e(0x29f9, &[0xdd, 0x9B], & ["decode_dd", "ALU[y] r[z]", "SBC A", "E"])] // SBC A, E
#[case::dd_sbc_a_ixh(0x29f9, &[0xdd, 0x9C], & ["decode_dd", "ALU[y] r[z]", "SBC A", "H", "IXH"])] // SBC A, IXH
#[case::dd_sbc_a_ixl(0x29f9, &[0xdd, 0x9D], & ["decode_dd", "ALU[y] r[z]", "SBC A", "L", "IXL"])] // SBC A, IXL
#[case::dd_sbc_a_ixd(0x29f9, &[0xdd, 0x9E, 0x05], & ["decode_dd", "ALU[y] r[z]", "SBC A", "(HL)", "(IX + d)"])] // SBC A, (IX + d)
#[case::dd_sbc_a_a(0x29f9, &[0xdd, 0x9F], & ["decode_dd", "ALU[y] r[z]", "SBC A", "A"])] // SBC A, A
#[case::dd_and_a_b(0x29f9, &[0xdd, 0xA0], & ["decode_dd", "ALU[y] r[z]", "AND A", "B"])] // AND A, B
#[case::dd_and_a_c(0x29f9, &[0xdd, 0xA1], & ["decode_dd", "ALU[y] r[z]", "AND A", "C"])] // AND A, C
#[case::dd_and_a_d(0x29f9, &[0xdd, 0xA2], & ["decode_dd", "ALU[y] r[z]", "AND A", "D"])] // AND A, D
#[case::dd_and_a_e(0x29f9, &[0xdd, 0xA3], & ["decode_dd", "ALU[y] r[z]", "AND A", "E"])] // AND A, E
#[case::dd_and_a_ixh(0x29f9, &[0xdd, 0xA4], & ["decode_dd", "ALU[y] r[z]", "AND A", "H", "IXH"])] // AND A, IXH
#[case::dd_and_a_ixl(0x29f9, &[0xdd, 0xA5], & ["decode_dd", "ALU[y] r[z]", "AND A", "L", "IXL"])] // AND A, IXL
#[case::dd_and_a_ixd(0x29f9, &[0xdd, 0xA6, 0x05], & ["decode_dd", "ALU[y] r[z]", "AND A", "(HL)", "(IX + d)"])] // AND A, (IX + d)
#[case::dd_and_a_a(0x29f9, &[0xdd, 0xA7], & ["decode_dd", "ALU[y] r[z]", "AND A", "A"])] // AND A, A
#[case::dd_xor_a_b(0x29f9, &[0xdd, 0xA8], & ["decode_dd", "ALU[y] r[z]", "XOR A", "B"])] // XOR A, B
#[case::dd_xor_a_c(0x29f9, &[0xdd, 0xA9], & ["decode_dd", "ALU[y] r[z]", "XOR A", "C"])] // XOR A, C
#[case::dd_xor_a_d(0x29f9, &[0xdd, 0xAA], & ["decode_dd", "ALU[y] r[z]", "XOR A", "D"])] // XOR A, D
#[case::dd_xor_a_e(0x29f9, &[0xdd, 0xAB], & ["decode_dd", "ALU[y] r[z]", "XOR A", "E"])] // XOR A, E
#[case::dd_xor_a_ixh(0x29f9, &[0xdd, 0xAC], & ["decode_dd", "ALU[y] r[z]", "XOR A", "H", "IXH"])] // XOR A, IXH
#[case::dd_xor_a_ixl(0x29f9, &[0xdd, 0xAD], & ["decode_dd", "ALU[y] r[z]", "XOR A", "L", "IXL"])] // XOR A, IXL
#[case::dd_xor_a_ixd(0x29f9, &[0xdd, 0xAE, 0x05], & ["decode_dd", "ALU[y] r[z]", "XOR A", "(HL)", "(IX + d)"])] // XOR A, (IX + d)
#[case::dd_xor_a_a(0x29f9, &[0xdd, 0xAF], & ["decode_dd", "ALU[y] r[z]", "XOR A", "A"])] // XOR A, A
#[case::dd_or_a_b(0x29f9, &[0xdd, 0xB0], & ["decode_dd", "ALU[y] r[z]", "OR A", "B"])] // OR A, B
#[case::dd_or_a_c(0x29f9, &[0xdd, 0xB1], & ["decode_dd", "ALU[y] r[z]", "OR A", "C"])] // OR A, C
#[case::dd_or_a_d(0x29f9, &[0xdd, 0xB2], & ["decode_dd", "ALU[y] r[z]", "OR A", "D"])] // OR A, D
#[case::dd_or_a_e(0x29f9, &[0xdd, 0xB3], & ["decode_dd", "ALU[y] r[z]", "OR A", "E"])] // OR A, E
#[case::dd_or_a_ixh(0x29f9, &[0xdd, 0xB4], & ["decode_dd", "ALU[y] r[z]", "OR A", "H", "IXH"])] // OR A, IXH
#[case::dd_or_a_ixl(0x29f9, &[0xdd, 0xB5], & ["decode_dd", "ALU[y] r[z]", "OR A", "L", "IXL"])] // OR A, IXL
#[case::dd_or_a_ixd(0x29f9, &[0xdd, 0xB6, 0x05], & ["decode_dd", "ALU[y] r[z]", "OR A", "(HL)", "(IX + d)"])] // OR A, (IX + d)
#[case::dd_or_a_a(0x29f9, &[0xdd, 0xB7], & ["decode_dd", "ALU[y] r[z]", "OR A", "A"])] // OR A, A
#[case::dd_cp_a_b(0x29f9, &[0xdd, 0xB8], & ["decode_dd", "ALU[y] r[z]", "CP A", "B"])] // CP A, B
#[case::dd_cp_a_c(0x29f9, &[0xdd, 0xB9], & ["decode_dd", "ALU[y] r[z]", "CP A", "C"])] // CP A, C
#[case::dd_cp_a_d(0x29f9, &[0xdd, 0xBA], & ["decode_dd", "ALU[y] r[z]", "CP A", "D"])] // CP A, D
#[case::dd_cp_a_e(0x29f9, &[0xdd, 0xBB], & ["decode_dd", "ALU[y] r[z]", "CP A", "E"])] // CP A, E
#[case::dd_cp_a_ixh(0x29f9, &[0xdd, 0xBC], & ["decode_dd", "ALU[y] r[z]", "CP A", "H", "IXH"])] // CP A, IXH
#[case::dd_cp_a_ixl(0x29f9, &[0xdd, 0xBD], & ["decode_dd", "ALU[y] r[z]", "CP A", "L", "IXL"])] // CP A, IXL
#[case::dd_cp_a_ixd(0x29f9, &[0xdd, 0xBE, 0x05], & ["decode_dd", "ALU[y] r[z]", "CP A", "(HL)", "(IX + d)"])] // CP A, (IX + d)
#[case::dd_cp_a_a(0x29f9, &[0xdd, 0xBF], & ["decode_dd", "ALU[y] r[z]", "CP A", "A"])] // CP A, A
// TODO: DDCB case
#[case::dd_pop_ix(0x29f9, &[0xdd, 0xe1], &["decode_dd", "POP rp2[p]", "HL", "IX"])] // POP IX
#[case::dd_ex_sp_ix(0x29f9, &[0xdd, 0xe3], &["decode_dd", "EX (SP), HL/IX/IY", "IX"])] // EX (SP), IX
#[case::dd_push_ix(0x29f9, &[0xdd, 0xe5], &["decode_dd", "PUSH rp2[p]", "HL", "IX"])] // PUSH IX
#[case::dd_jp_ix(0x29f9, &[0xdd, 0xe9], &["decode_dd", "JP HL", "IX"])] // JP IX
#[case::dd_ld_sp_ix(0x29f9, &[0xdd, 0xf9], &["decode_dd", "LD SP, HL", "IX"])]
// LD SP, IX
// LD A, A

// ----------------------------------------------------------------------------------------------------
// FD TABLE (IY INSTRUCTIONS) - duplicated from DD with IX->IY mapping
// ----------------------------------------------------------------------------------------------------
#[case::fd_inc_b(0x29f9, &[0xfd, 0x04], &["decode_fd", "INC r[y]", "B"])] // INC B
#[case::fd_dec_b(0x29f9, &[0xfd, 0x05], &["decode_fd", "DEC r[y]", "B"])] // DEC B
#[case::fd_ld_b_n(0x29f9, &[0xfd, 0x06, 0x42], &["decode_fd", "LD r[y], n", "B"])] // LD B, n
#[case::fd_add_iy_bc(0x29f9, &[0xfd, 0x09], &["decode_fd", "ADD HL/IX/IY, rp[p]", "IY", "BC"])] // ADD IY, BC
#[case::fd_inc_c(0x29f9, &[0xfd, 0x0C], &["decode_fd", "INC r[y]", "C"])] // INC C
#[case::fd_dec_c(0x29f9, &[0xfd, 0x0D], &["decode_fd", "DEC r[y]", "C"])] // DEC C
#[case::fd_ld_c_n(0x29f9, &[0xfd, 0x0E, 0x42], &["decode_fd", "LD r[y], n", "C"])] // LD C, n
#[case::fd_inc_d(0x29f9, &[0xfd, 0x14], &["decode_fd", "INC r[y]", "D"])] // INC D
#[case::fd_dec_d(0x29f9, &[0xfd, 0x15], &["decode_fd", "DEC r[y]", "D"])] // DEC D
#[case::fd_ld_d_n(0x29f9, &[0xfd, 0x16, 0x42], &["decode_fd", "LD r[y], n", "D"])] // LD D, n
#[case::fd_add_iy_de(0x29f9, &[0xfd, 0x19], &["decode_fd", "ADD HL/IX/IY, rp[p]", "IY", "DE"])] // ADD IY, DE
#[case::fd_inc_e(0x29f9, &[0xfd, 0x1C], &["decode_fd", "INC r[y]", "E"])] // INC E
#[case::fd_dec_e(0x29f9, &[0xfd, 0x1D], &["decode_fd", "DEC r[y]", "E"])] // DEC E
#[case::fd_ld_e_n(0x29f9, &[0xfd, 0x1E, 0x42], &["decode_fd", "LD r[y], n", "E"])] // LD E, n
#[case::fd_ld_iy_nn(0x29f9, &[0xfd, 0x21, 0x34, 0x12], &["decode_fd", "LD rp[p], nn", "HL", "IY"])] // LD IY, nn
#[case::fd_ld_nni_iy(0x29f9, &[0xfd, 0x22, 0x34, 0x12], &["decode_fd", "LD (nn), HL/IX/IY", "IY"])] // LD (nn), IY
#[case::fd_inc_iy(0x29f9, &[0xfd, 0x23], &["decode_fd", "INC rp[p]", "HL", "IY"])] // INC IY
#[case::fd_inc_iyh(0x29f9, &[0xfd, 0x24], &["decode_fd", "INC r[y]", "H", "IYH"])] // INC IYH
#[case::fd_dec_iyh(0x29f9, &[0xfd, 0x25], &["decode_fd", "DEC r[y]", "H", "IYH"])] // DEC IYH
#[case::fd_ld_iyh_n(0x29f9, &[0xfd, 0x26, 0x42], &["decode_fd", "LD r[y], n", "H", "IYH"])] // LD IYH, n
#[case::fd_add_iy_iy(0x29f9, &[0xfd, 0x29], &["decode_fd", "ADD HL/IX/IY, rp[p]", "IY", "HL", "IY"])] // ADD IY, IY
#[case::fd_ld_iy_nni(0x29f9, &[0xfd, 0x2A, 0x34, 0x12], &["decode_fd", "LD HL/IX/IY, (nn)", "IY"])] // LD IY, (nn)
#[case::fd_dec_iy(0x29f9, &[0xfd, 0x2B], &["decode_fd", "DEC rp[p]", "HL", "IY"])] // DEC IY
#[case::fd_inc_iyl(0x29f9, &[0xfd, 0x2C], &["decode_fd", "INC r[y]", "L", "IYL"])] // INC IYL
#[case::fd_dec_iyl(0x29f9, &[0xfd, 0x2D], &["decode_fd", "DEC r[y]", "L", "IYL"])] // DEC IYL
#[case::fd_ld_iyl_n(0x29f9, &[0xfd, 0x2E, 0x42], &["decode_fd", "LD r[y], n", "L", "IYL"])] // LD IYL, n
#[case::fd_inc_iyd(0x29f9, &[0xfd, 0x34], &["decode_fd", "INC r[y]", "(HL)", "(IY + d)"])] // INC (IY + d)
#[case::fd_dec_iyd(0x29f9, &[0xfd, 0x35], &["decode_fd", "DEC r[y]", "(HL)", "(IY + d)"])] // DEC (IY + d)
#[case::fd_ld_iyd_n(0x29f9, &[0xfd, 0x36, 0x42], &["decode_fd", "LD r[y], n", "(HL)", "(IY + d)"])] // LD (IY + d), n
#[case::fd_add_iy_sp(0x29f9, &[0xfd, 0x39], &["decode_fd", "ADD HL/IX/IY, rp[p]", "IY", "SP"])] // ADD IY, SP
#[case::fd_inc_a(0x29f9, &[0xfd, 0x3C], &["decode_fd", "INC r[y]", "A"])] // INC A
#[case::fd_dec_a(0x29f9, &[0xfd, 0x3D], &["decode_fd", "DEC r[y]", "A"])] // DEC A
#[case::fd_ld_a_n(0x29f9, &[0xfd, 0x3E, 0x42], &["decode_fd", "LD r[y], n", "A"])] // LD A, n
#[case::fd_ld_b_b(0x29f9, &[0xfd, 0x40], &["decode_fd", "LD r[y], r[z]", "B", "B"])] // LD B, B
#[case::fd_ld_b_c(0x29f9, &[0xfd, 0x41], &["decode_fd", "LD r[y], r[z]", "B", "C"])] // LD B, C
#[case::fd_ld_b_d(0x29f9, &[0xfd, 0x42], &["decode_fd", "LD r[y], r[z]", "B", "D"])] // LD B, D
#[case::fd_ld_b_e(0x29f9, &[0xfd, 0x43], &["decode_fd", "LD r[y], r[z]", "B", "E"])] // LD B, E
#[case::fd_ld_b_iyh(0x29f9, &[0xfd, 0x44], &["decode_fd", "LD r[y], r[z]", "B", "H", "IYH"])] // LD B, IYH
#[case::fd_ld_b_iyl(0x29f9, &[0xfd, 0x45], &["decode_fd", "LD r[y], r[z]", "B", "L", "IYL"])] // LD B, IYL
#[case::fd_ld_b_iyd(0x29f9, &[0xfd, 0x46, 0x05], &["decode_fd", "LD r[y], r[z]", "B", "(HL)", "(IY + d)"])] // LD B, (IY + d)
#[case::fd_ld_b_a(0x29f9, &[0xfd, 0x47], &["decode_fd", "LD r[y], r[z]", "B", "A"])] // LD B, A
#[case::fd_ld_c_b(0x29f9, &[0xfd, 0x48], &["decode_fd", "LD r[y], r[z]", "C", "B"])] // LD C, B
#[case::fd_ld_c_c(0x29f9, &[0xfd, 0x49], &["decode_fd", "LD r[y], r[z]", "C", "C"])] // LD C, C
#[case::fd_ld_c_d(0x29f9, &[0xfd, 0x4A], &["decode_fd", "LD r[y], r[z]", "C", "D"])] // LD C, D
#[case::fd_ld_c_e(0x29f9, &[0xfd, 0x4B], &["decode_fd", "LD r[y], r[z]", "C", "E"])] // LD C, E
#[case::fd_ld_c_iyh(0x29f9, &[0xfd, 0x4C], &["decode_fd", "LD r[y], r[z]", "C", "H", "IYH"])] // LD C, IYH
#[case::fd_ld_c_iyl(0x29f9, &[0xfd, 0x4D], &["decode_fd", "LD r[y], r[z]", "C", "L", "IYL"])] // LD C, IYL
#[case::fd_ld_c_iyd(0x29f9, &[0xfd, 0x4E, 0x05], &["decode_fd", "LD r[y], r[z]", "C", "(HL)", "(IY + d)"])] // LD C, (IY + d)
#[case::fd_ld_c_a(0x29f9, &[0xfd, 0x4F], &["decode_fd", "LD r[y], r[z]", "C", "A"])] // LD C, A
#[case::fd_ld_d_b(0x29f9, &[0xfd, 0x50], &["decode_fd", "LD r[y], r[z]", "D", "B"])] // LD D, B
#[case::fd_ld_d_c(0x29f9, &[0xfd, 0x51], &["decode_fd", "LD r[y], r[z]", "D", "C"])] // LD D, C
#[case::fd_ld_d_d(0x29f9, &[0xfd, 0x52], &["decode_fd", "LD r[y], r[z]", "D", "D"])] // LD D, D
#[case::fd_ld_d_e(0x29f9, &[0xfd, 0x53], &["decode_fd", "LD r[y], r[z]", "D", "E"])] // LD D, E
#[case::fd_ld_d_iyh(0x29f9, &[0xfd, 0x54], &["decode_fd", "LD r[y], r[z]", "D", "H", "IYH"])] // LD D, IYH
#[case::fd_ld_d_iyl(0x29f9, &[0xfd, 0x55], &["decode_fd", "LD r[y], r[z]", "D", "L", "IYL"])] // LD D, IYL
#[case::fd_ld_d_iyd(0x29f9, &[0xfd, 0x56, 0x05], &["decode_fd", "LD r[y], r[z]", "D", "(HL)", "(IY + d)"])] // LD D, (IY + d)
#[case::fd_ld_d_a(0x29f9, &[0xfd, 0x57], &["decode_fd", "LD r[y], r[z]", "D", "A"])] // LD D, A
#[case::fd_ld_e_b(0x29f9, &[0xfd, 0x58], &["decode_fd", "LD r[y], r[z]", "E", "B"])] // LD E, B
#[case::fd_ld_e_c(0x29f9, &[0xfd, 0x59], &["decode_fd", "LD r[y], r[z]", "E", "C"])] // LD E, C
#[case::fd_ld_e_d(0x29f9, &[0xfd, 0x5A], &["decode_fd", "LD r[y], r[z]", "E", "D"])] // LD E, D
#[case::fd_ld_e_e(0x29f9, &[0xfd, 0x5B], &["decode_fd", "LD r[y], r[z]", "E", "E"])] // LD E, E
#[case::fd_ld_e_iyh(0x29f9, &[0xfd, 0x5C], &["decode_fd", "LD r[y], r[z]", "E", "H", "IYH"])] // LD E, IYH
#[case::fd_ld_e_iyl(0x29f9, &[0xfd, 0x5D], &["decode_fd", "LD r[y], r[z]", "E", "L", "IYL"])] // LD E, IYL
#[case::fd_ld_e_iyd(0x29f9, &[0xfd, 0x5E, 0x05], &["decode_fd", "LD r[y], r[z]", "E", "(HL)", "(IY + d)"])] // LD E, (IY + d)
#[case::fd_ld_e_a(0x29f9, &[0xfd, 0x5F], &["decode_fd", "LD r[y], r[z]", "E", "A"])] // LD E, A
#[case::fd_ld_iyh_b(0x29f9, &[0xfd, 0x60], &["decode_fd", "LD r[y], r[z]", "H", "IYH", "B"])] // LD IYH, B
#[case::fd_ld_iyh_c(0x29f9, &[0xfd, 0x61], &["decode_fd", "LD r[y], r[z]", "H", "IYH", "C"])] // LD IYH, C
#[case::fd_ld_iyh_d(0x29f9, &[0xfd, 0x62], &["decode_fd", "LD r[y], r[z]", "H", "IYH", "D"])] // LD IYH, D
#[case::fd_ld_iyh_e(0x29f9, &[0xfd, 0x63], &["decode_fd", "LD r[y], r[z]", "H", "IYH", "E"])] // LD IYH, E
#[case::fd_ld_iyh_iyh (0x29f9, &[0xfd, 0x64], &["decode_fd", "LD r[y], r[z]", "H", "IYH", "H", "IYH"])] // LD IYH, IYH
#[case::fd_ld_iyh_iyl (0x29f9, &[0xfd, 0x65], &["decode_fd", "LD r[y], r[z]", "H", "IYH", "L", "IYL"])] // LD IYH, IYL
#[case::fd_ld_iyh_iyd (0x29f9, &[0xfd, 0x66, 0x05], &["decode_fd", "LD r[y], r[z]", "H", "IYH", "(HL)", "(IY + d)"])] // LD IYH, (IY + d)
#[case::fd_ld_iyh_a(0x29f9, &[0xfd, 0x67], &["decode_fd", "LD r[y], r[z]", "H", "IYH", "A"])] // LD IYH, A
#[case::fd_ld_iyl_b(0x29f9, &[0xfd, 0x68], &["decode_fd", "LD r[y], r[z]", "L", "IYL", "B"])] // LD IYL, B
#[case::fd_ld_iyl_c(0x29f9, &[0xfd, 0x69], &["decode_fd", "LD r[y], r[z]", "L", "IYL", "C"])] // LD IYL, C
#[case::fd_ld_iyl_d(0x29f9, &[0xfd, 0x6A], &["decode_fd", "LD r[y], r[z]", "L", "IYL", "D"])] // LD IYL, D
#[case::fd_ld_iyl_e(0x29f9, &[0xfd, 0x6B], &["decode_fd", "LD r[y], r[z]", "L", "IYL", "E"])] // LD IYL, E
#[case::fd_ld_iyl_iyh (0x29f9, &[0xfd, 0x6C], &["decode_fd", "LD r[y], r[z]", "L", "IYL", "H", "IYH"])] // LD IYL, IYH
#[case::fd_ld_iyl_iyl (0x29f9, &[0xfd, 0x6D], &["decode_fd", "LD r[y], r[z]", "L", "IYL", "L", "IYL"])] // LD IYL, IYL
#[case::fd_ld_iyl_iyd (0x29f9, &[0xfd, 0x6E, 0x05], &["decode_fd", "LD r[y], r[z]", "L", "IYL", "(HL)", "(IY + d)"])] // LD IYL, (IY + d)
#[case::fd_ld_iyl_a(0x29f9, &[0xfd, 0x6F], &["decode_fd", "LD r[y], r[z]", "L", "IYL", "A"])] // LD IYL, A
#[case::fd_ld_iyd_b(0x29f9, &[0xfd, 0x70, 0x05], &["decode_fd", "LD r[y], r[z]", "(HL)", "(IY + d)", "B"])] // LD (IY + d), B
#[case::fd_ld_iyd_c(0x29f9, &[0xfd, 0x71, 0x05], &["decode_fd", "LD r[y], r[z]", "(HL)", "(IY + d)", "C"])] // LD (IY + d), C
#[case::fd_ld_iyd_d(0x29f9, &[0xfd, 0x72, 0x05], &["decode_fd", "LD r[y], r[z]", "(HL)", "(IY + d)", "D"])] // LD (IY + d), D
#[case::fd_ld_iyd_e(0x29f9, &[0xfd, 0x73, 0x05], &["decode_fd", "LD r[y], r[z]", "(HL)", "(IY + d)", "E"])] // LD (IY + d), E
#[case::fd_ld_iyd_iyh (0x29f9, &[0xfd, 0x74, 0x05], &["decode_fd", "LD r[y], r[z]", "(HL)", "(IY + d)", "H", "IYH"])] // LD (IY + d), IYH
#[case::fd_ld_iyd_iyl (0x29f9, &[0xfd, 0x75, 0x05], &["decode_fd", "LD r[y], r[z]", "(HL)", "(IY + d)", "L", "IYL"])] // LD (IY + d), IYL
#[case::fd_halt(0x29f9, &[0xfd, 0x76], &["decode_fd", "HALT"])] // HALT (IY)
#[case::fd_ld_iyd_a(0x29f9, &[0xfd, 0x77, 0x05], &["decode_fd", "LD r[y], r[z]", "(HL)", "(IY + d)", "A"])] // LD (IY + d), A
#[case::fd_ld_a_b(0x29f9, &[0xfd, 0x78], &["decode_fd", "LD r[y], r[z]", "A", "B"])] // LD A, B
#[case::fd_ld_a_c(0x29f9, &[0xfd, 0x79], &["decode_fd", "LD r[y], r[z]", "A", "C"])] // LD A, C
#[case::fd_ld_a_d(0x29f9, &[0xfd, 0x7A], &["decode_fd", "LD r[y], r[z]", "A", "D"])] // LD A, D
#[case::fd_ld_a_e(0x29f9, &[0xfd, 0x7B], &["decode_fd", "LD r[y], r[z]", "A", "E"])] // LD A, E
#[case::fd_ld_a_iyh(0x29f9, &[0xfd, 0x7C], &["decode_fd", "LD r[y], r[z]", "A", "H", "IYH"])] // LD A, IYH
#[case::fd_ld_a_iyl(0x29f9, &[0xfd, 0x7D], &["decode_fd", "LD r[y], r[z]", "A", "L", "IYL"])] // LD A, IYL
#[case::fd_ld_a_iyd(0x29f9, &[0xfd, 0x7E, 0x05], &["decode_fd", "LD r[y], r[z]", "A", "(HL)", "(IY + d)"])] // LD A, (IY + d)
#[case::fd_ld_a_a(0x29f9, &[0xfd, 0x7F], &["decode_fd", "LD r[y], r[z]", "A", "A"])] // LD A, A
#[case::fd_add_a_b(0x29f9, &[0xfd, 0x80], &["decode_fd", "ALU[y] r[z]", "ADD A", "B"])] // ADD A, B
#[case::fd_add_a_c(0x29f9, &[0xfd, 0x81], &["decode_fd", "ALU[y] r[z]", "ADD A", "C"])] // ADD A, C
#[case::fd_add_a_d(0x29f9, &[0xfd, 0x82], &["decode_fd", "ALU[y] r[z]", "ADD A", "D"])] // ADD A, D
#[case::fd_add_a_e(0x29f9, &[0xfd, 0x83], &["decode_fd", "ALU[y] r[z]", "ADD A", "E"])] // ADD A, E
#[case::fd_add_a_iyh(0x29f9, &[0xfd, 0x84], &["decode_fd", "ALU[y] r[z]", "ADD A", "H", "IYH"])] // ADD A, IYH
#[case::fd_add_a_iyl(0x29f9, &[0xfd, 0x85], &["decode_fd", "ALU[y] r[z]", "ADD A", "L", "IYL"])] // ADD A, IYL
#[case::fd_add_a_iyd(0x29f9, &[0xfd, 0x86, 0x05], &["decode_fd", "ALU[y] r[z]", "ADD A", "(HL)", "(IY + d)"])] // ADD A, (IY + d)
#[case::fd_add_a_a(0x29f9, &[0xfd, 0x87], &["decode_fd", "ALU[y] r[z]", "ADD A", "A"])] // ADD A, A
#[case::fd_adc_a_b(0x29f9, &[0xfd, 0x88], &["decode_fd", "ALU[y] r[z]", "ADC A", "B"])] // ADC A, B
#[case::fd_adc_a_c(0x29f9, &[0xfd, 0x89], &["decode_fd", "ALU[y] r[z]", "ADC A", "C"])] // ADC A, C
#[case::fd_adc_a_d(0x29f9, &[0xfd, 0x8A], &["decode_fd", "ALU[y] r[z]", "ADC A", "D"])] // ADC A, D
#[case::fd_adc_a_e(0x29f9, &[0xfd, 0x8B], &["decode_fd", "ALU[y] r[z]", "ADC A", "E"])] // ADC A, E
#[case::fd_adc_a_iyh(0x29f9, &[0xfd, 0x8C], &["decode_fd", "ALU[y] r[z]", "ADC A", "H", "IYH"])] // ADC A, IYH
#[case::fd_adc_a_iyl(0x29f9, &[0xfd, 0x8D], &["decode_fd", "ALU[y] r[z]", "ADC A", "L", "IYL"])] // ADC A, IYL
#[case::fd_adc_a_iyd(0x29f9, &[0xfd, 0x8E, 0x05], &["decode_fd", "ALU[y] r[z]", "ADC A", "(HL)", "(IY + d)"])] // ADC A, (IY + d)
#[case::fd_adc_a_a(0x29f9, &[0xfd, 0x8F], &["decode_fd", "ALU[y] r[z]", "ADC A", "A"])] // ADC A, A
#[case::fd_sub_a_b(0x29f9, &[0xfd, 0x90], &["decode_fd", "ALU[y] r[z]", "SUB A", "B"])] // SUB A, B
#[case::fd_sub_a_c(0x29f9, &[0xfd, 0x91], &["decode_fd", "ALU[y] r[z]", "SUB A", "C"])] // SUB A, C
#[case::fd_sub_a_d(0x29f9, &[0xfd, 0x92], &["decode_fd", "ALU[y] r[z]", "SUB A", "D"])] // SUB A, D
#[case::fd_sub_a_e(0x29f9, &[0xfd, 0x93], &["decode_fd", "ALU[y] r[z]", "SUB A", "E"])] // SUB A, E
#[case::fd_sub_a_iyh(0x29f9, &[0xfd, 0x94], &["decode_fd", "ALU[y] r[z]", "SUB A", "H", "IYH"])] // SUB A, IYH
#[case::fd_sub_a_iyl(0x29f9, &[0xfd, 0x95], &["decode_fd", "ALU[y] r[z]", "SUB A", "L", "IYL"])] // SUB A, IYL
#[case::fd_sub_a_iyd(0x29f9, &[0xfd, 0x96, 0x05], &["decode_fd", "ALU[y] r[z]", "SUB A", "(HL)", "(IY + d)"])] // SUB A, (IY + d)
#[case::fd_sub_a_a(0x29f9, &[0xfd, 0x97], &["decode_fd", "ALU[y] r[z]", "SUB A", "A"])] // SUB A, A
#[case::fd_sbc_a_b(0x29f9, &[0xfd, 0x98], &["decode_fd", "ALU[y] r[z]", "SBC A", "B"])] // SBC A, B
#[case::fd_sbc_a_c(0x29f9, &[0xfd, 0x99], &["decode_fd", "ALU[y] r[z]", "SBC A", "C"])] // SBC A, C
#[case::fd_sbc_a_d(0x29f9, &[0xfd, 0x9A], &["decode_fd", "ALU[y] r[z]", "SBC A", "D"])] // SBC A, D
#[case::fd_sbc_a_e(0x29f9, &[0xfd, 0x9B], &["decode_fd", "ALU[y] r[z]", "SBC A", "E"])] // SBC A, E
#[case::fd_sbc_a_iyh(0x29f9, &[0xfd, 0x9C], &["decode_fd", "ALU[y] r[z]", "SBC A", "H", "IYH"])] // SBC A, IYH
#[case::fd_sbc_a_iyl(0x29f9, &[0xfd, 0x9D], &["decode_fd", "ALU[y] r[z]", "SBC A", "L", "IYL"])] // SBC A, IYL
#[case::fd_sbc_a_iyd(0x29f9, &[0xfd, 0x9E, 0x05], &["decode_fd", "ALU[y] r[z]", "SBC A", "(HL)", "(IY + d)"])] // SBC A, (IY + d)
#[case::fd_sbc_a_a(0x29f9, &[0xfd, 0x9F], &["decode_fd", "ALU[y] r[z]", "SBC A", "A"])] // SBC A, A
#[case::fd_and_a_b(0x29f9, &[0xfd, 0xA0], &["decode_fd", "ALU[y] r[z]", "AND A", "B"])] // AND A, B
#[case::fd_and_a_c(0x29f9, &[0xfd, 0xA1], &["decode_fd", "ALU[y] r[z]", "AND A", "C"])] // AND A, C
#[case::fd_and_a_d(0x29f9, &[0xfd, 0xA2], &["decode_fd", "ALU[y] r[z]", "AND A", "D"])] // AND A, D
#[case::fd_and_a_e(0x29f9, &[0xfd, 0xA3], &["decode_fd", "ALU[y] r[z]", "AND A", "E"])] // AND A, E
#[case::fd_and_a_iyh(0x29f9, &[0xfd, 0xA4], &["decode_fd", "ALU[y] r[z]", "AND A", "H", "IYH"])] // AND A, IYH
#[case::fd_and_a_iyl(0x29f9, &[0xfd, 0xA5], &["decode_fd", "ALU[y] r[z]", "AND A", "L", "IYL"])] // AND A, IYL
#[case::fd_and_a_iyd(0x29f9, &[0xfd, 0xA6, 0x05], &["decode_fd", "ALU[y] r[z]", "AND A", "(HL)", "(IY + d)"])] // AND A, (IY + d)
#[case::fd_and_a_a(0x29f9, &[0xfd, 0xA7], &["decode_fd", "ALU[y] r[z]", "AND A", "A"])] // AND A, A
#[case::fd_xor_a_b(0x29f9, &[0xfd, 0xA8], &["decode_fd", "ALU[y] r[z]", "XOR A", "B"])] // XOR A, B
#[case::fd_xor_a_c(0x29f9, &[0xfd, 0xA9], &["decode_fd", "ALU[y] r[z]", "XOR A", "C"])] // XOR A, C
#[case::fd_xor_a_d(0x29f9, &[0xfd, 0xAA], &["decode_fd", "ALU[y] r[z]", "XOR A", "D"])] // XOR A, D
#[case::fd_xor_a_e(0x29f9, &[0xfd, 0xAB], &["decode_fd", "ALU[y] r[z]", "XOR A", "E"])] // XOR A, E
#[case::fd_xor_a_iyh(0x29f9, &[0xfd, 0xAC], &["decode_fd", "ALU[y] r[z]", "XOR A", "H", "IYH"])] // XOR A, IYH
#[case::fd_xor_a_iyl(0x29f9, &[0xfd, 0xAD], &["decode_fd", "ALU[y] r[z]", "XOR A", "L", "IYL"])] // XOR A, IYL
#[case::fd_xor_a_iyd(0x29f9, &[0xfd, 0xAE, 0x05], &["decode_fd", "ALU[y] r[z]", "XOR A", "(HL)", "(IY + d)"])] // XOR A, (IY + d)
#[case::fd_xor_a_a(0x29f9, &[0xfd, 0xAF], &["decode_fd", "ALU[y] r[z]", "XOR A", "A"])] // XOR A, A
#[case::fd_or_a_b(0x29f9, &[0xfd, 0xB0], &["decode_fd", "ALU[y] r[z]", "OR A", "B"])] // OR A, B
#[case::fd_or_a_c(0x29f9, &[0xfd, 0xB1], &["decode_fd", "ALU[y] r[z]", "OR A", "C"])] // OR A, C
#[case::fd_or_a_d(0x29f9, &[0xfd, 0xB2], &["decode_fd", "ALU[y] r[z]", "OR A", "D"])] // OR A, D
#[case::fd_or_a_e(0x29f9, &[0xfd, 0xB3], &["decode_fd", "ALU[y] r[z]", "OR A", "E"])] // OR A, E
#[case::fd_or_a_iyh(0x29f9, &[0xfd, 0xB4], &["decode_fd", "ALU[y] r[z]", "OR A", "H", "IYH"])] // OR A, IYH
#[case::fd_or_a_iyl(0x29f9, &[0xfd, 0xB5], &["decode_fd", "ALU[y] r[z]", "OR A", "L", "IYL"])] // OR A, IYL
#[case::fd_or_a_iyd(0x29f9, &[0xfd, 0xB6, 0x05], &["decode_fd", "ALU[y] r[z]", "OR A", "(HL)", "(IY + d)"])] // OR A, (IY + d)
#[case::fd_or_a_a(0x29f9, &[0xfd, 0xB7], &["decode_fd", "ALU[y] r[z]", "OR A", "A"])] // OR A, A
#[case::fd_cp_a_b(0x29f9, &[0xfd, 0xB8], &["decode_fd", "ALU[y] r[z]", "CP A", "B"])] // CP A, B
#[case::fd_cp_a_c(0x29f9, &[0xfd, 0xB9], &["decode_fd", "ALU[y] r[z]", "CP A", "C"])] // CP A, C
#[case::fd_cp_a_d(0x29f9, &[0xfd, 0xBA], &["decode_fd", "ALU[y] r[z]", "CP A", "D"])] // CP A, D
#[case::fd_cp_a_e(0x29f9, &[0xfd, 0xBB], &["decode_fd", "ALU[y] r[z]", "CP A", "E"])] // CP A, E
#[case::fd_cp_a_iyh(0x29f9, &[0xfd, 0xBC], &["decode_fd", "ALU[y] r[z]", "CP A", "H", "IYH"])] // CP A, IYH
#[case::fd_cp_a_iyl(0x29f9, &[0xfd, 0xBD], &["decode_fd", "ALU[y] r[z]", "CP A", "L", "IYL"])] // CP A, IYL
#[case::fd_cp_a_iyd(0x29f9, &[0xfd, 0xBE, 0x05], &["decode_fd", "ALU[y] r[z]", "CP A", "(HL)", "(IY + d)"])] // CP A, (IY + d)
#[case::fd_cp_a_a(0x29f9, &[0xfd, 0xBF], &["decode_fd", "ALU[y] r[z]", "CP A", "A"])] // CP A, A
#[case::fd_pop_iy(0x29f9, &[0xfd, 0xe1], &["decode_fd", "POP rp2[p]", "HL", "IY"])] // POP IY
#[case::fd_ex_sp_iy(0x29f9, &[0xfd, 0xe3], &["decode_fd", "EX (SP), HL/IX/IY", "IY"])] // EX (SP), IY
#[case::fd_push_iy(0x29f9, &[0xfd, 0xe5], &["decode_fd", "PUSH rp2[p]", "HL", "IY"])] // PUSH IY
#[case::fd_jp_iy(0x29f9, &[0xfd, 0xe9], &["decode_fd", "JP HL", "IY"])] // JP (IY)
#[case::fd_ld_sp_iy(0x29f9, &[0xfd, 0xf9], &["decode_fd", "LD SP, HL", "IY"])] // LD SP, IY
// DOUBLE PREFIXED
// DDCB
#[case::ddcb_rlc_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0x00], &["decode_ddcb", "rot[y] (IX + d)", "RLC", "B"])] // RLC (IX + d), B
#[case::ddcb_rlc_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0x01], &["decode_ddcb", "rot[y] (IX + d)", "RLC", "C"])] // RLC (IX + d), C
#[case::ddcb_rlc_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0x02], &["decode_ddcb", "rot[y] (IX + d)", "RLC", "D"])] // RLC (IX + d), D
#[case::ddcb_rlc_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0x03], &["decode_ddcb", "rot[y] (IX + d)", "RLC", "E"])] // RLC (IX + d), E
#[case::ddcb_rlc_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0x04], &["decode_ddcb", "rot[y] (IX + d)", "RLC", "H"])] // RLC (IX + d), H
#[case::ddcb_rlc_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0x05], &["decode_ddcb", "rot[y] (IX + d)", "RLC", "L"])] // RLC (IX + d), L
#[case::ddcb_rlc_ixd(0x29f9, &[0xdd, 0xcb, 0x05, 0x06], &["decode_ddcb", "rot[y] (IX + d)", "RLC"])] // RLC (IX + d)
#[case::ddcb_rlc_ixd_a(0x29f9, &[0xdd, 0xcb, 0x05, 0x07], &["decode_ddcb", "rot[y] (IX + d)", "RLC", "A"])] // RLC (IX + d), A
#[case::ddcb_rrc_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0x08], &["decode_ddcb", "rot[y] (IX + d)", "RRC", "B"])] // RRC (IX + d), B
#[case::ddcb_rrc_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0x09], &["decode_ddcb", "rot[y] (IX + d)", "RRC", "C"])] // RRC (IX + d), C
#[case::ddcb_rrc_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0x0A], &["decode_ddcb", "rot[y] (IX + d)", "RRC", "D"])] // RRC (IX + d), D
#[case::ddcb_rrc_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0x0B], &["decode_ddcb", "rot[y] (IX + d)", "RRC", "E"])] // RRC (IX + d), E
#[case::ddcb_rrc_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0x0C], &["decode_ddcb", "rot[y] (IX + d)", "RRC", "H"])] // RRC (IX + d), H
#[case::ddcb_rrc_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0x0D], &["decode_ddcb", "rot[y] (IX + d)", "RRC", "L"])] // RRC (IX + d), L
#[case::ddcb_rrc_ixd(0x29f9, &[0xdd, 0xcb, 0x15, 0x0E], &["decode_ddcb", "rot[y] (IX + d)", "RRC"])] // RRC (IX + d)
#[case::ddcb_rrc_ixd_a(0x29f9, &[0xdd, 0xcb, 0x15, 0x0F], &["decode_ddcb", "rot[y] (IX + d)", "RRC", "A"])] // RRC (IX + d), A
#[case::ddcb_rl_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0x10], &["decode_ddcb", "rot[y] (IX + d)", "RL", "B"])] // RL (IX + d), B
#[case::ddcb_rl_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0x11], &["decode_ddcb", "rot[y] (IX + d)", "RL", "C"])] // RL (IX + d), C
#[case::ddcb_rl_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0x12], &["decode_ddcb", "rot[y] (IX + d)", "RL", "D"])] // RL (IX + d), D
#[case::ddcb_rl_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0x13], &["decode_ddcb", "rot[y] (IX + d)", "RL", "E"])] // RL (IX + d), E
#[case::ddcb_rl_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0x14], &["decode_ddcb", "rot[y] (IX + d)", "RL", "H"])] // RL (IX + d), H
#[case::ddcb_rl_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0x15], &["decode_ddcb", "rot[y] (IX + d)", "RL", "L"])] // RL (IX + d), L
#[case::ddcb_rl_ixd(0x29f9, &[0xdd, 0xcb, 0x15, 0x16], &["decode_ddcb", "rot[y] (IX + d)", "RL"])] // RL (IX + d)
#[case::ddcb_rl_ixd_a(0x29f9, &[0xdd, 0xcb, 0x15, 0x17], &["decode_ddcb", "rot[y] (IX + d)", "RL", "A"])] // RL (IX + d), A
#[case::ddcb_rr_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0x18], &["decode_ddcb", "rot[y] (IX + d)", "RR", "B"])] // RR (IX + d), B
#[case::ddcb_rr_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0x19], &["decode_ddcb", "rot[y] (IX + d)", "RR", "C"])] // RR (IX + d), C
#[case::ddcb_rr_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0x1A], &["decode_ddcb", "rot[y] (IX + d)", "RR", "D"])] // RR (IX + d), D
#[case::ddcb_rr_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0x1B], &["decode_ddcb", "rot[y] (IX + d)", "RR", "E"])] // RR (IX + d), E
#[case::ddcb_rr_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0x1C], &["decode_ddcb", "rot[y] (IX + d)", "RR", "H"])] // RR (IX + d), H
#[case::ddcb_rr_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0x1D], &["decode_ddcb", "rot[y] (IX + d)", "RR", "L"])] // RR (IX + d), L
#[case::ddcb_rr_ixd(0x29f9, &[0xdd, 0xcb, 0x15, 0x1E], &["decode_ddcb", "rot[y] (IX + d)", "RR"])] // RR (IX + d)
#[case::ddcb_rr_ixd_a(0x29f9, &[0xdd, 0xcb, 0x15, 0x1F], &["decode_ddcb", "rot[y] (IX + d)", "RR", "A"])] // RR (IX + d), A
#[case::ddcb_sla_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0x20], &["decode_ddcb", "rot[y] (IX + d)", "SLA", "B"])] // SLA (IX + d), B
#[case::ddcb_sla_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0x21], &["decode_ddcb", "rot[y] (IX + d)", "SLA", "C"])] // SLA (IX + d), C
#[case::ddcb_sla_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0x22], &["decode_ddcb", "rot[y] (IX + d)", "SLA", "D"])] // SLA (IX + d), D
#[case::ddcb_sla_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0x23], &["decode_ddcb", "rot[y] (IX + d)", "SLA", "E"])] // SLA (IX + d), E
#[case::ddcb_sla_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0x24], &["decode_ddcb", "rot[y] (IX + d)", "SLA", "H"])] // SLA (IX + d), H
#[case::ddcb_sla_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0x25], &["decode_ddcb", "rot[y] (IX + d)", "SLA", "L"])] // SLA (IX + d), L
#[case::ddcb_sla_ixd(0x29f9, &[0xdd, 0xcb, 0x15, 0x26], &["decode_ddcb", "rot[y] (IX + d)", "SLA"])] // SLA (IX + d)
#[case::ddcb_sla_ixd_a(0x29f9, &[0xdd, 0xcb, 0x15, 0x27], &["decode_ddcb", "rot[y] (IX + d)", "SLA", "A"])] // SLA (IX + d), A
#[case::ddcb_sra_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0x28], &["decode_ddcb", "rot[y] (IX + d)", "SRA", "B"])] // SRA (IX + d), B
#[case::ddcb_sra_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0x29], &["decode_ddcb", "rot[y] (IX + d)", "SRA", "C"])] // SRA (IX + d), C
#[case::ddcb_sra_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0x2A], &["decode_ddcb", "rot[y] (IX + d)", "SRA", "D"])] // SRA (IX + d), D
#[case::ddcb_sra_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0x2B], &["decode_ddcb", "rot[y] (IX + d)", "SRA", "E"])] // SRA (IX + d), E
#[case::ddcb_sra_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0x2C], &["decode_ddcb", "rot[y] (IX + d)", "SRA", "H"])] // SRA (IX + d), H
#[case::ddcb_sra_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0x2D], &["decode_ddcb", "rot[y] (IX + d)", "SRA", "L"])] // SRA (IX + d), L
#[case::ddcb_sra_ixd(0x29f9, &[0xdd, 0xcb, 0x15, 0x2E], &["decode_ddcb", "rot[y] (IX + d)", "SRA"])] // SRA (IX + d)
#[case::ddcb_sra_ixd_a(0x29f9, &[0xdd, 0xcb, 0x15, 0x2F], &["decode_ddcb", "rot[y] (IX + d)", "SRA", "A"])] // SRA (IX + d), A
#[case::ddcb_sll_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0x30], &["decode_ddcb", "rot[y] (IX + d)", "SLL", "B"])] // SLL (IX + d), B
#[case::ddcb_sll_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0x31], &["decode_ddcb", "rot[y] (IX + d)", "SLL", "C"])] // SLL (IX + d), C
#[case::ddcb_sll_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0x32], &["decode_ddcb", "rot[y] (IX + d)", "SLL", "D"])] // SLL (IX + d), D
#[case::ddcb_sll_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0x33], &["decode_ddcb", "rot[y] (IX + d)", "SLL", "E"])] // SLL (IX + d), E
#[case::ddcb_sll_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0x34], &["decode_ddcb", "rot[y] (IX + d)", "SLL", "H"])] // SLL (IX + d), H
#[case::ddcb_sll_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0x35], &["decode_ddcb", "rot[y] (IX + d)", "SLL", "L"])] // SLL (IX + d), L
#[case::ddcb_sll_ixd(0x29f9, &[0xdd, 0xcb, 0x15, 0x36], &["decode_ddcb", "rot[y] (IX + d)", "SLL"])] // SLL (IX + d)
#[case::ddcb_sll_ixd_a(0x29f9, &[0xdd, 0xcb, 0x15, 0x37], &["decode_ddcb", "rot[y] (IX + d)", "SLL", "A"])] // SLL (IX + d), A
#[case::ddcb_srl_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0x38], &["decode_ddcb", "rot[y] (IX + d)", "SRL", "B"])] // SRL (IX + d), B
#[case::ddcb_srl_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0x39], &["decode_ddcb", "rot[y] (IX + d)", "SRL", "C"])] // SRL (IX + d), C
#[case::ddcb_srl_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0x3A], &["decode_ddcb", "rot[y] (IX + d)", "SRL", "D"])] // SRL (IX + d), D
#[case::ddcb_srl_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0x3B], &["decode_ddcb", "rot[y] (IX + d)", "SRL", "E"])] // SRL (IX + d), E
#[case::ddcb_srl_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0x3C], &["decode_ddcb", "rot[y] (IX + d)", "SRL", "H"])] // SRL (IX + d), H
#[case::ddcb_srl_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0x3D], &["decode_ddcb", "rot[y] (IX + d)", "SRL", "L"])] // SRL (IX + d), L
#[case::ddcb_srl_ixd(0x29f9, &[0xdd, 0xcb, 0x15, 0x3E], &["decode_ddcb", "rot[y] (IX + d)", "SRL"])] // SRL (IX + d)
#[case::ddcb_srl_ixd_a(0x29f9, &[0xdd, 0xcb, 0x15, 0x3F], &["decode_ddcb", "rot[y] (IX + d)", "SRL", "A"])] // SRL (IX + d), A
// BITS ARE IN RANGES
#[case::ddcb_res_0_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0x80], &["decode_ddcb", "RES y, (IX + d)", "B"])] // RES 0, (IX + d), B
#[case::ddcb_res_0_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0x81], &["decode_ddcb", "RES y, (IX + d)", "C"])] // RES 0, (IX + d), C
#[case::ddcb_res_0_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0x82], &["decode_ddcb", "RES y, (IX + d)", "D"])] // RES 0, (IX + d), D
#[case::ddcb_res_0_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0x83], &["decode_ddcb", "RES y, (IX + d)", "E"])] // RES 0, (IX + d), E
#[case::ddcb_res_0_ixd_h(0x29f9, &[0xdd, 0xcb, 0x15, 0x84], &["decode_ddcb", "RES y, (IX + d)", "H"])] // RES 0, (IX + d), H
#[case::ddcb_res_0_ixd_l(0x29f9, &[0xdd, 0xcb, 0x15, 0x85], &["decode_ddcb", "RES y, (IX + d)", "L"])] // RES 0, (IX + d), L
#[case::ddcb_res_0_ixd(0x29f9, &[0xdd, 0xcb, 0x05, 0x86], &["decode_ddcb", "RES y, (IX + d)"])] // RES 0, (IX + d)
#[case::ddcb_res_0_ixd_a(0x29f9, &[0xdd, 0xcb, 0x05, 0x87], &["decode_ddcb", "RES y, (IX + d)", "A"])] // RES 0, (IX + d), A
#[case::ddcb_res_1_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0x88], &["decode_ddcb", "RES y, (IX + d)", "B"])] // RES 1, (IX + d), B
#[case::ddcb_res_1_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0x89], &["decode_ddcb", "RES y, (IX + d)", "C"])] // RES 1, (IX + d), C
#[case::ddcb_res_1_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0x8A], &["decode_ddcb", "RES y, (IX + d)", "D"])] // RES 1, (IX + d), D
#[case::ddcb_res_1_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0x8B], &["decode_ddcb", "RES y, (IX + d)", "E"])] // RES 1, (IX + d), E
#[case::ddcb_res_1_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0x8C], &["decode_ddcb", "RES y, (IX + d)", "H"])] // RES 1, (IX + d), H
#[case::ddcb_res_1_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0x8D], &["decode_ddcb", "RES y, (IX + d)", "L"])] // RES 1, (IX + d), L
#[case::ddcb_res_1_ixd(0x29f9, &[0xdd, 0xcb, 0x05, 0x8E], &["decode_ddcb", "RES y, (IX + d)"])] // RES 1, (IX + d)
#[case::ddcb_res_1_ixd_a(0x29f9, &[0xdd, 0xcb, 0x15, 0x8F], &["decode_ddcb", "RES y, (IX + d)", "A"])] // RES 1,( IX + d), A
#[case::ddcb_res_2_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0x90], &["decode_ddcb", "RES y, (IX + d)", "B"])] // RES 2, (IX + d), B
#[case::ddcb_res_2_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0x91], &["decode_ddcb", "RES y, (IX + d)", "C"])] // RES 2, (IX + d), C
#[case::ddcb_res_2_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0x92], &["decode_ddcb", "RES y, (IX + d)", "D"])] // RES 2, (IX + d), D
#[case::ddcb_res_2_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0x93], &["decode_ddcb", "RES y, (IX + d)", "E"])] // RES 2, (IX + d), E
#[case::ddcb_res_2_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0x94], &["decode_ddcb", "RES y, (IX + d)", "H"])] // RES 2, (IX + d), H
#[case::ddcb_res_2_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0x95], &["decode_ddcb", "RES y, (IX + d)", "L"])] // RES 2, (IX + d), L
#[case::ddcb_res_2_ixd(0x29f9, &[0xdd, 0xcb, 0x05, 0x96], &["decode_ddcb", "RES y, (IX + d)"])] // RES 2, (IX + d)
#[case::ddcb_res_2_ixd_a(0x29f9, &[0xdd, 0xcb, 0x05, 0x97], &["decode_ddcb", "RES y, (IX + d)", "A"])] // RES 2, (IX + d), A
#[case::ddcb_res_3_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0x98], &["decode_ddcb", "RES y, (IX + d)", "B"])] // RES 3, (IX + d), B
#[case::ddcb_res_3_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0x99], &["decode_ddcb", "RES y, (IX + d)", "C"])] // RES 3, (IX + d), C
#[case::ddcb_res_3_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0x9A], &["decode_ddcb", "RES y, (IX + d)", "D"])] // RES 3, (IX + d), D
#[case::ddcb_res_3_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0x9B], &["decode_ddcb", "RES y, (IX + d)", "E"])] // RES 3, (IX + d), E
#[case::ddcb_res_3_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0x9C], &["decode_ddcb", "RES y, (IX + d)", "H"])] // RES 3, (IX + d), H
#[case::ddcb_res_3_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0x9D], &["decode_ddcb", "RES y, (IX + d)", "L"])] // RES 3, (IX + d), L
#[case::ddcb_res_3_ixd(0x29f9, &[0xdd, 0xcb, 0x05, 0x9E], &["decode_ddcb", "RES y, (IX + d)"])] // RES 3, (IX + d)
#[case::ddcb_res_3_ixd_a(0x29f9, &[0xdd, 0xcb, 0x15, 0x9F], &["decode_ddcb", "RES y, (IX + d)", "A"])] // RES 3, (IX + d)
#[case::ddcb_res_4_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0xA0], &["decode_ddcb", "RES y, (IX + d)", "B"])] // RES 4, (IX + d), B
#[case::ddcb_res_4_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0xA1], &["decode_ddcb", "RES y, (IX + d)", "C"])] // RES 4, (IX + d), C
#[case::ddcb_res_4_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0xA2], &["decode_ddcb", "RES y, (IX + d)", "D"])] // RES 4, (IX + d), D
#[case::ddcb_res_4_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0xA3], &["decode_ddcb", "RES y, (IX + d)", "E"])] // RES 4, (IX + d), E
#[case::ddcb_res_4_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0xA4], &["decode_ddcb", "RES y, (IX + d)", "H"])] // RES 4, (IX + d), H
#[case::ddcb_res_4_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0xA5], &["decode_ddcb", "RES y, (IX + d)", "L"])] // RES 4, (IX + d), L
#[case::ddcb_res_4_ixd(0x29f9, &[0xdd, 0xcb, 0x05, 0xA6], &["decode_ddcb", "RES y, (IX + d)"])] // RES 4, (IX + d)
#[case::ddcb_res_4_ixd_a(0x29f9, &[0xdd, 0xcb, 0x05, 0xA7], &["decode_ddcb", "RES y, (IX + d)", "A"])] // RES 4, (IX + d), A
#[case::ddcb_res_5_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0xA8], &["decode_ddcb", "RES y, (IX + d)", "B"])] // RES 5, (IX + d), B
#[case::ddcb_res_5_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0xA9], &["decode_ddcb", "RES y, (IX + d)", "C"])] // RES 5, (IX + d), C
#[case::ddcb_res_5_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0xAA], &["decode_ddcb", "RES y, (IX + d)", "D"])] // RES 5, (IX + d), D
#[case::ddcb_res_5_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0xAB], &["decode_ddcb", "RES y, (IX + d)", "E"])] // RES 5, (IX + d), E
#[case::ddcb_res_5_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0xAC], &["decode_ddcb", "RES y, (IX + d)", "H"])] // RES 5, (IX + d), H
#[case::ddcb_res_5_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0xAD], &["decode_ddcb", "RES y, (IX + d)", "L"])] // RES 5, (IX + d), L
#[case::ddcb_res_5_ixd(0x29f9, &[0xdd, 0xcb, 0x05, 0xAE], &["decode_ddcb", "RES y, (IX + d)"])] // RES 5, (IX + d)
#[case::ddcb_res_5_ixd_a(0x29f9, &[0xdd, 0xcb, 0x05, 0xAF], &["decode_ddcb", "RES y, (IX + d)", "A"])] // RES 5, (IX + d), A
#[case::ddcb_res_6_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0xB0], &["decode_ddcb", "RES y, (IX + d)", "B"])] // RES 6, (IX + d), B
#[case::ddcb_res_6_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0xB1], &["decode_ddcb", "RES y, (IX + d)", "C"])] // RES 6, (IX + d), C
#[case::ddcb_res_6_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0xB2], &["decode_ddcb", "RES y, (IX + d)", "D"])] // RES 6, (IX + d), D
#[case::ddcb_res_6_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0xB3], &["decode_ddcb", "RES y, (IX + d)", "E"])] // RES 6, (IX + d), E
#[case::ddcb_res_6_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0xB4], &["decode_ddcb", "RES y, (IX + d)", "H"])] // RES 6, (IX + d), H
#[case::ddcb_res_6_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0xB5], &["decode_ddcb", "RES y, (IX + d)", "L"])] // RES 6, (IX + d), L
#[case::ddcb_res_6_ixd(0x29f9, &[0xdd, 0xcb, 0x05, 0xB6], &["decode_ddcb", "RES y, (IX + d)"])] // RES 6, (IX + d)
#[case::ddcb_res_6_ixd_a(0x29f9, &[0xdd, 0xcb, 0x05, 0xB7], &["decode_ddcb", "RES y, (IX + d)", "A"])] // RES 6, (IX + d), A
#[case::ddcb_res_7_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0xB8], &["decode_ddcb", "RES y, (IX + d)", "B"])] // RES 7, (IX + d), B
#[case::ddcb_res_7_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0xB9], &["decode_ddcb", "RES y, (IX + d)", "C"])] // RES 7, (IX + d), C
#[case::ddcb_res_7_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0xBA], &["decode_ddcb", "RES y, (IX + d)", "D"])] // RES 7, (IX + d), D
#[case::ddcb_res_7_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0xBB], &["decode_ddcb", "RES y, (IX + d)", "E"])] // RES 7, (IX + d), E
#[case::ddcb_res_7_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0xBC], &["decode_ddcb", "RES y, (IX + d)", "H"])] // RES 7, (IX + d), H
#[case::ddcb_res_7_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0xBD], &["decode_ddcb", "RES y, (IX + d)", "L"])] // RES 7, (IX + d), L
#[case::ddcb_res_7_ixd(0x29f9, &[0xdd, 0xcb, 0x05, 0xBE], &["decode_ddcb", "RES y, (IX + d)"])] // RES 7, (IX + d)
#[case::ddcb_res_7_ixd_a(0x29f9, &[0xdd, 0xcb, 0x05, 0xBF], &["decode_ddcb", "RES y, (IX + d)", "A"])] // RES 7, (IX + d), A
#[case::ddcb_set_0_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0xC0], &["decode_ddcb", "SET y, (IX + d)", "B"])] // SET 0, (IX + d), B
#[case::ddcb_set_0_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0xC1], &["decode_ddcb", "SET y, (IX + d)", "C"])] // SET 0, (IX + d), C
#[case::ddcb_set_0_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0xC2], &["decode_ddcb", "SET y, (IX + d)", "D"])] // SET 0, (IX + d), D
#[case::ddcb_set_0_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0xC3], &["decode_ddcb", "SET y, (IX + d)", "E"])] // SET 0, (IX + d), E
#[case::ddcb_set_0_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0xC4], &["decode_ddcb", "SET y, (IX + d)", "H"])] // SET 0, (IX + d), H
#[case::ddcb_set_0_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0xC5], &["decode_ddcb", "SET y, (IX + d)", "L"])] // SET 0, (IX + d), L
#[case::ddcb_set_0_ixd(0x29f9, &[0xdd, 0xcb, 0x05, 0xC6], &["decode_ddcb", "SET y, (IX + d)"])] // SET 0, (IX + d)
#[case::ddcb_set_0_ixd_a(0x29f9, &[0xdd, 0xcb, 0x05, 0xC7], &["decode_ddcb", "SET y, (IX + d)", "A"])] // SET 0, (IX + d), A
#[case::ddcb_set_1_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0xC8], &["decode_ddcb", "SET y, (IX + d)", "B"])] // SET 1, (IX + d), B
#[case::ddcb_set_1_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0xC9], &["decode_ddcb", "SET y, (IX + d)", "C"])] // SET 1, (IX + d), C
#[case::ddcb_set_1_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0xCA], &["decode_ddcb", "SET y, (IX + d)", "D"])] // SET 1, (IX + d), D
#[case::ddcb_set_1_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0xCB], &["decode_ddcb", "SET y, (IX + d)", "E"])] // SET 1, (IX + d), E
#[case::ddcb_set_1_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0xCC], &["decode_ddcb", "SET y, (IX + d)", "H"])] // SET 1, (IX + d), H
#[case::ddcb_set_1_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0xCD], &["decode_ddcb", "SET y, (IX + d)", "L"])] // SET 1, (IX + d), L
#[case::ddcb_set_1_ixd(0x29f9, &[0xdd, 0xcb, 0x05, 0xCE], &["decode_ddcb", "SET y, (IX + d)"])] // SET 1, (IX + d)
#[case::ddcb_set_1_ixd_a(0x29f9, &[0xdd, 0xcb, 0x15, 0xCF], &["decode_ddcb", "SET y, (IX + d)", "A"])] // SET 1, (IX + d), A
#[case::ddcb_set_2_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0xD0], &["decode_ddcb", "SET y, (IX + d)", "B"])] // SET 2, (IX + d), B
#[case::ddcb_set_2_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0xD1], &["decode_ddcb", "SET y, (IX + d)", "C"])] // SET 2, (IX + d), C
#[case::ddcb_set_2_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0xD2], &["decode_ddcb", "SET y, (IX + d)", "D"])] // SET 2, (IX + d), D
#[case::ddcb_set_2_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0xD3], &["decode_ddcb", "SET y, (IX + d)", "E"])] // SET 2, (IX + d), E
#[case::ddcb_set_2_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0xD4], &["decode_ddcb", "SET y, (IX + d)", "H"])] // SET 2, (IX + d), H
#[case::ddcb_set_2_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0xD5], &["decode_ddcb", "SET y, (IX + d)", "L"])] // SET 2, (IX + d), L
#[case::ddcb_set_2_ixd(0x29f9, &[0xdd, 0xcb, 0x05, 0xD6], &["decode_ddcb", "SET y, (IX + d)"])] // SET 2, (IX + d)
#[case::ddcb_set_2_ixd_a(0x29f9, &[0xdd, 0xcb, 0x05, 0xD7], &["decode_ddcb", "SET y, (IX + d)", "A"])] // SET 2, (IX + d), A
#[case::ddcb_set_3_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0xD8], &["decode_ddcb", "SET y, (IX + d)", "B"])] // SET 3, (IX + d), B
#[case::ddcb_set_3_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0xD9], &["decode_ddcb", "SET y, (IX + d)", "C"])] // SET 3, (IX + d), C
#[case::ddcb_set_3_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0xDA], &["decode_ddcb", "SET y, (IX + d)", "D"])] // SET 3, (IX + d), D
#[case::ddcb_set_3_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0xDB], &["decode_ddcb", "SET y, (IX + d)", "E"])] // SET 3, (IX + d), E
#[case::ddcb_set_3_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0xDC], &["decode_ddcb", "SET y, (IX + d)", "H"])] // SET 3, (IX + d), H
#[case::ddcb_set_3_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0xDD], &["decode_ddcb", "SET y, (IX + d)", "L"])] // SET 3, (IX + d), L
#[case::ddcb_set_3_ixd(0x29f9, &[0xdd, 0xcb, 0x05, 0xDE], &["decode_ddcb", "SET y, (IX + d)"])] // SET 3, (IX + d)
#[case::ddcb_set_3_ixd_a(0x29f9, &[0xdd, 0xcb, 0x15, 0xDF], &["decode_ddcb", "SET y, (IX + d)", "A"])] // SET 3, (IX + d), A
#[case::ddcb_set_4_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0xE0], &["decode_ddcb", "SET y, (IX + d)", "B"])] // SET 4, (IX + d), B
#[case::ddcb_set_4_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0xE1], &["decode_ddcb", "SET y, (IX + d)", "C"])] // SET 4, (IX + d), C
#[case::ddcb_set_4_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0xE2], &["decode_ddcb", "SET y, (IX + d)", "D"])] // SET 4, (IX + d), D
#[case::ddcb_set_4_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0xE3], &["decode_ddcb", "SET y, (IX + d)", "E"])] // SET 4, (IX + d), E
#[case::ddcb_set_4_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0xE4], &["decode_ddcb", "SET y, (IX + d)", "H"])] // SET 4, (IX + d), H
#[case::ddcb_set_4_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0xE5], &["decode_ddcb", "SET y, (IX + d)", "L"])] // SET 4, (IX + d), L
#[case::ddcb_set_4_ixd(0x29f9, &[0xdd, 0xcb, 0x05, 0xE6], &["decode_ddcb", "SET y, (IX + d)"])] // SET 4, (IX + d)
#[case::ddcb_set_4_ixd_a(0x29f9, &[0xdd, 0xcb, 0x05, 0xE7], &["decode_ddcb", "SET y, (IX + d)", "A"])] // SET 4, (IX + d), A
#[case::ddcb_set_5_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0xE8], &["decode_ddcb", "SET y, (IX + d)", "B"])] // SET 5, (IX + d), B
#[case::ddcb_set_5_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0xE9], &["decode_ddcb", "SET y, (IX + d)", "C"])] // SET 5, (IX + d), C
#[case::ddcb_set_5_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0xEA], &["decode_ddcb", "SET y, (IX + d)", "D"])] // SET 5, (IX + d), D
#[case::ddcb_set_5_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0xEB], &["decode_ddcb", "SET y, (IX + d)", "E"])] // SET 5, (IX + d), E
#[case::ddcb_set_5_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0xEC], &["decode_ddcb", "SET y, (IX + d)", "H"])] // SET 5, (IX + d), H
#[case::ddcb_set_5_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0xED], &["decode_ddcb", "SET y, (IX + d)", "L"])] // SET 5, (IX + d), L
#[case::ddcb_set_5_ixd(0x29f9, &[0xdd, 0xcb, 0x05, 0xEE], &["decode_ddcb", "SET y, (IX + d)"])] // SET 5, (IX + d)
#[case::ddcb_set_5_ixd_a(0x29f9, &[0xdd, 0xcb, 0x05, 0xEF], &["decode_ddcb", "SET y, (IX + d)", "A"])] // SET 5, (IX + d), A
#[case::ddcb_set_6_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0xF0], &["decode_ddcb", "SET y, (IX + d)", "B"])] // SET 6, (IX + d), B
#[case::ddcb_set_6_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0xF1], &["decode_ddcb", "SET y, (IX + d)", "C"])] // SET 6, (IX + d), C
#[case::ddcb_set_6_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0xF2], &["decode_ddcb", "SET y, (IX + d)", "D"])] // SET 6, (IX + d), D
#[case::ddcb_set_6_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0xF3], &["decode_ddcb", "SET y, (IX + d)", "E"])] // SET 6, (IX + d), E
#[case::ddcb_set_6_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0xF4], &["decode_ddcb", "SET y, (IX + d)", "H"])] // SET 6, (IX + d), H
#[case::ddcb_set_6_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0xF5], &["decode_ddcb", "SET y, (IX + d)", "L"])] // SET 6, (IX + d), L
#[case::ddcb_set_6_ixd(0x29f9, &[0xdd, 0xcb, 0x05, 0xF6], &["decode_ddcb", "SET y, (IX + d)"])] // SET 6, (IX + d)
#[case::ddcb_set_6_ixd_a(0x29f9, &[0xdd, 0xcb, 0x05, 0xF7], &["decode_ddcb", "SET y, (IX + d)", "A"])] // SET 6, (IX + d), A
#[case::ddcb_set_7_ixd_b(0x29f9, &[0xdd, 0xcb, 0x05, 0xF8], &["decode_ddcb", "SET y, (IX + d)", "B"])] // SET 7, (IX + d), B
#[case::ddcb_set_7_ixd_c(0x29f9, &[0xdd, 0xcb, 0x05, 0xF9], &["decode_ddcb", "SET y, (IX + d)", "C"])] // SET 7, (IX + d), C
#[case::ddcb_set_7_ixd_d(0x29f9, &[0xdd, 0xcb, 0x05, 0xFA], &["decode_ddcb", "SET y, (IX + d)", "D"])] // SET 7, (IX + d), D
#[case::ddcb_set_7_ixd_e(0x29f9, &[0xdd, 0xcb, 0x05, 0xFB], &["decode_ddcb", "SET y, (IX + d)", "E"])] // SET 7, (IX + d), E
#[case::ddcb_set_7_ixd_h(0x29f9, &[0xdd, 0xcb, 0x05, 0xFC], &["decode_ddcb", "SET y, (IX + d)", "H"])] // SET 7, (IX + d), H
#[case::ddcb_set_7_ixd_l(0x29f9, &[0xdd, 0xcb, 0x05, 0xFD], &["decode_ddcb", "SET y, (IX + d)", "L"])] // SET 7, (IX + d), L
#[case::ddcb_set_7_ixd(0x29f9, &[0xdd, 0xcb, 0x05, 0xFE], &["decode_ddcb", "SET y, (IX + d)"])] // SET 7, (IX + d)
#[case::ddcb_set_7_ixd_a(0x29f9, &[0xdd, 0xcb, 0x05, 0xFF], &["decode_ddcb", "SET y, (IX + d)", "A"])] // SET 7, (IX + d), A
// FDCB
#[case::fdcb_rlc_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0x00], &["decode_fdcb", "rot[y] (IY + d)", "RLC", "B"])] // RLC (IY + d), B
#[case::fdcb_rlc_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0x01], &["decode_fdcb", "rot[y] (IY + d)", "RLC", "C"])] // RLC (IY + d), C
#[case::fdcb_rlc_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0x02], &["decode_fdcb", "rot[y] (IY + d)", "RLC", "D"])] // RLC (IY + d), D
#[case::fdcb_rlc_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0x03], &["decode_fdcb", "rot[y] (IY + d)", "RLC", "E"])] // RLC (IY + d), E
#[case::fdcb_rlc_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0x04], &["decode_fdcb", "rot[y] (IY + d)", "RLC", "H"])] // RLC (IY + d), H
#[case::fdcb_rlc_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0x05], &["decode_fdcb", "rot[y] (IY + d)", "RLC", "L"])] // RLC (IY + d), L
#[case::fdcb_rlc_iyd(0x29f9, &[0xfd, 0xcb, 0x05, 0x06], &["decode_fdcb", "rot[y] (IY + d)", "RLC"])] // RLC (IY + d)
#[case::fdcb_rlc_iyd_a(0x29f9, &[0xfd, 0xcb, 0x05, 0x07], &["decode_fdcb", "rot[y] (IY + d)", "RLC", "A"])] // RLC (IY + d), A
#[case::fdcb_rrc_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0x08], &["decode_fdcb", "rot[y] (IY + d)", "RRC", "B"])] // RRC (IY + d), B
#[case::fdcb_rrc_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0x09], &["decode_fdcb", "rot[y] (IY + d)", "RRC", "C"])] // RRC (IY + d), C
#[case::fdcb_rrc_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0x0A], &["decode_fdcb", "rot[y] (IY + d)", "RRC", "D"])] // RRC (IY + d), D
#[case::fdcb_rrc_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0x0B], &["decode_fdcb", "rot[y] (IY + d)", "RRC", "E"])] // RRC (IY + d), E
#[case::fdcb_rrc_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0x0C], &["decode_fdcb", "rot[y] (IY + d)", "RRC", "H"])] // RRC (IY + d), H
#[case::fdcb_rrc_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0x0D], &["decode_fdcb", "rot[y] (IY + d)", "RRC", "L"])] // RRC (IY + d), L
#[case::fdcb_rrc_iyd(0x29f9, &[0xfd, 0xcb, 0x15, 0x0E], &["decode_fdcb", "rot[y] (IY + d)", "RRC"])] // RRC (IY + d)
#[case::fdcb_rrc_iyd_a(0x29f9, &[0xfd, 0xcb, 0x15, 0x0F], &["decode_fdcb", "rot[y] (IY + d)", "RRC", "A"])] // RRC (IY + d), A
#[case::fdcb_rl_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0x10], &["decode_fdcb", "rot[y] (IY + d)", "RL", "B"])] // RL (IY + d), B
#[case::fdcb_rl_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0x11], &["decode_fdcb", "rot[y] (IY + d)", "RL", "C"])] // RL (IY + d), C
#[case::fdcb_rl_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0x12], &["decode_fdcb", "rot[y] (IY + d)", "RL", "D"])] // RL (IY + d), D
#[case::fdcb_rl_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0x13], &["decode_fdcb", "rot[y] (IY + d)", "RL", "E"])] // RL (IY + d), E
#[case::fdcb_rl_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0x14], &["decode_fdcb", "rot[y] (IY + d)", "RL", "H"])] // RL (IY + d), H
#[case::fdcb_rl_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0x15], &["decode_fdcb", "rot[y] (IY + d)", "RL", "L"])] // RL (IY + d), L
#[case::fdcb_rl_iyd(0x29f9, &[0xfd, 0xcb, 0x15, 0x16], &["decode_fdcb", "rot[y] (IY + d)", "RL"])] // RL (IY + d)
#[case::fdcb_rl_iyd_a(0x29f9, &[0xfd, 0xcb, 0x15, 0x17], &["decode_fdcb", "rot[y] (IY + d)", "RL", "A"])] // RL (IY + d), A
#[case::fdcb_rr_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0x18], &["decode_fdcb", "rot[y] (IY + d)", "RR", "B"])] // RR (IY + d), B
#[case::fdcb_rr_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0x19], &["decode_fdcb", "rot[y] (IY + d)", "RR", "C"])] // RR (IY + d), C
#[case::fdcb_rr_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0x1A], &["decode_fdcb", "rot[y] (IY + d)", "RR", "D"])] // RR (IY + d), D
#[case::fdcb_rr_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0x1B], &["decode_fdcb", "rot[y] (IY + d)", "RR", "E"])] // RR (IY + d), E
#[case::fdcb_rr_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0x1C], &["decode_fdcb", "rot[y] (IY + d)", "RR", "H"])] // RR (IY + d), H
#[case::fdcb_rr_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0x1D], &["decode_fdcb", "rot[y] (IY + d)", "RR", "L"])] // RR (IY + d), L
#[case::fdcb_rr_iyd(0x29f9, &[0xfd, 0xcb, 0x15, 0x1E], &["decode_fdcb", "rot[y] (IY + d)", "RR"])] // RR (IY + d)
#[case::fdcb_rr_iyd_a(0x29f9, &[0xfd, 0xcb, 0x15, 0x1F], &["decode_fdcb", "rot[y] (IY + d)", "RR", "A"])] // RR (IY + d), A
#[case::fdcb_sla_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0x20], &["decode_fdcb", "rot[y] (IY + d)", "SLA", "B"])] // SLA (IY + d), B
#[case::fdcb_sla_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0x21], &["decode_fdcb", "rot[y] (IY + d)", "SLA", "C"])] // SLA (IY + d), C
#[case::fdcb_sla_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0x22], &["decode_fdcb", "rot[y] (IY + d)", "SLA", "D"])] // SLA (IY + d), D
#[case::fdcb_sla_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0x23], &["decode_fdcb", "rot[y] (IY + d)", "SLA", "E"])] // SLA (IY + d), E
#[case::fdcb_sla_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0x24], &["decode_fdcb", "rot[y] (IY + d)", "SLA", "H"])] // SLA (IY + d), H
#[case::fdcb_sla_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0x25], &["decode_fdcb", "rot[y] (IY + d)", "SLA", "L"])] // SLA (IY + d), L
#[case::fdcb_sla_iyd(0x29f9, &[0xfd, 0xcb, 0x15, 0x26], &["decode_fdcb", "rot[y] (IY + d)", "SLA"])] // SLA (IY + d)
#[case::fdcb_sla_iyd_a(0x29f9, &[0xfd, 0xcb, 0x15, 0x27], &["decode_fdcb", "rot[y] (IY + d)", "SLA", "A"])] // SLA (IY + d), A
#[case::fdcb_sra_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0x28], &["decode_fdcb", "rot[y] (IY + d)", "SRA", "B"])] // SRA (IY + d), B
#[case::fdcb_sra_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0x29], &["decode_fdcb", "rot[y] (IY + d)", "SRA", "C"])] // SRA (IY + d), C
#[case::fdcb_sra_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0x2A], &["decode_fdcb", "rot[y] (IY + d)", "SRA", "D"])] // SRA (IY + d), D
#[case::fdcb_sra_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0x2B], &["decode_fdcb", "rot[y] (IY + d)", "SRA", "E"])] // SRA (IY + d), E
#[case::fdcb_sra_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0x2C], &["decode_fdcb", "rot[y] (IY + d)", "SRA", "H"])] // SRA (IY + d), H
#[case::fdcb_sra_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0x2D], &["decode_fdcb", "rot[y] (IY + d)", "SRA", "L"])] // SRA (IY + d), L
#[case::fdcb_sra_iyd(0x29f9, &[0xfd, 0xcb, 0x15, 0x2E], &["decode_fdcb", "rot[y] (IY + d)", "SRA"])] // SRA (IY + d)
#[case::fdcb_sra_iyd_a(0x29f9, &[0xfd, 0xcb, 0x15, 0x2F], &["decode_fdcb", "rot[y] (IY + d)", "SRA", "A"])] // SRA (IY + d), A
#[case::fdcb_sll_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0x30], &["decode_fdcb", "rot[y] (IY + d)", "SLL", "B"])] // SLL (IY + d), B
#[case::fdcb_sll_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0x31], &["decode_fdcb", "rot[y] (IY + d)", "SLL", "C"])] // SLL (IY + d), C
#[case::fdcb_sll_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0x32], &["decode_fdcb", "rot[y] (IY + d)", "SLL", "D"])] // SLL (IY + d), D
#[case::fdcb_sll_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0x33], &["decode_fdcb", "rot[y] (IY + d)", "SLL", "E"])] // SLL (IY + d), E
#[case::fdcb_sll_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0x34], &["decode_fdcb", "rot[y] (IY + d)", "SLL", "H"])] // SLL (IY + d), H
#[case::fdcb_sll_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0x35], &["decode_fdcb", "rot[y] (IY + d)", "SLL", "L"])] // SLL (IY + d), L
#[case::fdcb_sll_iyd(0x29f9, &[0xfd, 0xcb, 0x15, 0x36], &["decode_fdcb", "rot[y] (IY + d)", "SLL"])] // SLL (IY + d)
#[case::fdcb_sll_iyd_a(0x29f9, &[0xfd, 0xcb, 0x15, 0x37], &["decode_fdcb", "rot[y] (IY + d)", "SLL", "A"])] // SLL (IY + d), A
#[case::fdcb_srl_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0x38], &["decode_fdcb", "rot[y] (IY + d)", "SRL", "B"])] // SRL (IY + d), B
#[case::fdcb_srl_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0x39], &["decode_fdcb", "rot[y] (IY + d)", "SRL", "C"])] // SRL (IY + d), C
#[case::fdcb_srl_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0x3A], &["decode_fdcb", "rot[y] (IY + d)", "SRL", "D"])] // SRL (IY + d), D
#[case::fdcb_srl_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0x3B], &["decode_fdcb", "rot[y] (IY + d)", "SRL", "E"])] // SRL (IY + d), E
#[case::fdcb_srl_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0x3C], &["decode_fdcb", "rot[y] (IY + d)", "SRL", "H"])] // SRL (IY + d), H
#[case::fdcb_srl_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0x3D], &["decode_fdcb", "rot[y] (IY + d)", "SRL", "L"])] // SRL (IY + d), L
#[case::fdcb_srl_iyd(0x29f9, &[0xfd, 0xcb, 0x15, 0x3E], &["decode_fdcb", "rot[y] (IY + d)", "SRL"])] // SRL (IY + d)
#[case::fdcb_srl_iyd_a(0x29f9, &[0xfd, 0xcb, 0x15, 0x3F], &["decode_fdcb", "rot[y] (IY + d)", "SRL", "A"])] // SRL (IY + d), A
// BITS ARE IN RANGES
#[case::fdcb_res_0_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0x80], &["decode_fdcb", "RES y, (IY + d)", "B"])] // RES 0, (IY + d), B
#[case::fdcb_res_0_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0x81], &["decode_fdcb", "RES y, (IY + d)", "C"])] // RES 0, (IY + d), C
#[case::fdcb_res_0_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0x82], &["decode_fdcb", "RES y, (IY + d)", "D"])] // RES 0, (IY + d), D
#[case::fdcb_res_0_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0x83], &["decode_fdcb", "RES y, (IY + d)", "E"])] // RES 0, (IY + d), E
#[case::fdcb_res_0_iyd_h(0x29f9, &[0xfd, 0xcb, 0x15, 0x84], &["decode_fdcb", "RES y, (IY + d)", "H"])] // RES 0, (IY + d), H
#[case::fdcb_res_0_iyd_l(0x29f9, &[0xfd, 0xcb, 0x15, 0x85], &["decode_fdcb", "RES y, (IY + d)", "L"])] // RES 0, (IY + d), L
#[case::fdcb_res_0_iyd(0x29f9, &[0xfd, 0xcb, 0x05, 0x86], &["decode_fdcb", "RES y, (IY + d)"])] // RES 0, (IY + d)
#[case::fdcb_res_0_iyd_a(0x29f9, &[0xfd, 0xcb, 0x05, 0x87], &["decode_fdcb", "RES y, (IY + d)", "A"])] // RES 0, (IY + d), A
#[case::fdcb_res_1_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0x88], &["decode_fdcb", "RES y, (IY + d)", "B"])] // RES 1, (IY + d), B
#[case::fdcb_res_1_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0x89], &["decode_fdcb", "RES y, (IY + d)", "C"])] // RES 1, (IY + d), C
#[case::fdcb_res_1_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0x8A], &["decode_fdcb", "RES y, (IY + d)", "D"])] // RES 1, (IY + d), D
#[case::fdcb_res_1_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0x8B], &["decode_fdcb", "RES y, (IY + d)", "E"])] // RES 1, (IY + d), E
#[case::fdcb_res_1_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0x8C], &["decode_fdcb", "RES y, (IY + d)", "H"])] // RES 1, (IY + d), H
#[case::fdcb_res_1_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0x8D], &["decode_fdcb", "RES y, (IY + d)", "L"])] // RES 1, (IY + d), L
#[case::fdcb_res_1_iyd(0x29f9, &[0xfd, 0xcb, 0x05, 0x8E], &["decode_fdcb", "RES y, (IY + d)"])] // RES 1, (IY + d)
#[case::fdcb_res_1_iyd_a(0x29f9, &[0xfd, 0xcb, 0x15, 0x8F], &["decode_fdcb", "RES y, (IY + d)", "A"])] // RES 1,( IY + d), A
#[case::fdcb_res_2_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0x90], &["decode_fdcb", "RES y, (IY + d)", "B"])] // RES 2, (IY + d), B
#[case::fdcb_res_2_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0x91], &["decode_fdcb", "RES y, (IY + d)", "C"])] // RES 2, (IY + d), C
#[case::fdcb_res_2_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0x92], &["decode_fdcb", "RES y, (IY + d)", "D"])] // RES 2, (IY + d), D
#[case::fdcb_res_2_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0x93], &["decode_fdcb", "RES y, (IY + d)", "E"])] // RES 2, (IY + d), E
#[case::fdcb_res_2_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0x94], &["decode_fdcb", "RES y, (IY + d)", "H"])] // RES 2, (IY + d), H
#[case::fdcb_res_2_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0x95], &["decode_fdcb", "RES y, (IY + d)", "L"])] // RES 2, (IY + d), L
#[case::fdcb_res_2_iyd(0x29f9, &[0xfd, 0xcb, 0x05, 0x96], &["decode_fdcb", "RES y, (IY + d)"])] // RES 2, (IY + d)
#[case::fdcb_res_2_iyd_a(0x29f9, &[0xfd, 0xcb, 0x05, 0x97], &["decode_fdcb", "RES y, (IY + d)", "A"])] // RES 2, (IY + d), A
#[case::fdcb_res_3_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0x98], &["decode_fdcb", "RES y, (IY + d)", "B"])] // RES 3, (IY + d), B
#[case::fdcb_res_3_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0x99], &["decode_fdcb", "RES y, (IY + d)", "C"])] // RES 3, (IY + d), C
#[case::fdcb_res_3_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0x9A], &["decode_fdcb", "RES y, (IY + d)", "D"])] // RES 3, (IY + d), D
#[case::fdcb_res_3_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0x9B], &["decode_fdcb", "RES y, (IY + d)", "E"])] // RES 3, (IY + d), E
#[case::fdcb_res_3_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0x9C], &["decode_fdcb", "RES y, (IY + d)", "H"])] // RES 3, (IY + d), H
#[case::fdcb_res_3_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0x9D], &["decode_fdcb", "RES y, (IY + d)", "L"])] // RES 3, (IY + d), L
#[case::fdcb_res_3_iyd(0x29f9, &[0xfd, 0xcb, 0x05, 0x9E], &["decode_fdcb", "RES y, (IY + d)"])] // RES 3, (IY + d)
#[case::fdcb_res_3_iyd_a(0x29f9, &[0xfd, 0xcb, 0x15, 0x9F], &["decode_fdcb", "RES y, (IY + d)", "A"])] // RES 3, (IY + d)
#[case::fdcb_res_4_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0xA0], &["decode_fdcb", "RES y, (IY + d)", "B"])] // RES 4, (IY + d), B
#[case::fdcb_res_4_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0xA1], &["decode_fdcb", "RES y, (IY + d)", "C"])] // RES 4, (IY + d), C
#[case::fdcb_res_4_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0xA2], &["decode_fdcb", "RES y, (IY + d)", "D"])] // RES 4, (IY + d), D
#[case::fdcb_res_4_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0xA3], &["decode_fdcb", "RES y, (IY + d)", "E"])] // RES 4, (IY + d), E
#[case::fdcb_res_4_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0xA4], &["decode_fdcb", "RES y, (IY + d)", "H"])] // RES 4, (IY + d), H
#[case::fdcb_res_4_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0xA5], &["decode_fdcb", "RES y, (IY + d)", "L"])] // RES 4, (IY + d), L
#[case::fdcb_res_4_iyd(0x29f9, &[0xfd, 0xcb, 0x05, 0xA6], &["decode_fdcb", "RES y, (IY + d)"])] // RES 4, (IY + d)
#[case::fdcb_res_4_iyd_a(0x29f9, &[0xfd, 0xcb, 0x05, 0xA7], &["decode_fdcb", "RES y, (IY + d)", "A"])] // RES 4, (IY + d), A
#[case::fdcb_res_5_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0xA8], &["decode_fdcb", "RES y, (IY + d)", "B"])] // RES 5, (IY + d), B
#[case::fdcb_res_5_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0xA9], &["decode_fdcb", "RES y, (IY + d)", "C"])] // RES 5, (IY + d), C
#[case::fdcb_res_5_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0xAA], &["decode_fdcb", "RES y, (IY + d)", "D"])] // RES 5, (IY + d), D
#[case::fdcb_res_5_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0xAB], &["decode_fdcb", "RES y, (IY + d)", "E"])] // RES 5, (IY + d), E
#[case::fdcb_res_5_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0xAC], &["decode_fdcb", "RES y, (IY + d)", "H"])] // RES 5, (IY + d), H
#[case::fdcb_res_5_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0xAD], &["decode_fdcb", "RES y, (IY + d)", "L"])] // RES 5, (IY + d), L
#[case::fdcb_res_5_iyd(0x29f9, &[0xfd, 0xcb, 0x05, 0xAE], &["decode_fdcb", "RES y, (IY + d)"])] // RES 5, (IY + d)
#[case::fdcb_res_5_iyd_a(0x29f9, &[0xfd, 0xcb, 0x05, 0xAF], &["decode_fdcb", "RES y, (IY + d)", "A"])] // RES 5, (IY + d), A
#[case::fdcb_res_6_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0xB0], &["decode_fdcb", "RES y, (IY + d)", "B"])] // RES 6, (IY + d), B
#[case::fdcb_res_6_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0xB1], &["decode_fdcb", "RES y, (IY + d)", "C"])] // RES 6, (IY + d), C
#[case::fdcb_res_6_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0xB2], &["decode_fdcb", "RES y, (IY + d)", "D"])] // RES 6, (IY + d), D
#[case::fdcb_res_6_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0xB3], &["decode_fdcb", "RES y, (IY + d)", "E"])] // RES 6, (IY + d), E
#[case::fdcb_res_6_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0xB4], &["decode_fdcb", "RES y, (IY + d)", "H"])] // RES 6, (IY + d), H
#[case::fdcb_res_6_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0xB5], &["decode_fdcb", "RES y, (IY + d)", "L"])] // RES 6, (IY + d), L
#[case::fdcb_res_6_iyd(0x29f9, &[0xfd, 0xcb, 0x05, 0xB6], &["decode_fdcb", "RES y, (IY + d)"])] // RES 6, (IY + d)
#[case::fdcb_res_6_iyd_a(0x29f9, &[0xfd, 0xcb, 0x05, 0xB7], &["decode_fdcb", "RES y, (IY + d)", "A"])] // RES 6, (IY + d), A
#[case::fdcb_res_7_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0xB8], &["decode_fdcb", "RES y, (IY + d)", "B"])] // RES 7, (IY + d), B
#[case::fdcb_res_7_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0xB9], &["decode_fdcb", "RES y, (IY + d)", "C"])] // RES 7, (IY + d), C
#[case::fdcb_res_7_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0xBA], &["decode_fdcb", "RES y, (IY + d)", "D"])] // RES 7, (IY + d), D
#[case::fdcb_res_7_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0xBB], &["decode_fdcb", "RES y, (IY + d)", "E"])] // RES 7, (IY + d), E
#[case::fdcb_res_7_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0xBC], &["decode_fdcb", "RES y, (IY + d)", "H"])] // RES 7, (IY + d), H
#[case::fdcb_res_7_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0xBD], &["decode_fdcb", "RES y, (IY + d)", "L"])] // RES 7, (IY + d), L
#[case::fdcb_res_7_iyd(0x29f9, &[0xfd, 0xcb, 0x05, 0xBE], &["decode_fdcb", "RES y, (IY + d)"])] // RES 7, (IY + d)
#[case::fdcb_res_7_iyd_a(0x29f9, &[0xfd, 0xcb, 0x05, 0xBF], &["decode_fdcb", "RES y, (IY + d)", "A"])] // RES 7, (IY + d), A
#[case::fdcb_set_0_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0xC0], &["decode_fdcb", "SET y, (IY + d)", "B"])] // SET 0, (IY + d), B
#[case::fdcb_set_0_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0xC1], &["decode_fdcb", "SET y, (IY + d)", "C"])] // SET 0, (IY + d), C
#[case::fdcb_set_0_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0xC2], &["decode_fdcb", "SET y, (IY + d)", "D"])] // SET 0, (IY + d), D
#[case::fdcb_set_0_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0xC3], &["decode_fdcb", "SET y, (IY + d)", "E"])] // SET 0, (IY + d), E
#[case::fdcb_set_0_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0xC4], &["decode_fdcb", "SET y, (IY + d)", "H"])] // SET 0, (IY + d), H
#[case::fdcb_set_0_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0xC5], &["decode_fdcb", "SET y, (IY + d)", "L"])] // SET 0, (IY + d), L
#[case::fdcb_set_0_iyd(0x29f9, &[0xfd, 0xcb, 0x05, 0xC6], &["decode_fdcb", "SET y, (IY + d)"])] // SET 0, (IY + d)
#[case::fdcb_set_0_iyd_a(0x29f9, &[0xfd, 0xcb, 0x05, 0xC7], &["decode_fdcb", "SET y, (IY + d)", "A"])] // SET 0, (IY + d), A
#[case::fdcb_set_1_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0xC8], &["decode_fdcb", "SET y, (IY + d)", "B"])] // SET 1, (IY + d), B
#[case::fdcb_set_1_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0xC9], &["decode_fdcb", "SET y, (IY + d)", "C"])] // SET 1, (IY + d), C
#[case::fdcb_set_1_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0xCA], &["decode_fdcb", "SET y, (IY + d)", "D"])] // SET 1, (IY + d), D
#[case::fdcb_set_1_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0xCB], &["decode_fdcb", "SET y, (IY + d)", "E"])] // SET 1, (IY + d), E
#[case::fdcb_set_1_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0xCC], &["decode_fdcb", "SET y, (IY + d)", "H"])] // SET 1, (IY + d), H
#[case::fdcb_set_1_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0xCD], &["decode_fdcb", "SET y, (IY + d)", "L"])] // SET 1, (IY + d), L
#[case::fdcb_set_1_iyd(0x29f9, &[0xfd, 0xcb, 0x05, 0xCE], &["decode_fdcb", "SET y, (IY + d)"])] // SET 1, (IY + d)
#[case::fdcb_set_1_iyd_a(0x29f9, &[0xfd, 0xcb, 0x15, 0xCF], &["decode_fdcb", "SET y, (IY + d)", "A"])] // SET 1, (IY + d), A
#[case::fdcb_set_2_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0xD0], &["decode_fdcb", "SET y, (IY + d)", "B"])] // SET 2, (IY + d), B
#[case::fdcb_set_2_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0xD1], &["decode_fdcb", "SET y, (IY + d)", "C"])] // SET 2, (IY + d), C
#[case::fdcb_set_2_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0xD2], &["decode_fdcb", "SET y, (IY + d)", "D"])] // SET 2, (IY + d), D
#[case::fdcb_set_2_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0xD3], &["decode_fdcb", "SET y, (IY + d)", "E"])] // SET 2, (IY + d), E
#[case::fdcb_set_2_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0xD4], &["decode_fdcb", "SET y, (IY + d)", "H"])] // SET 2, (IY + d), H
#[case::fdcb_set_2_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0xD5], &["decode_fdcb", "SET y, (IY + d)", "L"])] // SET 2, (IY + d), L
#[case::fdcb_set_2_iyd(0x29f9, &[0xfd, 0xcb, 0x05, 0xD6], &["decode_fdcb", "SET y, (IY + d)"])] // SET 2, (IY + d)
#[case::fdcb_set_2_iyd_a(0x29f9, &[0xfd, 0xcb, 0x05, 0xD7], &["decode_fdcb", "SET y, (IY + d)", "A"])] // SET 2, (IY + d), A
#[case::fdcb_set_3_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0xD8], &["decode_fdcb", "SET y, (IY + d)", "B"])] // SET 3, (IY + d), B
#[case::fdcb_set_3_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0xD9], &["decode_fdcb", "SET y, (IY + d)", "C"])] // SET 3, (IY + d), C
#[case::fdcb_set_3_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0xDA], &["decode_fdcb", "SET y, (IY + d)", "D"])] // SET 3, (IY + d), D
#[case::fdcb_set_3_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0xDB], &["decode_fdcb", "SET y, (IY + d)", "E"])] // SET 3, (IY + d), E
#[case::fdcb_set_3_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0xDC], &["decode_fdcb", "SET y, (IY + d)", "H"])] // SET 3, (IY + d), H
#[case::fdcb_set_3_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0xDD], &["decode_fdcb", "SET y, (IY + d)", "L"])] // SET 3, (IY + d), L
#[case::fdcb_set_3_iyd(0x29f9, &[0xfd, 0xcb, 0x05, 0xDE], &["decode_fdcb", "SET y, (IY + d)"])] // SET 3, (IY + d)
#[case::fdcb_set_3_iyd_a(0x29f9, &[0xfd, 0xcb, 0x15, 0xDF], &["decode_fdcb", "SET y, (IY + d)", "A"])] // SET 3, (IY + d), A
#[case::fdcb_set_4_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0xE0], &["decode_fdcb", "SET y, (IY + d)", "B"])] // SET 4, (IY + d), B
#[case::fdcb_set_4_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0xE1], &["decode_fdcb", "SET y, (IY + d)", "C"])] // SET 4, (IY + d), C
#[case::fdcb_set_4_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0xE2], &["decode_fdcb", "SET y, (IY + d)", "D"])] // SET 4, (IY + d), D
#[case::fdcb_set_4_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0xE3], &["decode_fdcb", "SET y, (IY + d)", "E"])] // SET 4, (IY + d), E
#[case::fdcb_set_4_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0xE4], &["decode_fdcb", "SET y, (IY + d)", "H"])] // SET 4, (IY + d), H
#[case::fdcb_set_4_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0xE5], &["decode_fdcb", "SET y, (IY + d)", "L"])] // SET 4, (IY + d), L
#[case::fdcb_set_4_iyd(0x29f9, &[0xfd, 0xcb, 0x05, 0xE6], &["decode_fdcb", "SET y, (IY + d)"])] // SET 4, (IY + d)
#[case::fdcb_set_4_iyd_a(0x29f9, &[0xfd, 0xcb, 0x05, 0xE7], &["decode_fdcb", "SET y, (IY + d)", "A"])] // SET 4, (IY + d), A
#[case::fdcb_set_5_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0xE8], &["decode_fdcb", "SET y, (IY + d)", "B"])] // SET 5, (IY + d), B
#[case::fdcb_set_5_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0xE9], &["decode_fdcb", "SET y, (IY + d)", "C"])] // SET 5, (IY + d), C
#[case::fdcb_set_5_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0xEA], &["decode_fdcb", "SET y, (IY + d)", "D"])] // SET 5, (IY + d), D
#[case::fdcb_set_5_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0xEB], &["decode_fdcb", "SET y, (IY + d)", "E"])] // SET 5, (IY + d), E
#[case::fdcb_set_5_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0xEC], &["decode_fdcb", "SET y, (IY + d)", "H"])] // SET 5, (IY + d), H
#[case::fdcb_set_5_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0xED], &["decode_fdcb", "SET y, (IY + d)", "L"])] // SET 5, (IY + d), L
#[case::fdcb_set_5_iyd(0x29f9, &[0xfd, 0xcb, 0x05, 0xEE], &["decode_fdcb", "SET y, (IY + d)"])] // SET 5, (IY + d)
#[case::fdcb_set_5_iyd_a(0x29f9, &[0xfd, 0xcb, 0x05, 0xEF], &["decode_fdcb", "SET y, (IY + d)", "A"])] // SET 5, (IY + d), A
#[case::fdcb_set_6_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0xF0], &["decode_fdcb", "SET y, (IY + d)", "B"])] // SET 6, (IY + d), B
#[case::fdcb_set_6_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0xF1], &["decode_fdcb", "SET y, (IY + d)", "C"])] // SET 6, (IY + d), C
#[case::fdcb_set_6_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0xF2], &["decode_fdcb", "SET y, (IY + d)", "D"])] // SET 6, (IY + d), D
#[case::fdcb_set_6_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0xF3], &["decode_fdcb", "SET y, (IY + d)", "E"])] // SET 6, (IY + d), E
#[case::fdcb_set_6_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0xF4], &["decode_fdcb", "SET y, (IY + d)", "H"])] // SET 6, (IY + d), H
#[case::fdcb_set_6_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0xF5], &["decode_fdcb", "SET y, (IY + d)", "L"])] // SET 6, (IY + d), L
#[case::fdcb_set_6_iyd(0x29f9, &[0xfd, 0xcb, 0x05, 0xF6], &["decode_fdcb", "SET y, (IY + d)"])] // SET 6, (IY + d)
#[case::fdcb_set_6_iyd_a(0x29f9, &[0xfd, 0xcb, 0x05, 0xF7], &["decode_fdcb", "SET y, (IY + d)", "A"])] // SET 6, (IY + d), A
#[case::fdcb_set_7_iyd_b(0x29f9, &[0xfd, 0xcb, 0x05, 0xF8], &["decode_fdcb", "SET y, (IY + d)", "B"])] // SET 7, (IY + d), B
#[case::fdcb_set_7_iyd_c(0x29f9, &[0xfd, 0xcb, 0x05, 0xF9], &["decode_fdcb", "SET y, (IY + d)", "C"])] // SET 7, (IY + d), C
#[case::fdcb_set_7_iyd_d(0x29f9, &[0xfd, 0xcb, 0x05, 0xFA], &["decode_fdcb", "SET y, (IY + d)", "D"])] // SET 7, (IY + d), D
#[case::fdcb_set_7_iyd_e(0x29f9, &[0xfd, 0xcb, 0x05, 0xFB], &["decode_fdcb", "SET y, (IY + d)", "E"])] // SET 7, (IY + d), E
#[case::fdcb_set_7_iyd_h(0x29f9, &[0xfd, 0xcb, 0x05, 0xFC], &["decode_fdcb", "SET y, (IY + d)", "H"])] // SET 7, (IY + d), H
#[case::fdcb_set_7_iyd_l(0x29f9, &[0xfd, 0xcb, 0x05, 0xFD], &["decode_fdcb", "SET y, (IY + d)", "L"])] // SET 7, (IY + d), L
#[case::fdcb_set_7_iyd(0x29f9, &[0xfd, 0xcb, 0x05, 0xFE], &["decode_fdcb", "SET y, (IY + d)"])] // SET 7, (IY + d)
#[case::fdcb_set_7_iyd_a(0x29f9, &[0xfd, 0xcb, 0x05, 0xFF], &["decode_fdcb", "SET y, (IY + d)", "A"])] // SET 7, (IY + d), A
fn test_opcode(
    #[case] starting_pc: u16,
    #[case] memory_contents: &[u8],
    #[case] expected_logs: &[&str],
) {
    let mut cpu = setup_cpu();
    for (i, &byte) in memory_contents.iter().enumerate() {
        cpu.memory
            .borrow_mut()
            .write(starting_pc.wrapping_add(i as u16), byte);
    }
    cpu.pc = starting_pc;
    cpu.tick();

    assert_eq!(
        expected_logs.len(),
        cpu.test_callback.0.len(),
        "Log length mismatch, expected {} entries but got {} entries. Expected logs: {:?}, Actual logs: {:?}",
        expected_logs.len(),
        cpu.test_callback.0.len(),
        expected_logs,
        cpu.test_callback.0.iter().rev().collect::<Vec<_>>()
    );

    // zip logs and cpu logs

    for (expected, actual) in expected_logs.iter().zip(cpu.test_callback.0.iter().rev()) {
        assert_eq!(
            expected, actual,
            "Log mismatch. Expected log: {:?}, Actual log: {:?}",
            expected, actual
        );
    }
}

#[rstest]
#[case::ed_noni_x0(0x29f9, &[0xed], 0x00..=0x3f, &[], &["decode_ed", "NONI"])] // ED NONI where X == 0
#[case::ed_noni_x3(0x29f9, &[0xed], 0xc0..=0xff, &[], &["decode_ed", "NONI"])] // ED NONI where X == 3
#[case(0x29f9, &[0xed], 0x7C..=0x9F, &[], &["decode_ed", "NONI"])]
#[case(0x29f9, &[0xed], 0xA4..=0xA7, &[], &["decode_ed", "NONI"])]
#[case(0x29f9, &[0xed], 0xAC..=0xAF, &[], &["decode_ed", "NONI"])]
#[case(0x29f9, &[0xed], 0xB4..=0xB7, &[], &["decode_ed", "NONI"])]
#[case(0x29f9, &[0xed], 0xBC..=0xBF, &[], &["decode_ed", "NONI"])]
#[case::ddbc_bit_0_ixd(0x29f9, &[0xdd, 0xcb, 0x05], 0x40..=0x47, &[0x00], &["decode_ddcb", "BIT y, (IX + d)"])] // BIT 0, (IX + d)
#[case::ddbc_bit_1_ixd(0x29f9, &[0xdd, 0xcb, 0x05], 0x48..=0x4F, &[0x00], &["decode_ddcb", "BIT y, (IX + d)"])] // BIT 1, (IX + d)
#[case::ddbc_bit_2_ixd(0x29f9, &[0xdd, 0xcb, 0x05], 0x50..=0x57, &[0x00], &["decode_ddcb", "BIT y, (IX + d)"])] // BIT 2, (IX + d)
#[case::ddbc_bit_3_ixd(0x29f9, &[0xdd, 0xcb, 0x05], 0x58..=0x5F, &[0x00], &["decode_ddcb", "BIT y, (IX + d)"])] // BIT 3, (IX + d)
#[case::ddbc_bit_4_ixd(0x29f9, &[0xdd, 0xcb, 0x05], 0x60..=0x67, &[0x00], &["decode_ddcb", "BIT y, (IX + d)"])] // BIT 4, (IX + d)
#[case::ddbc_bit_5_ixd(0x29f9, &[0xdd, 0xcb, 0x05], 0x68..=0x6F, &[0x00], &["decode_ddcb", "BIT y, (IX + d)"])] // BIT 5, (IX + d)
#[case::ddbc_bit_6_ixd(0x29f9, &[0xdd, 0xcb, 0x05], 0x70..=0x77, &[0x00], &["decode_ddcb", "BIT y, (IX + d)"])] // BIT 6, (IX + d)
#[case::ddbc_bit_7_ixd(0x29f9, &[0xdd, 0xcb, 0x05], 0x78..=0x7F, &[0x00], &["decode_ddcb", "BIT y, (IX + d)"])] // BIT 7, (IX + d)
#[case::fdcb_bit_0_iyd(0x29f9, &[0xfd, 0xcb, 0x05], 0x40..=0x47, &[0x00], &["decode_fdcb", "BIT y, (IY + d)"])] // BIT 0, (IY + d)
#[case::fdcb_bit_1_iyd(0x29f9, &[0xfd, 0xcb, 0x05], 0x48..=0x4F, &[0x00], &["decode_fdcb", "BIT y, (IY + d)"])] // BIT 1, (IY + d)
#[case::fdcb_bit_2_iyd(0x29f9, &[0xfd, 0xcb, 0x05], 0x50..=0x57, &[0x00], &["decode_fdcb", "BIT y, (IY + d)"])] // BIT 2, (IY + d)
#[case::fdcb_bit_3_iyd(0x29f9, &[0xfd, 0xcb, 0x05], 0x58..=0x5F, &[0x00], &["decode_fdcb", "BIT y, (IY + d)"])] // BIT 3, (IY + d)
#[case::fdcb_bit_4_iyd(0x29f9, &[0xfd, 0xcb, 0x05], 0x60..=0x67, &[0x00], &["decode_fdcb", "BIT y, (IY + d)"])] // BIT 4, (IY + d)
#[case::fdcb_bit_5_iyd(0x29f9, &[0xfd, 0xcb, 0x05], 0x68..=0x6F, &[0x00], &["decode_fdcb", "BIT y, (IY + d)"])] // BIT 5, (IY + d)
#[case::fdcb_bit_6_iyd(0x29f9, &[0xfd, 0xcb, 0x05], 0x70..=0x77, &[0x00], &["decode_fdcb", "BIT y, (IY + d)"])] // BIT 6, (IY + d)
#[case::fdcb_bit_7_iyd(0x29f9, &[0xfd, 0xcb, 0x05], 0x78..=0x7F, &[0x00], &["decode_fdcb", "BIT y, (IY + d)"])] // BIT 7, (IY + d)

fn test_opcode_range(
    #[case] starting_pc: u16,
    #[case] prefixes: &[u8],
    #[case] opcode_range: std::ops::RangeInclusive<u8>,
    #[case] suffixes: &[u8],
    #[case] expected_logs: &[&str],
) {
    for opcode in opcode_range {
        test_opcode(
            starting_pc,
            [prefixes, &[opcode], suffixes].concat().as_slice(),
            expected_logs,
        );
    }
}
