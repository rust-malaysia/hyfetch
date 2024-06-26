use std::path::PathBuf;
use std::str::FromStr;

#[cfg(feature = "autocomplete")]
use bpaf::ShellComp;
use bpaf::{construct, long, OptionParser, Parser};
use strum::VariantNames;

use crate::presets::Preset;
use crate::types::AnsiMode;

#[derive(Clone, Debug)]
pub struct Options {
    pub config: bool,
    pub config_file: Option<PathBuf>,
    pub preset: Option<Preset>,
    pub mode: Option<AnsiMode>,
    // pub backend: Option<Backend>,
    // pub backend_args: Option<String>,
    // pub colors_scale: Option<f32>,
    // pub colors_set_lightness: Option<f32>,
    // pub colors_use_overlay: bool,
    // pub june: bool,
    // pub debug: bool,
    // pub test_distro: Option<String>,
    // pub ascii_file: Option<PathBuf>,
    // pub test_print: bool,
    // pub ask_exit: bool,
}

pub fn options() -> OptionParser<Options> {
    let config = long("config").short('c').help("Configure hyfetch").switch();
    let config_file = long("config-file")
        .short('C')
        .help("Use another config file")
        .argument("CONFIG_FILE");
    #[cfg(feature = "autocomplete")]
    let config_file = config_file.complete_shell(ShellComp::Nothing);
    let config_file = config_file.optional();
    let preset = long("preset")
        .short('p')
        .help(&*format!(
            "Use preset
PRESET={{{}}}",
            Preset::VARIANTS.join(",")
        ))
        .argument::<String>("PRESET");
    #[cfg(feature = "autocomplete")]
    let preset = preset.complete(complete_preset);
    let preset = preset.parse(|s| Preset::from_str(&s)).optional();
    let mode = long("mode")
        .short('m')
        .help(&*format!(
            "Color mode
MODE={{{}}}",
            AnsiMode::VARIANTS.join(",")
        ))
        .argument::<String>("MODE");
    #[cfg(feature = "autocomplete")]
    let mode = mode.complete(complete_mode);
    let mode = mode.parse(|s| AnsiMode::from_str(&s)).optional();
    // TODO

    construct!(Options {
        config,
        config_file,
        preset,
        mode,
        // TODO
    })
    .to_options()
    .version(env!("CARGO_PKG_VERSION"))
}

#[cfg(feature = "autocomplete")]
fn complete_preset(input: &String) -> Vec<(String, Option<String>)> {
    Preset::VARIANTS
        .iter()
        .filter_map(|&name| {
            if name.starts_with(input) {
                Some((name.to_owned(), None))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

#[cfg(feature = "autocomplete")]
fn complete_mode(input: &String) -> Vec<(String, Option<String>)> {
    AnsiMode::VARIANTS
        .iter()
        .filter_map(|&name| {
            if name.starts_with(input) {
                Some((name.to_owned(), None))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_options() {
        options().check_invariants(false)
    }
}
