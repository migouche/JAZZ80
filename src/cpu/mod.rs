#![allow(clippy::upper_case_acronyms)]
#![cfg_attr(test, allow(clippy::too_many_arguments, clippy::type_complexity))]

pub mod alu;
mod block_instructions;

#[cfg(test)]
use std::collections::VecDeque;

use std::{cell::RefCell, rc::Rc};

use crate::{
    cpu::alu::{
        add_16, alu_op, bit, dec, inc, res,
        rot::{self, RotOperation},
        set, sub_16,
    },
    traits::{IODevice, MemoryMapper, SynchronousComponent},
};

#[cfg(test)]
macro_rules! test_log {
    ($self:expr, $fmt:expr $(, $($arg:tt)+)? ) => {
        $self.test_callback.1(&format!($fmt $(, $($arg)+)?), &mut $self.test_callback.0)
    };
}

#[cfg(not(test))]
macro_rules! test_log {
    ($self:expr, $fmt:expr $(, $($arg:tt)+)? ) => {
        ()
    };
}

mod decoding;

const _: fn(&mut Z80A, u8, PrefixAddressing) = Z80A::decode_unprefixed_legacy;

mod flags {
    pub const CARRY: u8 = 0b00000001;
    pub const ADD_SUB: u8 = 0b00000010;
    pub const PARITY_OVERFLOW: u8 = 0b00000100;
    pub const X: u8 = 0b00001000;
    pub const HALF_CARRY: u8 = 0b00010000;
    pub const Y: u8 = 0b00100000;
    pub const ZERO: u8 = 0b01000000;
    pub const SIGN: u8 = 0b10000000;
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum GPR {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum RegisterPair {
    BC,
    DE,
    HL,
    AF,
    SP,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RegOps {
    B,
    C,
    D,
    E,
    H,
    L,
    HLIndirect,
    A,
}

impl TryFrom<u8> for RegOps {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(RegOps::B),
            1 => Ok(RegOps::C),
            2 => Ok(RegOps::D),
            3 => Ok(RegOps::E),
            4 => Ok(RegOps::H),
            5 => Ok(RegOps::L),
            6 => Ok(RegOps::HLIndirect),
            7 => Ok(RegOps::A),
            _ => Err(format!("Invalid RegOps value: {}", value)),
        }
    }
}

impl std::fmt::Display for RegOps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegOps::B => write!(f, "B"),
            RegOps::C => write!(f, "C"),
            RegOps::D => write!(f, "D"),
            RegOps::E => write!(f, "E"),
            RegOps::H => write!(f, "H"),
            RegOps::L => write!(f, "L"),
            RegOps::HLIndirect => write!(f, "(HL)"),
            RegOps::A => write!(f, "A"),
        }
    }
}

impl std::str::FromStr for RegOps {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "B" => Ok(RegOps::B),
            "C" => Ok(RegOps::C),
            "D" => Ok(RegOps::D),
            "E" => Ok(RegOps::E),
            "H" => Ok(RegOps::H),
            "L" => Ok(RegOps::L),
            "(HL)" => Ok(RegOps::HLIndirect),
            "A" => Ok(RegOps::A),
            _ => Err(format!("Invalid register: {}", s)),
        }
    }
}

impl From<RegOps> for AddressingMode {
    fn from(op: RegOps) -> Self {
        match op {
            RegOps::B => AddressingMode::Register(GPR::B),
            RegOps::C => AddressingMode::Register(GPR::C),
            RegOps::D => AddressingMode::Register(GPR::D),
            RegOps::E => AddressingMode::Register(GPR::E),
            RegOps::H => AddressingMode::Register(GPR::H),
            RegOps::L => AddressingMode::Register(GPR::L),
            RegOps::HLIndirect => AddressingMode::RegisterIndirect(RegisterPair::HL),
            RegOps::A => AddressingMode::Register(GPR::A),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RpOps {
    BC,
    DE,
    HL,
    SP,
}

impl TryFrom<u8> for RpOps {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(RpOps::BC),
            1 => Ok(RpOps::DE),
            2 => Ok(RpOps::HL),
            3 => Ok(RpOps::SP),
            _ => Err(format!("Invalid Rp value: {}", value)),
        }
    }
}

impl std::fmt::Display for RpOps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpOps::BC => write!(f, "BC"),
            RpOps::DE => write!(f, "DE"),
            RpOps::HL => write!(f, "HL"),
            RpOps::SP => write!(f, "SP"),
        }
    }
}

impl std::str::FromStr for RpOps {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BC" => Ok(RpOps::BC),
            "DE" => Ok(RpOps::DE),
            "HL" => Ok(RpOps::HL),
            "SP" => Ok(RpOps::SP),
            _ => Err(format!("Invalid register pair: {}", s)),
        }
    }
}

impl From<RpOps> for RegisterPair {
    fn from(rp: RpOps) -> Self {
        match rp {
            RpOps::BC => RegisterPair::BC,
            RpOps::DE => RegisterPair::DE,
            RpOps::HL => RegisterPair::HL,
            RpOps::SP => RegisterPair::SP,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Rp2Ops {
    BC,
    DE,
    HL,
    AF,
}

impl TryFrom<u8> for Rp2Ops {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Rp2Ops::BC),
            1 => Ok(Rp2Ops::DE),
            2 => Ok(Rp2Ops::HL),
            3 => Ok(Rp2Ops::AF),
            _ => Err(format!("Invalid Rp2 value: {}", value)),
        }
    }
}

impl std::fmt::Display for Rp2Ops {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rp2Ops::BC => write!(f, "BC"),
            Rp2Ops::DE => write!(f, "DE"),
            Rp2Ops::HL => write!(f, "HL"),
            Rp2Ops::AF => write!(f, "AF"),
        }
    }
}

impl std::str::FromStr for Rp2Ops {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BC" => Ok(Rp2Ops::BC),
            "DE" => Ok(Rp2Ops::DE),
            "HL" => Ok(Rp2Ops::HL),
            "AF" => Ok(Rp2Ops::AF),
            _ => Err(format!("Invalid register pair 2: {}", s)),
        }
    }
}

impl From<Rp2Ops> for RegisterPair {
    fn from(rp: Rp2Ops) -> Self {
        match rp {
            Rp2Ops::BC => RegisterPair::BC,
            Rp2Ops::DE => RegisterPair::DE,
            Rp2Ops::HL => RegisterPair::HL,
            Rp2Ops::AF => RegisterPair::AF,
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Flag {
    C,
    N,
    PV,
    Y,
    H,
    X,
    Z,
    S,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum SystemRegister {
    I,
    R,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum IndexRegister {
    IX,
    IY,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum IndexRegisterPart {
    IXH,
    IXL,
    IYH,
    IYL,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum AddressingMode {
    Register(GPR),
    RegisterIndirect(RegisterPair),
    Indexed(IndexRegister, i8),
    Immediate(u8),
    Absolute(u16),
    System(SystemRegister),
    IndexRegisterPart(IndexRegisterPart),
    RegisterPair(RegisterPair),
    IndexRegister(IndexRegister),
    ImmediateExtended(u16),
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Condition {
    NZ,
    Z,
    NC,
    C,
    PO,
    PE,
    P,
    M,
}

impl TryFrom<u8> for Condition {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Condition::NZ),
            1 => Ok(Condition::Z),
            2 => Ok(Condition::NC),
            3 => Ok(Condition::C),
            4 => Ok(Condition::PO),
            5 => Ok(Condition::PE),
            6 => Ok(Condition::P),
            7 => Ok(Condition::M),
            _ => Err(format!("Invalid Condition value: {}", value)),
        }
    }
}

impl std::fmt::Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Condition::NZ => write!(f, "NZ"),
            Condition::Z => write!(f, "Z"),
            Condition::NC => write!(f, "NC"),
            Condition::C => write!(f, "C"),
            Condition::PO => write!(f, "PO"),
            Condition::PE => write!(f, "PE"),
            Condition::P => write!(f, "P"),
            Condition::M => write!(f, "M"),
        }
    }
}

impl std::str::FromStr for Condition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NZ" => Ok(Condition::NZ),
            "Z" => Ok(Condition::Z),
            "NC" => Ok(Condition::NC),
            "C" => Ok(Condition::C),
            "PO" => Ok(Condition::PO),
            "PE" => Ok(Condition::PE),
            "P" => Ok(Condition::P),
            "M" => Ok(Condition::M),
            _ => Err(format!("Invalid condition: {}", s)),
        }
    }
}

enum BlockInstruction {
    LDI,
    LDD,
    LDIR,
    LDDR,
    CPI,
    CPD,
    CPIR,
    CPDR,
    INI,
    IND,
    INIR,
    INDR,
    OUTI,
    OUTD,
    OTIR,
    OTDR,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum PrefixAddressing {
    HL,
    IX,
    IY,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ALUOperation {
    ADD,
    ADC,
    SUB,
    SBC,
    AND,
    XOR,
    OR,
    CP,
}

impl TryFrom<u8> for ALUOperation {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ALUOperation::ADD),
            1 => Ok(ALUOperation::ADC),
            2 => Ok(ALUOperation::SUB),
            3 => Ok(ALUOperation::SBC),
            4 => Ok(ALUOperation::AND),
            5 => Ok(ALUOperation::XOR),
            6 => Ok(ALUOperation::OR),
            7 => Ok(ALUOperation::CP),
            _ => Err(format!("Invalid ALU operation value: {}", value)),
        }
    }
}

impl std::fmt::Display for ALUOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ALUOperation::ADD => write!(f, "ADD"),
            ALUOperation::ADC => write!(f, "ADC"),
            ALUOperation::SUB => write!(f, "SUB"),
            ALUOperation::SBC => write!(f, "SBC"),
            ALUOperation::AND => write!(f, "AND"),
            ALUOperation::XOR => write!(f, "XOR"),
            ALUOperation::OR => write!(f, "OR"),
            ALUOperation::CP => write!(f, "CP"),
        }
    }
}

impl std::str::FromStr for ALUOperation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ADD" => Ok(ALUOperation::ADD),
            "ADC" => Ok(ALUOperation::ADC),
            "SUB" => Ok(ALUOperation::SUB),
            "SBC" => Ok(ALUOperation::SBC),
            "AND" => Ok(ALUOperation::AND),
            "XOR" => Ok(ALUOperation::XOR),
            "OR" => Ok(ALUOperation::OR),
            "CP" => Ok(ALUOperation::CP),
            _ => Err(format!("Invalid ALU operation: {}", s)),
        }
    }
}

#[derive(Default, Clone, Copy, Debug)]
struct AFSet {
    a: u8,
    f: u8,
}

#[derive(Default, Clone, Copy, Debug)]
struct GeneralSet {
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
}

pub(super) fn decode_opcode(opcode: u8) -> (u8, u8, u8, u8, bool) // x, y, z, p, q
    /* See http://www.z80.info/decoding.htm for more information
    x = the opcode's 1st octal digit (i.e. bits 7-6)
    y = the opcode's 2nd octal digit (i.e. bits 5-3)
    z = the opcode's 3rd octal digit (i.e. bits 2-0)
    p = y rightshifted one position (i.e. bits 5-4)
    q = y modulo 2 (i.e. bit 3) 
    */ {
    let x = (opcode >> 6) & 0b11;
    let y = (opcode >> 3) & 0b111;
    let z = opcode & 0b111;
    let p = (y >> 1) & 0b11;
    let q = (y & 0b1) == 1;
    (x, y, z, p, q)
}

pub struct Z80A {
    // registers
    af_registers: [AFSet; 2],
    active_af: usize,

    general_registers: [GeneralSet; 2],
    active_general: usize,

    // special registers
    pub(super) pc: u16,
    pub(super) sp: u16,
    ix: u16,
    iy: u16,
    i: u8,
    r: u8,

    pub(super) memory: Rc<RefCell<dyn MemoryMapper>>,
    devices: Vec<Rc<RefCell<dyn IODevice>>>,

    pub(super) halted: bool,
    pub(super) iff1: bool,
    pub(super) iff2: bool,
    interrupt_mode: u8,
    nmi_pending: bool,
    int_pending: bool,
    nmi_last_state: bool, // Track NMI line state for edge detection
    pub(super) iff_delay_count: u8,

    #[cfg(test)]
    test_callback: (
        VecDeque<String>,
        Box<dyn FnMut(&str, &mut VecDeque<String>)>,
    ),
}

impl Z80A {
    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    pub fn set_pc(&mut self, val: u16) {
        self.pc = val;
    }

    pub fn get_sp(&self) -> u16 {
        self.sp
    }

    pub fn set_sp(&mut self, val: u16) {
        self.sp = val;
    }

    pub fn get_ix(&self) -> u16 {
        self.ix
    }

    pub fn set_ix(&mut self, val: u16) {
        self.ix = val;
    }

    pub fn get_iy(&self) -> u16 {
        self.iy
    }

    pub fn set_iy(&mut self, val: u16) {
        self.iy = val;
    }

    pub fn is_halted(&self) -> bool {
        self.halted
    }

    pub fn get_iff1(&self) -> bool {
        self.iff1
    }

    pub fn get_iff2(&self) -> bool {
        self.iff2
    }

    pub fn get_interrupt_mode(&self) -> u8 {
        self.interrupt_mode
    }

    pub fn set_halted(&mut self, halted: bool) {
        self.halted = halted;
    }

    pub fn new(memory: Rc<RefCell<dyn MemoryMapper>>) -> Self {
        Z80A {
            af_registers: [AFSet::default(); 2],
            active_af: 0,
            general_registers: [GeneralSet::default(); 2],
            active_general: 0,
            pc: 0,
            sp: 0,
            ix: 0,
            iy: 0,
            i: 0,
            r: 0,
            memory,
            devices: Vec::new(),
            halted: false,
            iff1: false,
            iff2: false,
            interrupt_mode: 0,
            nmi_pending: false,
            int_pending: false,
            nmi_last_state: false,
            iff_delay_count: 0,

            #[cfg(test)]
            test_callback: (
                VecDeque::new(),
                Box::new(|s, q| q.push_front(s.to_string())),
            ),
        }
    }

    pub fn attach_device(&mut self, device: Rc<RefCell<dyn IODevice>>) {
        self.devices.push(device);
    }

    pub fn nmi(&mut self) {
        self.nmi_pending = true;
    }

    pub fn set_interrupt(&mut self, active: bool) {
        self.int_pending = active;
    }

    fn read_io(&mut self, port: u16) -> u8 {
        for dev in &self.devices {
            if let Some(val) = dev.borrow_mut().read_in(port) {
                return val;
            }
        }
        0xFF
    }

    fn write_io(&mut self, port: u16, data: u8) {
        for dev in &self.devices {
            if dev.borrow_mut().write_out(port, data) {
                return;
            }
        }
    }

    fn is_int_line_active(&self) -> bool {
        if self.int_pending {
            return true;
        }
        for dev in &self.devices {
            if dev.borrow().poll_interrupt() {
                return true;
            }
        }
        false
    }

    fn ack_interrupt(&mut self) -> u8 {
        for dev in &self.devices {
            if dev.borrow().poll_interrupt() {
                return dev.borrow_mut().ack_interrupt();
            }
        }
        0xFF
    }

    fn handle_nmi(&mut self) {
        self.iff2 = self.iff1;
        self.iff1 = false;
        self.push(self.pc);
        self.pc = 0x0066;
    }

    fn handle_int(&mut self) {
        self.iff1 = false;
        self.iff2 = false;

        match self.interrupt_mode {
            0 => {
                let opcode = self.ack_interrupt();
                self.decode(opcode);
            }
            1 => {
                self.ack_interrupt(); // Acknowledge interrupt to clear the line
                self.push(self.pc);
                self.pc = 0x0038;
            }
            2 => {
                let vector = self.ack_interrupt() & 0xFE;
                let addr = ((self.i as u16) << 8) | (vector as u16);
                self.push(self.pc);
                let dest = self.memory.borrow().read_word(addr);
                self.pc = dest;
            }
            _ => unreachable!(),
        }
    }

    fn fetch(&mut self) -> u8 {
        let data = self.memory.borrow().read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        data
    }

    fn fetch_word(&mut self) -> u16 {
        let low = self.fetch() as u16;
        let high = self.fetch() as u16;
        (high << 8) | low
    }

    fn fetch_displacement(&mut self) -> i8 {
        self.fetch() as i8
    }

    fn ex_af_af_prime(&mut self) {
        self.active_af = 1 - self.active_af;
    }

    fn exx(&mut self) {
        self.active_general = 1 - self.active_general;
    }

    pub fn set_register(&mut self, reg: GPR, value: u8) {
        match reg {
            GPR::A => self.af_registers[self.active_af].a = value,
            GPR::F => self.af_registers[self.active_af].f = value,
            GPR::B => self.general_registers[self.active_general].b = value,
            GPR::C => self.general_registers[self.active_general].c = value,
            GPR::D => self.general_registers[self.active_general].d = value,
            GPR::E => self.general_registers[self.active_general].e = value,
            GPR::H => self.general_registers[self.active_general].h = value,
            GPR::L => self.general_registers[self.active_general].l = value,
        }
    }

    pub fn get_shadow_register(&self, reg: GPR) -> u8 {
        match reg {
            GPR::A => self.af_registers[1 - self.active_af].a,
            GPR::F => self.af_registers[1 - self.active_af].f,
            GPR::B => self.general_registers[1 - self.active_general].b,
            GPR::C => self.general_registers[1 - self.active_general].c,
            GPR::D => self.general_registers[1 - self.active_general].d,
            GPR::E => self.general_registers[1 - self.active_general].e,
            GPR::H => self.general_registers[1 - self.active_general].h,
            GPR::L => self.general_registers[1 - self.active_general].l,
        }
    }

    pub fn set_shadow_register(&mut self, reg: GPR, value: u8) {
        match reg {
            GPR::A => self.af_registers[1 - self.active_af].a = value,
            GPR::F => self.af_registers[1 - self.active_af].f = value,
            GPR::B => self.general_registers[1 - self.active_general].b = value,
            GPR::C => self.general_registers[1 - self.active_general].c = value,
            GPR::D => self.general_registers[1 - self.active_general].d = value,
            GPR::E => self.general_registers[1 - self.active_general].e = value,
            GPR::H => self.general_registers[1 - self.active_general].h = value,
            GPR::L => self.general_registers[1 - self.active_general].l = value,
        }
    }

    pub fn get_register(&self, reg: GPR) -> u8 {
        match reg {
            GPR::A => self.af_registers[self.active_af].a,
            GPR::F => self.af_registers[self.active_af].f,
            GPR::B => self.general_registers[self.active_general].b,
            GPR::C => self.general_registers[self.active_general].c,
            GPR::D => self.general_registers[self.active_general].d,
            GPR::E => self.general_registers[self.active_general].e,
            GPR::H => self.general_registers[self.active_general].h,
            GPR::L => self.general_registers[self.active_general].l,
        }
    }

    pub fn get_register_pair(&self, pair: RegisterPair) -> u16 {
        match pair {
            RegisterPair::AF => {
                let regs = &self.af_registers[self.active_af];
                ((regs.a as u16) << 8) | (regs.f as u16)
            }
            RegisterPair::BC => {
                let regs = &self.general_registers[self.active_general];
                ((regs.b as u16) << 8) | (regs.c as u16)
            }
            RegisterPair::DE => {
                let regs = &self.general_registers[self.active_general];
                ((regs.d as u16) << 8) | (regs.e as u16)
            }
            RegisterPair::HL => {
                let regs = &self.general_registers[self.active_general];
                ((regs.h as u16) << 8) | (regs.l as u16)
            }
            RegisterPair::SP => self.sp,
        }
    }

    fn set_register_pair(&mut self, pair: RegisterPair, value: u16) {
        match pair {
            RegisterPair::AF => {
                let regs = &mut self.af_registers[self.active_af];
                regs.a = (value >> 8) as u8;
                regs.f = (value & 0xFF) as u8;
            }
            RegisterPair::BC => {
                let regs = &mut self.general_registers[self.active_general];
                regs.b = (value >> 8) as u8;
                regs.c = (value & 0xFF) as u8;
            }
            RegisterPair::DE => {
                let regs = &mut self.general_registers[self.active_general];
                regs.d = (value >> 8) as u8;
                regs.e = (value & 0xFF) as u8;
            }
            RegisterPair::HL => {
                let regs = &mut self.general_registers[self.active_general];
                regs.h = (value >> 8) as u8;
                regs.l = (value & 0xFF) as u8;
            }
            RegisterPair::SP => self.sp = value,
        }
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        let f = self.af_registers[self.active_af].f;
        match flag {
            Flag::C => f & flags::CARRY != 0,
            Flag::N => f & flags::ADD_SUB != 0,
            Flag::PV => f & flags::PARITY_OVERFLOW != 0,
            Flag::Y => f & flags::X != 0,
            Flag::H => f & flags::HALF_CARRY != 0,
            Flag::X => f & flags::Y != 0,
            Flag::Z => f & flags::ZERO != 0,
            Flag::S => f & flags::SIGN != 0,
        }
    }

    pub fn set_flag(&mut self, value: bool, flag: Flag) {
        let f = &mut self.af_registers[self.active_af].f;
        let mask = match flag {
            Flag::C => flags::CARRY,
            Flag::N => flags::ADD_SUB,
            Flag::PV => flags::PARITY_OVERFLOW,
            Flag::Y => flags::X,
            Flag::H => flags::HALF_CARRY,
            Flag::X => flags::Y,
            Flag::Z => flags::ZERO,
            Flag::S => flags::SIGN,
        };
        if value { *f |= mask } else { *f &= !mask }
    }

    fn set_system_register(&mut self, reg: SystemRegister, value: u16) {
        match reg {
            SystemRegister::I => self.i = value as u8,
            SystemRegister::R => self.r = value as u8,
        }
    }

    fn get_system_register(&self, reg: SystemRegister) -> u16 {
        match reg {
            SystemRegister::I => self.i as u16,
            SystemRegister::R => self.r as u16,
        }
    }

    fn set_index_register_part(&mut self, reg: IndexRegisterPart, value: u8) {
        match reg {
            IndexRegisterPart::IXH => self.ix = (self.ix & 0x00FF) | ((value as u16) << 8),
            IndexRegisterPart::IXL => self.ix = (self.ix & 0xFF00) | (value as u16),
            IndexRegisterPart::IYH => self.iy = (self.iy & 0x00FF) | ((value as u16) << 8),
            IndexRegisterPart::IYL => self.iy = (self.iy & 0xFF00) | (value as u16),
        }
    }

    fn get_index_register_part(&self, reg: IndexRegisterPart) -> u8 {
        match reg {
            IndexRegisterPart::IXH => (self.ix >> 8) as u8,
            IndexRegisterPart::IXL => (self.ix & 0x00FF) as u8,
            IndexRegisterPart::IYH => (self.iy >> 8) as u8,
            IndexRegisterPart::IYL => (self.iy & 0x00FF) as u8,
        }
    }

    fn read_8(&mut self, mode: AddressingMode) -> u8 {
        match mode {
            AddressingMode::Register(r) => self.get_register(r),
            AddressingMode::RegisterIndirect(rp) => {
                let addr = self.get_register_pair(rp);
                self.memory.borrow().read(addr)
            }
            AddressingMode::Indexed(idx, disp) => {
                let base = self.get_index_register(idx);
                let addr = base.wrapping_add_signed(disp as i16);
                self.memory.borrow().read(addr)
            }
            AddressingMode::Immediate(n) => n,
            AddressingMode::Absolute(addr) => self.memory.borrow().read(addr),
            AddressingMode::System(r) => (self.get_system_register(r) & 0xFF) as u8,
            AddressingMode::IndexRegisterPart(r) => self.get_index_register_part(r),
            _ => panic!("Invalid addressing mode for read_8."),
        }
    }

    fn write_8(&mut self, mode: AddressingMode, value: u8) {
        match mode {
            AddressingMode::Register(r) => self.set_register(r, value),
            AddressingMode::RegisterIndirect(rp) => {
                let addr = self.get_register_pair(rp);
                self.memory.borrow_mut().write(addr, value);
            }
            AddressingMode::Indexed(idx, disp) => {
                let base = self.get_index_register(idx);
                let addr = base.wrapping_add_signed(disp as i16);
                self.memory.borrow_mut().write(addr, value);
            }
            AddressingMode::Absolute(addr) => self.memory.borrow_mut().write(addr, value),
            AddressingMode::System(r) => self.set_system_register(r, value as u16),
            AddressingMode::IndexRegisterPart(r) => self.set_index_register_part(r, value),
            _ => panic!("Invalid addressing mode for write_8."),
        }
    }

    fn read_16(&mut self, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::RegisterPair(rp) => self.get_register_pair(rp),
            AddressingMode::IndexRegister(idx) => self.get_index_register(idx),
            AddressingMode::System(r) => self.get_system_register(r),
            AddressingMode::ImmediateExtended(nn) => nn,
            AddressingMode::Absolute(addr) => self.memory.borrow().read_word(addr),
            _ => panic!("Invalid addressing mode for read_16."),
        }
    }

    fn write_16(&mut self, mode: AddressingMode, value: u16) {
        match mode {
            AddressingMode::RegisterPair(rp) => self.set_register_pair(rp, value),
            AddressingMode::IndexRegister(idx) => self.set_index_register(idx, value),
            AddressingMode::System(r) => self.set_system_register(r, value),
            AddressingMode::Absolute(addr) => self.memory.borrow_mut().write_word(addr, value),
            _ => panic!("Invalid addressing mode for write_16."),
        }
    }

    fn get_index_register(&self, index: IndexRegister) -> u16 {
        match index {
            IndexRegister::IX => self.ix,
            IndexRegister::IY => self.iy,
        }
    }

    fn set_index_register(&mut self, index: IndexRegister, value: u16) {
        match index {
            IndexRegister::IX => self.ix = value,
            IndexRegister::IY => self.iy = value,
        }
    }

    fn push(&mut self, value: u16) {
        self.sp = self.sp.wrapping_sub(2);
        self.memory.borrow_mut().write_word(self.sp, value);
    }

    fn pop(&mut self) -> u16 {
        let value = self.memory.borrow().read_word(self.sp);
        self.sp = self.sp.wrapping_add(2);
        value
    }

    // TABLES FROM http://www.z80.info/decoding.htm

    fn table_r(&mut self, p: u8) -> RegOps {
        match RegOps::try_from(p) {
            Ok(r) => {
                test_log!(self, "{}", r);
                r
            }
            Err(e) => panic!("{}", e),
        }
    }

    fn table_alu(&mut self, y: u8) -> ALUOperation {
        match ALUOperation::try_from(y) {
            Ok(op) => {
                test_log!(self, "{} A", op);
                op
            }
            Err(e) => panic!("{}", e),
        }
    }

    fn alu_op(&mut self, op: ALUOperation, value: u8) {
        match op {
            ALUOperation::ADD => {
                let (result, flags) = alu_op::add(self.get_register(GPR::A), value, false);
                self.set_register(GPR::A, result);
                self.af_registers[self.active_af].f = flags;
            }
            ALUOperation::ADC => {
                let carry = self.get_flag(Flag::C);
                let (result, flags) = alu_op::add(self.get_register(GPR::A), value, carry);
                self.set_register(GPR::A, result);
                self.af_registers[self.active_af].f = flags;
            }
            ALUOperation::SUB => {
                let (result, flags) = alu_op::sub(self.get_register(GPR::A), value, false);
                self.set_register(GPR::A, result);
                self.af_registers[self.active_af].f = flags;
            }
            ALUOperation::SBC => {
                let carry = self.get_flag(Flag::C);
                let (result, flags) = alu_op::sub(self.get_register(GPR::A), value, carry);
                self.set_register(GPR::A, result);
                self.af_registers[self.active_af].f = flags;
            }
            ALUOperation::AND => {
                let (result, flags) = alu_op::and(self.get_register(GPR::A), value);
                self.set_register(GPR::A, result);
                self.af_registers[self.active_af].f = flags;
            }
            ALUOperation::OR => {
                let (result, flags) = alu_op::or(self.get_register(GPR::A), value);
                self.set_register(GPR::A, result);
                self.af_registers[self.active_af].f = flags;
            }
            ALUOperation::XOR => {
                let (result, flags) = alu_op::xor(self.get_register(GPR::A), value);
                self.set_register(GPR::A, result);
                self.af_registers[self.active_af].f = flags;
            }
            ALUOperation::CP => {
                let (_, flags) = alu_op::sub(self.get_register(GPR::A), value, false);
                // result is ignored for CP
                self.af_registers[self.active_af].f = flags;
            }
        }
    }

    fn inc_op(&mut self, dest: AddressingMode) {
        let value = self.read_8(dest);
        let (result, flags) = inc(value);

        // Preserve Carry flag
        let current_carry = self.get_flag(Flag::C) as u8;
        self.af_registers[self.active_af].f = flags | current_carry;

        self.write_8(dest, result);
    }

    fn dec_op(&mut self, dest: AddressingMode) {
        let value = self.read_8(dest);
        let (result, flags) = dec(value);

        // Preserve Carry flag
        let current_carry = self.af_registers[self.active_af].f & flags::CARRY;
        self.af_registers[self.active_af].f = flags | current_carry;

        self.write_8(dest, result);
    }

    fn inc_16_op(&mut self, dest: AddressingMode) {
        let val = self.read_16(dest);
        let result = val.wrapping_add(1);
        self.write_16(dest, result);
    }

    fn dec_16_op(&mut self, dest: AddressingMode) {
        let val = self.read_16(dest);
        let result = val.wrapping_sub(1);
        self.write_16(dest, result);
    }

    fn add_16_op(&mut self, dest: AddressingMode, src: AddressingMode, use_carry: bool) {
        let val_dest = self.read_16(dest);
        let val_src = self.read_16(src);

        let (result, flags) = add_16(
            val_dest,
            val_src,
            self.af_registers[self.active_af].f,
            use_carry,
        );
        self.af_registers[self.active_af].f = flags;

        self.write_16(dest, result);
    }

    fn sub_16_op(&mut self, dest: AddressingMode, src: AddressingMode, use_carry: bool) {
        let val_dest = self.read_16(dest);
        let val_src = self.read_16(src);

        let (result, flags) = sub_16(
            val_dest,
            val_src,
            self.af_registers[self.active_af].f,
            use_carry,
        );
        self.af_registers[self.active_af].f = flags | flags::ADD_SUB;

        self.write_16(dest, result);
    }

    fn table_rp(&mut self, p: u8) -> RegisterPair {
        match RpOps::try_from(p) {
            Ok(rp) => {
                test_log!(self, "{}", rp);
                rp.into()
            }
            Err(e) => panic!("{}", e),
        }
    }

    fn table_rp2(&mut self, p: u8) -> RegisterPair {
        match Rp2Ops::try_from(p) {
            Ok(rp) => {
                test_log!(self, "{}", rp);
                rp.into()
            }
            Err(e) => panic!("{}", e),
        }
    }

    fn table_cc(&mut self, y: u8) -> Condition {
        match Condition::try_from(y) {
            Ok(c) => {
                test_log!(self, "{}", c);
                c
            }
            Err(e) => panic!("{}", e),
        }
    }

    fn table_bli(&mut self, a: u8, b: u8) -> BlockInstruction {
        match (a, b) {
            (4, 0) => {
                test_log!(self, "LDI");
                BlockInstruction::LDI
            }
            (4, 1) => {
                test_log!(self, "CPI");
                BlockInstruction::CPI
            }
            (4, 2) => {
                test_log!(self, "INI");
                BlockInstruction::INI
            }
            (4, 3) => {
                test_log!(self, "OUTI");
                BlockInstruction::OUTI
            }
            (5, 0) => {
                test_log!(self, "LDD");
                BlockInstruction::LDD
            }
            (5, 1) => {
                test_log!(self, "CPD");
                BlockInstruction::CPD
            }
            (5, 2) => {
                test_log!(self, "IND");
                BlockInstruction::IND
            }
            (5, 3) => {
                test_log!(self, "OUTD");
                BlockInstruction::OUTD
            }
            (6, 0) => {
                test_log!(self, "LDIR");
                BlockInstruction::LDIR
            }
            (6, 1) => {
                test_log!(self, "CPIR");
                BlockInstruction::CPIR
            }
            (6, 2) => {
                test_log!(self, "INIR");
                BlockInstruction::INIR
            }
            (6, 3) => {
                test_log!(self, "OTIR");
                BlockInstruction::OTIR
            }
            (7, 0) => {
                test_log!(self, "LDDR");
                BlockInstruction::LDDR
            }
            (7, 1) => {
                test_log!(self, "CPDR");
                BlockInstruction::CPDR
            }
            (7, 2) => {
                test_log!(self, "INDR");
                BlockInstruction::INDR
            }
            (7, 3) => {
                test_log!(self, "OTDR");
                BlockInstruction::OTDR
            }
            _ => unreachable!("Invalid a,b values"), // should never happen
        }
    }

    fn execute_block_instruction(&mut self, instruction: BlockInstruction) {
        block_instructions::execute(self, instruction);
    }

    fn transform_register(
        &mut self,
        reg: AddressingMode,
        addressing: PrefixAddressing,
    ) -> AddressingMode {
        let (index, high, low, _name) = match addressing {
            PrefixAddressing::HL => return reg,
            PrefixAddressing::IX => (IndexRegister::IX, IndexRegisterPart::IXH, IndexRegisterPart::IXL, "IX"),
            PrefixAddressing::IY => (IndexRegister::IY, IndexRegisterPart::IYH, IndexRegisterPart::IYL, "IY"),
        };
        match reg {
            AddressingMode::Register(GPR::H) => {
                test_log!(self, "{}H", _name);
                AddressingMode::IndexRegisterPart(high)
            }
            AddressingMode::Register(GPR::L) => {
                test_log!(self, "{}L", _name);
                AddressingMode::IndexRegisterPart(low)
            }
            AddressingMode::RegisterIndirect(RegisterPair::HL) => {
                test_log!(self, "({} + d)", _name);
                AddressingMode::Indexed(index, self.fetch_displacement())
            }
            AddressingMode::RegisterPair(RegisterPair::HL) => {
                test_log!(self, "{}", _name);
                AddressingMode::IndexRegister(index)
            }
            _ => reg,
        }
    }

    fn evaluate_condition(&mut self, condition: Condition) -> bool {
        match condition {
            Condition::NZ => !self.get_flag(Flag::Z),
            Condition::Z => self.get_flag(Flag::Z),
            Condition::NC => !self.get_flag(Flag::C),
            Condition::C => self.get_flag(Flag::C),
            Condition::PO => !self.get_flag(Flag::PV),
            Condition::PE => self.get_flag(Flag::PV),
            Condition::P => !self.get_flag(Flag::S),
            Condition::M => self.get_flag(Flag::S),
        }
    }

    fn decode_unprefixed(&mut self, opcode: u8, addressing: PrefixAddressing) {
        decoding::decode_unprefixed(self, opcode, addressing);
    }

    fn decode_unprefixed_legacy(&mut self, opcode: u8, addressing: PrefixAddressing) {
        //test_log!(self, "decode_unprefixed");
        match addressing {
            PrefixAddressing::HL => test_log!(self, "decode_unprefixed"),
            PrefixAddressing::IX => test_log!(self, "decode_dd"),
            PrefixAddressing::IY => test_log!(self, "decode_fd"),
        }

        let (x, y, z, p, q) = decode_opcode(opcode);
        match x {
            0 => match z {
                0 => match y {
                    // Relative jumps and assorted ops
                    0 => test_log!(self, "NOP"), // NOP

                    1 => {
                        test_log!(self, "EX AF, AF'");
                        self.ex_af_af_prime();
                    }
                    2 => {
                        // DJNZ d
                        test_log!(self, "DJNZ d");

                        let d = self.fetch_displacement();
                        let b = self.get_register(GPR::B).wrapping_sub(1);
                        self.set_register(GPR::B, b);
                        if b != 0 {
                            self.pc = self.pc.wrapping_add(d as i16 as u16);
                        }
                    }
                    3 => {
                        // JR d
                        test_log!(self, "JR d");
                        let d = self.fetch_displacement();
                        self.pc = self.pc.wrapping_add(d as i16 as u16);
                    }
                    4..=7 => {
                        // JR cc[y-4], d
                        test_log!(self, "JR cc[y-4], d");
                        let condition = self.table_cc(y - 4);
                        let d = self.fetch_displacement();
                        if self.evaluate_condition(condition) {
                            self.pc = self.pc.wrapping_add(d as i16 as u16);
                        }
                    }
                    _ => unreachable!("Invalid y value"), // should never happen
                },

                1 => {
                    if q {
                        test_log!(self, "ADD HL/IX/IY, rp[p]");
                        let dest = self.transform_register(
                            AddressingMode::RegisterPair(RegisterPair::HL),
                            addressing,
                        );
                        let rp = self.table_rp(p);
                        let src =
                            self.transform_register(AddressingMode::RegisterPair(rp), addressing);
                        self.add_16_op(dest, src, false);
                    } else {
                        // 16-bit load immediate/add
                        // LD rp[p], nn
                        test_log!(self, "LD rp[p], nn");
                        let nn = self.fetch_word();
                        let rp = self.table_rp(p);
                        let src =
                            self.transform_register(AddressingMode::RegisterPair(rp), addressing);
                        self.ld_16(src, AddressingMode::ImmediateExtended(nn));
                    }
                }

                2 => match (q, p) {
                    // Indirect loading
                    (false, 0) => {
                        test_log!(self, "LD (BC), A");
                        self.ld(
                            AddressingMode::RegisterIndirect(RegisterPair::BC),
                            AddressingMode::Register(GPR::A),
                        )
                    } // LD (BC), A
                    (false, 1) => {
                        test_log!(self, "LD (DE), A");
                        self.ld(
                            AddressingMode::RegisterIndirect(RegisterPair::DE),
                            AddressingMode::Register(GPR::A),
                        )
                    } // LD (DE), A
                    (false, 2) => {
                        test_log!(self, "LD (nn), HL/IX/IY");
                        let addr = self.fetch_word();
                        let transformed = self.transform_register(
                            AddressingMode::RegisterPair(RegisterPair::HL),
                            addressing,
                        );
                        self.ld_16(AddressingMode::Absolute(addr), transformed)
                    } // LD (nn), HL or LD (nn), IX/IY
                    (false, 3) => {
                        test_log!(self, "LD (nn), A");
                        let addr = self.fetch_word();
                        self.ld(
                            AddressingMode::Absolute(addr),
                            AddressingMode::Register(GPR::A),
                        )
                    } // LD (nn), A
                    (true, 0) => {
                        test_log!(self, "LD A, (BC)");
                        self.ld(
                            AddressingMode::Register(GPR::A),
                            AddressingMode::RegisterIndirect(RegisterPair::BC),
                        )
                    } // LD A, (BC)
                    (true, 1) => {
                        test_log!(self, "LD A, (DE)");
                        self.ld(
                            AddressingMode::Register(GPR::A),
                            AddressingMode::RegisterIndirect(RegisterPair::DE),
                        )
                    } // LD A, (DE)
                    (true, 2) => {
                        test_log!(self, "LD HL/IX/IY, (nn)");
                        let addr = self.fetch_word();
                        let transformed = self.transform_register(
                            AddressingMode::RegisterPair(RegisterPair::HL),
                            addressing,
                        );
                        self.ld_16(transformed, AddressingMode::Absolute(addr))
                    } // LD HL, (nn) or LD IX/IY, (nn)
                    (true, 3) => {
                        test_log!(self, "LD A, (nn)");
                        let addr = self.fetch_word();
                        self.ld(
                            AddressingMode::Register(GPR::A),
                            AddressingMode::Absolute(addr),
                        )
                    } // LD A, (nn)
                    _ => unreachable!("Invalid q, p values"), // should never happen
                },

                3 => {
                    if !q {
                        // 16-bit INC/DEC
                        test_log!(self, "INC rp[p]");
                        let rp = self.table_rp(p);
                        let dest =
                            self.transform_register(AddressingMode::RegisterPair(rp), addressing);
                        self.inc_16_op(dest);
                    } else {
                        test_log!(self, "DEC rp[p]");
                        let rp = self.table_rp(p);
                        let dest =
                            self.transform_register(AddressingMode::RegisterPair(rp), addressing);
                        self.dec_16_op(dest);
                    }
                }

                4 => {
                    test_log!(self, "INC r[y]");
                    let reg = self.table_r(y);
                    let dest = self.transform_register(reg.into(), addressing);
                    self.inc_op(dest);
                }
                5 => {
                    test_log!(self, "DEC r[y]");
                    let reg = self.table_r(y);
                    let dest = self.transform_register(reg.into(), addressing);
                    self.dec_op(dest);
                }
                6 => {
                    test_log!(self, "LD r[y], n");
                    let reg = self.table_r(y);
                    let dest = self.transform_register(reg.into(), addressing);
                    let n = self.fetch();
                    self.ld(dest, AddressingMode::Immediate(n))
                } //  LD r[y], n (still have to transform r[y] if IX/IY prefixed)
                7 => {
                    // Assorted operations on accumulator/flags
                    match y {
                        0 => {
                            // RLCA
                            test_log!(self, "RLCA");
                            let a = self.get_register(GPR::A);
                            let (res, f) = rot::rlc(a);
                            self.set_register(GPR::A, res);

                            self.set_flag((f & flags::CARRY) != 0, Flag::C);
                            self.set_flag(false, Flag::N);
                            self.set_flag(false, Flag::H);
                        }
                        1 => {
                            // RRCA
                            test_log!(self, "RRCA");
                            let a = self.get_register(GPR::A);
                            let (res, f) = rot::rrc(a);
                            self.set_register(GPR::A, res);

                            self.set_flag((f & flags::CARRY) != 0, Flag::C);
                            self.set_flag(false, Flag::N);
                            self.set_flag(false, Flag::H);
                        }
                        2 => {
                            // RLA
                            test_log!(self, "RLA");
                            let a = self.get_register(GPR::A);
                            let carry = self.get_flag(Flag::C);
                            let (res, f) = rot::rl(a, carry);
                            self.set_register(GPR::A, res);

                            self.set_flag((f & flags::CARRY) != 0, Flag::C);
                            self.set_flag(false, Flag::N);
                            self.set_flag(false, Flag::H);
                        }
                        3 => {
                            // RRA
                            test_log!(self, "RRA");
                            let a = self.get_register(GPR::A);
                            let carry = self.get_flag(Flag::C);
                            let (res, f) = rot::rr(a, carry);
                            self.set_register(GPR::A, res);

                            self.set_flag((f & flags::CARRY) != 0, Flag::C);
                            self.set_flag(false, Flag::N);
                            self.set_flag(false, Flag::H);
                        }
                        4 => {
                            // DAA
                            test_log!(self, "DAA");
                            self.daa();
                        }
                        5 => {
                            // CPL
                            test_log!(self, "CPL");

                            let a = self.get_register(GPR::A);
                            let result = a ^ 0xFF;
                            self.set_register(GPR::A, result);
                            self.set_flag(true, Flag::N);
                            self.set_flag(true, Flag::H);
                            // undocummented x and y flags
                            // set bit 3 and 5 according to result
                            self.set_flag((result & flags::X) != 0, Flag::X);
                            self.set_flag((result & flags::Y) != 0, Flag::Y);
                        }
                        6 => {
                            //  SCF
                            test_log!(self, "SCF");

                            self.set_flag(true, Flag::C);
                            self.set_flag(false, Flag::N);
                            self.set_flag(false, Flag::H);
                            // undocummented x and y flags
                            let a = self.get_register(GPR::A);
                            self.set_flag((a & flags::X) != 0, Flag::X);
                            self.set_flag((a & flags::Y) != 0, Flag::Y);
                        }
                        7 => {
                            // CCF
                            test_log!(self, "CCF");

                            let current_carry = self.get_flag(Flag::C);
                            self.set_flag(!current_carry, Flag::C);
                            self.set_flag(false, Flag::N);
                            // NOTE: H flag is set to previous C flag
                            self.set_flag(current_carry, Flag::H);

                            // undocummented x and y flags
                            let a = self.get_register(GPR::A);
                            self.set_flag((a & flags::X) != 0, Flag::X);
                            self.set_flag((a & flags::Y) != 0, Flag::Y);
                        }
                        _ => unreachable!("Invalid y value"), // should never happen
                    }
                }
                _ => unreachable!("Invalid z value"), // should never happen
            },
            1 => {
                if (z == 6) && (y == 6) {
                    test_log!(self, "HALT");
                    self.halted = true;
                } else {
                    test_log!(self, "LD r[y], r[z]");
                    let reg = self.table_r(y);
                    let dest = self.transform_register(reg.into(), addressing);
                    let reg = self.table_r(z);
                    let src = self.transform_register(reg.into(), addressing);
                    self.ld(dest, src); // LD r[y], r[z] (still have to transform r[y] and r[z] if IX/IY prefixed)
                }
            }

            2 => {
                // ALU[y] r[z]
                test_log!(self, "ALU[y] r[z]"); // TODO: exhaustive tests for these
                let alu_op = self.table_alu(y);
                let reg = self.table_r(z);
                let src = self.transform_register(reg.into(), addressing);
                let value = self.read_8(src);
                self.alu_op(alu_op, value);
            }
            3 => match z {
                0 => {
                    // RET cc[y]
                    test_log!(self, "RET cc[y]");
                    let condition = self.table_cc(y);
                    if self.evaluate_condition(condition) {
                        let ret_addr = self.pop();
                        self.pc = ret_addr;
                    }
                }
                1 => match (q, p) {
                    // POP & various ops
                    (false, _) => {
                        // POP rp2[p]
                        test_log!(self, "POP rp2[p]");
                        let rp = self.table_rp2(p);
                        let dest =
                            self.transform_register(AddressingMode::RegisterPair(rp), addressing);
                        let value = self.pop();
                        self.ld_16(dest, AddressingMode::ImmediateExtended(value));
                    }
                    (true, 0) => {
                        // RET
                        test_log!(self, "RET");

                        let ret_addr = self.pop();
                        self.pc = ret_addr;
                    }
                    (true, 1) => {
                        test_log!(self, "EXX");
                        self.exx();
                    }
                    (true, 2) => {
                        // JP HL/IX/IY
                        test_log!(self, "JP HL");
                        let src = self.transform_register(
                            AddressingMode::RegisterPair(RegisterPair::HL),
                            addressing,
                        );
                        let addr = self.read_16(src);
                        self.pc = addr;
                    }
                    (true, 3) => {
                        // LD SP, HL (or LD SP, IX/IY if prefixed)
                        test_log!(self, "LD SP, HL");
                        let src = self.transform_register(
                            AddressingMode::RegisterPair(RegisterPair::HL),
                            addressing,
                        );
                        self.ld_16(AddressingMode::RegisterPair(RegisterPair::SP), src);
                    }
                    _ => unreachable!("Invalid q, p values"), // should never happen
                },
                2 => {
                    // JP cc[y], nn
                    test_log!(self, "JP cc[y], nn");
                    let condition = self.table_cc(y);
                    let addr = self.fetch_word();
                    if self.evaluate_condition(condition) {
                        self.pc = addr;
                    }
                }
                3 => match y {
                    // Assorted operations
                    0 => {
                        // JP nn
                        test_log!(self, "JP nn");
                        let addr = self.fetch_word();
                        self.pc = addr;
                    }
                    1 => unreachable!("CB prefix, should be handled separately"), // should never reach this
                    2 => {
                        test_log!(self, "OUT (n), A");
                        let n = self.fetch();
                        let a = self.get_register(GPR::A);
                        let port = n as u16;
                        self.write_io(port, a);
                    }
                    3 => {
                        test_log!(self, "IN A, (n)");
                        let n = self.fetch();
                        let port = n as u16;
                        let val = self.read_io(port);
                        self.set_register(GPR::A, val);
                    }
                    4 => {
                        // EX (SP), HL (or EX (SP), IX/IY if prefixed)
                        test_log!(self, "EX (SP), HL/IX/IY");
                        let temp_l = self.memory.borrow().read(self.sp);
                        let temp_h = self.memory.borrow().read(self.sp.wrapping_add(1));
                        let register_pair = self.transform_register(
                            AddressingMode::RegisterPair(RegisterPair::HL),
                            addressing,
                        );

                        let rp = self.read_16(register_pair);
                        self.memory.borrow_mut().write(self.sp, (rp & 0xFF) as u8);
                        self.memory
                            .borrow_mut()
                            .write(self.sp.wrapping_add(1), (rp >> 8) as u8);

                        self.write_16(register_pair, ((temp_h as u16) << 8) | (temp_l as u16));
                    }
                    5 => {
                        test_log!(self, "EX DE, HL");
                        let de = self.get_register_pair(RegisterPair::DE);
                        let hl = self.get_register_pair(RegisterPair::HL);
                        self.set_register_pair(RegisterPair::DE, hl);
                        self.set_register_pair(RegisterPair::HL, de);
                    }
                    6 => {
                        test_log!(self, "DI");
                        self.iff1 = false;
                        self.iff2 = false;
                        self.iff_delay_count = 0;
                    }
                    7 => {
                        test_log!(self, "EI");
                        self.iff_delay_count = 2;
                    }
                    _ => unreachable!("Invalid y value"), // should never happen
                },
                4 => {
                    // CALL cc[y], nn
                    test_log!(self, "CALL cc[y], nn");
                    let condition = self.table_cc(y);
                    let addr = self.fetch_word();
                    if self.evaluate_condition(condition) {
                        self.push(self.pc);
                        self.pc = addr;
                    }
                }
                5 => match (q, p) {
                    // PUSH & various ops
                    (false, _) => {
                        // PUSH rp2[p]
                        test_log!(self, "PUSH rp2[p]");
                        let rp = self.table_rp2(p);
                        let src =
                            self.transform_register(AddressingMode::RegisterPair(rp), addressing);
                        let value = self.read_16(src);
                        self.push(value);
                    }
                    (true, 0) => {
                        // CALL nn
                        test_log!(self, "CALL nn");
                        let addr = self.fetch_word();
                        self.push(self.pc);
                        self.pc = addr;
                    }
                    (true, 1) => unreachable!("Shouldn't reach ED prefix here"), // should never reach this
                    (true, 2) => unreachable!("Shouldn't reach DD prefix here"), // should never reach this
                    (true, 3) => unreachable!("Shouldn't reach FD prefix here"), // should never reach this
                    _ => unreachable!("Invalid q, p values"), // should never happen
                },
                6 => {
                    // ALU[y] n
                    test_log!(self, "ALU[y] n");
                    let alu_op = self.table_alu(y);
                    let n = self.fetch();
                    self.alu_op(alu_op, n);
                }
                7 => {
                    // RST y*8
                    test_log!(self, "RST y*8");
                    let rst_addr = (y as u16) * 8;
                    test_log!(self, "{:02X}h", rst_addr);
                    self.push(self.pc);
                    self.pc = rst_addr;
                }
                _ => unreachable!("Invalid z value"), // should never happen
            },
            _ => unreachable!("Invalid x value"), // should never happen
        }
    }

    fn decode_cb(&mut self, opcode: u8) {
        test_log!(self, "decode_cb");
        let (x, y, z, _p, _q) = decode_opcode(opcode);
        match x {
            0 => {
                test_log!(self, "rot[y] r[z]");
                self.rot(y, z) // NOTE: rot[y] r[z]
            }
            1 => {
                test_log!(self, "BIT y, r[z]");
                test_log!(self, "{}", y);
                let reg = self.table_r(z);
                self.bit(y, reg.into())
            } // NOTE: BIT y, r[z]
            2 => {
                test_log!(self, "RES y, r[z]");
                test_log!(self, "{}", y);
                let reg = self.table_r(z);
                self.res(y, reg.into())
            } // NOTE: RES y, r[z]
            3 => {
                test_log!(self, "SET y, r[z]");
                test_log!(self, "{}", y);
                let reg = self.table_r(z);
                self.set(y, reg.into())
            } // NOTE: SET y, r[z]
            _ => unreachable!("Invalid x value"), // should never happen
        }
    }

    fn decode_ed(&mut self, opcode: u8) {
        decoding::decode_ed(self, opcode)
    }

    fn decode_dd(&mut self, opcode: u8) {
        //test_log!(self, "decode_dd");
        match opcode {
            0xDD | 0xED | 0xFD => {
                // Current DD is ignored (NONI), fetch next byte and process it
                test_log!(self, "DD NONI");
                let next_opcode = self.fetch();
                self.decode(next_opcode)
            }
            0xCB => {
                let displacement = self.fetch();
                let op = self.fetch();
                self.decode_ddcb_fdcb(op, IndexRegister::IX, displacement)
            }
            _ => self.decode_unprefixed(opcode, PrefixAddressing::IX),
        }
    }

    fn decode_fd(&mut self, opcode: u8) {
        //test_log!(self, "decode_fd");
        match opcode {
            0xDD | 0xED | 0xFD => {
                // Current FD is ignored (NONI), fetch next byte and process it
                test_log!(self, "FD NONI");
                let next_opcode = self.fetch();
                self.decode(next_opcode)
            }
            0xCB => {
                let displacement = self.fetch();
                let op = self.fetch();
                self.decode_ddcb_fdcb(op, IndexRegister::IY, displacement)
            }
            _ => self.decode_unprefixed(opcode, PrefixAddressing::IY),
        }
    }

    fn decode_ddcb_fdcb(&mut self, opcode: u8, indexing: IndexRegister, displacement: u8) {
        match indexing {
            IndexRegister::IX => test_log!(self, "decode_ddcb"),
            IndexRegister::IY => test_log!(self, "decode_fdcb"),
        }
        let (x, y, z, _p, _q) = decode_opcode(opcode);
        let addressing_mode = AddressingMode::Indexed(indexing, displacement as i8);

        match x {
            0 => {
                // rot[y] (IX/IY+d)
                test_log!(self, "rot[y] ({:?} + d)", indexing);
                // Fetch value from (IX/IY+d)
                let value = self.read_8(addressing_mode);
                let operation = self.table_rot(y);

                // Perform rotation
                let (result, flags) = match operation {
                    RotOperation::RLC => rot::rlc(value),
                    RotOperation::RRC => rot::rrc(value),
                    RotOperation::RL => rot::rl(value, self.get_flag(Flag::C)),
                    RotOperation::RR => rot::rr(value, self.get_flag(Flag::C)),
                    RotOperation::SLA => rot::sla(value),
                    RotOperation::SRA => rot::sra(value),
                    RotOperation::SLL => rot::sll(value),
                    RotOperation::SRL => rot::srl(value),
                };

                self.set_register(GPR::F, flags);
                self.write_8(addressing_mode, result);

                // Undocumented: copy result to register if z != 6
                if z != 6 {
                    let dest_reg = self.table_r(z);
                    self.write_8(dest_reg.into(), result);
                }
            }
            1 => {
                test_log!(self, "BIT y, ({:?} + d)", indexing);
                // Standard BIT instruction on (IX/IY+d)
                // z is ignored (typically 6, but others behave same)
                self.bit(y, addressing_mode);
            }
            2 => {
                test_log!(self, "RES y, ({:?} + d)", indexing);
                // Perform RES on (IX/IY+d)
                let value = self.read_8(addressing_mode);
                let result = res(value, y);
                self.write_8(addressing_mode, result);

                // Undocumented: copy result to register if z != 6
                if z != 6 {
                    let dest_reg = self.table_r(z);
                    self.write_8(dest_reg.into(), result);
                }
            }
            3 => {
                test_log!(self, "SET y, ({:?} + d)", indexing);
                // Perform SET on (IX/IY+d)
                let value = self.read_8(addressing_mode);
                let result = set(value, y);
                self.write_8(addressing_mode, result);

                // Undocumented: copy result to register if z != 6
                if z != 6 {
                    let dest_reg = self.table_r(z);
                    self.write_8(dest_reg.into(), result);
                }
            }
            _ => unreachable!("Invalid x value"), // should never happen
        }
    }

    fn decode(&mut self, opcode: u8) {
        match opcode {
            0xCB => {
                let op = self.fetch();
                self.decode_cb(op)
            }
            0xED => {
                let op = self.fetch();
                self.decode_ed(op)
            }

            0xDD => {
                let op = self.fetch();
                self.decode_dd(op)
            }
            0xFD => {
                let op = self.fetch();
                self.decode_fd(op)
            }

            _ => self.decode_unprefixed(opcode, PrefixAddressing::HL),
        }
    }

    // register ops

    fn ld(&mut self, dest: AddressingMode, src: AddressingMode) {
        let value = self.read_8(src);
        self.write_8(dest, value);
    }

    fn ld_16(&mut self, dest: AddressingMode, src: AddressingMode) {
        let value = self.read_16(src);
        self.write_16(dest, value);
    }

    fn table_rot(&mut self, y: u8) -> RotOperation {
        match RotOperation::try_from(y) {
            Ok(op) => {
                test_log!(self, "{}", op);
                op
            }
            Err(e) => panic!("{}", e),
        }
    }

    fn rot(&mut self, y: u8, z: u8) {
        let operation = self.table_rot(y);
        let reg = self.table_r(z);
        let value = self.read_8(reg.into());

        let (result, flags) = match operation {
            RotOperation::RLC => rot::rlc(value),
            RotOperation::RRC => rot::rrc(value),
            RotOperation::RL => rot::rl(value, self.get_flag(Flag::C)),
            RotOperation::RR => rot::rr(value, self.get_flag(Flag::C)),
            RotOperation::SLA => rot::sla(value),
            RotOperation::SRA => rot::sra(value),
            RotOperation::SLL => rot::sll(value),
            RotOperation::SRL => rot::srl(value),
        };
        self.set_register(GPR::A, flags); // Wait, flags in F? Yes. set_register(GPR::F, flags) likely.
        self.set_register(GPR::F, flags);
        self.write_8(reg.into(), result);
    }

    fn bit(&mut self, y: u8, reg: AddressingMode) {
        let value = self.read_8(reg);

        let prev_c = self.get_flag(Flag::C);
        self.set_register(GPR::F, bit(value, y));

        self.set_flag(prev_c, Flag::C); // C flag is not affected
    }

    fn res(&mut self, y: u8, reg: AddressingMode) {
        let value = self.read_8(reg);
        let result = res(value, y);
        self.write_8(reg, result);
    }

    fn set(&mut self, y: u8, reg: AddressingMode) {
        let value = self.read_8(reg);
        let result = set(value, y);
        self.write_8(reg, result);
    }

    fn daa(&mut self) {
        let n = self.get_flag(Flag::N);
        let h = self.get_flag(Flag::H);
        let c = self.get_flag(Flag::C);
        let mut a = self.get_register(GPR::A);
        let adjust_low = h || a & 0x0f > 9;
        let adjust_high = c || a > 0x99;
        let adjustment = match (adjust_low, adjust_high, n) {
            (false, false, _) => 0,
            (true, false, false) => 0x06,
            (false, true, false) => 0x60,
            (true, true, false) => 0x66,
            (true, false, true) => 0xFA,
            (false, true, true) => 0xA0,
            (true, true, true) => 0x9A,
        };

        if adjust_high { self.set_flag(true, Flag::C); }
        self.set_flag(match (n, h) {
            (true, false) => false,
            (true, true) => a & 0x0f < 6,
            (false, _) => a & 0x0f >= 0x0a,
        }, Flag::H);
        a = a.wrapping_add(adjustment);

        self.set_register(GPR::A, a);

        self.set_flag((a & flags::SIGN) == flags::SIGN, Flag::S);
        self.set_flag(a == 0, Flag::Z);
        self.set_flag(a.count_ones().is_multiple_of(2), Flag::PV);
        self.set_flag((a & flags::X) == flags::X, Flag::X);
        self.set_flag((a & flags::Y) == flags::Y, Flag::Y);
    }
}

impl SynchronousComponent for Z80A {
    fn tick(&mut self) {
        self.poll_nmi();
        if self.handle_pending_interrupts() || self.halted { return; }

        let opcode = self.fetch();
        self.decode(opcode);

        // Handle Delayed Interrupt Enable (EI)
        if self.iff_delay_count > 0 {
            self.iff_delay_count -= 1;
            if self.iff_delay_count == 0 {
                self.iff1 = true;
                self.iff2 = true;
            }
        }
    }
}

impl Z80A {
    fn poll_nmi(&mut self) {
        let mut nmi_active = false;
        for dev in &self.devices {
            if dev.borrow().poll_nmi() {
                nmi_active = true;
                break;
            }
        }

        if nmi_active && !self.nmi_last_state {
            self.nmi_pending = true;
        }
        self.nmi_last_state = nmi_active;
    }

    fn handle_pending_interrupts(&mut self) -> bool {
        if self.nmi_pending {
            self.nmi_pending = false;
            self.halted = false; // Wake from HALT
            self.handle_nmi();
            return true;
        }

        // Handle Maskable Interrupts (Level triggered)
        // Check if interrupts enabled and pending.
        // Important: If we just executed EI (iff_delay_count > 0), iff1 is not yet true.
        // Also check if delay is active (interrupts are inhibited during the instruction after EI)
        if self.iff1 && self.is_int_line_active() && self.iff_delay_count == 0 {
            self.halted = false; // Wake from HALT
            self.handle_int();
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests;
