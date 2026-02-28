use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::assembler::encoding::{
    encode_instruction, ensure_range, parse_numeric_literal, parse_value,
};
use crate::assembler::error::AssemblerError;

#[derive(Debug, Clone)]
struct Statement {
    line_no: usize,
    kind: StatementKind,
    operation: String,
    arguments: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatementKind {
    DirectiveOrg,
    DirectiveDb,
    DirectiveDw,
    Instruction,
}

pub fn assemble_file(path: impl AsRef<Path>, origin: usize) -> Result<Vec<u8>, AssemblerError> {
    let source =
        fs::read_to_string(path).map_err(|error| AssemblerError::new(error.to_string(), None))?;
    assemble_text(&source, origin)
}

pub fn assemble_text(source: &str, origin: usize) -> Result<Vec<u8>, AssemblerError> {
    let (statements, labels) = parse_source(source, origin)?;
    encode_statements(&statements, &labels, origin)
}

fn parse_source(
    source: &str,
    origin: usize,
) -> Result<(Vec<Statement>, HashMap<String, usize>), AssemblerError> {
    let mut statements = Vec::new();
    let mut labels = HashMap::new();
    let mut program_counter = origin;

    for (line_no, raw_line) in source.lines().enumerate() {
        let line_no = line_no + 1;
        let content = strip_comments(raw_line).trim().to_owned();
        if content.is_empty() {
            continue;
        }

        let (line_labels, remainder) = split_labels(&content, line_no)?;
        for label in line_labels {
            if labels.contains_key(&label) {
                return Err(AssemblerError::new(
                    format!("duplicate label '{label}'"),
                    Some(line_no),
                ));
            }
            labels.insert(label, program_counter);
        }

        if remainder.is_empty() {
            continue;
        }

        let (operation, arguments) = split_operation_and_arguments(&remainder);
        let normalized = normalize_operation(&operation);
        let kind = classify_operation(&normalized);

        statements.push(Statement {
            line_no,
            kind,
            operation: normalized.clone(),
            arguments: arguments.clone(),
        });

        match kind {
            StatementKind::DirectiveOrg => {
                if arguments.len() != 1 {
                    return Err(AssemblerError::new(
                        "ORG expects exactly one argument",
                        Some(line_no),
                    ));
                }

                let target = parse_numeric_literal(&arguments[0], line_no)? as usize;
                if target < origin {
                    return Err(AssemblerError::new(
                        format!("ORG target 0x{target:03X} cannot be below origin 0x{origin:03X}"),
                        Some(line_no),
                    ));
                }
                if target < program_counter {
                    return Err(AssemblerError::new(
                        format!(
                            "ORG target 0x{target:03X} cannot move backwards from 0x{program_counter:03X}"
                        ),
                        Some(line_no),
                    ));
                }
                program_counter = target;
            }
            StatementKind::DirectiveDb => {
                if arguments.is_empty() {
                    return Err(AssemblerError::new(
                        "DB expects at least one argument",
                        Some(line_no),
                    ));
                }
                program_counter += count_db_bytes(&arguments, line_no)?;
            }
            StatementKind::DirectiveDw => {
                if arguments.is_empty() {
                    return Err(AssemblerError::new(
                        "DW expects at least one argument",
                        Some(line_no),
                    ));
                }
                program_counter += 2 * arguments.len();
            }
            StatementKind::Instruction => {
                program_counter += 2;
            }
        }
    }

    Ok((statements, labels))
}

fn encode_statements(
    statements: &[Statement],
    labels: &HashMap<String, usize>,
    origin: usize,
) -> Result<Vec<u8>, AssemblerError> {
    let mut output = Vec::new();
    let mut current_address = origin;

    for statement in statements {
        match statement.kind {
            StatementKind::DirectiveOrg => {
                let target =
                    parse_value(&statement.arguments[0], labels, statement.line_no)? as usize;
                if target < current_address {
                    return Err(AssemblerError::new(
                        format!(
                            "ORG target 0x{target:03X} cannot move backwards from 0x{current_address:03X}"
                        ),
                        Some(statement.line_no),
                    ));
                }
                output.extend(vec![0; target - current_address]);
                current_address = target;
            }
            StatementKind::DirectiveDb => {
                let db_values = encode_db_values(&statement.arguments, labels, statement.line_no)?;
                current_address += db_values.len();
                output.extend(db_values);
            }
            StatementKind::DirectiveDw => {
                for argument in &statement.arguments {
                    let word = parse_value(argument, labels, statement.line_no)?;
                    ensure_range(word, 0, 0xFFFF, "word", statement.line_no)?;
                    output.push(((word >> 8) & 0xFF) as u8);
                    output.push((word & 0xFF) as u8);
                    current_address += 2;
                }
            }
            StatementKind::Instruction => {
                let opcode = encode_instruction(
                    &statement.operation,
                    &statement.arguments,
                    labels,
                    statement.line_no,
                )?;
                output.push(((opcode >> 8) & 0xFF) as u8);
                output.push((opcode & 0xFF) as u8);
                current_address += 2;
            }
        }
    }

    Ok(output)
}

fn strip_comments(line: &str) -> String {
    let mut in_quote: Option<char> = None;

    for (index, ch) in line.char_indices() {
        if ch == '\'' || ch == '"' {
            if in_quote.is_none() {
                in_quote = Some(ch);
            } else if in_quote == Some(ch) {
                in_quote = None;
            }
        } else if (ch == ';' || ch == '#') && in_quote.is_none() {
            return line[..index].to_owned();
        }
    }

    line.to_owned()
}

fn split_labels(content: &str, line_no: usize) -> Result<(Vec<String>, String), AssemblerError> {
    let mut labels = Vec::new();
    let mut remainder = content.trim().to_owned();

    loop {
        let Some(colon_index) = remainder.find(':') else {
            return Ok((labels, remainder.trim().to_owned()));
        };

        let before = remainder[..colon_index].trim();
        let after = remainder[colon_index + 1..].trim();

        if before.is_empty() || before.chars().any(char::is_whitespace) {
            return Ok((labels, remainder.trim().to_owned()));
        }

        validate_label(before, line_no)?;
        labels.push(before.to_owned());
        remainder = after.to_owned();

        if remainder.is_empty() {
            return Ok((labels, String::new()));
        }
    }
}

fn validate_label(label: &str, line_no: usize) -> Result<(), AssemblerError> {
    let mut chars = label.chars();
    let first = chars
        .next()
        .ok_or_else(|| AssemblerError::new(format!("invalid label '{label}'"), Some(line_no)))?;

    if !first.is_ascii_alphabetic() && first != '_' {
        return Err(AssemblerError::new(
            format!("invalid label '{label}'"),
            Some(line_no),
        ));
    }

    if chars.any(|ch| !ch.is_ascii_alphanumeric() && ch != '_') {
        return Err(AssemblerError::new(
            format!("invalid label '{label}'"),
            Some(line_no),
        ));
    }

    Ok(())
}

fn split_operation_and_arguments(text: &str) -> (String, Vec<String>) {
    let mut parts = text.splitn(2, char::is_whitespace);
    let operation = parts.next().unwrap_or("").to_owned();
    let arguments = parts.next().map(split_arguments).unwrap_or_default();
    (operation, arguments)
}

fn split_arguments(text: &str) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }

    let mut args = Vec::new();
    let mut token = String::new();
    let mut in_quote: Option<char> = None;

    for ch in text.chars() {
        if ch == '\'' || ch == '"' {
            if in_quote.is_none() {
                in_quote = Some(ch);
            } else if in_quote == Some(ch) {
                in_quote = None;
            }
            token.push(ch);
            continue;
        }

        if ch == ',' && in_quote.is_none() {
            let value = token.trim();
            if !value.is_empty() {
                args.push(value.to_owned());
            }
            token.clear();
            continue;
        }

        token.push(ch);
    }

    let tail = token.trim();
    if !tail.is_empty() {
        args.push(tail.to_owned());
    }

    args
}

fn normalize_operation(operation: &str) -> String {
    let op = operation.trim().trim_start_matches('.');
    op.to_ascii_uppercase()
}

fn classify_operation(operation: &str) -> StatementKind {
    match operation {
        "ORG" => StatementKind::DirectiveOrg,
        "DB" => StatementKind::DirectiveDb,
        "DW" => StatementKind::DirectiveDw,
        _ => StatementKind::Instruction,
    }
}

fn count_db_bytes(arguments: &[String], line_no: usize) -> Result<usize, AssemblerError> {
    let mut total = 0;

    for argument in arguments {
        if let Some(text) = parse_string_literal(argument) {
            total += text.chars().count();
        } else {
            total += 1;
        }
    }

    if total == 0 {
        return Err(AssemblerError::new("DB produced no bytes", Some(line_no)));
    }

    Ok(total)
}

fn parse_string_literal(token: &str) -> Option<String> {
    let value = token.trim();
    if value.len() >= 2 {
        let first = value.chars().next()?;
        let last = value.chars().last()?;
        if (first == '\'' || first == '"') && first == last {
            return Some(value[1..value.len() - 1].to_owned());
        }
    }
    None
}

fn encode_db_values(
    arguments: &[String],
    labels: &HashMap<String, usize>,
    line_no: usize,
) -> Result<Vec<u8>, AssemblerError> {
    let mut values = Vec::new();

    for argument in arguments {
        if let Some(text) = parse_string_literal(argument) {
            values.extend(text.chars().map(|ch| (ch as u32 & 0xFF) as u8));
            continue;
        }

        let byte = parse_value(argument, labels, line_no)?;
        ensure_range(byte, 0, 0xFF, "byte", line_no)?;
        values.push(byte as u8);
    }

    Ok(values)
}
