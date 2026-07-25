//! SeaCad command-line bootstrap.

#![forbid(unsafe_code)]

use std::env;
use std::process::ExitCode;

const HELP: &str = "SeaCad DXF verification CLI\n\nUsage: seacad [--help | --version]\n\nThe inspect, verify, and corpus commands arrive in later milestones.";

fn main() -> ExitCode {
    ExitCode::from(run(env::args().skip(1)))
}

fn run(mut args: impl Iterator<Item = String>) -> u8 {
    match args.next().as_deref() {
        None | Some("--help" | "-h") => {
            println!("{HELP}");
            0
        }
        Some("--version" | "-V") => {
            println!("seacad {}", seacad_dxf_core::core_version());
            0
        }
        Some(argument) => {
            eprintln!("unknown argument: {argument}\n\n{HELP}");
            2
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn help_and_version_succeed() {
        assert_eq!(super::run(["--help".to_owned()].into_iter()), 0);
        assert_eq!(super::run(["--version".to_owned()].into_iter()), 0);
    }

    #[test]
    fn unknown_argument_is_rejected() {
        assert_eq!(super::run(["inspect".to_owned()].into_iter()), 2);
    }
}
