pub mod assembler;
pub mod chip8_emulator;

pub use chip8_emulator::app::{run_emulator_app, run_emulator_headless};
pub use chip8_emulator::cpu::{execute_cycle, execute_opcode, tick_timers};
pub use chip8_emulator::error::Chip8Error;
pub use chip8_emulator::quirks::{
    load_quirks_profile, load_quirks_profile_from_env, Chip8Quirks, MODERN_QUIRKS, ORIGINAL_QUIRKS,
};
pub use chip8_emulator::state::{
    clear_display, create_state, first_pressed_key, load_rom, reset_state, set_key_state,
    EmulatorState,
};
