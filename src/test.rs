#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io::Write,
    };

    use crate::Chip8Emulator;

    #[test]
    fn test_initialization() {
        let emu = Chip8Emulator::new();
        assert_eq!(emu.pc, 0x200);
    }

    #[test]
    fn test_rom_loading() {
        // 1. Create a dummy ROM file
        let rom_path = "test_rom.bin";
        let test_data = [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF];
        {
            let mut file = File::create(rom_path).expect("Failed to create test ROM file.");
            file.write_all(&test_data)
                .expect("Failed to write to test ROM file.");
        }

        // 2. Load the ROM using the emulator
        let mut emu = Chip8Emulator::new();
        emu.load_rom(rom_path).expect("Failed to load ROM.");

        // 3. Check if ROM was loaded correctly into memory
        for (i, &byte) in test_data.iter().enumerate() {
            assert_eq!(
                emu.memory[0x0200 + i],
                byte,
                "Memory mismatch at index {:x}.",
                0x0200 + i
            );
        }

        // Cleanup: Remove the dummy ROM file after testing
        fs::remove_file(rom_path).expect("Failed to remove test ROM file.");
    }
}
