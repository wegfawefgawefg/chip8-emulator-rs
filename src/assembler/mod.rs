pub mod assembler;
pub mod encoding;
pub mod error;

pub use assembler::{assemble_file, assemble_text};
pub use error::AssemblerError;
