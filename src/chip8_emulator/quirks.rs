use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chip8Quirks {
    pub shift_uses_vy: bool,
    pub load_store_increment_i: bool,
    pub jump_with_vx: bool,
    pub draw_wrap: bool,
}

pub const ORIGINAL_QUIRKS: Chip8Quirks = Chip8Quirks {
    shift_uses_vy: true,
    load_store_increment_i: true,
    jump_with_vx: false,
    draw_wrap: false,
};

pub const MODERN_QUIRKS: Chip8Quirks = Chip8Quirks {
    shift_uses_vy: false,
    load_store_increment_i: false,
    jump_with_vx: true,
    draw_wrap: true,
};

pub fn load_quirks_profile(profile: &str) -> Result<Chip8Quirks, String> {
    match profile.trim().to_ascii_lowercase().as_str() {
        "original" => Ok(ORIGINAL_QUIRKS),
        "modern" => Ok(MODERN_QUIRKS),
        other => Err(format!(
            "invalid CHIP8_QUIRKS '{other}', expected one of: modern, original"
        )),
    }
}

pub fn load_quirks_profile_from_env() -> Result<(String, Chip8Quirks), String> {
    let profile = env::var("CHIP8_QUIRKS").unwrap_or_else(|_| "original".to_owned());
    let normalized = profile.trim().to_ascii_lowercase();
    let quirks = load_quirks_profile(&normalized)?;
    Ok((normalized, quirks))
}
