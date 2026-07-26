//! SeaCad DXF inspection and verification command-line interface.

#![forbid(unsafe_code)]

use std::{
    env,
    ffi::OsString,
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

use clap::{Arg, ArgAction, ArgMatches, Command, error::ErrorKind, value_parser};
use seacad_dxf_core::{
    DxfAsciiDocumentConformance, DxfAsciiRawDocument, DxfByteSource, DxfCancellationToken,
    DxfDiagnosticSeverity, DxfError, DxfFileSource, DxfPhysicalFormat, DxfReadMode, DxfReadOptions,
    DxfResourceProfile, NoopDxfReadObserver, probe_dxf_physical_format, scan_dxf_source,
};
use serde::Serialize;

const JSON_SCHEMA_VERSION: &str = "v1";
const CLI_INTERNAL_ERROR: &str = "CLI-E0001";
const CLI_UNSUPPORTED_FORMAT: &str = "CLI-E0002";
const CLI_UNKNOWN_FORMAT: &str = "CLI-E0003";
const CLI_RECOVERED_NOT_VERIFIED: &str = "CLI-E0004";
const CLI_OUTPUT_ERROR: &str = "CLI-E0005";

fn main() -> ExitCode {
    let args: Vec<OsString> = env::args_os().collect();
    let stdout = io::stdout();
    let stderr = io::stderr();
    let mut stdout = stdout.lock();
    let mut stderr = stderr.lock();
    ExitCode::from(run(args, &mut stdout, &mut stderr))
}

fn run(args: Vec<OsString>, stdout: &mut dyn Write, stderr: &mut dyn Write) -> u8 {
    let matches = match cli_command().try_get_matches_from(args) {
        Ok(matches) => matches,
        Err(error) => {
            let is_display = matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            );
            let write_result = if is_display {
                write!(stdout, "{error}")
            } else {
                write!(stderr, "{error}")
            };
            return if write_result.is_err() {
                1
            } else if is_display {
                0
            } else {
                2
            };
        }
    };

    let options = match CliOptions::from_matches(&matches) {
        Ok(options) => options,
        Err(message) => {
            let _ignored = writeln!(stderr, "{CLI_INTERNAL_ERROR}: {message}");
            return 1;
        }
    };
    let outcome = execute(&options);
    let write_result = if options.json {
        render_json(stdout, &outcome.report)
    } else if outcome.exit_code == 0 {
        render_text(stdout, &outcome.report)
    } else {
        render_text(stderr, &outcome.report)
    };
    if let Err(error) = write_result {
        let _ignored = writeln!(
            stderr,
            "{CLI_OUTPUT_ERROR}: failed to write CLI output ({:?})",
            error.kind()
        );
        1
    } else {
        outcome.exit_code
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CliAction {
    Inspect,
    Verify,
}

impl CliAction {
    const fn name(self) -> &'static str {
        match self {
            Self::Inspect => "inspect",
            Self::Verify => "verify",
        }
    }
}

struct CliOptions {
    action: CliAction,
    input: PathBuf,
    read_mode: DxfReadMode,
    profile: DxfResourceProfile,
    json: bool,
    show_path: bool,
}

impl CliOptions {
    fn from_matches(matches: &ArgMatches) -> Result<Self, &'static str> {
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
        Ok(Self {
            action,
            input,
            read_mode,
            profile,
            json: matches.get_flag("json"),
            show_path: matches.get_flag("show-path"),
        })
    }
}

fn cli_command() -> Command {
    Command::new("seacad")
        .version(seacad_dxf_core::core_version())
        .about("Lossless DXF inspection and verification")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .disable_help_subcommand(true)
        .arg(
            Arg::new("json")
                .long("json")
                .action(ArgAction::SetTrue)
                .global(true)
                .help("Emit stable JSON schema v1"),
        )
        .arg(
            Arg::new("large")
                .long("large")
                .action(ArgAction::SetTrue)
                .global(true)
                .help("Opt in to the Large resource profile"),
        )
        .arg(
            Arg::new("show-path")
                .long("show-path")
                .action(ArgAction::SetTrue)
                .global(true)
                .help("Include the input path in output"),
        )
        .subcommand(file_command(
            "inspect",
            "Inspect raw ASCII DXF framing; Compatible mode is the default",
            "compatible",
        ))
        .subcommand(file_command(
            "verify",
            "Verify strict raw ASCII DXF framing; Strict mode is the default",
            "strict",
        ))
}

fn file_command(name: &'static str, about: &'static str, default_mode: &'static str) -> Command {
    Command::new(name)
        .about(about)
        .arg(
            Arg::new("input")
                .value_name("FILE")
                .required(true)
                .value_parser(value_parser!(PathBuf))
                .help("DXF file to read"),
        )
        .arg(
            Arg::new("mode")
                .long("mode")
                .value_parser(["strict", "compatible"])
                .default_value(default_mode),
        )
}

struct CliOutcome {
    report: CliReport,
    exit_code: u8,
}

#[derive(Serialize)]
struct CliReport {
    schema_version: &'static str,
    command: &'static str,
    status: &'static str,
    options: OptionsReport,
    source: SourceReport,
    format: FormatReport,
    document: Option<DocumentReport>,
    diagnostics: Vec<DiagnosticReport>,
    error: Option<ErrorReport>,
}

#[derive(Serialize)]
struct OptionsReport {
    read_mode: &'static str,
    resource_profile: &'static str,
    language: &'static str,
}

#[derive(Serialize)]
struct SourceReport {
    id: Option<String>,
    bytes: Option<u64>,
    path: Option<String>,
}

#[derive(Serialize)]
struct FormatReport {
    physical: &'static str,
}

#[derive(Serialize)]
struct DocumentReport {
    conformance: &'static str,
    groups: u64,
    eof_occurrence: Option<u64>,
    trailing_bytes: u64,
    diagnostics_truncated: bool,
}

#[derive(Serialize)]
struct DiagnosticReport {
    code: &'static str,
    severity: &'static str,
    span: Option<SpanReport>,
}

#[derive(Serialize)]
struct SpanReport {
    start: u64,
    end: u64,
}

#[derive(Serialize)]
struct ErrorReport {
    code: String,
    message: String,
}

fn execute(options: &CliOptions) -> CliOutcome {
    let mut report = base_report(options);
    let source = match DxfFileSource::open(&options.input, options.profile) {
        Ok(source) => source,
        Err(error) => return core_failure(report, error),
    };
    report.source.bytes = Some(source.len());

    let physical = match probe_dxf_physical_format(&source, options.profile) {
        Ok(physical) => physical,
        Err(error) => return identified_core_failure(report, &source, options.profile, error),
    };
    report.format.physical = physical_name(physical);
    match physical {
        DxfPhysicalFormat::AsciiCandidate => open_ascii(report, &source, options),
        DxfPhysicalFormat::Binary => identified_cli_failure(
            report,
            &source,
            options.profile,
            "unsupported",
            CLI_UNSUPPORTED_FORMAT,
            "Binary DXF is not implemented in this milestone",
        ),
        DxfPhysicalFormat::Unknown => identified_cli_failure(
            report,
            &source,
            options.profile,
            "invalid",
            CLI_UNKNOWN_FORMAT,
            "The source is not a recognized ASCII or Binary DXF candidate",
        ),
        _ => identified_cli_failure(
            report,
            &source,
            options.profile,
            "unsupported",
            CLI_UNSUPPORTED_FORMAT,
            "The physical DXF format is not supported",
        ),
    }
}

fn open_ascii(mut report: CliReport, source: &DxfFileSource, options: &CliOptions) -> CliOutcome {
    let cancellation = DxfCancellationToken::default();
    let mut observer = NoopDxfReadObserver;
    let read_options = DxfReadOptions::new(options.read_mode, options.profile);
    let document =
        match DxfAsciiRawDocument::open(source, read_options, &cancellation, &mut observer) {
            Ok(document) => document,
            Err(error) => return identified_core_failure(report, source, options.profile, error),
        };

    report.source.id = Some(document.source_id().to_string());
    let groups = match u64::try_from(document.groups().len()) {
        Ok(groups) => groups,
        Err(_) => {
            return cli_failure(
                report,
                "invalid",
                CLI_INTERNAL_ERROR,
                "Group count does not fit the CLI schema",
            );
        }
    };
    let conformance = conformance_name(document.conformance());
    report.document = Some(DocumentReport {
        conformance,
        groups,
        eof_occurrence: document.eof_occurrence(),
        trailing_bytes: document.trailing_span().map_or(0, |span| span.len()),
        diagnostics_truncated: document.diagnostics_were_truncated(),
    });
    report.diagnostics = document
        .diagnostics()
        .iter()
        .map(|diagnostic| DiagnosticReport {
            code: diagnostic.code().as_str(),
            severity: severity_name(diagnostic.severity()),
            span: diagnostic.span().map(|span| SpanReport {
                start: span.start(),
                end: span.end(),
            }),
        })
        .collect();

    let recovered = document.conformance() == DxfAsciiDocumentConformance::Recovered;
    match (options.action, recovered) {
        (CliAction::Inspect, true) => {
            report.status = "recovered";
            CliOutcome {
                report,
                exit_code: 0,
            }
        }
        (CliAction::Inspect, false) => {
            report.status = "ok";
            CliOutcome {
                report,
                exit_code: 0,
            }
        }
        (CliAction::Verify, true) => cli_failure(
            report,
            "not_verified",
            CLI_RECOVERED_NOT_VERIFIED,
            "Recovered framing is inspect/verbatim-only and is not verified",
        ),
        (CliAction::Verify, false) => {
            report.status = "verified";
            CliOutcome {
                report,
                exit_code: 0,
            }
        }
    }
}

fn base_report(options: &CliOptions) -> CliReport {
    CliReport {
        schema_version: JSON_SCHEMA_VERSION,
        command: options.action.name(),
        status: "pending",
        options: OptionsReport {
            read_mode: read_mode_name(options.read_mode),
            resource_profile: profile_name(options.profile),
            language: "en",
        },
        source: SourceReport {
            id: None,
            bytes: None,
            path: if options.show_path {
                options.input.to_str().map(str::to_owned)
            } else {
                None
            },
        },
        format: FormatReport {
            physical: "unknown",
        },
        document: None,
        diagnostics: Vec::new(),
        error: None,
    }
}

fn identified_core_failure(
    mut report: CliReport,
    source: &DxfFileSource,
    profile: DxfResourceProfile,
    error: DxfError,
) -> CliOutcome {
    attach_source_identity(&mut report, source, profile);
    core_failure(report, error)
}

fn core_failure(mut report: CliReport, error: DxfError) -> CliOutcome {
    report.status = "invalid";
    report.error = Some(ErrorReport {
        code: error.code().as_str().to_owned(),
        message: error.to_string(),
    });
    CliOutcome {
        report,
        exit_code: 1,
    }
}

fn identified_cli_failure(
    mut report: CliReport,
    source: &DxfFileSource,
    profile: DxfResourceProfile,
    status: &'static str,
    code: &'static str,
    message: &'static str,
) -> CliOutcome {
    attach_source_identity(&mut report, source, profile);
    cli_failure(report, status, code, message)
}

fn attach_source_identity(
    report: &mut CliReport,
    source: &DxfFileSource,
    profile: DxfResourceProfile,
) {
    let cancellation = DxfCancellationToken::default();
    let mut observer = NoopDxfReadObserver;
    if let Ok(receipt) = scan_dxf_source(source, profile, &cancellation, &mut observer) {
        report.source.id = Some(receipt.source_id().to_string());
    }
}

fn cli_failure(
    mut report: CliReport,
    status: &'static str,
    code: &'static str,
    message: &'static str,
) -> CliOutcome {
    report.status = status;
    report.error = Some(ErrorReport {
        code: code.to_owned(),
        message: message.to_owned(),
    });
    CliOutcome {
        report,
        exit_code: 1,
    }
}

fn render_json(writer: &mut dyn Write, report: &CliReport) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut *writer, report).map_err(io::Error::other)?;
    writeln!(writer)
}

fn render_text(writer: &mut dyn Write, report: &CliReport) -> io::Result<()> {
    writeln!(writer, "SeaCad {}", report.command)?;
    writeln!(writer, "Status: {}", status_text(report.status))?;
    writeln!(writer, "Read mode: {}", report.options.read_mode)?;
    writeln!(
        writer,
        "Resource profile: {}",
        report.options.resource_profile
    )?;
    writeln!(
        writer,
        "Physical format: {}",
        physical_text(report.format.physical)
    )?;
    if let Some(path) = &report.source.path {
        writeln!(writer, "Path: {path}")?;
    }
    if let Some(bytes) = report.source.bytes {
        writeln!(writer, "Bytes: {bytes}")?;
    }
    if let Some(source_id) = &report.source.id {
        writeln!(writer, "Source ID: {source_id}")?;
    }
    if let Some(document) = &report.document {
        writeln!(writer, "Conformance: {}", document.conformance)?;
        writeln!(writer, "Groups: {}", document.groups)?;
        let eof = document
            .eof_occurrence
            .map_or_else(|| "absent".to_owned(), |value| value.to_string());
        writeln!(writer, "EOF occurrence: {eof}")?;
        writeln!(writer, "Trailing bytes: {}", document.trailing_bytes)?;
    }
    writeln!(writer, "Diagnostics: {}", report.diagnostics.len())?;
    for diagnostic in &report.diagnostics {
        if let Some(span) = &diagnostic.span {
            writeln!(
                writer,
                "  {} {} [{}, {})",
                diagnostic.code, diagnostic.severity, span.start, span.end
            )?;
        } else {
            writeln!(writer, "  {} {}", diagnostic.code, diagnostic.severity)?;
        }
    }
    if let Some(error) = &report.error {
        writeln!(writer, "Error {}: {}", error.code, error.message)?;
    }
    Ok(())
}

fn status_text(status: &str) -> &str {
    match status {
        "ok" => "OK",
        "verified" => "VERIFIED",
        "recovered" => "RECOVERED",
        "not_verified" => "NOT VERIFIED",
        "invalid" => "INVALID",
        "unsupported" => "UNSUPPORTED",
        _ => status,
    }
}

fn physical_text(physical: &str) -> &str {
    match physical {
        "ascii_candidate" => "ASCII DXF candidate",
        "binary" => "Binary DXF",
        "unknown" => "Unknown",
        _ => physical,
    }
}

const fn read_mode_name(mode: DxfReadMode) -> &'static str {
    match mode {
        DxfReadMode::Strict => "strict",
        DxfReadMode::Compatible => "compatible",
        _ => "unknown",
    }
}

const fn profile_name(profile: DxfResourceProfile) -> &'static str {
    match profile {
        DxfResourceProfile::Safe => "safe",
        DxfResourceProfile::Large => "large",
        _ => "unknown",
    }
}

const fn physical_name(format: DxfPhysicalFormat) -> &'static str {
    match format {
        DxfPhysicalFormat::AsciiCandidate => "ascii_candidate",
        DxfPhysicalFormat::Binary => "binary",
        DxfPhysicalFormat::Unknown => "unknown",
        _ => "unknown",
    }
}

const fn conformance_name(conformance: DxfAsciiDocumentConformance) -> &'static str {
    match conformance {
        DxfAsciiDocumentConformance::Strict => "strict",
        DxfAsciiDocumentConformance::Recovered => "recovered",
        _ => "unknown",
    }
}

const fn severity_name(severity: DxfDiagnosticSeverity) -> &'static str {
    match severity {
        DxfDiagnosticSeverity::Info => "info",
        DxfDiagnosticSeverity::Warning => "warning",
        DxfDiagnosticSeverity::Error => "error",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        ffi::OsString,
        fs, io,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use serde_json::Value;

    static NEXT_TEMP_DIRECTORY_ID: AtomicU64 = AtomicU64::new(0);
    const STRICT: &[u8] = b"0\nSECTION\n2\nENTITIES\n0\nENDSEC\n0\nEOF\n";

    #[test]
    fn help_version_and_usage_have_stable_exit_codes() -> Result<(), Box<dyn Error>> {
        let (_, stdout, _) = invoke(["seacad", "--help"])?;
        assert!(stdout.contains("inspect"));
        let (version_exit, version, _) = invoke(["seacad", "--version"])?;
        assert_eq!(version_exit, 0);
        assert!(version.starts_with("seacad "));
        let (usage_exit, _, usage_error) = invoke(["seacad", "unknown"])?;
        assert_eq!(usage_exit, 2);
        assert!(usage_error.contains("unrecognized subcommand"));
        Ok(())
    }

    #[test]
    fn inspect_json_is_v1_and_hides_path_by_default() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let input = directory.write("private-name.dxf", STRICT)?;
        let input_text = path_text(&input)?;
        let (exit, stdout, stderr) = invoke_os(vec![
            OsString::from("seacad"),
            OsString::from("inspect"),
            input.as_os_str().to_owned(),
            OsString::from("--json"),
        ])?;
        assert_eq!(exit, 0);
        assert!(stderr.is_empty());
        assert!(!stdout.contains(input_text));
        let report: Value = serde_json::from_str(&stdout)?;
        assert_eq!(report["schema_version"], "v1");
        assert_eq!(report["command"], "inspect");
        assert_eq!(report["status"], "ok");
        assert_eq!(report["options"]["read_mode"], "compatible");
        assert_eq!(report["source"]["path"], Value::Null);
        assert_eq!(report["document"]["groups"], 4);
        assert_eq!(report["document"]["conformance"], "strict");
        Ok(())
    }

    #[test]
    fn show_path_is_explicit_and_json_keys_remain_stable() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let input = directory.write("shown.dxf", STRICT)?;
        let input_text = path_text(&input)?;
        let (exit, stdout, _) = invoke_os(vec![
            OsString::from("seacad"),
            OsString::from("inspect"),
            input.as_os_str().to_owned(),
            OsString::from("--json"),
            OsString::from("--show-path"),
        ])?;
        assert_eq!(exit, 0);
        let report: Value = serde_json::from_str(&stdout)?;
        assert_eq!(report["schema_version"], "v1");
        assert_eq!(report["options"]["language"], "en");
        assert_eq!(report["source"]["path"], input_text);
        Ok(())
    }

    #[test]
    fn inspect_recovers_but_verify_rejects_recovered_framing() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let input = directory.write("missing-eof.dxf", b"0\nSECTION\n0\nENDSEC\n")?;
        let inspect = invoke_file("inspect", &input, &["--json"])?;
        assert_eq!(inspect.0, 0);
        let inspect_report: Value = serde_json::from_str(&inspect.1)?;
        assert_eq!(inspect_report["status"], "recovered");
        assert_eq!(inspect_report["diagnostics"][0]["code"], "DXF-W0203");

        let verify = invoke_file("verify", &input, &["--mode", "compatible", "--json"])?;
        assert_eq!(verify.0, 1);
        let verify_report: Value = serde_json::from_str(&verify.1)?;
        assert_eq!(verify_report["status"], "not_verified");
        assert_eq!(
            verify_report["error"]["code"],
            super::CLI_RECOVERED_NOT_VERIFIED
        );
        Ok(())
    }

    #[test]
    fn verify_strict_passes_and_malformed_pair_fails_with_core_code() -> Result<(), Box<dyn Error>>
    {
        let directory = TestDirectory::new()?;
        let valid = directory.write("valid.dxf", STRICT)?;
        let verified = invoke_file("verify", &valid, &["--json"])?;
        assert_eq!(verified.0, 0);
        let verified_report: Value = serde_json::from_str(&verified.1)?;
        assert_eq!(verified_report["status"], "verified");
        assert_eq!(verified_report["options"]["read_mode"], "strict");

        let invalid = directory.write("invalid-private.dxf", b"0\n")?;
        let invalid_text = path_text(&invalid)?;
        let failed = invoke_file("inspect", &invalid, &["--json"])?;
        assert_eq!(failed.0, 1);
        assert!(failed.2.is_empty());
        assert!(!failed.1.contains(invalid_text));
        let failed_report: Value = serde_json::from_str(&failed.1)?;
        assert_eq!(failed_report["status"], "invalid");
        assert_eq!(failed_report["error"]["code"], "DXF-E0202");
        assert!(failed_report["source"]["id"].is_string());

        let missing = directory.path.join("missing-private.dxf");
        let missing_text = path_text(&missing)?;
        let missing_result = invoke_file("inspect", &missing, &["--json"])?;
        assert_eq!(missing_result.0, 1);
        assert!(!missing_result.1.contains(missing_text));
        let missing_report: Value = serde_json::from_str(&missing_result.1)?;
        assert_eq!(missing_report["status"], "invalid");
        assert_eq!(missing_report["source"]["path"], Value::Null);
        Ok(())
    }

    #[test]
    fn binary_and_unknown_inputs_are_identified_without_path_leakage() -> Result<(), Box<dyn Error>>
    {
        let directory = TestDirectory::new()?;
        let mut binary = seacad_dxf_core::DXF_BINARY_SENTINEL.to_vec();
        binary.extend_from_slice(b"payload");
        let binary_path = directory.write("binary-secret.dxf", &binary)?;
        let binary_result = invoke_file("inspect", &binary_path, &["--json"])?;
        assert_eq!(binary_result.0, 1);
        let binary_report: Value = serde_json::from_str(&binary_result.1)?;
        assert_eq!(binary_report["status"], "unsupported");
        assert_eq!(binary_report["format"]["physical"], "binary");
        assert_eq!(
            binary_report["error"]["code"],
            super::CLI_UNSUPPORTED_FORMAT
        );
        assert!(binary_report["source"]["id"].is_string());

        let unknown_path = directory.write("unknown-secret.dxf", b"")?;
        let unknown_result = invoke_file("inspect", &unknown_path, &["--json"])?;
        assert_eq!(unknown_result.0, 1);
        let unknown_report: Value = serde_json::from_str(&unknown_result.1)?;
        assert_eq!(unknown_report["status"], "invalid");
        assert_eq!(unknown_report["error"]["code"], super::CLI_UNKNOWN_FORMAT);
        Ok(())
    }

    fn invoke<const N: usize>(args: [&str; N]) -> Result<(u8, String, String), Box<dyn Error>> {
        invoke_os(args.into_iter().map(OsString::from).collect())
    }

    fn invoke_file(
        command: &str,
        input: &Path,
        trailing: &[&str],
    ) -> Result<(u8, String, String), Box<dyn Error>> {
        let mut args = vec![
            OsString::from("seacad"),
            OsString::from(command),
            input.as_os_str().to_owned(),
        ];
        args.extend(trailing.iter().map(OsString::from));
        invoke_os(args)
    }

    fn invoke_os(args: Vec<OsString>) -> Result<(u8, String, String), Box<dyn Error>> {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = super::run(args, &mut stdout, &mut stderr);
        Ok((exit, String::from_utf8(stdout)?, String::from_utf8(stderr)?))
    }

    fn path_text(path: &Path) -> Result<&str, io::Error> {
        path.to_str().ok_or(io::Error::new(
            io::ErrorKind::InvalidData,
            "test path is not Unicode",
        ))
    }

    struct TestDirectory {
        path: PathBuf,
    }

    impl TestDirectory {
        fn new() -> io::Result<Self> {
            for _ in 0..100 {
                let id = NEXT_TEMP_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
                let path =
                    std::env::temp_dir().join(format!("seacad-cli-{}-{id}", std::process::id()));
                match fs::create_dir(&path) {
                    Ok(()) => return Ok(Self { path }),
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                    Err(error) => return Err(error),
                }
            }
            Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "unable to reserve temporary CLI test directory",
            ))
        }

        fn write(&self, name: &str, bytes: &[u8]) -> io::Result<PathBuf> {
            let path = self.path.join(name);
            fs::write(&path, bytes)?;
            Ok(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
