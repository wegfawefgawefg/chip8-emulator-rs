use std::path::PathBuf;

use clap::Parser;

use chip8_emulator_rs::assembler::{assemble_file, AssemblerError};

#[derive(Debug, Parser)]
#[command(name = "chip8-asm")]
#[command(about = "Assemble CHIP-8 source into a ROM")]
struct Args {
    source: PathBuf,

    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    #[arg(long, default_value = "0x200")]
    origin: String,
}

fn parse_origin(text: &str) -> Result<usize, AssemblerError> {
    let value = if let Some(rest) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        usize::from_str_radix(rest, 16)
            .map_err(|_| AssemblerError::new(format!("invalid --origin value '{text}'"), None))?
    } else {
        text.parse::<usize>()
            .map_err(|_| AssemblerError::new(format!("invalid --origin value '{text}'"), None))?
    };
    Ok(value)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let output_path = args
        .output
        .unwrap_or_else(|| args.source.with_extension("ch8"));

    let origin = parse_origin(&args.origin)?;
    let rom_bytes = assemble_file(&args.source, origin)?;

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&output_path, &rom_bytes)?;

    println!(
        "wrote {} bytes to {}",
        rom_bytes.len(),
        output_path.display()
    );
    Ok(())
}
