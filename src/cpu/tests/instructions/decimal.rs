use crate::traits::SynchronousComponent;

const DAA_OPCODE: u8 = 0x27;

fn parse_line_to_test_case(line: &str) -> Option<(u8, bool, bool, bool, u8, bool, bool, bool)> {
    let mut parts = line.split_whitespace();

    if parts.next()? != "N" {
        return None;
    }
    let n_flag = parts.next()? == "1";

    if parts.next()? != "C" {
        return None;
    }
    let c_flag = parts.next()? == "1";

    if parts.next()? != "H" {
        return None;
    }
    let h_flag = parts.next()? == "1";

    let initial_a = u8::from_str_radix(parts.next()?, 16).ok()?;

    if parts.next()? != "N" {
        return None;
    }
    let expected_n_flag = parts.next()? == "1";

    if parts.next()? != "C" {
        return None;
    }
    let expected_c_flag = parts.next()? == "1";

    if parts.next()? != "H" {
        return None;
    }
    let expected_h_flag = parts.next()? == "1";

    let expected_a = u8::from_str_radix(parts.next()?, 16).ok()?;

    Some((
        initial_a,
        n_flag,
        c_flag,
        h_flag,
        expected_a,
        expected_n_flag,
        expected_c_flag,
        expected_h_flag,
    ))
}

#[test]
fn test_daa_file() {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("test_files")
        .join("daa.txt");
    println!("Reading test file: {:?}", file);
    // read every line from the file and parse it to a test case

    let contents = std::fs::read_to_string(file).expect("Failed to read test file");
    for line in contents.lines() {
        if let Some((
            initial_a,
            n_flag,
            c_flag,
            h_flag,
            expected_a,
            expected_n,
            expected_c,
            expected_h,
        )) = parse_line_to_test_case(line)
        {
            test_daa_instruction(
                initial_a, n_flag, c_flag, h_flag, expected_a, expected_n, expected_c, expected_h,
            );
        } // ignore if fails
    }
}

fn test_daa_instruction(
    initial_a: u8,
    n_flag: bool,
    c_flag: bool,
    h_flag: bool,
    expected_a: u8,
    expected_n: bool,
    expected_c: bool,
    expected_h: bool,
) {
    use crate::cpu::{Flag, GPR, tests::setup_cpu};

    let mut cpu = setup_cpu();
    cpu.pc = 0x1000;
    cpu.memory.borrow_mut().write(cpu.pc, DAA_OPCODE);

    // Setup initial state
    cpu.set_register(GPR::A, initial_a);
    cpu.set_flag(n_flag, Flag::N);
    cpu.set_flag(h_flag, Flag::H);
    cpu.set_flag(c_flag, Flag::C);

    // Execute DAA
    cpu.tick();

    // Assert results
    let result_a = cpu.get_register(GPR::A);

    assert_eq!(
        cpu.test_callback.0.iter().rev().collect::<Vec<_>>(),
        ["decode_unprefixed", "DAA"],
        "Instruction execution trace mismatch, got: {:?}",
        cpu.test_callback.0.iter().rev().collect::<Vec<_>>()
    );

    assert_eq!(
        result_a, expected_a,
        "Accumulator mismatch! Input: {:#04X}, N: {}, H: {}, C: {}",
        initial_a, n_flag, h_flag, c_flag
    );

    assert_eq!(
        n_flag, expected_n,
        "N flag should not change: Input N: {}, Expected N: {}",
        n_flag, expected_n
    );

    assert_eq!(cpu.get_flag(Flag::C), expected_c, "C flag mismatch");
    assert_eq!(cpu.get_flag(Flag::H), expected_h, "H flag mismatch");
    assert_eq!(cpu.get_flag(Flag::N), expected_n, "N flag mismatch");

    // Check standard flags
    assert_eq!(cpu.get_flag(Flag::Z), result_a == 0, "Z flag mismatch");
    assert_eq!(
        cpu.get_flag(Flag::S),
        (result_a & 0x80) != 0,
        "S flag mismatch"
    );
}
