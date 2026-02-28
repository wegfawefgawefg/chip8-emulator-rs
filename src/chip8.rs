use log::{info, log_enabled};
use std::io::{Read, Result};
use std::{fs::File, path::Path};

pub struct Chip8Emulator {
    pub key_inputs: [u8; 16],
    pub memory: [u8; Chip8Emulator::MEMORY_SIZE],
    pub pc: usize,
    pub index: u16,
    pub registers: [u8; 16],
    pub stack: [u16; 16],

    pub sound_timer: u8,
    pub delay_timer: u8,

    pub gfx: [u8; Chip8Emulator::SCREEN_WIDTH * Chip8Emulator::SCREEN_HEIGHT],
    pub font: [u8; 80],
}

impl Chip8Emulator {
    const CYCLE_LOGGING: bool = false;
    const MEMORY_SIZE: usize = 4096;
    const SCREEN_WIDTH: usize = 64;
    const SCREEN_HEIGHT: usize = 32;

    pub fn new() -> Chip8Emulator {
        Chip8Emulator {
            key_inputs: [0; 16],
            memory: [0; Chip8Emulator::MEMORY_SIZE],
            pc: 0x200,
            index: 0,
            registers: [0; 16],
            stack: [0; 16],

            sound_timer: 0,
            delay_timer: 0,

            gfx: [0; Chip8Emulator::SCREEN_WIDTH * Chip8Emulator::SCREEN_HEIGHT],

            font: [
                0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
                0x20, 0x60, 0x20, 0x20, 0x70, // 1
                0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
                0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
                0x90, 0x90, 0xF0, 0x10, 0x10, // 4
                0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
                0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
                0xF0, 0x10, 0x20, 0x40, 0x40, // 7
                0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
                0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
                0xF0, 0x90, 0xF0, 0x90, 0x90, // A
                0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
                0xF0, 0x80, 0x80, 0x80, 0xF0, // C
                0xE0, 0x90, 0x90, 0x90, 0xE0, // D
                0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
                0xF0, 0x80, 0xF0, 0x80, 0x80, // F
            ],
        }
    }

    pub fn key_map(key: u8) -> Option<u8> {
        match key {
            K_1 => Some(0x01),
            K_2 => Some(0x02),
            K_3 => Some(0x03),
            K_4 => Some(0x0C),
            K_q => Some(0x04),
            K_w => Some(0x05),
            K_e => Some(0x06),
            K_r => Some(0x0D),
            K_a => Some(0x07),
            K_s => Some(0x08),
            K_d => Some(0x09),
            K_f => Some(0x0E),
            K_z => Some(0x0A),
            K_x => Some(0x00),
            K_c => Some(0x0B),
            K_v => Some(0x0F),
            _ => None, // Return None or a default value for unknown keys
        }
    }

    pub fn load_font(&mut self) {
        info!("load font"); // Using the log crate for logging
        for i in 0..80 {
            self.memory[i] = self.font[i];
        }
    }

    pub fn load_rom(&mut self, path: &str) -> Result<()> {
        let path_obj = Path::new(path);
        let mut file = File::open(path_obj)?;

        let mut rom = Vec::new();
        file.read_to_end(&mut rom)?;

        for (i, &byte) in rom.iter().enumerate() {
            self.memory[0x0200 + i] = byte;
        }

        if log_enabled!(log::Level::Info) {
            Chip8Emulator::log_rom_content(&rom);
        }

        let start = 0x0200;
        let end = 0x0200 + rom.len();
        info!("loaded rom into mem starting at {:x} - to {:x}", start, end);

        Ok(())
    }

    pub fn log_rom_content(rom: &[u8]) {
        let mut bts: Vec<Vec<String>> = Vec::new();
        let mut bts_row: Vec<String> = Vec::new();

        for (i, &byte) in rom.iter().enumerate() {
            let hex_str = format!("0x{:02x}", byte);
            bts_row.push(hex_str);

            if (i + 1) % 16 == 0 {
                bts.push(bts_row.clone());
                bts_row.clear();
            }
        }
        if !bts_row.is_empty() {
            bts.push(bts_row);
        }

        let info_str = format!(
            "loading rom:\n{}",
            bts.into_iter()
                .map(|row| row.join(" "))
                .collect::<Vec<String>>()
                .join("\n")
        );
        info!("{}", info_str);
    }

    pub fn inc_pc(&mut self) {
        if self.pc >= Self::MEMORY_SIZE {
            if Self::CYCLE_LOGGING {
                log::error!("program counter exceeded program memory");
            }
            panic!("program counter exceeded program memory");
        } else if self.pc == Self::MEMORY_SIZE - 2 && Self::CYCLE_LOGGING {
            log::warn!("program counter is about to wrap");
        }

        self.pc = self.pc.wrapping_add(2);
    }

    pub fn dec_pc(&mut self) {
        self.pc = self.pc.wrapping_sub(2);
    }

    pub fn print_stack(&self) {
        if Self::CYCLE_LOGGING {
            let stack_dump: Vec<String> =
                self.stack.iter().map(|&ptr| format!("{:x}", ptr)).collect();
            log::info!("======== stack_height: {}", self.stack.len());
            log::info!("======== stack: [{:?}]", stack_dump.join(", "));
        }
    }

    pub fn print_registers(&self) {
        if Self::CYCLE_LOGGING {
            let reg_dump: Vec<String> = self
                .registers
                .iter()
                .map(|&ptr| format!("{:x}", ptr))
                .collect();
            log::info!("======== registers: [{:?}]", reg_dump.join(", "));
        }
    }

    pub fn print_keys(&self) {
        if Self::CYCLE_LOGGING {
            let mut key_dump = vec![" ".to_string()];
            for y in 0..4 {
                let mut line = vec![];
                for x in 0..4 {
                    let i = (y * 4) + x;
                    let k = self.key_inputs[i];
                    line.push(k.to_string());
                }
                key_dump.push(line.join(", "));
            }
            log::info!("======== keys: [{}]", key_dump.join("\t\t\n "));
        }
    }

    pub fn cycle(&mut self) {
        if Self::CYCLE_LOGGING {
            log::info!("========================= cycle ================================");
        }

        let op = ((self.memory[self.pc] as u16) << 8) | (self.memory[self.pc + 1] as u16);
        self.inc_pc();

        if self.print_loaded_ops && Self::CYCLE_LOGGING {
            log::info!("======== pc: {:x}", self.pc);
            self.print_registers();
            self.print_stack();
            self.print_keys();
            log::info!("======== current op: {:x}", self.op);
        }

        let op_type = self.op & 0xF000;
        self.process(op_type);

        self.delay_timer = self.delay_timer.saturating_sub(1);
        self.sound_timer = self.sound_timer.saturating_sub(1);
        if self.sound_timer > 0 {
            self.play_tone();
        }
    }

    pub fn process(&mut self, op: u16) {
        match self.isa.get(&op) {
            Some(instruction) => instruction(),
            None => {
                if CYCLE_LOGGING {
                    log::error!("invalid instruction: 0x{:02x}", op);
                }
                panic!("invalid instruction: 0x{:02x}", op);
            }
        }
    }

    pub fn play_tone(&mut self) {}
}
