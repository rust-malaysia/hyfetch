use std::borrow::Cow;
use std::ffi::OsStr;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{cmp, env, fmt, io};
use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use regex::Regex;
use tracing::debug;

use crate::distros::Distro;
use crate::presets::ColorProfile;
use crate::types::ColorAlignMode;
use crate::types::PathOrString;

/// Ask the user to provide an input among a list of options
/// 
/// prompt: Input prompt
/// options: Options
/// default: Default option
/// show_ops: Show options
/// return: Selection
pub fn literal_input<T>(prompt: String, options: impl IntoIterator<Item = String>, default: String, show_ops: bool) -> Result<String, >
where
    T: std::iter::Iterator
{
    let options = Vec::from_iter(options);
    
    if (show_ops) {
        // let op_text = options.join("|")
        // TODO: printc function from color_util
    } else {
        // TODO: printc function from color_util
    }
    let option_text = options.join("|");

    let mut selection = String::new();
    io::stdin().read_line(&mut selection)?;
    if (selection.is_empty()) {
        selection = default;
    }

    let mut lows = options.into_iter().map(|s| s.to_lowercase());

    loop {
        let sel = lows
            .find(|x| x == &selection || x.starts_with(&selection));
        
        match sel {
            None => {
                println!("Invalid selection! {selection} is not one of {option_text}");
                io::stdin().read_line(&mut selection)?;
            },
            Some(s) => return Ok(s)
        }
    }
}

pub fn term_size() -> io::Result<(u16, u16)> {
    termion::terminal_size()
}

/// Get distro ascii width, height ignoring color code
pub fn ascii_size(asc: &String) -> Result<(usize, usize)> {
    let re_neofetch_color = Regex::new("\\${c[0-9]}").unwrap();
    re_neofetch_color.replace_all(asc, "");
    let width = asc.split("\n").fold(0, |acc, line| cmp::max(acc, line.len()));
    let height = asc.split("\n").clone().count();
    return Ok((width, height));
}

/// Make sure every line are the same width
pub fn normalize_ascii(asc: String) -> String {
    let width = ascii_size(&asc).unwrap().0;
    let lines: String = asc.split("\n").into_iter().fold("".to_string(),|acc, line| format!("{}{:width$}\n", acc, line));
    return lines;
}

/// Fill the missing starting placeholders.
pub fn fill_starting(asc: String) -> String {
    todo!()
}

/// Return the file if the file exists, or return none. Useful for chaining 'or's
pub fn if_file(f: PathOrString) -> Option<PathBuf> {
    let file: PathBuf;
    match f {
        PathOrString::P(path_buf) => file = path_buf,
        PathOrString::S(path_string) => file = PathBuf::from(path_string)
    }
    if file.as_path().is_file() {
        return Some(file.to_path_buf());
    }
    return None;
}

struct ColorAlignment {
    mode: ColorAlignMode,
    custom_colors: HashMap<u16, u16>,
    fore_back: Option<(u16, u16)>
}

impl ColorAlignment {
    /// Use the color alignment to recolor an ascii art
    fn recolor_ascii(&self, asc: String, preset: ColorProfile) -> String {
        let asc = fill_starting(asc);

        match self.mode {
            ColorAlignMode::Horizontal => {},
            ColorAlignMode::Vertical => {},
            ColorAlignMode::Custom => {
                preset = preset.unique_colors();
            }
        }
    }
}

/// Gets the absolute path of the neofetch command.
pub fn get_command_path() -> Result<PathBuf> {
    if let Ok(workspace_dir) = env::var("CARGO_WORKSPACE_DIR") {
        let path = Path::new(&workspace_dir);
        if path.exists() {
            let path = path.join("neofetch");
            match path.try_exists() {
                Ok(true) => {
                    return path.canonicalize().context("Failed to canonicalize path");
                },
                Ok(false) => {
                    Err(anyhow!("{path:?} does not exist or is not readable"))?;
                },
                Err(err) => {
                    Err(err)
                        .with_context(|| format!("Failed to check for existence of {path:?}"))?;
                },
            }
        }
    }

    let Ok(path_env) = env::var("PATH") else {
        return Err(anyhow!("`PATH` env var is not set or invalid"));
    };

    for search_path in env::split_paths(&path_env) {
        let path = search_path.join("neowofetch");
        if !path.is_file() {
            continue;
        }
        return path.canonicalize().context("Failed to canonicalize path");
    }

    Err(anyhow!("neofetch command not found"))
}

/// Gets the distro ascii of the current distro. Or if distro is specified, get
/// the specific distro's ascii art instead.
#[tracing::instrument(level = "debug")]
pub fn get_distro_ascii<S>(distro: Option<S>) -> Result<String>
where
    S: AsRef<str> + fmt::Debug,
{
    let distro: Cow<_> = if let Some(distro) = distro.as_ref() {
        distro.as_ref().into()
    } else {
        get_distro_name()
            .context("Failed to get distro name")?
            .into()
    };
    debug!(%distro, "distro name");

    if let Some(distro) = Distro::detect(&distro) {
        return Ok(distro.ascii_art().to_owned());
    }

    todo!()
}

/// Runs neofetch command, returning the piped stdout output.
#[tracing::instrument(level = "debug")]
fn run_neofetch_command_piped<S>(args: &[S]) -> Result<String>
where
    S: AsRef<OsStr> + fmt::Debug,
{
    let mut command = make_neofetch_command(args).context("Failed to make neofetch command")?;

    let output = command
        .output()
        .context("Failed to execute neofetch as child process")?;
    debug!(?output, "neofetch output");

    if !output.status.success() {
        let err = if let Some(code) = output.status.code() {
            anyhow!("neofetch process exited with status code: {code}")
        } else {
            #[cfg(unix)]
            {
                anyhow!(
                    "neofetch process terminated by signal: {}",
                    output
                        .status
                        .signal()
                        .expect("either one of status code or signal should be set")
                )
            }
            #[cfg(not(unix))]
            unimplemented!("status code not expected to be `None` on non-Unix platforms")
        };
        Err(err)?;
    }

    let out = String::from_utf8(output.stdout)
        .context("Failed to process neofetch output as it contains invalid UTF-8")?
        .trim()
        .to_owned();
    Ok(out)
}

fn make_neofetch_command<S>(args: &[S]) -> Result<Command>
where
    S: AsRef<OsStr>,
{
    #[cfg(not(windows))]
    {
        let mut command = Command::new("bash");
        command.arg(get_command_path().context("Failed to get neofetch command path")?);
        command.args(args);
        Ok(command)
    }
    #[cfg(windows)]
    {
        todo!()
    }
}

#[tracing::instrument(level = "debug")]
fn get_distro_name() -> Result<String> {
    run_neofetch_command_piped(&["ascii_distro_name"])
        .context("Failed to get distro name from neofetch")
}
