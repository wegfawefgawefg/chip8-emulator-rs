use std::fs;
use std::path::{Path, PathBuf};

use crate::chip8_emulator::config::{
    FONT_BYTES, KEY_COUNT, MEMORY_SIZE, PROGRAM_START, REGISTER_COUNT, SCREEN_HEIGHT, SCREEN_WIDTH,
};
use crate::chip8_emulator::error::Chip8Error;

#[derive(Debug, Clone)]
pub struct EmulatorState {
    pub memory: [u8; MEMORY_SIZE],
    pub registers: [u8; REGISTER_COUNT],
    pub stack: Vec<u16>,
    pub key_inputs: [u8; KEY_COUNT],
    pub screen_buffer: [u8; SCREEN_WIDTH * SCREEN_HEIGHT],
    pub pc: usize,
    pub index: usize,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub should_draw: bool,
    pub exited: bool,
    pub op: u16,
    pub rom_path: Option<PathBuf>,
}

impl Default for EmulatorState {
    fn default() -> Self {
        Self {
            memory: [0; MEMORY_SIZE],
            registers: [0; REGISTER_COUNT],
            stack: Vec::new(),
            key_inputs: [0; KEY_COUNT],
            screen_buffer: [0; SCREEN_WIDTH * SCREEN_HEIGHT],
            pc: PROGRAM_START,
            index: 0,
            delay_timer: 0,
            sound_timer: 0,
            should_draw: true,
            exited: false,
            op: 0,
            rom_path: None,
        }
    }
}

pub fn create_state(rom_path: Option<&Path>) -> Result<EmulatorState, Chip8Error> {
    let mut state = EmulatorState::default();
    reset_state(&mut state, rom_path)?;
    Ok(state)
}

pub fn reset_state(state: &mut EmulatorState, rom_path: Option<&Path>) -> Result<(), Chip8Error> {
    state.memory = [0; MEMORY_SIZE];
    state.registers = [0; REGISTER_COUNT];
    state.stack.clear();
    state.key_inputs = [0; KEY_COUNT];
    clear_display(state);

    state.pc = PROGRAM_START;
    state.index = 0;
    state.delay_timer = 0;
    state.sound_timer = 0;
    state.exited = false;
    state.op = 0;

    load_font(state);

    if let Some(path) = rom_path {
        state.rom_path = Some(path.to_path_buf());
    }

    if let Some(path) = state.rom_path.clone() {
        load_rom(state, &path)?;
    }

    Ok(())
}

pub fn clear_display(state: &mut EmulatorState) {
    state.screen_buffer = [0; SCREEN_WIDTH * SCREEN_HEIGHT];
    state.should_draw = true;
}

pub fn load_font(state: &mut EmulatorState) {
    state.memory[..FONT_BYTES.len()].copy_from_slice(&FONT_BYTES);
}

pub fn load_rom(state: &mut EmulatorState, path: &Path) -> Result<(), Chip8Error> {
    let rom_bytes = fs::read(path)?;
    let max_size = MEMORY_SIZE - PROGRAM_START;

    if rom_bytes.len() > max_size {
        return Err(Chip8Error::RomTooLarge {
            size: rom_bytes.len(),
            max: max_size,
        });
    }

    let start = PROGRAM_START;
    let end = PROGRAM_START + rom_bytes.len();
    state.memory[start..end].copy_from_slice(&rom_bytes);
    state.rom_path = Some(path.to_path_buf());

    Ok(())
}

pub fn first_pressed_key(state: &EmulatorState) -> Option<u8> {
    state
        .key_inputs
        .iter()
        .position(|pressed| *pressed == 1)
        .map(|index| index as u8)
}

pub fn set_key_state(state: &mut EmulatorState, key_index: usize, is_pressed: bool) {
    if key_index >= KEY_COUNT {
        return;
    }

    state.key_inputs[key_index] = u8::from(is_pressed);
}
