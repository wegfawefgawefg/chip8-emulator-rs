use chip8_emulator_rs::{run_emulator_headless, ORIGINAL_QUIRKS};

#[test]
fn headless_stops_on_exit_opcode() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), [0x00, 0xFD]).unwrap();

    let state = run_emulator_headless(ORIGINAL_QUIRKS, tmp.path(), 10, 700).unwrap();

    assert!(state.exited);
}

#[test]
fn white_dot_rom_draws_pixels() {
    let rom_path =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("roms/white_dot_wasd.ch8");
    let state = run_emulator_headless(ORIGINAL_QUIRKS, &rom_path, 64, 700).unwrap();

    assert!(
        state.screen_buffer.iter().any(|pixel| *pixel == 1),
        "expected white_dot_wasd ROM to draw at least one lit pixel"
    );
}
