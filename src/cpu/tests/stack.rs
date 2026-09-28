use rstest::rstest;

enum StackOperation {
    Push(u16),
    Pop(u16), // we well compare the popped value to this
}

#[rstest]
#[case(
    0x1000,
    &[], // Start with empty memory
    &[StackOperation::Push(0x1234)],
    0x0FFE // SP should move down by 2
)]
#[case::wraparound(
    0xFFFF,
    &[(0xFFFF, 0x34), (0x0000, 0x12)], // Pre-fill the wrap points
    &[StackOperation::Pop(0x1234)],
    0x0001 // Final SP after two increments
)]
#[case(
    0x0001,
    &[],
    &[StackOperation::Push(0xABCD)],
    0xFFFF
)]
#[case(
    0x2000,
    &[(0x2000, 0x78), (0x2001, 0x56)], // Low byte at SP, High byte at SP+1
    &[
        StackOperation::Pop(0x5678),
        StackOperation::Push(0x9ABC),
        StackOperation::Pop(0x9ABC)
    ],
    0x2002
)]
fn test_stack(
    #[case] initial_sp: u16,        // Where SP starts before operations
    #[case] mem_init: &[(u16, u8)], // List of (Address, Value) to pre-load
    #[case] operations: &[StackOperation],
    #[case] expected_final_sp: u16,
) {
    let mut cpu = crate::cpu::tests::setup_cpu();

    // 1. Setup Memory (Pre-load existing stack data if any)
    for &(addr, val) in mem_init {
        cpu.memory.write(addr, val);
    }

    // 2. Set SP
    cpu.sp = initial_sp;

    // 3. Run Operations
    for operation in operations {
        match operation {
            StackOperation::Push(value) => {
                cpu.push(*value);
            }
            StackOperation::Pop(expected_value) => {
                let popped_value = cpu.pop();
                assert_eq!(
                    popped_value, *expected_value,
                    "Popped value mismatch! Expected: 0x{:04X}, Got: 0x{:04X}",
                    expected_value, popped_value
                );
            }
        }
    }

    // 4. Final SP Check
    assert_eq!(
        cpu.sp, expected_final_sp,
        "Final SP mismatch! Expected: 0x{:04X}, Got: 0x{:04X}",
        expected_final_sp, cpu.sp
    );
}
