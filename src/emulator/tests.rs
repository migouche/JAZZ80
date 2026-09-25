use super::*;

fn program(mut cpu: Z80A, bytes: &[u8]) -> Z80A {
    for (i, b) in bytes.iter().enumerate() {
        cpu.memory.write(i as u16, *b);
    }
    cpu
}

#[test]
fn snapshot_matches_cpu_state() {
    let mut machine = Machine::new();
    machine.cpu.set_pc(0x1234);
    machine.cpu.set_sp(0xABCD);
    machine.cpu.set_ix(0x1111);
    machine.cpu.set_iy(0x2222);
    machine.cpu.set_register(GPR::A, 0x42);
    machine.cpu.set_register(GPR::H, 0x99);

    let snap = machine.snapshot();
    assert_eq!(snap.pc, 0x1234);
    assert_eq!(snap.sp, 0xABCD);
    assert_eq!(snap.ix, 0x1111);
    assert_eq!(snap.iy, 0x2222);
    assert_eq!(snap.regs[0], 0x42);
    assert_eq!(snap.regs[6], 0x99);
    assert_eq!(snap.memory.len(), 0x10000);
}

#[test]
fn run_slice_zero_budget_executes_nothing() {
    let mut machine = Machine::new();
    let breakpoints = HashSet::new();
    let result = machine.run_slice(0, &breakpoints);
    assert_eq!(result, SliceResult::BudgetExhausted { cycles: 0 });
    assert_eq!(machine.cpu.get_pc(), 0);
}

#[test]
fn run_slice_executes_up_to_max_ticks() {
    let mut machine = Machine::new();
    let mut bytes = vec![0x00u8; 16];
    bytes.push(0x76);
    machine.cpu = program(machine.cpu, &bytes);

    let breakpoints = HashSet::new();
    let result = machine.run_slice(3, &breakpoints);
    assert_eq!(result, SliceResult::BudgetExhausted { cycles: 3 });
    assert_eq!(machine.cpu.get_pc(), 3);

    let result = machine.run_slice(100, &breakpoints);
    assert_eq!(result, SliceResult::Halted { cycles: 14 });
    assert_eq!(machine.cpu.get_pc(), 17);
}

#[test]
fn run_slice_reports_cycles_across_repeated_slices() {
    let mut machine = Machine::new();
    machine.cpu = program(machine.cpu, &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    let breakpoints = HashSet::new();

    let mut total = 0;
    for _ in 0..3 {
        if let SliceResult::BudgetExhausted { cycles } = machine.run_slice(2, &breakpoints) {
            total += cycles;
        }
    }
    assert_eq!(total, 6);
    assert_eq!(machine.cpu.get_pc(), 6);
}

#[test]
fn run_slice_stops_at_breakpoint_before_execution() {
    let mut machine = Machine::new();
    machine.cpu = program(machine.cpu, &[0x3E, 0x42, 0x3E, 0x99]);

    let mut breakpoints = HashSet::new();
    breakpoints.insert(0x0002);

    let result = machine.run_slice(100, &breakpoints);
    assert_eq!(result, SliceResult::HitBreakpoint { cycles: 1 });
    assert_eq!(machine.cpu.get_pc(), 0x0002);
    assert_eq!(machine.cpu.get_register(GPR::A), 0x42);

    let result = machine.run_slice(0, &breakpoints);
    assert_eq!(result, SliceResult::HitBreakpoint { cycles: 0 });
    assert_eq!(machine.cpu.get_pc(), 0x0002);
    assert_eq!(machine.cpu.get_register(GPR::A), 0x42);
}

#[test]
fn run_slice_returns_halted_while_cpu_is_halted() {
    let mut machine = Machine::new();
    machine.cpu = program(machine.cpu, &[0x76]);

    let breakpoints = HashSet::new();
    let result = machine.run_slice(1, &breakpoints);
    assert_eq!(result, SliceResult::Halted { cycles: 1 });
    assert!(machine.cpu.is_halted());
}

#[test]
fn run_slice_wakes_halted_cpu_on_nmi() {
    let mut machine = Machine::new();
    machine.cpu = program(machine.cpu, &[0x76]);
    machine.cpu.memory.write(0x0066, 0xC9);

    let breakpoints = HashSet::new();
    assert_eq!(
        machine.run_slice(10, &breakpoints),
        SliceResult::Halted { cycles: 1 }
    );

    machine.cpu.nmi();
    let result = machine.run_slice(4, &breakpoints);
    assert!(!matches!(result, SliceResult::Halted { .. }));
    assert!(!machine.cpu.is_halted());
}

#[test]
fn run_slice_wakes_halted_cpu_on_maskable_interrupt() {
    let mut machine = Machine::new();
    machine.cpu = program(machine.cpu, &[0xFB, 0x76]);
    machine.cpu.memory.write(0x0038, 0xC9);

    let breakpoints = HashSet::new();
    assert_eq!(
        machine.run_slice(10, &breakpoints),
        SliceResult::Halted { cycles: 2 }
    );

    machine.cpu.set_interrupt(true);
    let mut woke = false;
    for _ in 0..4 {
        let result = machine.run_slice(10, &breakpoints);
        if !matches!(result, SliceResult::Halted { .. }) && !machine.cpu.is_halted() {
            woke = true;
            break;
        }
    }
    assert!(woke, "maskable interrupt did not wake the halted CPU");
}

#[test]
fn memory_helpers_expose_the_same_backing_memory() {
    let mut machine = Machine::new();
    machine.memory_mut().write(0x1000, 0xAB);
    assert_eq!(machine.cpu.memory.read(0x1000), 0xAB);
}

#[test]
fn snapshot_includes_register_view() {
    let mut machine = Machine::new();
    machine.cpu.set_register(GPR::D, 0x56);
    machine.cpu.set_register(GPR::E, 0x66);
    let snap = machine.snapshot();
    assert_eq!(snap.regs[4], 0x56);
    assert_eq!(snap.regs[5], 0x66);
}
