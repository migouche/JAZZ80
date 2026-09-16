use super::*;

fn write_argument(dos: &mut VirtualDOS, argument: &str) {
    for byte in argument.bytes() {
        assert!(dos.write_out(0x20, byte));
    }
}

#[test]
fn virtual_dos_routes_commands_and_preserves_handled_results() {
    let mut dos = VirtualDOS::new(0x20);

    assert!(!dos.write_out(0x21 + 0x10, 0));
    write_argument(&mut dos, "docs");
    assert!(dos.write_out(0x21, 1));
    assert_eq!(dos.read_in(0x22), Some(0));

    assert!(dos.write_out(0x21, 0xFF));
    assert_eq!(dos.read_in(0x22), Some(0));

    assert!(dos.write_out(0x21, 0));
    write_argument(&mut dos, "docs");
    assert!(dos.write_out(0x21, 2));
    assert_eq!(dos.cwd, "/docs");

    assert!(dos.write_out(0x21, 0));
    write_argument(&mut dos, "missing");
    assert!(dos.write_out(0x21, 2));
    assert_eq!(dos.read_in(0x22), Some(1));
}

#[test]
fn virtual_dos_file_command_buffers_data_and_clears_status_after_reads() {
    let mut dos = VirtualDOS::new(0x20);
    dos.fs
        .insert("/readme".to_string(), Inode::File(vec![b'O', b'K']));

    write_argument(&mut dos, "readme");
    assert!(dos.write_out(0x21, 4));
    assert_eq!(dos.read_in(0x22), Some(2));
    assert_eq!(dos.read_in(0x20), Some(b'O'));
    assert_eq!(dos.read_in(0x22), Some(2));
    assert_eq!(dos.read_in(0x20), Some(b'K'));
    assert_eq!(dos.read_in(0x22), Some(0));

    assert!(dos.write_out(0x21, 0));
    assert_eq!(dos.read_in(0x22), Some(0));
}
