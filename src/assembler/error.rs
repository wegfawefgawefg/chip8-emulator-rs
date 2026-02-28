use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct AssemblerError {
    pub message: String,
    pub line_no: Option<usize>,
}

impl AssemblerError {
    pub fn new(message: impl Into<String>, line_no: Option<usize>) -> Self {
        Self {
            message: message.into(),
            line_no,
        }
    }
}

impl Display for AssemblerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(line_no) = self.line_no {
            write!(f, "line {line_no}: {}", self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for AssemblerError {}
