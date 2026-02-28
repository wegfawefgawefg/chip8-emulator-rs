use std::path::Path;
use std::time::Instant;

use crate::chip8_emulator::config::{MEMORY_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::chip8_emulator::cpu::{execute_cycle, tick_timers};
use crate::chip8_emulator::error::Chip8Error;
use crate::chip8_emulator::quirks::Chip8Quirks;
use crate::chip8_emulator::state::{create_state, set_key_state, EmulatorState};

pub fn run_emulator_headless(
    quirks: Chip8Quirks,
    rom_path: &Path,
    max_cycles: usize,
    cpu_hz: usize,
) -> Result<EmulatorState, Chip8Error> {
    if max_cycles == 0 {
        return Err(Chip8Error::InvalidArgument("max_cycles must be > 0"));
    }
    if cpu_hz == 0 {
        return Err(Chip8Error::InvalidArgument("cpu_hz must be > 0"));
    }

    let mut state = create_state(Some(rom_path))?;
    let cycles_per_timer_tick = usize::max(1, cpu_hz / 60);

    for i in 0..max_cycles {
        if state.exited {
            break;
        }

        execute_cycle(&mut state, quirks)?;

        if ((i + 1) % cycles_per_timer_tick) == 0 {
            tick_timers(&mut state, None);
        }
    }

    Ok(state)
}

pub fn run_emulator_app(
    quirks: Chip8Quirks,
    rom_path: &Path,
    scale: usize,
    cpu_hz: usize,
    target_fps: usize,
) -> Result<EmulatorState, Chip8Error> {
    use raylib::prelude::{Color, KeyboardKey, RaylibDraw};

    if scale == 0 {
        return Err(Chip8Error::InvalidArgument("scale must be > 0"));
    }
    if cpu_hz == 0 {
        return Err(Chip8Error::InvalidArgument("cpu_hz must be > 0"));
    }
    if target_fps == 0 {
        return Err(Chip8Error::InvalidArgument("target_fps must be > 0"));
    }

    let mut state = create_state(Some(rom_path))?;

    let width = (SCREEN_WIDTH * scale) as i32;
    let height = (SCREEN_HEIGHT * scale) as i32;
    let (mut rl, thread) = raylib::init()
        .size(width, height)
        .title("chip8-emulator-rs")
        .build();
    rl.set_target_fps(target_fps as u32);

    let key_map = [
        (KeyboardKey::KEY_ONE, 0x1usize),
        (KeyboardKey::KEY_TWO, 0x2),
        (KeyboardKey::KEY_THREE, 0x3),
        (KeyboardKey::KEY_FOUR, 0xC),
        (KeyboardKey::KEY_Q, 0x4),
        (KeyboardKey::KEY_W, 0x5),
        (KeyboardKey::KEY_E, 0x6),
        (KeyboardKey::KEY_R, 0xD),
        (KeyboardKey::KEY_A, 0x7),
        (KeyboardKey::KEY_S, 0x8),
        (KeyboardKey::KEY_D, 0x9),
        (KeyboardKey::KEY_F, 0xE),
        (KeyboardKey::KEY_Z, 0xA),
        (KeyboardKey::KEY_X, 0x0),
        (KeyboardKey::KEY_C, 0xB),
        (KeyboardKey::KEY_V, 0xF),
    ];

    let cycle_interval = 1.0f32 / cpu_hz as f32;
    let timer_interval = 1.0f32 / 60.0;
    let max_cycles_per_frame = usize::max(1, (cpu_hz / target_fps) * 3);
    let mut accumulated_time = 0.0f32;
    let mut timer_accumulated_time = 0.0f32;
    let mut front_buffer = state.screen_buffer;
    let mut previous_tick = Instant::now();
    let mut frame_in_progress_after_clear = false;
    let mut has_draw_since_clear = false;

    while !rl.window_should_close() && !state.exited {
        if rl.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            break;
        }

        for (key, mapped) in key_map {
            set_key_state(&mut state, mapped, rl.is_key_down(key));
        }

        let now = Instant::now();
        let frame_dt = (now - previous_tick).as_secs_f32().min(0.1);
        previous_tick = now;
        accumulated_time += frame_dt;
        timer_accumulated_time += frame_dt;

        let mut cycles_run = 0;
        while accumulated_time >= cycle_interval
            && cycles_run < max_cycles_per_frame
            && !state.exited
        {
            // For CLS-framed ROMs (like snake), publish the completed frame right before
            // the next clear starts the next frame.
            if state.pc <= (MEMORY_SIZE - 2) {
                let next_opcode =
                    ((state.memory[state.pc] as u16) << 8) | state.memory[state.pc + 1] as u16;
                if next_opcode == 0x00E0 && has_draw_since_clear {
                    front_buffer = state.screen_buffer;
                    has_draw_since_clear = false;
                }
            }

            let pc_before = state.pc;
            execute_cycle(&mut state, quirks)?;
            if state.op == 0x00E0 {
                frame_in_progress_after_clear = true;
                has_draw_since_clear = false;
            }
            if (state.op & 0xF000) == 0xD000 {
                if frame_in_progress_after_clear {
                    has_draw_since_clear = true;
                } else {
                    // ROMs that don't use CLS still update smoothly.
                    front_buffer = state.screen_buffer;
                }
            }
            // If ROM blocks on LD Vx, K after drawing a frame (title screens),
            // publish what we have even without a subsequent CLS boundary.
            if (state.op & 0xF0FF) == 0xF00A && state.pc == pc_before && has_draw_since_clear {
                front_buffer = state.screen_buffer;
                has_draw_since_clear = false;
            }
            accumulated_time -= cycle_interval;
            cycles_run += 1;
        }

        while timer_accumulated_time >= timer_interval && !state.exited {
            tick_timers(&mut state, None);
            timer_accumulated_time -= timer_interval;
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        for (index, value) in front_buffer.iter().enumerate() {
            if *value == 0 {
                continue;
            }
            let x = (index % SCREEN_WIDTH) as i32;
            let y = (index / SCREEN_WIDTH) as i32;
            d.draw_rectangle(
                x * scale as i32,
                y * scale as i32,
                scale as i32,
                scale as i32,
                Color::WHITE,
            );
        }
        state.should_draw = false;
    }

    Ok(state)
}
