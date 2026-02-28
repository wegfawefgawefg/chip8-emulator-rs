use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Chip8Error {
    Io(std::io::Error),
    RomTooLarge { size: usize, max: usize },
    ProgramCounterOutOfBounds(usize),
    InvalidOpcode(u16),
    StackUnderflow,
    InvalidArgument(&'static str),
}

impl Display for Chip8Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(f, "io error: {error}"),
            Self::RomTooLarge { size, max } => {
                write!(f, "ROM too large: {size} bytes (max {max})")
            }
            Self::ProgramCounterOutOfBounds(pc) => {
                write!(f, "program counter exceeded program memory: 0x{pc:03x}")
            }
            Self::InvalidOpcode(opcode) => write!(f, "invalid opcode: 0x{opcode:04x}"),
            Self::StackUnderflow => write!(f, "return instruction with empty stack"),
            Self::InvalidArgument(argument) => write!(f, "invalid argument: {argument}"),
        }
    }
}

impl std::error::Error for Chip8Error {}

impl From<std::io::Error> for Chip8Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
