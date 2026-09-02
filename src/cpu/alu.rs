use crate::cpu::flags;

pub mod rot {
    use crate::cpu::flags;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum RotOperation {
        RLC,
        RRC,
        RL,
        RR,
        SLA,
        SRA,
        SLL,
        SRL,
    }

    impl TryFrom<u8> for RotOperation {
        type Error = String;
        fn try_from(value: u8) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(RotOperation::RLC),
                1 => Ok(RotOperation::RRC),
                2 => Ok(RotOperation::RL),
                3 => Ok(RotOperation::RR),
                4 => Ok(RotOperation::SLA),
                5 => Ok(RotOperation::SRA),
                6 => Ok(RotOperation::SLL),
                7 => Ok(RotOperation::SRL),
                _ => Err(format!("Invalid RotOperation value: {}", value)),
            }
        }
    }

    impl std::fmt::Display for RotOperation {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                RotOperation::RLC => write!(f, "RLC"),
                RotOperation::RRC => write!(f, "RRC"),
                RotOperation::RL => write!(f, "RL"),
                RotOperation::RR => write!(f, "RR"),
                RotOperation::SLA => write!(f, "SLA"),
                RotOperation::SRA => write!(f, "SRA"),
                RotOperation::SLL => write!(f, "SLL"),
                RotOperation::SRL => write!(f, "SRL"),
            }
        }
    }

    impl std::str::FromStr for RotOperation {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "RLC" => Ok(RotOperation::RLC),
                "RRC" => Ok(RotOperation::RRC),
                "RL" => Ok(RotOperation::RL),
                "RR" => Ok(RotOperation::RR),
                "SLA" => Ok(RotOperation::SLA),
                "SRA" => Ok(RotOperation::SRA),
                "SLL" => Ok(RotOperation::SLL),
                "SRL" => Ok(RotOperation::SRL),
                _ => Err(format!("Invalid rotation operation: {}", s)),
            }
        }
    }

    fn calculate_flags(result: u8, carry: bool) -> u8 {
        let z = result == 0;
        let s = (result & 0x80) != 0;
        let pv = result.count_ones().is_multiple_of(2);
        let x = (result & 0x08) != 0;
        let y = (result & 0x20) != 0;
        // Carry is passed as parameter

        (if carry { flags::CARRY } else { 0x00 })
            | (if pv { flags::PARITY_OVERFLOW } else { 0x00 })
            | (if x { flags::X } else { 0x00 })
            | (if y { flags::Y } else { 0x00 })
            | (if z { flags::ZERO } else { 0x00 })
            | (if s { flags::SIGN } else { 0x00 })
    }

    pub fn rlc(value: u8) -> (u8, u8) {
        let carry = (value & 0x80) != 0;
        let result = (value << 1) | if carry { 1 } else { 0 };
        let flags = calculate_flags(result, carry);
        (result, flags)
    }

    pub fn rrc(value: u8) -> (u8, u8) {
        let carry = (value & 0x01) != 0;
        let result = (value >> 1) | if carry { 0x80 } else { 0 };
        let flags = calculate_flags(result, carry);
        (result, flags)
    }

    pub fn rl(value: u8, carry_in: bool) -> (u8, u8) {
        let carry = (value & 0x80) != 0;
        let result = (value << 1) | if carry_in { 1 } else { 0 };
        let flags = calculate_flags(result, carry);
        (result, flags)
    }

    pub fn rr(value: u8, carry_in: bool) -> (u8, u8) {
        let carry = (value & 0x01) != 0;
        let result = (value >> 1) | if carry_in { 0x80 } else { 0 };
        let flags = calculate_flags(result, carry);
        (result, flags)
    }

    pub fn sla(value: u8) -> (u8, u8) {
        let carry = (value & 0x80) != 0;
        let result = value << 1;
        let flags = calculate_flags(result, carry);
        (result, flags)
    }

    pub fn sra(value: u8) -> (u8, u8) {
        let carry = (value & 0x01) != 0;
        let msb = value & 0x80;
        let result = (value >> 1) | msb;
        let flags = calculate_flags(result, carry);
        (result, flags)
    }

    pub fn sll(value: u8) -> (u8, u8) {
        let carry = (value & 0x80) != 0;
        let result = (value << 1) | 0x01;
        let flags = calculate_flags(result, carry);
        (result, flags)
    }

    pub fn srl(value: u8) -> (u8, u8) {
        let carry = (value & 0x01) != 0;
        let result = value >> 1;
        let flags = calculate_flags(result, carry);
        (result, flags)
    }
}

pub fn bit(value: u8, bit_position: u8) -> u8 {
    let bit: bool = (value & (1 << bit_position)) != 0;

    let z = !bit;
    let s = bit_position == 7 && bit;
    let pv = z;
    let h = true;
    let n = false;
    let c = false; // unaffected, will use mask later to ignore
    let f3 = bit_position == 3 && bit;
    let f5 = bit_position == 5 && bit;

    (if c { flags::CARRY } else { 0x00 })
        | (if n { flags::ADD_SUB } else { 0x00 })
        | (if pv { flags::PARITY_OVERFLOW } else { 0x00 })
        | (if f3 { flags::X } else { 0x00 })
        | (if h { flags::HALF_CARRY } else { 0x00 })
        | (if f5 { flags::Y } else { 0x00 })
        | (if z { flags::ZERO } else { 0x00 })
        | (if s { flags::SIGN } else { 0x00 })
}

pub fn res(value: u8, bit_position: u8) -> u8 {
    value & !(1 << bit_position)
}

pub fn set(value: u8, bit_position: u8) -> u8 {
    value | (1 << bit_position)
}

pub mod alu_op {
    use crate::cpu::flags;

    pub fn add(a: u8, b: u8, carry_in: bool) -> (u8, u8) {
        let (intermediate_sum, carry1) = a.overflowing_add(b);
        let (final_sum, carry2) = intermediate_sum.overflowing_add(if carry_in { 1 } else { 0 });
        let carry_out = carry1 || carry2;

        let h = ((a ^ b ^ final_sum) & 0x10) != 0;
        let z = final_sum == 0;
        let s = (final_sum & 0x80) != 0;
        let pv = ((a ^ final_sum) & (b ^ final_sum) & 0x80) != 0;
        let x = (final_sum & 0x08) != 0;
        let y = (final_sum & 0x20) != 0;

        let flags = (if carry_out { flags::CARRY } else { 0x00 })
            | (if pv { flags::PARITY_OVERFLOW } else { 0x00 })
            | (if x { flags::X } else { 0x00 })
            | (if y { flags::Y } else { 0x00 })
            | (if h { flags::HALF_CARRY } else { 0x00 })
            | (if z { flags::ZERO } else { 0x00 })
            | (if s { flags::SIGN } else { 0x00 });

        (final_sum, flags)
    }

    pub fn sub(a: u8, b: u8, carry_in: bool) -> (u8, u8) {
        let (intermediate_diff, borrow1) = a.overflowing_sub(b);
        let (final_diff, borrow2) = intermediate_diff.overflowing_sub(if carry_in { 1 } else { 0 });
        let carry_out = borrow1 || borrow2;

        let h = (a & 0x0F) < ((b & 0x0F) + if carry_in { 1 } else { 0 });
        let z = final_diff == 0;
        let s = (final_diff & 0x80) != 0;
        let pv = ((a ^ b) & (a ^ final_diff) & 0x80) != 0;
        let x = (final_diff & 0x08) != 0;
        let y = (final_diff & 0x20) != 0;
        let n = true;

        let flags = (if carry_out { flags::CARRY } else { 0x00 })
            | (if n { flags::ADD_SUB } else { 0x00 })
            | (if pv { flags::PARITY_OVERFLOW } else { 0x00 })
            | (if x { flags::X } else { 0x00 })
            | (if h { flags::HALF_CARRY } else { 0x00 })
            | (if y { flags::Y } else { 0x00 })
            | (if z { flags::ZERO } else { 0x00 })
            | (if s { flags::SIGN } else { 0x00 });

        (final_diff, flags)
    }

    pub fn and(a: u8, b: u8) -> (u8, u8) {
        let result = a & b;
        let s = (result & 0x80) != 0;
        let z = result == 0;
        let h = true;
        let pv = result.count_ones().is_multiple_of(2);
        let n = false;
        let c = false;
        let x = (result & 0x08) != 0;
        let y = (result & 0x20) != 0;

        let flags = (if c { flags::CARRY } else { 0x00 })
            | (if n { flags::ADD_SUB } else { 0x00 })
            | (if pv { flags::PARITY_OVERFLOW } else { 0x00 })
            | (if x { flags::X } else { 0x00 })
            | (if h { flags::HALF_CARRY } else { 0x00 })
            | (if y { flags::Y } else { 0x00 })
            | (if z { flags::ZERO } else { 0x00 })
            | (if s { flags::SIGN } else { 0x00 });
        (result, flags)
    }

    pub fn or(a: u8, b: u8) -> (u8, u8) {
        let result = a | b;
        let s = (result & 0x80) != 0;
        let z = result == 0;
        let h = false;
        let pv = result.count_ones().is_multiple_of(2);
        let n = false;
        let c = false;
        let x = (result & 0x08) != 0;
        let y = (result & 0x20) != 0;

        let flags = (if c { flags::CARRY } else { 0x00 })
            | (if n { flags::ADD_SUB } else { 0x00 })
            | (if pv { flags::PARITY_OVERFLOW } else { 0x00 })
            | (if x { flags::X } else { 0x00 })
            | (if h { flags::HALF_CARRY } else { 0x00 })
            | (if y { flags::Y } else { 0x00 })
            | (if z { flags::ZERO } else { 0x00 })
            | (if s { flags::SIGN } else { 0x00 });
        (result, flags)
    }

    pub fn xor(a: u8, b: u8) -> (u8, u8) {
        let result = a ^ b;
        let s = (result & 0x80) != 0;
        let z = result == 0;
        let h = false;
        let pv = result.count_ones().is_multiple_of(2);
        let n = false;
        let c = false;
        let x = (result & 0x08) != 0;
        let y = (result & 0x20) != 0;

        let flags = (if c { flags::CARRY } else { 0x00 })
            | (if n { flags::ADD_SUB } else { 0x00 })
            | (if pv { flags::PARITY_OVERFLOW } else { 0x00 })
            | (if x { flags::X } else { 0x00 })
            | (if h { flags::HALF_CARRY } else { 0x00 })
            | (if y { flags::Y } else { 0x00 })
            | (if z { flags::ZERO } else { 0x00 })
            | (if s { flags::SIGN } else { 0x00 });
        (result, flags)
    }
}

pub fn inc(value: u8) -> (u8, u8) {
    let result = value.wrapping_add(1);
    let z = result == 0;
    let s = (result & 0x80) != 0;
    let h = (value & 0x0F) == 0x0F;
    let pv = value == 0x7F;
    let n = false;
    let x = (result & 0x08) != 0;
    let y = (result & 0x20) != 0;

    // Note: C flag is not affected by INC, caller must preserve it.
    let flags = (if n { flags::ADD_SUB } else { 0x00 })
        | (if pv { flags::PARITY_OVERFLOW } else { 0x00 })
        | (if x { flags::X } else { 0x00 })
        | (if h { flags::HALF_CARRY } else { 0x00 })
        | (if y { flags::Y } else { 0x00 })
        | (if z { flags::ZERO } else { 0x00 })
        | (if s { flags::SIGN } else { 0x00 });

    (result, flags)
}

pub fn add_16(a: u16, b: u16, current_flags: u8, use_carry: bool) -> (u16, u8) {
    let carry_in = if use_carry && (current_flags & flags::CARRY) != 0 {
        1
    } else {
        0
    };

    let (res_1, c1) = a.overflowing_add(b);
    let (result, c2) = res_1.overflowing_add(carry_in);
    let c = c1 || c2;

    // Half Carry is carry from bit 11 to 12
    let h = ((a & 0x0FFF) + (b & 0x0FFF) + carry_in) > 0x0FFF;

    let n = false;
    // X and Y are ALWAYS bits 11 and 13 of the result
    let x = (result & 0x0800) != 0;
    let y = (result & 0x2000) != 0;

    let (s, z, pv) = if use_carry {
        let s = (result & 0x8000) != 0;
        let z = result == 0;
        // Overflow: operands same sign, result different sign
        let pv = ((a ^ result) & (b ^ result) & 0x8000) != 0;
        (s, z, pv)
    } else {
        (
            (current_flags & flags::SIGN) != 0,
            (current_flags & flags::ZERO) != 0,
            (current_flags & flags::PARITY_OVERFLOW) != 0,
        )
    };

    let flags = (if c { flags::CARRY } else { 0x00 })
        | (if n { flags::ADD_SUB } else { 0x00 })
        | (if pv { flags::PARITY_OVERFLOW } else { 0x00 })
        | (if x { flags::X } else { 0x00 })
        | (if h { flags::HALF_CARRY } else { 0x00 })
        | (if y { flags::Y } else { 0x00 })
        | (if z { flags::ZERO } else { 0x00 })
        | (if s { flags::SIGN } else { 0x00 });

    (result, flags)
}

pub fn sub_16(a: u16, b: u16, current_flags: u8, use_carry: bool) -> (u16, u8) {
    let carry_in = if use_carry && (current_flags & flags::CARRY) != 0 {
        1
    } else {
        0
    };

    let (res_1, b1) = a.overflowing_sub(b);
    let (result, b2) = res_1.overflowing_sub(carry_in);
    let c = b1 || b2;

    // Half Borrow is borrow from bit 12
    let h = (a & 0x0FFF) < ((b & 0x0FFF) + carry_in);

    let n = true;
    let x = (result & 0x0800) != 0;
    let y = (result & 0x2000) != 0;

    let (s, z, pv) = if use_carry {
        let s = (result & 0x8000) != 0;
        let z = result == 0;
        // Overflow: operands different sign, result sign differs from 'a'
        let pv = ((a ^ b) & (a ^ result) & 0x8000) != 0;
        (s, z, pv)
    } else {
        (
            (current_flags & flags::SIGN) != 0,
            (current_flags & flags::ZERO) != 0,
            (current_flags & flags::PARITY_OVERFLOW) != 0,
        )
    };

    let flags = (if c { flags::CARRY } else { 0x00 })
        | (if n { flags::ADD_SUB } else { 0x00 })
        | (if pv { flags::PARITY_OVERFLOW } else { 0x00 })
        | (if x { flags::X } else { 0x00 })
        | (if h { flags::HALF_CARRY } else { 0x00 })
        | (if y { flags::Y } else { 0x00 })
        | (if z { flags::ZERO } else { 0x00 })
        | (if s { flags::SIGN } else { 0x00 });

    (result, flags)
}

pub fn dec(value: u8) -> (u8, u8) {
    let result = value.wrapping_sub(1);
    let z = result == 0;
    let s = (result & 0x80) != 0;
    let h = (value & 0x0F) == 0x00;
    let pv = value == 0x80;
    let n = true;
    let x = (result & 0x08) != 0;
    let y = (result & 0x20) != 0;

    // Note: C flag is not affected by DEC, caller must preserve it.
    let flags = (if n { flags::ADD_SUB } else { 0x00 })
        | (if pv { flags::PARITY_OVERFLOW } else { 0x00 })
        | (if x { flags::X } else { 0x00 })
        | (if h { flags::HALF_CARRY } else { 0x00 })
        | (if y { flags::Y } else { 0x00 })
        | (if z { flags::ZERO } else { 0x00 })
        | (if s { flags::SIGN } else { 0x00 });

    (result, flags)
}
