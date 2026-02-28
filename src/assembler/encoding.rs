use crate::assembler::error::AssemblerError;

pub fn parse_numeric_literal(token: &str, line_no: usize) -> Result<i32, AssemblerError> {
    let mut value = token.trim().to_owned();
    if let Some(rest) = value.strip_prefix('$') {
        value = format!("0x{rest}");
    }

    if value.len() >= 3 && value.starts_with('\'') && value.ends_with('\'') {
        let inner = &value[1..value.len() - 1];
        let mut chars = inner.chars();
        if let (Some(ch), None) = (chars.next(), chars.next()) {
            return Ok(ch as i32);
        }
    }

    if let Ok(parsed) = parse_prefixed_int(&value) {
        return Ok(parsed);
    }

    Err(AssemblerError::new(
        format!("invalid value '{value}'"),
        Some(line_no),
    ))
}

fn parse_prefixed_int(token: &str) -> Result<i32, std::num::ParseIntError> {
    if let Some(rest) = token
        .strip_prefix("0x")
        .or_else(|| token.strip_prefix("0X"))
    {
        return i32::from_str_radix(rest, 16);
    }
    if let Some(rest) = token
        .strip_prefix("0b")
        .or_else(|| token.strip_prefix("0B"))
    {
        return i32::from_str_radix(rest, 2);
    }
    if let Some(rest) = token
        .strip_prefix("0o")
        .or_else(|| token.strip_prefix("0O"))
    {
        return i32::from_str_radix(rest, 8);
    }
    token.parse::<i32>()
}

pub fn parse_value(
    token: &str,
    labels: &std::collections::HashMap<String, usize>,
    line_no: usize,
) -> Result<i32, AssemblerError> {
    let token = token.trim();
    if let Some(value) = labels.get(token) {
        return Ok(*value as i32);
    }
    parse_numeric_literal(token, line_no)
}

pub fn ensure_range(
    value: i32,
    minimum: i32,
    maximum: i32,
    label: &str,
    line_no: usize,
) -> Result<(), AssemblerError> {
    if value < minimum || value > maximum {
        return Err(AssemblerError::new(
            format!("{label} out of range: {value} (expected {minimum}..{maximum})"),
            Some(line_no),
        ));
    }
    Ok(())
}

pub fn parse_register(token: &str, line_no: usize) -> Result<u16, AssemblerError> {
    let value = token.trim().to_ascii_uppercase();
    let reg_text = value.strip_prefix('V').ok_or_else(|| {
        AssemblerError::new(format!("expected register, got '{token}'"), Some(line_no))
    })?;

    if reg_text.len() == 1 {
        let ch = reg_text.chars().next().unwrap_or(' ');
        if ch.is_ascii_hexdigit() {
            return Ok(ch.to_digit(16).unwrap_or(0) as u16);
        }
    }

    if reg_text.chars().all(|ch| ch.is_ascii_digit()) {
        let reg = reg_text.parse::<u16>().map_err(|_| {
            AssemblerError::new(format!("invalid register '{token}'"), Some(line_no))
        })?;
        if reg <= 15 {
            return Ok(reg);
        }
    }

    Err(AssemblerError::new(
        format!("invalid register '{token}'"),
        Some(line_no),
    ))
}

pub fn is_register(token: &str) -> bool {
    parse_register(token, 0)
        .map(|reg| reg <= 15)
        .unwrap_or(false)
}

fn expect_arg_count(
    mnemonic: &str,
    arguments: &[String],
    expected: usize,
    line_no: usize,
) -> Result<(), AssemblerError> {
    if arguments.len() != expected {
        return Err(AssemblerError::new(
            format!(
                "{mnemonic} expects {expected} argument(s), got {}",
                arguments.len()
            ),
            Some(line_no),
        ));
    }
    Ok(())
}

pub fn encode_instruction(
    mnemonic: &str,
    arguments: &[String],
    labels: &std::collections::HashMap<String, usize>,
    line_no: usize,
) -> Result<u16, AssemblerError> {
    let op = mnemonic.to_ascii_uppercase();

    if op == "CLS" {
        expect_arg_count(&op, arguments, 0, line_no)?;
        return Ok(0x00E0);
    }

    if op == "RET" {
        expect_arg_count(&op, arguments, 0, line_no)?;
        return Ok(0x00EE);
    }

    if op == "EXIT" {
        expect_arg_count(&op, arguments, 0, line_no)?;
        return Ok(0x00FD);
    }

    if op == "JP" {
        if arguments.len() == 1 {
            let address = parse_value(&arguments[0], labels, line_no)?;
            ensure_range(address, 0, 0x0FFF, "address", line_no)?;
            return Ok(0x1000 | address as u16);
        }
        if arguments.len() == 2 {
            let x_reg = parse_register(&arguments[0], line_no)?;
            let nn = parse_value(&arguments[1], labels, line_no)?;
            ensure_range(nn, 0, 0x00FF, "byte", line_no)?;
            return Ok(0xB000 | (x_reg << 8) | nn as u16);
        }
        return Err(AssemblerError::new(
            "JP expects one or two arguments",
            Some(line_no),
        ));
    }

    if op == "CALL" {
        expect_arg_count(&op, arguments, 1, line_no)?;
        let address = parse_value(&arguments[0], labels, line_no)?;
        ensure_range(address, 0, 0x0FFF, "address", line_no)?;
        return Ok(0x2000 | address as u16);
    }

    if op == "SE" {
        expect_arg_count(&op, arguments, 2, line_no)?;
        let x_reg = parse_register(&arguments[0], line_no)?;
        if is_register(&arguments[1]) {
            let y_reg = parse_register(&arguments[1], line_no)?;
            return Ok(0x5000 | (x_reg << 8) | (y_reg << 4));
        }
        let nn = parse_value(&arguments[1], labels, line_no)?;
        ensure_range(nn, 0, 0x00FF, "byte", line_no)?;
        return Ok(0x3000 | (x_reg << 8) | nn as u16);
    }

    if op == "SNE" {
        expect_arg_count(&op, arguments, 2, line_no)?;
        let x_reg = parse_register(&arguments[0], line_no)?;
        if is_register(&arguments[1]) {
            let y_reg = parse_register(&arguments[1], line_no)?;
            return Ok(0x9000 | (x_reg << 8) | (y_reg << 4));
        }
        let nn = parse_value(&arguments[1], labels, line_no)?;
        ensure_range(nn, 0, 0x00FF, "byte", line_no)?;
        return Ok(0x4000 | (x_reg << 8) | nn as u16);
    }

    if op == "LD" {
        expect_arg_count(&op, arguments, 2, line_no)?;
        let dest = arguments[0].trim().to_ascii_uppercase();
        let src = arguments[1].trim().to_ascii_uppercase();

        if is_register(&dest) {
            let x_reg = parse_register(&dest, line_no)?;
            if is_register(&src) {
                let y_reg = parse_register(&src, line_no)?;
                return Ok(0x8000 | (x_reg << 8) | (y_reg << 4));
            }
            if src == "DT" {
                return Ok(0xF007 | (x_reg << 8));
            }
            if src == "K" {
                return Ok(0xF00A | (x_reg << 8));
            }
            if src == "[I]" {
                return Ok(0xF065 | (x_reg << 8));
            }
            let nn = parse_value(&arguments[1], labels, line_no)?;
            ensure_range(nn, 0, 0x00FF, "byte", line_no)?;
            return Ok(0x6000 | (x_reg << 8) | nn as u16);
        }

        if dest == "I" {
            let address = parse_value(&arguments[1], labels, line_no)?;
            ensure_range(address, 0, 0x0FFF, "address", line_no)?;
            return Ok(0xA000 | address as u16);
        }
        if dest == "DT" {
            return Ok(0xF015 | (parse_register(&arguments[1], line_no)? << 8));
        }
        if dest == "ST" {
            return Ok(0xF018 | (parse_register(&arguments[1], line_no)? << 8));
        }
        if dest == "F" {
            return Ok(0xF029 | (parse_register(&arguments[1], line_no)? << 8));
        }
        if dest == "B" {
            return Ok(0xF033 | (parse_register(&arguments[1], line_no)? << 8));
        }
        if dest == "[I]" {
            return Ok(0xF055 | (parse_register(&arguments[1], line_no)? << 8));
        }

        return Err(AssemblerError::new(
            format!(
                "unsupported LD form: {}, {}",
                arguments[0].trim(),
                arguments[1].trim()
            ),
            Some(line_no),
        ));
    }

    if op == "ADD" {
        expect_arg_count(&op, arguments, 2, line_no)?;
        let dest = arguments[0].trim().to_ascii_uppercase();

        if dest == "I" {
            return Ok(0xF01E | (parse_register(&arguments[1], line_no)? << 8));
        }

        let x_reg = parse_register(&arguments[0], line_no)?;
        if is_register(&arguments[1]) {
            let y_reg = parse_register(&arguments[1], line_no)?;
            return Ok(0x8004 | (x_reg << 8) | (y_reg << 4));
        }

        let nn = parse_value(&arguments[1], labels, line_no)?;
        ensure_range(nn, 0, 0x00FF, "byte", line_no)?;
        return Ok(0x7000 | (x_reg << 8) | nn as u16);
    }

    if ["OR", "AND", "XOR", "SUB", "SUBN"].contains(&op.as_str()) {
        expect_arg_count(&op, arguments, 2, line_no)?;
        let x_reg = parse_register(&arguments[0], line_no)?;
        let y_reg = parse_register(&arguments[1], line_no)?;
        let tail = match op.as_str() {
            "OR" => 0x1,
            "AND" => 0x2,
            "XOR" => 0x3,
            "SUB" => 0x5,
            "SUBN" => 0x7,
            _ => unreachable!(),
        };
        return Ok(0x8000 | (x_reg << 8) | (y_reg << 4) | tail);
    }

    if ["SHR", "SHL"].contains(&op.as_str()) {
        if arguments.len() != 1 && arguments.len() != 2 {
            return Err(AssemblerError::new(
                format!("{op} expects one or two arguments"),
                Some(line_no),
            ));
        }
        let x_reg = parse_register(&arguments[0], line_no)?;
        let y_reg = if arguments.len() == 2 {
            parse_register(&arguments[1], line_no)?
        } else {
            x_reg
        };
        let tail = if op == "SHR" { 0x6 } else { 0xE };
        return Ok(0x8000 | (x_reg << 8) | (y_reg << 4) | tail);
    }

    if op == "RND" {
        expect_arg_count(&op, arguments, 2, line_no)?;
        let x_reg = parse_register(&arguments[0], line_no)?;
        let nn = parse_value(&arguments[1], labels, line_no)?;
        ensure_range(nn, 0, 0x00FF, "byte", line_no)?;
        return Ok(0xC000 | (x_reg << 8) | nn as u16);
    }

    if op == "DRW" {
        expect_arg_count(&op, arguments, 3, line_no)?;
        let x_reg = parse_register(&arguments[0], line_no)?;
        let y_reg = parse_register(&arguments[1], line_no)?;
        let n = parse_value(&arguments[2], labels, line_no)?;
        ensure_range(n, 0, 0x000F, "nibble", line_no)?;
        return Ok(0xD000 | (x_reg << 8) | (y_reg << 4) | n as u16);
    }

    if op == "SKP" {
        expect_arg_count(&op, arguments, 1, line_no)?;
        let x_reg = parse_register(&arguments[0], line_no)?;
        return Ok(0xE09E | (x_reg << 8));
    }

    if op == "SKNP" {
        expect_arg_count(&op, arguments, 1, line_no)?;
        let x_reg = parse_register(&arguments[0], line_no)?;
        return Ok(0xE0A1 | (x_reg << 8));
    }

    Err(AssemblerError::new(
        format!("unknown instruction '{mnemonic}'"),
        Some(line_no),
    ))
}
