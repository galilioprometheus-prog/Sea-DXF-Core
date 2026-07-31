//! Stable command-line definition and parsed option contract.

use std::{
    io::{self, Write},
    path::PathBuf,
};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use seacad_dxf_core::{DxfReadMode, DxfResourceProfile};

use crate::locale::CliLanguage;

pub(super) fn write_command_help(writer: &mut dyn Write, language: CliLanguage) -> io::Result<()> {
    let mut buffer = Vec::new();
    cli_command(language).write_help(&mut buffer)?;
    buffer.push(b'\n');
    writer.write_all(&buffer)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CliAction {
    Inspect,
    Verify,
}

impl CliAction {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Inspect => "inspect",
            Self::Verify => "verify",
        }
    }
}

pub(super) struct CliOptions {
    pub(super) action: CliAction,
    pub(super) input: PathBuf,
    pub(super) read_mode: DxfReadMode,
    pub(super) profile: DxfResourceProfile,
    pub(super) language: CliLanguage,
    pub(super) json: bool,
    pub(super) show_path: bool,
}

impl CliOptions {
    pub(super) fn from_matches(matches: &ArgMatches) -> Result<Self, &'static str> {
        let (action, command) = match matches.subcommand() {
            Some(("inspect", command)) => (CliAction::Inspect, command),
            Some(("verify", command)) => (CliAction::Verify, command),
            _ => return Err("missing command parser state"),
        };
        let input = command
            .get_one::<PathBuf>("input")
            .cloned()
            .ok_or("missing input parser state")?;
        let read_mode = match command.get_one::<String>("mode").map(String::as_str) {
            Some("strict") => DxfReadMode::Strict,
            Some("compatible") => DxfReadMode::Compatible,
            _ => return Err("invalid mode parser state"),
        };
        let profile = if matches.get_flag("large") {
            DxfResourceProfile::Large
        } else {
            DxfResourceProfile::Safe
        };
        let language = matches
            .get_one::<String>("language")
            .and_then(|value| CliLanguage::from_code(value))
            .ok_or("invalid language parser state")?;
        Ok(Self {
            action,
            input,
            read_mode,
            profile,
            language,
            json: matches.get_flag("json"),
            show_path: matches.get_flag("show-path"),
        })
    }
}

pub(super) fn cli_command(language: CliLanguage) -> Command {
    let text = language.catalog();
    let options_heading = text.options_heading;
    Command::new("seacad")
        .version(seacad_dxf_core::core_version())
        .about(text.application_about)
        .override_usage(text.root_usage)
        .help_template(language.help_template())
        .subcommand_help_heading(text.commands_heading)
        .subcommand_value_name(text.command_value_name)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .disable_help_subcommand(true)
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(
            Arg::new("json")
                .long("json")
                .action(ArgAction::SetTrue)
                .global(true)
                .help_heading(options_heading)
                .help(text.emit_json_help),
        )
        .arg(
            Arg::new("large")
                .long("large")
                .action(ArgAction::SetTrue)
                .global(true)
                .help_heading(options_heading)
                .help(text.large_profile_help),
        )
        .arg(
            Arg::new("show-path")
                .long("show-path")
                .action(ArgAction::SetTrue)
                .global(true)
                .help_heading(options_heading)
                .help(text.show_path_help),
        )
        .arg(
            Arg::new("language")
                .long("lang")
                .value_name(text.language_value_name)
                .value_parser(["en", "vi"])
                .default_value("en")
                .hide_default_value(true)
                .hide_possible_values(true)
                .global(true)
                .help_heading(options_heading)
                .help(text.language_help),
        )
        .arg(localized_help_arg(language, options_heading))
        .arg(localized_version_arg(language, options_heading))
        .subcommand(file_command(
            language,
            "inspect",
            text.inspect_about,
            "compatible",
        ))
        .subcommand(file_command(
            language,
            "verify",
            text.verify_about,
            "strict",
        ))
}

fn file_command(
    language: CliLanguage,
    name: &'static str,
    about: &'static str,
    default_mode: &'static str,
) -> Command {
    let text = language.catalog();
    let options_heading = text.options_heading;
    let usage = match name {
        "inspect" => text.inspect_usage,
        "verify" => text.verify_usage,
        _ => text.generic_file_usage,
    };
    Command::new(name)
        .about(about)
        .override_usage(usage)
        .help_template(language.help_template())
        .disable_help_flag(true)
        .arg(
            Arg::new("input")
                .value_name(text.file_value_name)
                .required(true)
                .value_parser(value_parser!(PathBuf))
                .help_heading(text.arguments_heading)
                .help(text.file_help),
        )
        .arg(
            Arg::new("mode")
                .long("mode")
                .value_name(text.mode_value_name)
                .value_parser(["strict", "compatible"])
                .default_value(default_mode)
                .hide_default_value(true)
                .hide_possible_values(true)
                .help_heading(options_heading)
                .help(text.mode_help),
        )
        .arg(localized_help_arg(language, options_heading))
}

fn localized_help_arg(language: CliLanguage, heading: &'static str) -> Arg {
    Arg::new("help")
        .short('h')
        .long("help")
        .action(ArgAction::Help)
        .help_heading(heading)
        .help(language.catalog().print_help)
}

fn localized_version_arg(language: CliLanguage, heading: &'static str) -> Arg {
    Arg::new("version")
        .short('V')
        .long("version")
        .action(ArgAction::Version)
        .help_heading(heading)
        .help(language.catalog().print_version)
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::{file_command, write_command_help};
    use crate::locale::CliLanguage;

    #[test]
    fn help_contract_covers_root_and_subcommand_usage() -> Result<(), Box<dyn Error>> {
        let mut root_help = Vec::new();
        write_command_help(&mut root_help, CliLanguage::English)?;
        let root_help = String::from_utf8(root_help)?;
        assert!(root_help.contains("Usage: seacad [OPTIONS] <COMMAND>"));
        assert!(root_help.ends_with('\n'));

        let cases = [
            (
                CliLanguage::English,
                "inspect",
                "Usage: seacad inspect [OPTIONS] <FILE>",
            ),
            (
                CliLanguage::English,
                "verify",
                "Usage: seacad verify [OPTIONS] <FILE>",
            ),
            (
                CliLanguage::Vietnamese,
                "inspect",
                "Cách dùng: seacad inspect [TÙY_CHỌN] <TỆP>",
            ),
            (
                CliLanguage::Vietnamese,
                "verify",
                "Cách dùng: seacad verify [TÙY_CHỌN] <TỆP>",
            ),
        ];
        for (language, name, expected_usage) in cases {
            let mut help = Vec::new();
            file_command(language, name, "about", "strict").write_help(&mut help)?;
            assert!(String::from_utf8(help)?.contains(expected_usage));
        }
        Ok(())
    }
}
