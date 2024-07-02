use std::num::ParseFloatError;

use anyhow::Result;
use deranged::RangedU8;
use derive_more::{From, FromStr, Into};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Represents the lightness component in HSL.
///
/// The range of valid values is
/// `(`[`Lightness::MIN`]`..=`[`Lightness::MAX`]`)`.
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Deserialize, Into, Serialize)]
pub struct Lightness(f32);

#[derive(Debug, Error)]
pub enum LightnessError {
    #[error(
        "invalid lightness {0}, expected value between {} and {}",
        Lightness::MIN,
        Lightness::MAX
    )]
    OutOfRange(f32),
}

#[derive(Debug, Error)]
pub enum ParseLightnessError {
    #[error("invalid float")]
    InvalidFloat(#[from] ParseFloatError),
    #[error("invalid lightness")]
    InvalidLightness(#[from] LightnessError),
}

/// An indexed color where the color palette is the set of colors used in
/// neofetch ascii art.
///
/// The range of valid values as supported in neofetch is
/// `(`[`NeofetchAsciiIndexedColor::MIN`]`..
/// =`[`NeofetchAsciiIndexedColor::MAX`]`)`.
#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Deserialize,
    From,
    FromStr,
    Into,
    Serialize,
)]
pub struct NeofetchAsciiIndexedColor(
    RangedU8<{ NeofetchAsciiIndexedColor::MIN }, { NeofetchAsciiIndexedColor::MAX }>,
);

/// An indexed color where the color palette is the set of unique colors in a
/// preset.
///
/// The range of valid values depends on the number of unique colors in a
/// certain preset.
#[derive(
    Copy,
    Clone,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    Debug,
    Deserialize,
    From,
    FromStr,
    Into,
    Serialize,
)]
pub struct PresetIndexedColor(usize);

impl Lightness {
    const MAX: f32 = 1.0f32;
    const MIN: f32 = 0.0f32;

    pub fn new(value: f32) -> Result<Self, LightnessError> {
        if !(Self::MIN..=Self::MAX).contains(&value) {
            return Err(LightnessError::OutOfRange(value));
        }

        Ok(Self(value))
    }
}

impl TryFrom<f32> for Lightness {
    type Error = LightnessError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Lightness::new(value)
    }
}

impl FromStr for Lightness {
    type Err = ParseLightnessError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Lightness::new(s.parse()?)?)
    }
}

impl NeofetchAsciiIndexedColor {
    const MAX: u8 = 6;
    const MIN: u8 = 1;
}
