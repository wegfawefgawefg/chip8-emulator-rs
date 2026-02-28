use rand::random;

use crate::chip8_emulator::config::{MEMORY_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::chip8_emulator::error::Chip8Error;
use crate::chip8_emulator::quirks::Chip8Quirks;
use crate::chip8_emulator::state::{clear_display, first_pressed_key, EmulatorState};

fn x_register_index(opcode: u16) -> usize {
    ((opcode & 0x0F00) >> 8) as usize
}

fn y_register_index(opcode: u16) -> usize {
    ((opcode & 0x00F0) >> 4) as usize
}

fn address_nnn(opcode: u16) -> usize {
    (opcode & 0x0FFF) as usize
}

fn byte_nn(opcode: u16) -> u8 {
    (opcode & 0x00FF) as u8
}

fn nibble_n(opcode: u16) -> u8 {
    (opcode & 0x000F) as u8
}

pub fn execute_cycle(state: &mut EmulatorState, quirks: Chip8Quirks) -> Result<(), Chip8Error> {
    if state.pc > (MEMORY_SIZE - 2) {
        return Err(Chip8Error::ProgramCounterOutOfBounds(state.pc));
    }

    let opcode = ((state.memory[state.pc] as u16) << 8) | state.memory[state.pc + 1] as u16;
    state.pc += 2;

    execute_opcode(state, opcode, quirks)
}

pub fn tick_timers(state: &mut EmulatorState, mut sound_callback: Option<&mut dyn FnMut()>) {
    state.delay_timer = state.delay_timer.saturating_sub(1);

    if state.sound_timer > 0 {
        state.sound_timer -= 1;
        if let Some(callback) = sound_callback.as_mut() {
            callback();
        }
    }
}

pub fn execute_opcode(
    state: &mut EmulatorState,
    opcode: u16,
    quirks: Chip8Quirks,
) -> Result<(), Chip8Error> {
    state.op = opcode;

    match opcode & 0xF000 {
        0x0000 => handle_family_0(state, opcode),
        0x1000 => {
            state.pc = address_nnn(opcode);
            Ok(())
        }
        0x2000 => {
            state.stack.push(state.pc as u16);
            state.pc = address_nnn(opcode);
            Ok(())
        }
        0x3000 => {
            if state.registers[x_register_index(opcode)] == byte_nn(opcode) {
                state.pc += 2;
            }
            Ok(())
        }
        0x4000 => {
            if state.registers[x_register_index(opcode)] != byte_nn(opcode) {
                state.pc += 2;
            }
            Ok(())
        }
        0x5000 => handle_opcode_5xy0_skip_eq_register(state, opcode),
        0x6000 => {
            state.registers[x_register_index(opcode)] = byte_nn(opcode);
            Ok(())
        }
        0x7000 => {
            let x_reg = x_register_index(opcode);
            state.registers[x_reg] = state.registers[x_reg].wrapping_add(byte_nn(opcode));
            Ok(())
        }
        0x8000 => handle_family_8(state, opcode, quirks),
        0x9000 => handle_opcode_9xy0_skip_neq_register(state, opcode),
        0xA000 => {
            state.index = address_nnn(opcode);
            Ok(())
        }
        0xB000 => {
            let jump_register = if quirks.jump_with_vx {
                x_register_index(opcode)
            } else {
                0
            };
            state.pc = (address_nnn(opcode) + state.registers[jump_register] as usize) & 0x0FFF;
            Ok(())
        }
        0xC000 => {
            state.registers[x_register_index(opcode)] = random::<u8>() & byte_nn(opcode);
            Ok(())
        }
        0xD000 => handle_opcode_dxyn_draw(state, opcode, quirks),
        0xE000 => handle_family_e(state, opcode),
        0xF000 => handle_family_f(state, opcode, quirks),
        _ => Err(Chip8Error::InvalidOpcode(opcode)),
    }
}

fn handle_family_0(state: &mut EmulatorState, opcode: u16) -> Result<(), Chip8Error> {
    match opcode {
        0x00E0 => {
            clear_display(state);
            Ok(())
        }
        0x00EE => {
            let ret = state.stack.pop().ok_or(Chip8Error::StackUnderflow)?;
            state.pc = ret as usize;
            Ok(())
        }
        0x00FD => {
            state.exited = true;
            Ok(())
        }
        _ => Ok(()),
    }
}

fn handle_opcode_5xy0_skip_eq_register(
    state: &mut EmulatorState,
    opcode: u16,
) -> Result<(), Chip8Error> {
    if nibble_n(opcode) != 0 {
        return Err(Chip8Error::InvalidOpcode(opcode));
    }

    if state.registers[x_register_index(opcode)] == state.registers[y_register_index(opcode)] {
        state.pc += 2;
    }

    Ok(())
}

fn handle_family_8(
    state: &mut EmulatorState,
    opcode: u16,
    quirks: Chip8Quirks,
) -> Result<(), Chip8Error> {
    let x_reg = x_register_index(opcode);
    let y_reg = y_register_index(opcode);

    match nibble_n(opcode) {
        0x0 => {
            state.registers[x_reg] = state.registers[y_reg];
            Ok(())
        }
        0x1 => {
            state.registers[x_reg] |= state.registers[y_reg];
            Ok(())
        }
        0x2 => {
            state.registers[x_reg] &= state.registers[y_reg];
            Ok(())
        }
        0x3 => {
            state.registers[x_reg] ^= state.registers[y_reg];
            Ok(())
        }
        0x4 => {
            let (result, carry) = state.registers[x_reg].overflowing_add(state.registers[y_reg]);
            state.registers[0xF] = u8::from(carry);
            state.registers[x_reg] = result;
            Ok(())
        }
        0x5 => {
            state.registers[0xF] = u8::from(state.registers[x_reg] >= state.registers[y_reg]);
            state.registers[x_reg] = state.registers[x_reg].wrapping_sub(state.registers[y_reg]);
            Ok(())
        }
        0x6 => {
            let source = if quirks.shift_uses_vy { y_reg } else { x_reg };
            let value = state.registers[source];
            state.registers[0xF] = value & 0x1;
            state.registers[x_reg] = value >> 1;
            Ok(())
        }
        0x7 => {
            state.registers[0xF] = u8::from(state.registers[y_reg] >= state.registers[x_reg]);
            state.registers[x_reg] = state.registers[y_reg].wrapping_sub(state.registers[x_reg]);
            Ok(())
        }
        0xE => {
            let source = if quirks.shift_uses_vy { y_reg } else { x_reg };
            let value = state.registers[source];
            state.registers[0xF] = (value & 0x80) >> 7;
            state.registers[x_reg] = value.wrapping_shl(1);
            Ok(())
        }
        _ => Err(Chip8Error::InvalidOpcode(opcode)),
    }
}

fn handle_opcode_9xy0_skip_neq_register(
    state: &mut EmulatorState,
    opcode: u16,
) -> Result<(), Chip8Error> {
    if nibble_n(opcode) != 0 {
        return Err(Chip8Error::InvalidOpcode(opcode));
    }

    if state.registers[x_register_index(opcode)] != state.registers[y_register_index(opcode)] {
        state.pc += 2;
    }

    Ok(())
}

fn handle_opcode_dxyn_draw(
    state: &mut EmulatorState,
    opcode: u16,
    quirks: Chip8Quirks,
) -> Result<(), Chip8Error> {
    let x_start = (state.registers[x_register_index(opcode)] as usize) % SCREEN_WIDTH;
    let y_start = (state.registers[y_register_index(opcode)] as usize) % SCREEN_HEIGHT;
    let height = nibble_n(opcode) as usize;

    let mut collision = 0;

    for row in 0..height {
        let mut y_pos = y_start + row;
        if quirks.draw_wrap {
            y_pos %= SCREEN_HEIGHT;
        } else if y_pos >= SCREEN_HEIGHT {
            break;
        }

        let sprite_address = state.index + row;
        if sprite_address >= MEMORY_SIZE {
            return Err(Chip8Error::ProgramCounterOutOfBounds(sprite_address));
        }

        let sprite_row = state.memory[sprite_address];

        for bit in 0..8 {
            let mut x_pos = x_start + bit;
            if quirks.draw_wrap {
                x_pos %= SCREEN_WIDTH;
            } else if x_pos >= SCREEN_WIDTH {
                break;
            }

            let pixel = (sprite_row >> (7 - bit)) & 0x1;
            if pixel == 0 {
                continue;
            }

            let location = x_pos + (y_pos * SCREEN_WIDTH);
            if state.screen_buffer[location] == 1 {
                collision = 1;
            }
            state.screen_buffer[location] ^= 1;
        }
    }

    state.registers[0xF] = collision;
    state.should_draw = true;

    Ok(())
}

fn handle_family_e(state: &mut EmulatorState, opcode: u16) -> Result<(), Chip8Error> {
    let key = (state.registers[x_register_index(opcode)] & 0x0F) as usize;

    match byte_nn(opcode) {
        0x9E => {
            if state.key_inputs[key] == 1 {
                state.pc += 2;
            }
            Ok(())
        }
        0xA1 => {
            if state.key_inputs[key] == 0 {
                state.pc += 2;
            }
            Ok(())
        }
        _ => Err(Chip8Error::InvalidOpcode(opcode)),
    }
}

fn handle_family_f(
    state: &mut EmulatorState,
    opcode: u16,
    quirks: Chip8Quirks,
) -> Result<(), Chip8Error> {
    let x_reg = x_register_index(opcode);

    match byte_nn(opcode) {
        0x07 => {
            state.registers[x_reg] = state.delay_timer;
            Ok(())
        }
        0x0A => {
            if let Some(key) = first_pressed_key(state) {
                state.registers[x_reg] = key;
            } else {
                state.pc = state.pc.saturating_sub(2);
            }
            Ok(())
        }
        0x15 => {
            state.delay_timer = state.registers[x_reg];
            Ok(())
        }
        0x18 => {
            state.sound_timer = state.registers[x_reg];
            Ok(())
        }
        0x1E => {
            state.index = (state.index + state.registers[x_reg] as usize) & 0x0FFF;
            Ok(())
        }
        0x29 => {
            state.index = ((state.registers[x_reg] & 0x0F) as usize) * 5;
            Ok(())
        }
        0x33 => {
            let value = state.registers[x_reg];
            state.memory[state.index] = value / 100;
            state.memory[state.index + 1] = (value % 100) / 10;
            state.memory[state.index + 2] = value % 10;
            Ok(())
        }
        0x55 => {
            for index in 0..=x_reg {
                state.memory[state.index + index] = state.registers[index];
            }
            if quirks.load_store_increment_i {
                state.index = (state.index + x_reg + 1) & 0x0FFF;
            }
            Ok(())
        }
        0x65 => {
            for index in 0..=x_reg {
                state.registers[index] = state.memory[state.index + index];
            }
            if quirks.load_store_increment_i {
                state.index = (state.index + x_reg + 1) & 0x0FFF;
            }
            Ok(())
        }
        _ => Err(Chip8Error::InvalidOpcode(opcode)),
    }
}
