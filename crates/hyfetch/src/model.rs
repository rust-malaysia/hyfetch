use crate::presets::Preset;
use serde::{Deserialize, Serialize};
use std::fs;
//use types::AnsiMode;

#[derive(Serialize, Deserialize, Debug)]
struct MyStruct {
    preset: String,
    mode: CustomMode,
    light_dark: CustomLightDark,
    lightness: CustomLightness,
    color_align: CustomColorAlign,
    backend: CustomBackend,
    args: String,
    distro: String,
    pride_month_shown: Vec<i32>, // This is deprecated
    pride_month_disable: bool,
}

#[derive(Serialize, Deserialize, Debug)]
enum CustomMode {
    Default,
    Ansi,
    #[serde(rename = "8bit")]
    Eightbit,
    Rgb,
}

#[derive(Serialize, Deserialize, Debug)]
enum CustomLightDark {
    Light,
    Dark,
}

#[derive(Serialize, Deserialize, Debug)]
enum CustomLightness {
    Float,
    None,
}

#[derive(Serialize, Deserialize, Debug)]
enum CustomColorAlign {
    Horizontal,
}

#[derive(Serialize, Deserialize, Debug)]
enum CustomBackend {
    Neofetch,
}
