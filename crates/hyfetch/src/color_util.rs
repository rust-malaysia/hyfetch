use anyhow::{anyhow, Context, Result};
use rgb::RGB8;

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
