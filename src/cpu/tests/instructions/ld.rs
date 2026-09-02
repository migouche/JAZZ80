use crate::{
    cpu::{
        AddressingMode, GPR, IndexRegister, IndexRegisterPart, RegisterPair, SystemRegister,
        tests::setup_cpu,
    },
    traits::SynchronousComponent,
};
use rstest::rstest;

#[rstest]
// MAIN TABLE
#[case::ld_b_b(GPR::B, GPR::B, 0x42)] // ld B, B
#[case::ld_b_c(GPR::B, GPR::C, 0x34)] // ld B, C
#[case::ld_b_d(GPR::B, GPR::D, 0x56)] // ld B, D
#[case::ld_b_e(GPR::B, GPR::E, 0x78)] // ld B, E
#[case::ld_b_h(GPR::B, GPR::H, 0x9A)] // ld B, H
#[case::ld_b_l(GPR::B, GPR::L, 0xBC)] // ld B, L
#[case::ld_b_a(GPR::B, GPR::A, 0xFF)] // ld B, A
#[case::ld_c_b(GPR::C, GPR::B, 0x42)] // ld C, B
#[case::ld_c_c(GPR::C, GPR::C, 0x34)] // ld C, C
#[case::ld_c_d(GPR::C, GPR::D, 0x56)] // ld C, D
#[case::ld_c_e(GPR::C, GPR::E, 0x78)] // ld C, E
#[case::ld_c_h(GPR::C, GPR::H, 0x9A)] // ld C, H
#[case::ld_c_l(GPR::C, GPR::L, 0xBC)] // ld C, L
#[case::ld_c_a(GPR::C, GPR::A, 0xFF)] // ld C, A
#[case::ld_d_b(GPR::D, GPR::B, 0x42)] // ld D, B
#[case::ld_d_c(GPR::D, GPR::C, 0x34)] // ld D, C
#[case::ld_d_d(GPR::D, GPR::D, 0x56)] // ld D, D
#[case::ld_d_e(GPR::D, GPR::E, 0x78)] // ld D, E
#[case::ld_d_h(GPR::D, GPR::H, 0x9A)] // ld D, H
#[case::ld_d_l(GPR::D, GPR::L, 0xBC)] // ld D, L
#[case::ld_d_a(GPR::D, GPR::A, 0xFF)] // ld D, A
#[case::ld_e_b(GPR::E, GPR::B, 0x42)] // ld E, B
#[case::ld_e_c(GPR::E, GPR::C, 0x34)] // ld E, C
#[case::ld_e_d(GPR::E, GPR::D, 0x56)] // ld E, D
#[case::ld_e_e(GPR::E, GPR::E, 0x78)] // ld E, E
#[case::ld_e_h(GPR::E, GPR::H, 0x9A)] // ld E, H
#[case::ld_e_l(GPR::E, GPR::L, 0xBC)] // ld E, L
#[case::ld_e_a(GPR::E, GPR::A, 0xFF)] // ld E, A
#[case::ld_h_b(GPR::H, GPR::B, 0x42)] // ld H, B
#[case::ld_h_c(GPR::H, GPR::C, 0x34)] // ld H, C
#[case::ld_h_d(GPR::H, GPR::D, 0x56)] // ld H, D
#[case::ld_h_e(GPR::H, GPR::E, 0x78)] // ld H, E
#[case::ld_h_h(GPR::H, GPR::H, 0x9A)] // ld H, H
#[case::ld_h_l(GPR::H, GPR::L, 0xBC)] // ld H, L
#[case::ld_h_a(GPR::H, GPR::A, 0xFF)] // ld H, A
#[case::ld_l_b(GPR::L, GPR::B, 0x42)] // ld L, B
#[case::ld_l_c(GPR::L, GPR::C, 0x34)] // ld L, C
#[case::ld_l_d(GPR::L, GPR::D, 0x56)] // ld L, D
#[case::ld_l_e(GPR::L, GPR::E, 0x78)] // ld L, E
#[case::ld_l_h(GPR::L, GPR::H, 0x9A)] // ld L, H
#[case::ld_l_l(GPR::L, GPR::L, 0xBC)] // ld L, L
#[case::ld_l_a(GPR::L, GPR::A, 0xFF)] // ld L, A
#[case::ld_a_b(GPR::A, GPR::B, 0x42)] // ld A, B
#[case::ld_a_c(GPR::A, GPR::C, 0x34)] // ld A, C
#[case::ld_a_d(GPR::A, GPR::D, 0x56)] // ld A, D
#[case::ld_a_e(GPR::A, GPR::E, 0x78)] // ld A, E
#[case::ld_a_h(GPR::A, GPR::H, 0x9A)] // ld A, H
#[case::ld_a_l(GPR::A, GPR::L, 0xBC)] // ld A, L
#[case::ld_a_a(GPR::A, GPR::A, 0xFF)] // ld A, A
fn test_register_register(#[case] src: GPR, #[case] dest: GPR, #[case] value: u8) {
    let mut cpu = setup_cpu();
    cpu.set_register(src, value);
    cpu.ld(
        AddressingMode::Register(dest),
        AddressingMode::Register(src),
    );
    assert_eq!(cpu.get_register(dest), value);
}

#[rstest]
// MAIN TABLE
#[case::ld_b_n(GPR::B, 0x34)] // ld B, n
#[case::ld_d_n(GPR::D, 0x7F)] // ld D, n
#[case::ld_h_n(GPR::H, 0x56)] // ld H, n
#[case::ld_c_n(GPR::C, 0x19)] // ld C, n
#[case::ld_e_n(GPR::E, 0x88)] // ld E, n
#[case::ld_l_n(GPR::L, 0xAB)] // ld L, n
#[case::ld_a_n(GPR::A, 0xFF)] // ld A, n
fn test_register_immediate(#[case] dest: GPR, #[case] value: u8) {
    let mut cpu = setup_cpu();
    cpu.ld(
        AddressingMode::Register(dest),
        AddressingMode::Immediate(value),
    );
    assert_eq!(cpu.get_register(dest), value);
}

#[rstest]
// MAIN TABLE
#[case::ld_bc_nn(RegisterPair::BC, 0x12, 0x34)] // ld BC, nn
#[case::ld_de_nn(RegisterPair::DE, 0x56, 0x78)] // ld DE, nn
#[case::ld_hl_nn(RegisterPair::HL, 0x9A, 0xBC)] // ld HL, nn
#[case::ld_sp_nn(RegisterPair::SP, 0xDE, 0xF0)] // ld SP, nn
fn test_register_pair_immediate(#[case] pair: RegisterPair, #[case] low: u8, #[case] high: u8) {
    let mut cpu = setup_cpu();
    let val = ((high as u16) << 8) | (low as u16);
    cpu.ld_16(
        AddressingMode::RegisterPair(pair),
        AddressingMode::ImmediateExtended(val),
    );
    assert_eq!(cpu.get_register_pair(pair), val);
}

#[rstest]
// MAIN TABLE
#[case::ld_bc_nn(RegisterPair::BC, GPR::A, 0x5619, 0x42)] // ld (BC), A
#[case::ld_de_nn(RegisterPair::DE, GPR::A, 0x1234, 0x7F)] // ld (DE), A
#[case::ld_hl_a(RegisterPair::HL, GPR::A, 0x9ABC, 0x56)] // ld (HL), A
#[case::ld_hl_b(RegisterPair::HL, GPR::B, 0xDEF0, 0x78)] // ld (HL), B
#[case::ld_hl_c(RegisterPair::HL, GPR::C, 0x1234, 0x7F)] // ld (HL), C
#[case::ld_hl_d(RegisterPair::HL, GPR::D, 0x5678, 0x9A)] // ld (HL), D
#[case::ld_hl_e(RegisterPair::HL, GPR::E, 0x9ABC, 0xBC)] // ld (HL), E
#[case::ld_hl_h(RegisterPair::HL, GPR::H, 0xDEF0, 0xDE)] // ld (HL), H
#[case::ld_hl_l(RegisterPair::HL, GPR::L, 0x3456, 0x56)] // ld (HL), L //NOTE, for this one L must match the L in HL (otherwise it's impossible scenario)
fn test_register_pair_indirect_register(
    #[case] pair: RegisterPair,
    #[case] src: GPR,
    #[case] addr: u16,
    #[case] value: u8,
) {
    let mut cpu = setup_cpu();
    cpu.set_register(src, value);
    cpu.set_register_pair(pair, addr);
    cpu.ld(
        AddressingMode::RegisterIndirect(pair),
        AddressingMode::Register(src),
    );
    assert_eq!(cpu.memory.borrow().read(addr), value);
}

#[rstest]
// MAIN TABLE
#[case::ld_a_bc(GPR::A, RegisterPair::BC, 0x5619, 0x42)] // ld A, (BC)
#[case::ld_a_de(GPR::A, RegisterPair::DE, 0x1234, 0x7F)] // ld A, (DE)
#[case::ld_b_hl(GPR::B, RegisterPair::HL, 0x9ABC, 0x56)] // ld B, (HL)
#[case::ld_c_hl(GPR::C, RegisterPair::HL, 0xDEF0, 0x78)] // ld C, (HL)
#[case::ld_d_hl(GPR::D, RegisterPair::HL, 0x1234, 0x7F)] // ld D, (HL)
#[case::ld_e_hl(GPR::E, RegisterPair::HL, 0x5678, 0x9A)] // ld E, (HL)
#[case::ld_h_hl(GPR::H, RegisterPair::HL, 0x9ABC, 0xBC)] // ld H, (HL)
#[case::ld_l_hl(GPR::L, RegisterPair::HL, 0xDEF0, 0xDE)] // ld L, (HL)
#[case::ld_a_hl(GPR::A, RegisterPair::HL, 0x3456, 0xEF)] // ld A, (HL)
fn test_register_register_pair_indirect(
    #[case] src: GPR,
    #[case] pair: RegisterPair,
    #[case] addr: u16,
    #[case] value: u8,
) {
    let mut cpu = setup_cpu();
    cpu.memory.borrow_mut().write(addr, value);
    cpu.set_register_pair(pair, addr);
    cpu.ld(
        AddressingMode::Register(src),
        AddressingMode::RegisterIndirect(pair),
    );
    assert_eq!(cpu.get_register(src), value);
}

#[rstest]
// MAIN TABLE
#[case::ld_bc_n(RegisterPair::HL, 0x5619, 0x42)] // ld (HL), n
fn test_register_pair_indirect_immediate(
    #[case] pair: RegisterPair,
    #[case] addr: u16,
    #[case] value: u8,
) {
    let mut cpu = setup_cpu();
    cpu.set_register_pair(pair, addr);
    cpu.ld(
        AddressingMode::RegisterIndirect(pair),
        AddressingMode::Immediate(value),
    );
    assert_eq!(cpu.memory.borrow().read(addr), value);
}

#[rstest]
// MAIN TABLE
#[case::ld_a_nn(GPR::A, 0x1234, 0x7F)] // ld A, (nn)
fn test_register_absolute(#[case] dest: GPR, #[case] addr: u16, #[case] value: u8) {
    let mut cpu = setup_cpu();
    cpu.memory.borrow_mut().write(addr, value);
    cpu.ld(
        AddressingMode::Register(dest),
        AddressingMode::Absolute(addr),
    );
    assert_eq!(cpu.get_register(dest), value);
}

#[rstest]
// MAIN TABLE
#[case::ld_hl_nn(RegisterPair::HL, 0x9ABC, 0x56)] // ld HL, (nn)

// ED TABLE
#[case::ld_bc_nn(RegisterPair::BC, 0x5619, 0x42)] // ld BC, (nn)
#[case::ld_de_nn(RegisterPair::DE, 0x1234, 0x7F)] // ld DE, (nn)
#[case::ld_sp_nn(RegisterPair::SP, 0xDEF0, 0x78)] // ld SP, (nn)
fn test_register_pair_absolute(#[case] pair: RegisterPair, #[case] addr: u16, #[case] value: u8) {
    let mut cpu = setup_cpu();
    cpu.memory.borrow_mut().write(addr, value);
    cpu.ld_16(
        AddressingMode::RegisterPair(pair),
        AddressingMode::Absolute(addr),
    );
    assert_eq!(cpu.get_register_pair(pair), value as u16);
}

#[rstest]
// MAIN TABLE
#[case::ld_nn_a(0x1234, 0x7F)] // ld (nn), A
fn test_absolute_register(#[case] addr: u16, #[case] value: u8) {
    let mut cpu = setup_cpu();
    cpu.set_register(GPR::A, value);
    cpu.ld(
        AddressingMode::Absolute(addr),
        AddressingMode::Register(GPR::A),
    );
    assert_eq!(cpu.memory.borrow().read(addr), value);
}

#[rstest]
// ED TABLE
#[case::ld_nn_bc(0x5619, 0x42)] // ld (nn), BC
#[case::ld_nn_de(0x1234, 0x7F)] // ld (nn), DE
#[case::ld_nn_hl(0x9ABC, 0x56)] // ld (nn), HL
#[case::ld_nn_sp(0xDEF0, 0x78)] // ld (nn), SP
// DD TABLE
#[case::ld_nn_ix(0x3456, 0x9A)] // ld (nn), IX
fn test_absolute_register_pair(#[case] addr: u16, #[case] value: u16) {
    let mut cpu = setup_cpu();
    cpu.set_register_pair(RegisterPair::BC, value);
    cpu.ld_16(
        AddressingMode::Absolute(addr),
        AddressingMode::RegisterPair(RegisterPair::BC),
    );
    assert_eq!(cpu.memory.borrow().read_word(addr), value);
}

#[rstest]
// ED TABLE
#[case::ld_i_a(
    AddressingMode::System(SystemRegister::I),
    AddressingMode::Register(GPR::A)
)] // ld I, A
#[case::ld_a_i(
    AddressingMode::Register(GPR::A),
    AddressingMode::System(SystemRegister::I)
)] // ld A, I
#[case::ld_r_a(
    AddressingMode::System(SystemRegister::R),
    AddressingMode::Register(GPR::A)
)] // ld R, A
#[case::ld_a_r(
    AddressingMode::Register(GPR::A),
    AddressingMode::System(SystemRegister::R)
)] // ld A, R
fn test_ld_system_gpr(#[case] dest: AddressingMode, #[case] src: AddressingMode) {
    let mut cpu = setup_cpu();
    // Set src value
    match src {
        AddressingMode::Register(r) => cpu.set_register(r, 0x42),
        AddressingMode::System(r) => cpu.set_system_register(r, 0x42),
        _ => panic!("Invalid src"),
    }

    cpu.ld(dest, src);

    // Check dest value
    match dest {
        AddressingMode::Register(r) => assert_eq!(cpu.get_register(r), 0x42),
        AddressingMode::System(r) => assert_eq!(cpu.get_system_register(r), 0x42),
        _ => panic!("Invalid dest"),
    }
}

#[rstest]
// DD TABLE
#[case::ld_ixh_n(IndexRegisterPart::IXH, 0x34)] // ld IXH, n
#[case::ld_ixl_n(IndexRegisterPart::IXL, 0x12)] // ld IXL, n
fn test_index_part_immediate(#[case] dest: IndexRegisterPart, #[case] value: u8) {
    let mut cpu = setup_cpu();
    cpu.ld(
        AddressingMode::IndexRegisterPart(dest),
        AddressingMode::Immediate(value),
    );
    assert_eq!(cpu.get_index_register_part(dest), value);
}

#[rstest]
// DD TABLE
#[case::ld_ix_nn(IndexRegister::IX, 0x3412)] // ld IX, (nn)
fn test_index_register_immediate_extended(#[case] dest: IndexRegister, #[case] value: u16) {
    let mut cpu = setup_cpu();
    cpu.ld_16(
        AddressingMode::IndexRegister(dest),
        AddressingMode::ImmediateExtended(value),
    );
    assert_eq!(cpu.get_index_register(dest), value);
}

#[rstest]
// DD TABLE
#[case::ld_ixd_n(IndexRegister::IX, 0x1234, 0x42)] // ld (IX+d), n
fn test_indexed_immediate(#[case] base: IndexRegister, #[case] addr: u16, #[case] value: u8) {
    let mut cpu = setup_cpu();
    cpu.set_index_register(base, addr);
    cpu.ld(
        AddressingMode::Indexed(base, 0),
        AddressingMode::Immediate(value),
    );
    assert_eq!(cpu.memory.borrow().read(addr), value);
}

#[rstest]
// DD TABLE
#[case::ld_b_ixh(GPR::B, IndexRegisterPart::IXH, 0x12)] // ld B, IXH
#[case::ld_b_ixl(GPR::B, IndexRegisterPart::IXL, 0x34)] // ld B, IXL
#[case::ld_c_ixh(GPR::C, IndexRegisterPart::IXH, 0x12)] // ld C, IXH
#[case::ld_c_ixl(GPR::C, IndexRegisterPart::IXL, 0x34)] // ld C, IXL

fn test_register_index_part(#[case] dest: GPR, #[case] src: IndexRegisterPart, #[case] value: u8) {
    let mut cpu = setup_cpu();
    cpu.set_index_register_part(src, value);
    cpu.ld(
        AddressingMode::Register(dest),
        AddressingMode::IndexRegisterPart(src),
    );
    assert_eq!(cpu.get_register(dest), value);
}

#[rstest]
// DD TABLE
#[case::ld_ixh_b(IndexRegisterPart::IXH, GPR::B, 0x34)] // ld IXH, B
fn test_index_part_register(#[case] dest: IndexRegisterPart, #[case] src: GPR, #[case] value: u8) {
    let mut cpu = setup_cpu();
    cpu.set_register(src, value);
    cpu.ld(
        AddressingMode::IndexRegisterPart(dest),
        AddressingMode::Register(src),
    );
    assert_eq!(cpu.get_index_register_part(dest), value);
}

#[rstest]
// DD TABLE
#[case::ld_b_ixd(GPR::B, IndexRegister::IX, 0x1234, 0x42, 0x01)] // ld B, (IX+d)
#[case::ld_c_ixd(GPR::C, IndexRegister::IX, 0x1234, 0x56, 0xFE)] // ld C, (IX+d)
#[case::ld_d_ixd(GPR::D, IndexRegister::IX, 0x1234, 0x78, 0x10)] // ld D, (IX+d)
#[case::ld_e_ixd(GPR::E, IndexRegister::IX, 0x1234, 0x4A, 0xF0)] // ld E, (IX+d)
#[case::ld_h_ixd(GPR::H, IndexRegister::IX, 0x1234, 0x4C, 0x7F)] // ld H, (IX+d)
#[case::ld_l_ixd(GPR::L, IndexRegister::IX, 0x1234, 0x3E, 0x80)] // ld L, (IX+d)
#[case::ld_a_ixd(GPR::A, IndexRegister::IX, 0x1234, 0x4F, 0x00)] // ld A, (IX+d)
fn test_register_indexed(
    #[case] dest: GPR,
    #[case] base: IndexRegister,
    #[case] addr: u16,
    #[case] displacement: i8,
    #[case] value: u8,
) {
    let mut cpu = setup_cpu();
    cpu.memory
        .borrow_mut()
        .write(addr.wrapping_add_signed(displacement as i16), value);
    cpu.set_index_register(base, addr);
    cpu.ld(
        AddressingMode::Register(dest),
        AddressingMode::Indexed(base, displacement),
    );
    assert_eq!(cpu.get_register(dest), value);
}

// testing with hard-coded bytes
#[rstest]
// LD B, r
#[case::ld_b_b(0, [(GPR::B, 0x42), (GPR::C, 0)], &[0x40], [(GPR::B, 0x42), (GPR::C, 0)])]
#[case::ld_b_c(0, [(GPR::B, 0), (GPR::C, 0x42)], &[0x41], [(GPR::B, 0x42), (GPR::C, 0x42)])]
#[case::ld_b_d(0, [(GPR::B, 0), (GPR::D, 0x42)], &[0x42], [(GPR::B, 0x42), (GPR::D, 0x42)])]
#[case::ld_b_e(0, [(GPR::B, 0), (GPR::E, 0x42)], &[0x43], [(GPR::B, 0x42), (GPR::E, 0x42)])]
#[case::ld_b_h(0, [(GPR::B, 0), (GPR::H, 0x42)], &[0x44], [(GPR::B, 0x42), (GPR::H, 0x42)])]
#[case::ld_b_l(0, [(GPR::B, 0), (GPR::L, 0x42)], &[0x45], [(GPR::B, 0x42), (GPR::L, 0x42)])]
#[case::ld_b_a(0, [(GPR::B, 0), (GPR::A, 0x42)], &[0x47], [(GPR::B, 0x42), (GPR::A, 0x42)])]
// LD C, r
#[case::ld_c_b(0, [(GPR::C, 0), (GPR::B, 0x42)], &[0x48], [(GPR::C, 0x42), (GPR::B, 0x42)])]
#[case::ld_c_c(0, [(GPR::C, 0x42), (GPR::B, 0)], &[0x49], [(GPR::C, 0x42), (GPR::B, 0)])]
#[case::ld_c_d(0, [(GPR::C, 0), (GPR::D, 0x42)], &[0x4A], [(GPR::C, 0x42), (GPR::D, 0x42)])]
#[case::ld_c_e(0, [(GPR::C, 0), (GPR::E, 0x42)], &[0x4B], [(GPR::C, 0x42), (GPR::E, 0x42)])]
#[case::ld_c_h(0, [(GPR::C, 0), (GPR::H, 0x42)], &[0x4C], [(GPR::C, 0x42), (GPR::H, 0x42)])]
#[case::ld_c_l(0, [(GPR::C, 0), (GPR::L, 0x42)], &[0x4D], [(GPR::C, 0x42), (GPR::L, 0x42)])]
#[case::ld_c_a(0, [(GPR::C, 0), (GPR::A, 0x42)], &[0x4F], [(GPR::C, 0x42), (GPR::A, 0x42)])]
// LD D, r
#[case::ld_d_b(0, [(GPR::D, 0), (GPR::B, 0x42)], &[0x50], [(GPR::D, 0x42), (GPR::B, 0x42)])]
#[case::ld_d_c(0, [(GPR::D, 0), (GPR::C, 0x42)], &[0x51], [(GPR::D, 0x42), (GPR::C, 0x42)])]
#[case::ld_d_d(0, [(GPR::D, 0x42), (GPR::B, 0)], &[0x52], [(GPR::D, 0x42), (GPR::B, 0)])]
#[case::ld_d_e(0, [(GPR::D, 0), (GPR::E, 0x42)], &[0x53], [(GPR::D, 0x42), (GPR::E, 0x42)])]
#[case::ld_d_h(0, [(GPR::D, 0), (GPR::H, 0x42)], &[0x54], [(GPR::D, 0x42), (GPR::H, 0x42)])]
#[case::ld_d_l(0, [(GPR::D, 0), (GPR::L, 0x42)], &[0x55], [(GPR::D, 0x42), (GPR::L, 0x42)])]
#[case::ld_d_a(0, [(GPR::D, 0), (GPR::A, 0x42)], &[0x57], [(GPR::D, 0x42), (GPR::A, 0x42)])]
// LD E, r
#[case::ld_e_b(0, [(GPR::E, 0), (GPR::B, 0x42)], &[0x58], [(GPR::E, 0x42), (GPR::B, 0x42)])]
#[case::ld_e_c(0, [(GPR::E, 0), (GPR::C, 0x42)], &[0x59], [(GPR::E, 0x42), (GPR::C, 0x42)])]
#[case::ld_e_d(0, [(GPR::E, 0), (GPR::D, 0x42)], &[0x5A], [(GPR::E, 0x42), (GPR::D, 0x42)])]
#[case::ld_e_e(0, [(GPR::E, 0x42), (GPR::B, 0)], &[0x5B], [(GPR::E, 0x42), (GPR::B, 0)])]
#[case::ld_e_h(0, [(GPR::E, 0), (GPR::H, 0x42)], &[0x5C], [(GPR::E, 0x42), (GPR::H, 0x42)])]
#[case::ld_e_l(0, [(GPR::E, 0), (GPR::L, 0x42)], &[0x5D], [(GPR::E, 0x42), (GPR::L, 0x42)])]
#[case::ld_e_a(0, [(GPR::E, 0), (GPR::A, 0x42)], &[0x5F], [(GPR::E, 0x42), (GPR::A, 0x42)])]
// LD H, r
#[case::ld_h_b(0, [(GPR::H, 0), (GPR::B, 0x42)], &[0x60], [(GPR::H, 0x42), (GPR::B, 0x42)])]
#[case::ld_h_c(0, [(GPR::H, 0), (GPR::C, 0x42)], &[0x61], [(GPR::H, 0x42), (GPR::C, 0x42)])]
#[case::ld_h_d(0, [(GPR::H, 0), (GPR::D, 0x42)], &[0x62], [(GPR::H, 0x42), (GPR::D, 0x42)])]
#[case::ld_h_e(0, [(GPR::H, 0), (GPR::E, 0x42)], &[0x63], [(GPR::H, 0x42), (GPR::E, 0x42)])]
#[case::ld_h_h(0, [(GPR::H, 0x42), (GPR::B, 0)], &[0x64], [(GPR::H, 0x42), (GPR::B, 0)])]
#[case::ld_h_l(0, [(GPR::H, 0), (GPR::L, 0x42)], &[0x65], [(GPR::H, 0x42), (GPR::L, 0x42)])]
#[case::ld_h_a(0, [(GPR::H, 0), (GPR::A, 0x42)], &[0x67], [(GPR::H, 0x42), (GPR::A, 0x42)])]
// LD L, r
#[case::ld_l_b(0, [(GPR::L, 0), (GPR::B, 0x42)], &[0x68], [(GPR::L, 0x42), (GPR::B, 0x42)])]
#[case::ld_l_c(0, [(GPR::L, 0), (GPR::C, 0x42)], &[0x69], [(GPR::L, 0x42), (GPR::C, 0x42)])]
#[case::ld_l_d(0, [(GPR::L, 0), (GPR::D, 0x42)], &[0x6A], [(GPR::L, 0x42), (GPR::D, 0x42)])]
#[case::ld_l_e(0, [(GPR::L, 0), (GPR::E, 0x42)], &[0x6B], [(GPR::L, 0x42), (GPR::E, 0x42)])]
#[case::ld_l_h(0, [(GPR::L, 0), (GPR::H, 0x42)], &[0x6C], [(GPR::L, 0x42), (GPR::H, 0x42)])]
#[case::ld_l_l(0, [(GPR::L, 0x42), (GPR::B, 0)], &[0x6D], [(GPR::L, 0x42), (GPR::B, 0)])]
#[case::ld_l_a(0, [(GPR::L, 0), (GPR::A, 0x42)], &[0x6F], [(GPR::L, 0x42), (GPR::A, 0x42)])]
// LD A, r
#[case::ld_a_b(0, [(GPR::A, 0), (GPR::B, 0x42)], &[0x78], [(GPR::A, 0x42), (GPR::B, 0x42)])]
#[case::ld_a_c(0, [(GPR::A, 0), (GPR::C, 0x42)], &[0x79], [(GPR::A, 0x42), (GPR::C, 0x42)])]
#[case::ld_a_d(0, [(GPR::A, 0), (GPR::D, 0x42)], &[0x7A], [(GPR::A, 0x42), (GPR::D, 0x42)])]
#[case::ld_a_e(0, [(GPR::A, 0), (GPR::E, 0x42)], &[0x7B], [(GPR::A, 0x42), (GPR::E, 0x42)])]
#[case::ld_a_h(0, [(GPR::A, 0), (GPR::H, 0x42)], &[0x7C], [(GPR::A, 0x42), (GPR::H, 0x42)])]
#[case::ld_a_l(0, [(GPR::A, 0), (GPR::L, 0x42)], &[0x7D], [(GPR::A, 0x42), (GPR::L, 0x42)])]
#[case::ld_a_a(0, [(GPR::A, 0x42), (GPR::B, 0)], &[0x7F], [(GPR::A, 0x42), (GPR::B, 0)])]
fn test_ld_gpr(
    #[case] initial_pc: u16,
    #[case] initial_regs: [(GPR, u8); 2],
    #[case] memory_bytes: &[u8],
    #[case] expected_regs: [(GPR, u8); 2],
) {
    let mut cpu = setup_cpu();
    cpu.pc = initial_pc;
    for (reg, val) in initial_regs.iter() {
        cpu.set_register(*reg, *val);
    }
    for (i, byte) in memory_bytes.iter().enumerate() {
        cpu.memory
            .borrow_mut()
            .write(initial_pc.wrapping_add(i as u16), *byte);
    }

    cpu.tick();

    // Check expected register values
    for (reg, expected_val) in expected_regs.iter() {
        assert_eq!(
            cpu.get_register(*reg),
            *expected_val,
            "Register {:?} mismatch, expected {:02X}, got {:02X}",
            reg,
            expected_val,
            cpu.get_register(*reg)
        );
    }
}

#[rstest]
#[case::ld_b_n(0, GPR::B, 0x06, 0x42)]
#[case::ld_c_n(0, GPR::C, 0x0E, 0x42)]
#[case::ld_d_n(0, GPR::D, 0x16, 0x42)]
#[case::ld_e_n(0, GPR::E, 0x1E, 0x42)]
#[case::ld_h_n(0, GPR::H, 0x26, 0x42)]
#[case::ld_l_n(0, GPR::L, 0x2E, 0x42)]
#[case::ld_a_n(0, GPR::A, 0x3E, 0x42)]
fn test_ld_immediate(
    #[case] initial_pc: u16,
    #[case] dest_reg: GPR,
    #[case] opcode: u8,
    #[case] value: u8,
) {
    let mut cpu = setup_cpu();
    cpu.pc = initial_pc;
    cpu.memory.borrow_mut().write(initial_pc, opcode);
    cpu.memory.borrow_mut().write(initial_pc + 1, value);

    cpu.tick();

    assert_eq!(cpu.get_register(dest_reg), value);
}

#[rstest]
#[case::ld_b_hl(0, GPR::B, 0x46, 0x42, 0x1000)]
#[case::ld_c_hl(0, GPR::C, 0x4E, 0x42, 0x1000)]
#[case::ld_d_hl(0, GPR::D, 0x56, 0x42, 0x1000)]
#[case::ld_e_hl(0, GPR::E, 0x5E, 0x42, 0x1000)]
#[case::ld_h_hl(0, GPR::H, 0x66, 0x42, 0x1000)]
#[case::ld_l_hl(0, GPR::L, 0x6E, 0x42, 0x1000)]
#[case::ld_a_hl(0, GPR::A, 0x7E, 0x42, 0x1000)]
fn test_ld_indirect_hl(
    #[case] initial_pc: u16,
    #[case] dest_reg: GPR,
    #[case] opcode: u8,
    #[case] value: u8,
    #[case] addr: u16,
) {
    let mut cpu = setup_cpu();
    cpu.pc = initial_pc;
    cpu.memory.borrow_mut().write(initial_pc, opcode);
    cpu.set_register_pair(RegisterPair::HL, addr);
    cpu.memory.borrow_mut().write(addr, value);

    cpu.tick();

    assert_eq!(cpu.get_register(dest_reg), value);
}

#[rstest]
#[case::ld_hl_b(0, GPR::B, 0x70, 0x42, 0x1000)]
#[case::ld_hl_c(0, GPR::C, 0x71, 0x42, 0x1000)]
#[case::ld_hl_d(0, GPR::D, 0x72, 0x42, 0x1000)]
#[case::ld_hl_e(0, GPR::E, 0x73, 0x42, 0x1000)]
#[case::ld_hl_h(0, GPR::H, 0x74, 0x10, 0x1000)]
#[case::ld_hl_l(0, GPR::L, 0x75, 0x00, 0x1000)]
#[case::ld_hl_a(0, GPR::A, 0x77, 0x42, 0x1000)]
fn test_ld_store_hl(
    #[case] initial_pc: u16,
    #[case] src_reg: GPR,
    #[case] opcode: u8,
    #[case] value: u8,
    #[case] addr: u16,
) {
    let mut cpu = setup_cpu();
    cpu.pc = initial_pc;
    cpu.memory.borrow_mut().write(initial_pc, opcode);
    cpu.set_register(src_reg, value);
    cpu.set_register_pair(RegisterPair::HL, addr);

    cpu.tick();

    assert_eq!(cpu.memory.borrow().read(addr), value);
}

#[rstest]
#[case::ld_hl_n(0, 0x36, 0x42, 0x1000)]
fn test_ld_store_immediate_hl(
    #[case] initial_pc: u16,
    #[case] opcode: u8,
    #[case] value: u8,
    #[case] addr: u16,
) {
    let mut cpu = setup_cpu();
    cpu.pc = initial_pc;
    cpu.memory.borrow_mut().write(initial_pc, opcode);
    cpu.memory.borrow_mut().write(initial_pc + 1, value);
    cpu.set_register_pair(RegisterPair::HL, addr);

    cpu.tick();

    assert_eq!(cpu.memory.borrow().read(addr), value);
}

#[rstest]
#[case::ld_ix_nn(0, IndexRegister::IX, 0xDD, 0x21, 0x1234)]
#[case::ld_iy_nn(0, IndexRegister::IY, 0xFD, 0x21, 0x1234)]
fn test_ld_index_immediate_extended(
    #[case] initial_pc: u16,
    #[case] reg: IndexRegister,
    #[case] prefix: u8,
    #[case] opcode: u8,
    #[case] value: u16,
) {
    let mut cpu = setup_cpu();
    cpu.pc = initial_pc;
    cpu.memory.borrow_mut().write(initial_pc, prefix);
    cpu.memory.borrow_mut().write(initial_pc + 1, opcode);
    cpu.memory
        .borrow_mut()
        .write(initial_pc + 2, (value & 0xFF) as u8);
    cpu.memory
        .borrow_mut()
        .write(initial_pc + 3, (value >> 8) as u8);

    cpu.tick();

    assert_eq!(cpu.get_index_register(reg), value);
}

#[rstest]
#[case::ld_b_ix_d(0, GPR::B, 0xDD, 0x46, 0x42, 0x1000, 5)] // ld b, (ix + 5)
#[case::ld_c_ix_d(0, GPR::C, 0xDD, 0x4E, 0x42, 0x1000, -5)] // ld c, (ix - 5)
#[case::ld_a_iy_d(0, GPR::A, 0xFD, 0x7E, 0x42, 0x1000, 0)] // ld a, (iy + 0)
fn test_ld_indexed_load(
    #[case] initial_pc: u16,
    #[case] dest_reg: GPR,
    #[case] prefix: u8,
    #[case] opcode: u8,
    #[case] value: u8,
    #[case] base_addr: u16,
    #[case] offset: i8,
) {
    let mut cpu = setup_cpu();
    cpu.pc = initial_pc;
    cpu.memory.borrow_mut().write(initial_pc, prefix);
    cpu.memory.borrow_mut().write(initial_pc + 1, opcode);
    cpu.memory.borrow_mut().write(initial_pc + 2, offset as u8);

    let target_addr = base_addr.wrapping_add_signed(offset as i16);
    cpu.memory.borrow_mut().write(target_addr, value);

    if prefix == 0xDD {
        cpu.set_index_register(IndexRegister::IX, base_addr);
    } else {
        cpu.set_index_register(IndexRegister::IY, base_addr);
    }

    cpu.tick();

    assert_eq!(cpu.get_register(dest_reg), value);
}

#[rstest]
#[case::ld_ix_displacement_immediate(0xDD, 0x1000)]
#[case::ld_iy_displacement_immediate(0xFD, 0x2000)]
fn test_ld_indexed_immediate_fetch_order(#[case] prefix: u8, #[case] base_addr: u16) {
    let mut cpu = setup_cpu();
    let displacement = 0x05_i8;
    let value = 0xA5;

    cpu.pc = 0;
    cpu.memory.borrow_mut().write(0, prefix);
    cpu.memory.borrow_mut().write(1, 0x36); // LD (IX/IY+d), n
    cpu.memory.borrow_mut().write(2, displacement as u8);
    cpu.memory.borrow_mut().write(3, value);

    let index_register = if prefix == 0xDD {
        IndexRegister::IX
    } else {
        IndexRegister::IY
    };
    cpu.set_index_register(index_register, base_addr);

    cpu.tick();

    assert_eq!(
        cpu.memory
            .borrow()
            .read(base_addr.wrapping_add_signed(displacement as i16)),
        value
    );
    assert_eq!(cpu.get_pc(), 4);
}

#[rstest]
#[case::ld_a_bc(0, 0x0A, 0x42, 0x1000)] // LD A, (BC)
#[case::ld_a_de(0, 0x1A, 0x42, 0x1000)] // LD A, (DE)
fn test_ld_a_indirect_pair(
    #[case] initial_pc: u16,
    #[case] opcode: u8,
    #[case] value: u8,
    #[case] addr: u16,
) {
    let mut cpu = setup_cpu();
    cpu.pc = initial_pc;
    cpu.memory.borrow_mut().write(initial_pc, opcode);
    if opcode == 0x0A {
        cpu.set_register_pair(RegisterPair::BC, addr);
    } else {
        cpu.set_register_pair(RegisterPair::DE, addr);
    }
    cpu.memory.borrow_mut().write(addr, value);

    cpu.tick();

    assert_eq!(cpu.get_register(GPR::A), value);
}

#[rstest]
#[case::ld_bc_a(0, 0x02, 0x42, 0x1000)] // LD (BC), A
#[case::ld_de_a(0, 0x12, 0x42, 0x1000)] // LD (DE), A
fn test_ld_indirect_pair_a(
    #[case] initial_pc: u16,
    #[case] opcode: u8,
    #[case] value: u8,
    #[case] addr: u16,
) {
    let mut cpu = setup_cpu();
    cpu.pc = initial_pc;
    cpu.memory.borrow_mut().write(initial_pc, opcode);
    cpu.set_register(GPR::A, value);
    if opcode == 0x02 {
        cpu.set_register_pair(RegisterPair::BC, addr);
    } else {
        cpu.set_register_pair(RegisterPair::DE, addr);
    }

    cpu.tick();

    assert_eq!(cpu.memory.borrow().read(addr), value);
}

#[rstest]
#[case::ld_a_nn(0, 0x3A, 0x42, 0x1000)] // LD A, (nn)
fn test_ld_a_absolute(
    #[case] initial_pc: u16,
    #[case] opcode: u8,
    #[case] value: u8,
    #[case] addr: u16,
) {
    let mut cpu = setup_cpu();
    cpu.pc = initial_pc;
    cpu.memory.borrow_mut().write(initial_pc, opcode);
    cpu.memory
        .borrow_mut()
        .write(initial_pc + 1, (addr & 0xFF) as u8);
    cpu.memory
        .borrow_mut()
        .write(initial_pc + 2, (addr >> 8) as u8);
    cpu.memory.borrow_mut().write(addr, value);

    cpu.tick();

    assert_eq!(cpu.get_register(GPR::A), value);
}

#[rstest]
#[case::ld_nn_a(0, 0x32, 0x42, 0x1000)] // LD (nn), A
fn test_ld_absolute_a(
    #[case] initial_pc: u16,
    #[case] opcode: u8,
    #[case] value: u8,
    #[case] addr: u16,
) {
    let mut cpu = setup_cpu();
    cpu.pc = initial_pc;
    cpu.memory.borrow_mut().write(initial_pc, opcode);
    cpu.memory
        .borrow_mut()
        .write(initial_pc + 1, (addr & 0xFF) as u8);
    cpu.memory
        .borrow_mut()
        .write(initial_pc + 2, (addr >> 8) as u8);
    cpu.set_register(GPR::A, value);

    cpu.tick();

    assert_eq!(cpu.memory.borrow().read(addr), value);
}

#[rstest]
#[case::ld_ixh_n(0, IndexRegisterPart::IXH, 0xDD, 0x26, 0x42)]
#[case::ld_ixl_n(0, IndexRegisterPart::IXL, 0xDD, 0x2E, 0x42)]
#[case::ld_iyh_n(0, IndexRegisterPart::IYH, 0xFD, 0x26, 0x42)]
#[case::ld_iyl_n(0, IndexRegisterPart::IYL, 0xFD, 0x2E, 0x42)]
fn test_ld_index_parts_immediate(
    #[case] initial_pc: u16,
    #[case] dest: IndexRegisterPart,
    #[case] prefix: u8,
    #[case] opcode: u8,
    #[case] value: u8,
) {
    let mut cpu = setup_cpu();
    cpu.pc = initial_pc;
    cpu.memory.borrow_mut().write(initial_pc, prefix);
    cpu.memory.borrow_mut().write(initial_pc + 1, opcode);
    cpu.memory.borrow_mut().write(initial_pc + 2, value);

    cpu.tick();

    assert_eq!(cpu.get_index_register_part(dest), value);
}

#[rstest]
#[case::ld_b_ixh(0, GPR::B, IndexRegisterPart::IXH, 0xDD, 0x44, 0x42)]
#[case::ld_c_ixl(0, GPR::C, IndexRegisterPart::IXL, 0xDD, 0x4D, 0x42)]
fn test_ld_from_index_part(
    #[case] initial_pc: u16,
    #[case] dest: GPR,
    #[case] src: IndexRegisterPart,
    #[case] prefix: u8,
    #[case] opcode: u8,
    #[case] value: u8,
) {
    let mut cpu = setup_cpu();
    cpu.pc = initial_pc;
    cpu.memory.borrow_mut().write(initial_pc, prefix);
    cpu.memory.borrow_mut().write(initial_pc + 1, opcode);

    cpu.set_index_register_part(src, value);

    cpu.tick();

    assert_eq!(cpu.get_register(dest), value);
}

#[rstest]
#[case::ld_ixh_b(0, IndexRegisterPart::IXH, GPR::B, 0xDD, 0x60, 0x42)]
#[case::ld_ixl_c(0, IndexRegisterPart::IXL, GPR::C, 0xDD, 0x69, 0x42)]
fn test_ld_to_index_part(
    #[case] initial_pc: u16,
    #[case] dest: IndexRegisterPart,
    #[case] src: GPR,
    #[case] prefix: u8,
    #[case] opcode: u8,
    #[case] value: u8,
) {
    let mut cpu = setup_cpu();
    cpu.pc = initial_pc;
    cpu.memory.borrow_mut().write(initial_pc, prefix);
    cpu.memory.borrow_mut().write(initial_pc + 1, opcode);

    cpu.set_register(src, value);

    cpu.tick();

    assert_eq!(cpu.get_index_register_part(dest), value);
}
