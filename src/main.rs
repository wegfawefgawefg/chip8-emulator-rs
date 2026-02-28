use std::path::PathBuf;

use clap::Parser;

use chip8_emulator_rs::{load_quirks_profile, run_emulator_app, run_emulator_headless, Chip8Error};

#[derive(Debug, Parser)]
#[command(name = "chip8-emulator-rs")]
#[command(about = "Run the CHIP-8 emulator")]
struct Args {
    #[arg(long, default_value = "roms/chip8-test-suite.ch8")]
    rom: PathBuf,

    #[arg(long, default_value = "original", value_parser = ["original", "modern"])]
    quirks: String,

    #[arg(long, default_value_t = 16)]
    scale: usize,

    #[arg(long, default_value_t = 700)]
    hz: usize,

    #[arg(long, default_value_t = 60)]
    fps: usize,

    #[arg(long, default_value_t = 2000)]
    max_cycles: usize,

    #[arg(long)]
    headless: bool,
}

fn main() -> Result<(), Chip8Error> {
    let args = Args::parse();
    let quirks = load_quirks_profile(&args.quirks)
        .map_err(|_| Chip8Error::InvalidArgument("quirks must be original or modern"))?;

    if args.headless {
        let state = run_emulator_headless(quirks, &args.rom, args.max_cycles, args.hz)?;
        println!(
            "headless finished: exited={} pc=0x{:03x}",
            state.exited, state.pc
        );
        return Ok(());
    }

    let _state = run_emulator_app(quirks, &args.rom, args.scale, args.hz, args.fps)?;
    Ok(())
}
