use anyhow::{anyhow, Context, Result};
use rgb::RGB8;

use crate::types::AnsiMode;

const MINECRAFT_COLORS: [(&str, &str); 30] = [
    // Minecraft formatting codes
    ("&0", "\x1b[38;5;0m"),
    ("&1", "\x1b[38;5;4m"),
    ("&2", "\x1b[38;5;2m"),
    ("&3", "\x1b[38;5;6m"),
    ("&4", "\x1b[38;5;1m"),
    ("&5", "\x1b[38;5;5m"),
    ("&6", "\x1b[38;5;3m"),
    ("&7", "\x1b[38;5;7m"),
    ("&8", "\x1b[38;5;8m"),
    ("&9", "\x1b[38;5;12m"),
    ("&a", "\x1b[38;5;10m"),
    ("&b", "\x1b[38;5;14m"),
    ("&c", "\x1b[38;5;9m"),
    ("&d", "\x1b[38;5;13m"),
    ("&e", "\x1b[38;5;11m"),
    ("&f", "\x1b[38;5;15m"),
    // Enable bold text
    ("&l", "\x1b[1m"),
    // Enable italic text
    ("&o", "\x1b[3m"),
    // Enable underlined text
    ("&n", "\x1b[4m"),
    // Enable hidden text
    ("&k", "\x1b[8m"),
    // Enable strikethrough text
    ("&m", "\x1b[9m"),
    // Reset everything
    ("&r", "\x1b[0m"),
    // Line break
    ("&-", "\n"),
    // Reset text color
    ("&~", "\x1b[39m"),
    // Reset background color
    ("&*", "\x1b[49m"),
    // Disable bold text
    ("&L", "\x1b[22m"),
    // Disable italic text
    ("&O", "\x1b[23m"),
    // Disable underlined text
    ("&N", "\x1b[24m"),
    // Disable hidden text
    ("&K", "\x1b[28m"),
    // Disable strikethrough text
    ("&M", "\x1b[29m"),
];

pub fn color(mut msg: String) -> Result<String> {
    for (code, esc) in MINECRAFT_COLORS {
        msg = msg.replace(code, esc)
    }

    while msg.contains("&gf(") || msg.contains("&gb(") {
        let i = if msg.contains("&gf(") {
            msg.find("&gf(").context("invalid msg format")?
        } else {
            msg.find("&gb(").context("invalid msg format")?
        };
        let end = msg[i..].find(')').context("invalid msg format")? + i;
        let code = &msg[i + 4..end];
        let fore = &msg[i + 2..i + 3] == "f";

        let rgb = if let Some(hex_code) = code.strip_prefix('#') {
            if hex_code.len() != 6 {
                return Err(anyhow!("invalid format"));
            }

            (
                u8::from_str_radix(&hex_code[0..2], 16).context("invalid msg format")?,
                u8::from_str_radix(&hex_code[2..4], 16).context("invalid msg format")?,
                u8::from_str_radix(&hex_code[4..6], 16).context("invalid msg format")?,
            )
        } else {
            let code = code.replace([',', ';'], " ").replace("  ", " ");
            let splits: Vec<&str> = code.split(' ').collect();
            (
                splits[0].parse().context("error parsing")?,
                splits[1].parse().context("error parsing")?,
                splits[2].parse().context("error parsing")?,
            )
        };

        let ansi_code = rgb::RGB8 {
            r: rgb.0,
            g: rgb.1,
            b: rgb.2,
        };
        msg = format!(
            "{}{}{}",
            &msg[..i],
            ansi_code.to_ansi(None, fore),
            &msg[end + 1..]
        );
    }
    Ok(msg)
}

pub trait FromHex {
    /// Creates color from hex code.
    fn from_hex<S>(hex: S) -> Result<RGB8>
    where
        S: AsRef<str>;
}

impl FromHex for RGB8 {
    fn from_hex<S>(hex: S) -> Result<RGB8>
    where
        S: AsRef<str>,
    {
        let hex = hex.as_ref();

        let hex = hex.strip_prefix('#').unwrap_or(hex);
        if hex.len() != 6 {
            Err(anyhow!("invalid length for hex color"))?;
        }

        let r =
            u8::from_str_radix(&hex[0..2], 16).context("Failed to parse hex color component")?;
        let g =
            u8::from_str_radix(&hex[2..4], 16).context("Failed to parse hex color component")?;
        let b =
            u8::from_str_radix(&hex[4..6], 16).context("Failed to parse hex color component")?;

        Ok(RGB8::new(r, g, b))
    }
}

pub trait ToAnsi {
    fn to_ansi(&self, mode: Option<AnsiMode>, foreground: bool) -> String;
    fn to_ansi_8bit(&self, foreground: bool) -> String;
    fn to_ansi_rgb(&self, foreground: bool) -> String;
}

impl ToAnsi for RGB8 {
    fn to_ansi(&self, mode: Option<AnsiMode>, foreground: bool) -> String {
        match mode {
            Some(AnsiMode::Rgb) => self.to_ansi_rgb(foreground),
            Some(AnsiMode::Ansi256) => self.to_ansi_8bit(foreground),
            _ => self.to_ansi_rgb(foreground),
        }
    }

    fn to_ansi_rgb(&self, foreground: bool) -> String {
        let c = if foreground { "38" } else { "48" };

        format!("\x1b[{c};2;{};{};{}m", self.r, self.g, self.b)
    }

    fn to_ansi_8bit(&self, foreground: bool) -> String {
        let (r, g, b) = (self.r as f64, self.g as f64, self.b as f64);
        let mut sep = 42.5;

        let gray;
        loop {
            if r < sep || g < sep || b < sep {
                gray = r < sep && g < sep && b < sep;
                break;
            }
            sep += 42.5;
        }

        let color: i32 = if gray {
            232 + (r + g + b) as i32 / 33
        } else {
            16 + (r / 256.0 * 6.0) as i32 * 36
                + (g / 256.0 * 6.0) as i32 * 6
                + (b / 256.0 * 6.0) as i32
        };

        let c = if foreground { "38" } else { "48" };
        format!("\x1b[{};5;{}m", c, color)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_minecraft_codes() {
        let input = "&aHello &bWorld!";
        let expected = "\u{1b}[38;5;10mHello \u{1b}[38;5;14mWorld!";
        assert_eq!(color(input.to_string()).unwrap(), expected);
    }

    #[test]
    fn test_color_hex_code() {
        let input = "Hello &gf(#FF0000)Red&gf(#00FF00)Green&gf(#0000FF)Blue";
        let expected =
            "Hello \u{1b}[38;2;255;0;0mRed\u{1b}[38;2;0;255;0mGreen\u{1b}[38;2;0;0;255mBlue";
        assert_eq!(color(input.to_string()).unwrap(), expected);
    }

    #[test]
    fn test_color_rgb_code() {
        let input = "Hello &gf(255, 0, 0)Red&gf(0, 255, 0)Green&gf(0, 0, 255)Blue";
        let expected =
            "Hello \u{1b}[38;2;255;0;0mRed\u{1b}[38;2;0;255;0mGreen\u{1b}[38;2;0;0;255mBlue";
        assert_eq!(color(input.to_string()).unwrap(), expected);
    }

    #[test]
    fn test_color_invalid_format() {
        let input = "Hello &gf(#FF00)Invalid";
        assert!(color(input.to_string()).is_err());
    }
}
