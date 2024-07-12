use std::borrow::Cow;
use std::ffi::OsStr;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::{env, fmt};

use aho_corasick::AhoCorasick;
use anyhow::{anyhow, Context, Result};
use indexmap::IndexMap;
#[cfg(windows)]
use normpath::PathExt as _;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;
use tracing::debug;
use unicode_segmentation::UnicodeSegmentation;

use crate::color_util::{
    color, printc, ForegroundBackground, NeofetchAsciiIndexedColor, PresetIndexedColor,
    ToAnsiString,
};
use crate::distros::Distro;
use crate::presets::ColorProfile;
use crate::types::{AnsiMode, Backend, LightDark};
use crate::utils::{find_file, find_in_path, process_command_status};

const NEOFETCH_COLOR_PATTERNS: [&str; 6] = ["${c1}", "${c2}", "${c3}", "${c4}", "${c5}", "${c6}"];
static NEOFETCH_COLORS_AC: OnceLock<AhoCorasick> = OnceLock::new();

type ForeBackColorPair = (NeofetchAsciiIndexedColor, NeofetchAsciiIndexedColor);

#[derive(Clone, Eq, PartialEq, Debug, Deserialize, Serialize)]
#[serde(tag = "mode")]
#[serde(rename_all = "lowercase")]
pub enum ColorAlignment {
    Horizontal {
        #[serde(skip)]
        fore_back: Option<ForeBackColorPair>,
    },
    Vertical {
        #[serde(skip)]
        fore_back: Option<ForeBackColorPair>,
    },
    Custom {
        #[serde(rename = "custom_colors")]
        #[serde(deserialize_with = "crate::utils::index_map_serde::deserialize")]
        colors: IndexMap<NeofetchAsciiIndexedColor, PresetIndexedColor>,
    },
}

impl ColorAlignment {
    /// Creates a new color alignment, with the specified foreground-background
    /// configuration.
    pub fn with_fore_back(&self, fore_back: Option<ForeBackColorPair>) -> Result<Self> {
        match self {
            Self::Horizontal { .. } => Ok(Self::Horizontal { fore_back }),
            Self::Vertical { .. } => {
                if fore_back.is_some() {
                    debug!(
                        "foreground-background configuration not implemented for vertical color \
                         alignment; ignoring"
                    );
                }
                Ok(Self::Vertical { fore_back: None })
            },
            Self::Custom { colors } => {
                if fore_back.is_some() {
                    return Err(anyhow!(
                        "foreground-background configuration not supported for custom colors"
                    ));
                }
                Ok(Self::Custom {
                    colors: colors.clone(),
                })
            },
        }
    }

    /// Uses the color alignment to recolor an ascii art.
    #[tracing::instrument(level = "debug", skip(asc))]
    pub fn recolor_ascii(
        &self,
        asc: String,
        color_profile: ColorProfile,
        color_mode: AnsiMode,
        term: LightDark,
    ) -> Result<String> {
        let reset = color("&~&*", color_mode).expect("color reset should not be invalid");

        let asc = match self {
            &Self::Horizontal {
                fore_back: Some((fore, back)),
            }
            | &Self::Vertical {
                fore_back: Some((fore, back)),
            } => {
                let fore: u8 = fore.into();
                let back: u8 = back.into();

                let asc = fill_starting(asc)
                    .context("failed to fill in starting neofetch color codes")?;

                // Replace foreground colors
                let asc = asc.replace(
                    &format!("${{c{fore}}}"),
                    &color(
                        match term {
                            LightDark::Light => "&0",
                            LightDark::Dark => "&f",
                        },
                        color_mode,
                    )
                    .expect("foreground color should not be invalid"),
                );

                let lines: Vec<_> = asc.split('\n').collect();

                // Add new colors
                let asc = match self {
                    Self::Horizontal { .. } => {
                        let ColorProfile { colors } = {
                            let length = lines.len();
                            let length: u8 =
                                length.try_into().expect("`length` should fit in `u8`");
                            color_profile
                                .with_length(length)
                                .context("failed to spread color profile to length")?
                        };
                        let mut asc = String::new();
                        for (i, line) in lines.into_iter().enumerate() {
                            let line = line.replace(
                                &format!("${{c{back}}}"),
                                &colors[i].to_ansi_string(color_mode, {
                                    // note: this is "background" in the ascii art, but foreground
                                    // text in terminal
                                    ForegroundBackground::Foreground
                                }),
                            );
                            asc.push_str(&line);
                            asc.push_str(&reset);
                            asc.push('\n');
                        }
                        asc
                    },
                    Self::Vertical { .. } => {
                        unimplemented!(
                            "vertical color alignment with fore and back colors not implemented"
                        );
                    },
                    _ => {
                        unreachable!();
                    },
                };

                // Remove existing colors
                let asc = {
                    let ac = NEOFETCH_COLORS_AC
                        .get_or_init(|| AhoCorasick::new(NEOFETCH_COLOR_PATTERNS).unwrap());
                    const N: usize = NEOFETCH_COLOR_PATTERNS.len();
                    const REPLACEMENTS: [&str; N] = [""; N];
                    ac.replace_all(&asc, &REPLACEMENTS)
                };

                asc
            },
            Self::Horizontal { fore_back: None } | Self::Vertical { fore_back: None } => {
                // Remove existing colors
                let asc = {
                    let ac = NEOFETCH_COLORS_AC
                        .get_or_init(|| AhoCorasick::new(NEOFETCH_COLOR_PATTERNS).unwrap());
                    const N: usize = NEOFETCH_COLOR_PATTERNS.len();
                    const REPLACEMENTS: [&str; N] = [""; N];
                    ac.replace_all(&asc, &REPLACEMENTS)
                };

                let lines: Vec<_> = asc.split('\n').collect();

                // Add new colors
                match self {
                    Self::Horizontal { .. } => {
                        let ColorProfile { colors } = {
                            let length = lines.len();
                            let length: u8 =
                                length.try_into().expect("`length` should fit in `u8`");
                            color_profile
                                .with_length(length)
                                .context("failed to spread color profile to length")?
                        };
                        let mut asc = String::new();
                        for (i, line) in lines.into_iter().enumerate() {
                            asc.push_str(
                                &colors[i]
                                    .to_ansi_string(color_mode, ForegroundBackground::Foreground),
                            );
                            asc.push_str(line);
                            asc.push_str(&reset);
                            asc.push('\n');
                        }
                        asc
                    },
                    Self::Vertical { .. } => {
                        let mut asc = String::new();
                        for line in lines {
                            let line = color_profile
                                .color_text(
                                    line,
                                    color_mode,
                                    ForegroundBackground::Foreground,
                                    false,
                                )
                                .context("failed to color text using color profile")?;
                            asc.push_str(&line);
                            asc.push_str(&reset);
                            asc.push('\n');
                        }
                        asc
                    },
                    _ => {
                        unreachable!();
                    },
                }
            },
            Self::Custom {
                colors: custom_colors,
            } => {
                let asc = fill_starting(asc)
                    .context("failed to fill in starting neofetch color codes")?;

                let ColorProfile { colors } = color_profile.unique_colors();

                // Apply colors
                let asc = {
                    let ac = NEOFETCH_COLORS_AC
                        .get_or_init(|| AhoCorasick::new(NEOFETCH_COLOR_PATTERNS).unwrap());
                    const N: usize = NEOFETCH_COLOR_PATTERNS.len();
                    let mut replacements = vec![Cow::from(""); N];
                    for (&ai, &pi) in custom_colors {
                        let ai: u8 = ai.into();
                        let pi: u8 = pi.into();
                        replacements[ai as usize - 1] = colors[pi as usize]
                            .to_ansi_string(color_mode, ForegroundBackground::Foreground)
                            .into();
                    }
                    ac.replace_all(&asc, &replacements)
                };

                asc
            },
        };

        Ok(asc)
    }

    /// Gets recommended foreground-background configuration for distro, or
    /// `None` if the distro ascii is not suitable for fore-back configuration.
    pub fn fore_back(distro: Distro) -> Option<ForeBackColorPair> {
        match distro {
            Distro::Anarchy
            | Distro::ArchStrike
            | Distro::Astra_Linux
            | Distro::Chapeau
            | Distro::Fedora
            | Distro::GalliumOS
            | Distro::KrassOS
            | Distro::Kubuntu
            | Distro::Lubuntu
            | Distro::openEuler
            | Distro::Peppermint
            | Distro::Pop__OS
            | Distro::Ubuntu_Cinnamon
            | Distro::Ubuntu_Kylin
            | Distro::Ubuntu_MATE
            | Distro::Ubuntu_old
            | Distro::Ubuntu_Studio
            | Distro::Ubuntu_Sway
            | Distro::Ultramarine_Linux
            | Distro::Univention
            | Distro::Vanilla
            | Distro::Xubuntu => Some((2u8.try_into().unwrap(), 1u8.try_into().unwrap())),

            Distro::Antergos => Some((1u8.try_into().unwrap(), 2u8.try_into().unwrap())),

            _ => None,
        }
    }
}

/// Asks the user to provide an input among a list of options.
pub fn literal_input<'a, S>(
    prompt: S,
    options: &[&'a str],
    default: &str,
    show_options: bool,
    color_mode: AnsiMode,
) -> Result<&'a str>
where
    S: AsRef<str>,
{
    let prompt = prompt.as_ref();

    if show_options {
        let options_text = options
            .iter()
            .map(|&o| {
                if o == default {
                    format!("&l&n{o}&L&N")
                } else {
                    o.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("|");
        printc(format!("{prompt} ({options_text})"), color_mode)
            .context("failed to print input prompt")?;
    } else {
        printc(format!("{prompt} (default: {default})"), color_mode)
            .context("failed to print input prompt")?;
    }

    let find_selection = |sel: &str| {
        if sel.is_empty() {
            return None;
        }

        // Find exact match
        if let Some(selected) = options.iter().find(|&&o| o.to_lowercase() == sel) {
            return Some(selected);
        }

        // Find starting abbreviation
        if let Some(selected) = options.iter().find(|&&o| o.to_lowercase().starts_with(sel)) {
            return Some(selected);
        }

        None
    };

    loop {
        let mut buf = String::new();
        print!("> ");
        io::stdout().flush()?;
        io::stdin()
            .read_line(&mut buf)
            .context("failed to read line from input")?;
        let selection = {
            let selection = buf.trim_end_matches(&['\r', '\n']);
            if selection.is_empty() {
                default.to_owned()
            } else {
                selection.to_lowercase()
            }
        };

        if let Some(selected) = find_selection(&selection) {
            println!();

            return Ok(selected);
        } else {
            let options_text = options.join("|");
            println!("Invalid selection! {selection} is not one of {options_text}");
        }
    }
}

/// Gets the absolute path of the neofetch command.
pub fn neofetch_path() -> Result<Option<PathBuf>> {
    if let Some(workspace_dir) = env::var_os("CARGO_WORKSPACE_DIR") {
        debug!(
            ?workspace_dir,
            "CARGO_WORKSPACE_DIR env var is set; using neofetch from project directory"
        );
        let workspace_path = Path::new(&workspace_dir);
        let workspace_path = match workspace_path.try_exists() {
            Ok(true) => workspace_path,
            Ok(false) => {
                return Err(anyhow!(
                    "{workspace_path:?} does not exist or is not readable"
                ));
            },
            Err(err) => {
                return Err(err).with_context(|| {
                    format!("failed to check for existence of {workspace_path:?}")
                });
            },
        };
        let neofetch_path = workspace_path.join("neofetch");
        return find_file(&neofetch_path)
            .with_context(|| format!("failed to check existence of file {neofetch_path:?}"));
    }

    let neowofetch_path = find_in_path("neowofetch")
        .context("failed to check existence of `neowofetch` in `PATH`")?;

    // Fall back to `neowofetch` in directory of current executable
    let neowofetch_path = if neowofetch_path.is_some() {
        neowofetch_path
    } else {
        let current_exe_path = env::current_exe()
            .and_then(|p| {
                #[cfg(not(windows))]
                {
                    p.canonicalize()
                }
                #[cfg(windows)]
                {
                    p.normalize().map(|p| p.into())
                }
            })
            .context("failed to get path of current running executable")?;
        let neowofetch_path = current_exe_path
            .parent()
            .expect("parent should not be `None`")
            .join("neowofetch");
        find_file(&neowofetch_path)
            .with_context(|| format!("failed to check existence of file {neowofetch_path:?}"))?
    };

    Ok(neowofetch_path)
}

/// Ensures git bash installation for Windows.
///
/// Returns the path to git bash.
#[cfg(windows)]
pub fn ensure_git_bash() -> Result<PathBuf> {
    let git_bash_path = {
        // Bundled git bash
        let current_exe_path = env::current_exe()
            .and_then(|p| p.normalize().map(|p| p.into()))
            .context("failed to get path of current running executable")?;
        let bash_path = current_exe_path
            .parent()
            .expect("parent should not be `None`")
            .join("git/bin/bash.exe");
        if bash_path.is_file() {
            Some(bash_path)
        } else {
            None
        }
    };
    let git_bash_path = git_bash_path.or_else(|| {
        let program_files_path = env::var_os("ProgramFiles")?;
        let bash_path = Path::new(&program_files_path).join("Git/bin/bash.exe");
        if bash_path.is_file() {
            Some(bash_path)
        } else {
            None
        }
    });
    let git_bash_path = git_bash_path.or_else(|| {
        let program_files_x86_path = env::var_os("ProgramFiles(x86)")?;
        let bash_path = Path::new(&program_files_x86_path).join("Git/bin/bash.exe");
        if bash_path.is_file() {
            Some(bash_path)
        } else {
            None
        }
    });

    let git_bash_path = git_bash_path.context("failed to find git bash executable")?;

    Ok(git_bash_path)
}

/// Gets the distro ascii of the current distro. Or if distro is specified, get
/// the specific distro's ascii art instead.
#[tracing::instrument(level = "debug")]
pub fn get_distro_ascii<S>(distro: Option<S>) -> Result<(String, Option<ForeBackColorPair>)>
where
    S: AsRef<str> + fmt::Debug,
{
    let distro: Cow<_> = if let Some(distro) = distro.as_ref() {
        distro.as_ref().into()
    } else {
        get_distro_name()
            .context("failed to get distro name")?
            .into()
    };
    debug!(%distro, "distro name");

    // Try new codegen-based detection method
    if let Some(distro) = Distro::detect(&distro) {
        return Ok((
            normalize_ascii(distro.ascii_art()),
            ColorAlignment::fore_back(distro),
        ));
    }

    debug!(%distro, "could not find a match for distro; falling back to neofetch");

    // Old detection method that calls neofetch
    let asc = run_neofetch_command_piped(&["print_ascii", "--ascii_distro", distro.as_ref()])
        .context("failed to get ascii art from neofetch")?;

    // Unescape backslashes here because backslashes are escaped in neofetch for
    // printf
    let asc = asc.replace(r"\\", r"\");

    Ok((normalize_ascii(asc), None))
}

pub fn run(asc: String, backend: Backend, args: Option<&Vec<String>>) -> Result<()> {
    match backend {
        Backend::Neofetch => {
            run_neofetch(asc, args).context("failed to run neofetch")?;
        },
        Backend::Fastfetch => {
            run_fastfetch(asc, args, false).context("failed to run fastfetch")?;
        },
        Backend::FastfetchOld => {
            run_fastfetch(asc, args, true).context("failed to run fastfetch")?;
        },
        Backend::Qwqfetch => {
            todo!();
        },
    }

    Ok(())
}

/// Gets distro ascii width and height, ignoring color code.
pub fn ascii_size<S>(asc: S) -> (u8, u8)
where
    S: AsRef<str>,
{
    let asc = asc.as_ref();

    let asc = {
        let ac =
            NEOFETCH_COLORS_AC.get_or_init(|| AhoCorasick::new(NEOFETCH_COLOR_PATTERNS).unwrap());
        const N: usize = NEOFETCH_COLOR_PATTERNS.len();
        const REPLACEMENTS: [&str; N] = [""; N];
        ac.replace_all(asc, &REPLACEMENTS)
    };

    let Some(width) = asc
        .split('\n')
        .map(|line| line.graphemes(true).count())
        .max()
    else {
        unreachable!();
    };
    let width: u8 = width.try_into().expect("`width` should fit in `u8`");
    let height = asc.split('\n').count();
    let height: u8 = height.try_into().expect("`height` should fit in `u8`");

    (width, height)
}

/// Makes sure every line are the same width.
fn normalize_ascii<S>(asc: S) -> String
where
    S: AsRef<str>,
{
    let asc = asc.as_ref();

    let (w, _) = ascii_size(asc);

    let mut buf = String::new();
    for line in asc.split('\n') {
        let (line_w, _) = ascii_size(line);
        buf.push_str(line);
        let pad = " ".repeat((w - line_w) as usize);
        buf.push_str(&pad);
        buf.push('\n');
    }

    buf
}

/// Fills the missing starting placeholders.
///
/// e.g. `"${c1}...\n..."` -> `"${c1}...\n${c1}..."`
fn fill_starting<S>(asc: S) -> Result<String>
where
    S: AsRef<str>,
{
    let asc = asc.as_ref();

    let ac = NEOFETCH_COLORS_AC.get_or_init(|| AhoCorasick::new(NEOFETCH_COLOR_PATTERNS).unwrap());

    let mut new = String::new();
    let mut last = None;
    for line in asc.split('\n') {
        let mut matches = ac.find_iter(line).peekable();

        match matches.peek() {
            Some(m) if m.start() == 0 || line[0..m.start()].trim_end_matches(' ').is_empty() => {
                // line starts with neofetch color code, do nothing
            },
            _ => {
                new.push_str(
                    last.context("failed to find neofetch color code from a previous line")?,
                );
            },
        }
        new.push_str(line);
        new.push('\n');

        // Get the last placeholder for the next line
        if let Some(m) = matches.last() {
            last = Some(&line[m.span()])
        }
    }

    Ok(new)
}

/// Runs neofetch command, returning the piped stdout output.
fn run_neofetch_command_piped<S>(args: &[S]) -> Result<String>
where
    S: AsRef<OsStr> + fmt::Debug,
{
    let mut command = make_neofetch_command(args)?;

    let output = command
        .output()
        .context("failed to execute neofetch as child process")?;
    debug!(?output, "neofetch output");
    process_command_status(&output.status).context("neofetch command exited with error")?;

    let out = String::from_utf8(output.stdout)
        .context("failed to process neofetch output as it contains invalid UTF-8")?
        .trim()
        .to_owned();
    Ok(out)
}

fn make_neofetch_command<S>(args: &[S]) -> Result<Command>
where
    S: AsRef<OsStr>,
{
    let neofetch_path = neofetch_path().context("failed to get neofetch path")?;
    let neofetch_path = neofetch_path.context("neofetch command not found")?;

    debug!(?neofetch_path, "neofetch path");

    #[cfg(not(windows))]
    {
        let mut command = Command::new("bash");
        command.arg(neofetch_path);
        command.args(args);
        Ok(command)
    }
    #[cfg(windows)]
    {
        let git_bash_path = ensure_git_bash().context("failed to get git bash path")?;
        let mut command = Command::new(git_bash_path);
        command.arg(neofetch_path);
        command.args(args);
        Ok(command)
    }
}

#[tracing::instrument(level = "debug")]
fn get_distro_name() -> Result<String> {
    run_neofetch_command_piped(&["ascii_distro_name"])
        .context("failed to get distro name from neofetch")
}

/// Runs neofetch with colors.
#[tracing::instrument(level = "debug", skip(asc))]
fn run_neofetch(asc: String, args: Option<&Vec<String>>) -> Result<()> {
    // Escape backslashes here because backslashes are escaped in neofetch for
    // printf
    let asc = asc.replace('\\', r"\\");

    // Write temp file
    let mut temp_file =
        NamedTempFile::with_prefix("ascii.txt").context("failed to create temp file for ascii")?;
    temp_file
        .write_all(asc.as_bytes())
        .context("failed to write ascii to temp file")?;

    // Call neofetch with the temp file
    let temp_file_path = temp_file.into_temp_path();
    let args = {
        let mut v = vec![
            "--ascii",
            "--source",
            temp_file_path
                .to_str()
                .expect("temp file path should not contain invalid UTF-8"),
            "--ascii-colors",
        ];
        if let Some(args) = args {
            let args: Vec<_> = args.iter().map(|s| &**s).collect();
            v.extend(args);
        }
        v
    };
    let mut command = make_neofetch_command(&args)?;

    debug!(?command, "neofetch command");

    let status = command
        .status()
        .context("failed to execute neofetch command as child process")?;
    process_command_status(&status).context("neofetch command exited with error")?;

    Ok(())
}

fn fastfetch_path() -> Result<Option<PathBuf>> {
    let fastfetch_path =
        find_in_path("fastfetch").context("failed to check existence of `fastfetch` in `PATH`")?;

    // Fall back to `fastfetch` in directory of current executable
    let current_exe_path = env::current_exe()
        .and_then(|p| {
            #[cfg(not(windows))]
            {
                p.canonicalize()
            }
            #[cfg(windows)]
            {
                p.normalize().map(|p| p.into())
            }
        })
        .context("failed to get path of current running executable")?;
    let current_exe_dir_path = current_exe_path
        .parent()
        .expect("parent should not be `None`");
    let fastfetch_path = if fastfetch_path.is_some() {
        fastfetch_path
    } else {
        let fastfetch_path = current_exe_dir_path.join("fastfetch");
        find_file(&fastfetch_path)
            .with_context(|| format!("failed to check existence of file {fastfetch_path:?}"))?
    };

    // Bundled fastfetch
    #[cfg(unix)]
    let fastfetch_path = if fastfetch_path.is_some() {
        fastfetch_path
    } else {
        let fastfetch_path = current_exe_dir_path.join("fastfetch/usr/bin/fastfetch");
        find_file(&fastfetch_path)
            .with_context(|| format!("failed to check existence of file {fastfetch_path:?}"))?
    };
    let fastfetch_path = if fastfetch_path.is_some() {
        fastfetch_path
    } else {
        let fastfetch_path = current_exe_dir_path.join("fastfetch/fastfetch");
        find_file(&fastfetch_path)
            .with_context(|| format!("failed to check existence of file {fastfetch_path:?}"))?
    };
    #[cfg(windows)]
    let fastfetch_path = if fastfetch_path.is_some() {
        fastfetch_path
    } else {
        let fastfetch_path = current_exe_dir_path.join("fastfetch/fastfetch.exe");
        find_file(&fastfetch_path)
            .with_context(|| format!("failed to check existence of file {fastfetch_path:?}"))?
    };

    Ok(fastfetch_path)
}

/// Runs fastfetch with colors.
#[tracing::instrument(level = "debug", skip(asc))]
fn run_fastfetch(asc: String, args: Option<&Vec<String>>, legacy: bool) -> Result<()> {
    // Find fastfetch binary
    let fastfetch_path = fastfetch_path().context("failed to get fastfetch path")?;
    let fastfetch_path = fastfetch_path.context("fastfetch command not found")?;

    debug!(?fastfetch_path, "fastfetch path");

    // Write temp file
    let mut temp_file =
        NamedTempFile::with_prefix("ascii.txt").context("failed to create temp file for ascii")?;
    temp_file
        .write_all(asc.as_bytes())
        .context("failed to write ascii to temp file")?;

    // Call fastfetch with the temp file
    let temp_file_path = temp_file.into_temp_path();
    let mut command = Command::new(fastfetch_path);
    command.arg(if legacy { "--raw" } else { "--file-raw" });
    command.arg(&temp_file_path);
    if let Some(args) = args {
        command.args(args);
    }

    debug!(?command, "fastfetch command");

    let status = command
        .status()
        .context("failed to execute fastfetch command as child process")?;
    if status.code() == Some(144) {
        eprintln!(
            "exit code 144 detected; please upgrade fastfetch to >=1.8.0 or use the \
             'fastfetch-old' backend"
        );
    }
    process_command_status(&status).context("fastfetch command exited with error")?;

    Ok(())
}
