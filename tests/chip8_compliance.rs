use chip8_emulator_rs::{
    create_state, execute_cycle, execute_opcode, tick_timers, MODERN_QUIRKS, ORIGINAL_QUIRKS,
};

#[test]
fn ex9e_skips_when_key_pressed() {
    let mut state = create_state(None).unwrap();
    state.registers[1] = 0xA;
    state.key_inputs[0xA] = 1;
    let start_pc = state.pc;

    execute_opcode(&mut state, 0xE19E, ORIGINAL_QUIRKS).unwrap();

    assert_eq!(state.pc, start_pc + 2);
}

#[test]
fn exa1_skips_when_key_not_pressed() {
    let mut state = create_state(None).unwrap();
    state.registers[1] = 0xA;
    state.key_inputs[0xA] = 0;
    let start_pc = state.pc;

    execute_opcode(&mut state, 0xE1A1, ORIGINAL_QUIRKS).unwrap();

    assert_eq!(state.pc, start_pc + 2);
}

#[test]
fn fx33_stores_bcd_digits() {
    let mut state = create_state(None).unwrap();
    state.registers[2] = 231;
    state.index = 0x300;

    execute_opcode(&mut state, 0xF233, ORIGINAL_QUIRKS).unwrap();

    assert_eq!(state.memory[0x300..0x303], [2, 3, 1]);
}

#[test]
fn fx65_reads_registers_and_increments_i() {
    let mut state = create_state(None).unwrap();
    state.index = 0x300;
    state.memory[0x300..0x303].copy_from_slice(&[0xAA, 0xBB, 0xCC]);

    execute_opcode(&mut state, 0xF265, ORIGINAL_QUIRKS).unwrap();

    assert_eq!(state.registers[0..3], [0xAA, 0xBB, 0xCC]);
    assert_eq!(state.index, 0x303);
}

#[test]
fn seven_xnn_wraps_at_8_bits() {
    let mut state = create_state(None).unwrap();
    state.registers[0] = 0xFF;

    execute_opcode(&mut state, 0x7002, ORIGINAL_QUIRKS).unwrap();

    assert_eq!(state.registers[0], 0x01);
}

#[test]
fn eight_xy6_uses_vy_as_source() {
    let mut state = create_state(None).unwrap();
    state.registers[1] = 0x00;
    state.registers[2] = 0x03;

    execute_opcode(&mut state, 0x8126, ORIGINAL_QUIRKS).unwrap();

    assert_eq!(state.registers[1], 0x01);
    assert_eq!(state.registers[2], 0x03);
    assert_eq!(state.registers[0xF], 0x01);
}

#[test]
fn eight_xye_uses_vy_as_source() {
    let mut state = create_state(None).unwrap();
    state.registers[1] = 0x00;
    state.registers[2] = 0x80;

    execute_opcode(&mut state, 0x812E, ORIGINAL_QUIRKS).unwrap();

    assert_eq!(state.registers[1], 0x00);
    assert_eq!(state.registers[2], 0x80);
    assert_eq!(state.registers[0xF], 0x01);
}

#[test]
fn dxyn_sets_collision_flag_without_losing_it() {
    let mut state = create_state(None).unwrap();
    state.registers[0] = 2;
    state.registers[1] = 3;
    state.index = 0x300;
    state.memory[0x300] = 0x80;
    let loc = 2 + (3 * 64);
    state.screen_buffer[loc] = 1;

    execute_opcode(&mut state, 0xD011, ORIGINAL_QUIRKS).unwrap();

    assert_eq!(state.registers[0xF], 1);
    assert_eq!(state.screen_buffer[loc], 0);
}

#[test]
fn dxyn_wraps_start_coordinates() {
    let mut state = create_state(None).unwrap();
    state.registers[0] = 66;
    state.registers[1] = 33;
    state.index = 0x300;
    state.memory[0x300] = 0x80;

    execute_opcode(&mut state, 0xD011, ORIGINAL_QUIRKS).unwrap();

    let wrapped_loc = 2 + (64);
    assert_eq!(state.screen_buffer[wrapped_loc], 1);
}

#[test]
fn eight_xy6_uses_vx_source_in_modern_profile() {
    let mut state = create_state(None).unwrap();
    state.registers[1] = 0x03;
    state.registers[2] = 0x00;

    execute_opcode(&mut state, 0x8126, MODERN_QUIRKS).unwrap();

    assert_eq!(state.registers[1], 0x01);
    assert_eq!(state.registers[0xF], 0x01);
}

#[test]
fn fx65_does_not_increment_i_in_modern_profile() {
    let mut state = create_state(None).unwrap();
    state.index = 0x300;
    state.memory[0x300..0x303].copy_from_slice(&[0xAA, 0xBB, 0xCC]);

    execute_opcode(&mut state, 0xF265, MODERN_QUIRKS).unwrap();

    assert_eq!(state.registers[0..3], [0xAA, 0xBB, 0xCC]);
    assert_eq!(state.index, 0x300);
}

#[test]
fn fx55_does_not_increment_i_in_modern_profile() {
    let mut state = create_state(None).unwrap();
    state.index = 0x300;
    state.registers[0..3].copy_from_slice(&[0x11, 0x22, 0x33]);

    execute_opcode(&mut state, 0xF255, MODERN_QUIRKS).unwrap();

    assert_eq!(state.memory[0x300..0x303], [0x11, 0x22, 0x33]);
    assert_eq!(state.index, 0x300);
}

#[test]
fn bxnn_jump_uses_vx_in_modern_profile() {
    let mut state = create_state(None).unwrap();
    state.registers[0] = 0x05;
    state.registers[1] = 0x10;

    execute_opcode(&mut state, 0xB123, MODERN_QUIRKS).unwrap();

    assert_eq!(state.pc, 0x133);
}

#[test]
fn dxyn_wraps_pixels_in_modern_profile() {
    let mut state = create_state(None).unwrap();
    state.registers[0] = 63;
    state.registers[1] = 31;
    state.index = 0x300;
    state.memory[0x300] = 0xC0;

    execute_opcode(&mut state, 0xD011, MODERN_QUIRKS).unwrap();

    let pixel_a = 63 + (31 * 64);
    let pixel_b = 31 * 64;
    assert_eq!(state.screen_buffer[pixel_a], 1);
    assert_eq!(state.screen_buffer[pixel_b], 1);
}

#[test]
fn execute_cycle_does_not_tick_timers() {
    let mut state = create_state(None).unwrap();
    state.delay_timer = 5;
    state.sound_timer = 5;
    state.memory[state.pc] = 0x00;
    state.memory[state.pc + 1] = 0xE0;

    execute_cycle(&mut state, ORIGINAL_QUIRKS).unwrap();

    assert_eq!(state.delay_timer, 5);
    assert_eq!(state.sound_timer, 5);
}

#[test]
fn tick_timers_decrements_sound_and_delay() {
    let mut state = create_state(None).unwrap();
    state.delay_timer = 2;
    state.sound_timer = 2;
    let mut beep_count = 0;

    tick_timers(
        &mut state,
        Some(&mut || {
            beep_count += 1;
        }),
    );
    tick_timers(
        &mut state,
        Some(&mut || {
            beep_count += 1;
        }),
    );

    assert_eq!(state.delay_timer, 0);
    assert_eq!(state.sound_timer, 0);
    assert_eq!(beep_count, 2);
}
