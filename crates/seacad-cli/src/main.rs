//! SeaCad DXF inspection and verification command-line interface.

#![forbid(unsafe_code)]

mod locale;

use std::{
    env,
    ffi::OsString,
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

use clap::{Arg, ArgAction, ArgMatches, Command, error::ErrorKind, value_parser};
use seacad_dxf_core::{
    DxfAsciiDocumentConformance, DxfAsciiRawDocument, DxfBinaryDocumentConformance,
    DxfBinaryRawDocument, DxfByteSource, DxfCancellationToken, DxfDiagnostic,
    DxfDiagnosticSeverity, DxfError, DxfFileSource, DxfPhysicalFormat, DxfReadMode, DxfReadOptions,
    DxfResourceProfile, NoopDxfReadObserver, probe_dxf_physical_format, scan_dxf_source,
};
use serde::Serialize;

use crate::locale::{CliLanguage, write_usage_error};

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
    let language = CliLanguage::detect(&args);
    let matches = match cli_command(language).try_get_matches_from(args) {
        Ok(matches) => matches,
        Err(error) => {
            let is_display = matches!(
                error.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            );
            let is_help_on_missing =
                error.kind() == ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand;
            let write_result = if is_display {
                write!(stdout, "{error}")
            } else if is_help_on_missing {
                write_command_help(stderr, language)
            } else {
                write_usage_error(stderr, &error, language)
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
            let label = language.pick("Internal CLI error", "Lỗi nội bộ CLI");
            let _ignored = writeln!(stderr, "{CLI_INTERNAL_ERROR}: {label}: {message}");
            return 1;
        }
    };
    let outcome = execute(&options);
    let write_result = if options.json {
        render_json(stdout, &outcome.report)
    } else if outcome.exit_code == 0 {
        render_text(stdout, &outcome.report, options.language)
    } else {
        render_text(stderr, &outcome.report, options.language)
    };
    if let Err(error) = write_result {
        let message = options
            .language
            .pick("failed to write CLI output", "không thể ghi đầu ra CLI");
        let _ignored = writeln!(stderr, "{CLI_OUTPUT_ERROR}: {message} ({:?})", error.kind());
        1
    } else {
        outcome.exit_code
    }
}

fn write_command_help(writer: &mut dyn Write, language: CliLanguage) -> io::Result<()> {
    let mut buffer = Vec::new();
    cli_command(language).write_help(&mut buffer)?;
    buffer.push(b'\n');
    writer.write_all(&buffer)
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
    language: CliLanguage,
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

fn cli_command(language: CliLanguage) -> Command {
    let options_heading = language.pick("Options", "Tùy chọn");
    Command::new("seacad")
        .version(seacad_dxf_core::core_version())
        .about(language.pick(
            "Lossless DXF inspection and verification",
            "Kiểm tra và xác minh DXF không làm mất dữ liệu",
        ))
        .override_usage(language.pick("seacad [OPTIONS] <COMMAND>", "seacad [TÙY_CHỌN] <LỆNH>"))
        .help_template(language.help_template())
        .subcommand_help_heading(language.pick("Commands", "Lệnh"))
        .subcommand_value_name(language.pick("COMMAND", "LỆNH"))
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
                .help(language.pick("Emit stable JSON schema v1", "Xuất JSON schema v1 ổn định")),
        )
        .arg(
            Arg::new("large")
                .long("large")
                .action(ArgAction::SetTrue)
                .global(true)
                .help_heading(options_heading)
                .help(language.pick(
                    "Opt in to the Large resource profile",
                    "Chủ động dùng hồ sơ tài nguyên Large",
                )),
        )
        .arg(
            Arg::new("show-path")
                .long("show-path")
                .action(ArgAction::SetTrue)
                .global(true)
                .help_heading(options_heading)
                .help(language.pick(
                    "Include the input path in output",
                    "Hiện đường dẫn đầu vào trong kết quả",
                )),
        )
        .arg(
            Arg::new("language")
                .long("lang")
                .value_name(language.pick("LANG", "NGÔN_NGỮ"))
                .value_parser(["en", "vi"])
                .default_value("en")
                .hide_default_value(true)
                .hide_possible_values(true)
                .global(true)
                .help_heading(options_heading)
                .help(language.pick(
                    "Human output language: en (default) or vi",
                    "Ngôn ngữ hiển thị: en (mặc định) hoặc vi",
                )),
        )
        .arg(localized_help_arg(language, options_heading))
        .arg(localized_version_arg(language, options_heading))
        .subcommand(file_command(
            language,
            "inspect",
            language.pick(
                "Inspect raw ASCII/Binary DXF framing; Compatible mode is the default",
                "Kiểm tra framing DXF ASCII/Binary thô; mặc định là Compatible",
            ),
            "compatible",
        ))
        .subcommand(file_command(
            language,
            "verify",
            language.pick(
                "Verify strict raw ASCII/Binary DXF framing; Strict mode is the default",
                "Xác minh framing DXF ASCII/Binary nghiêm ngặt; mặc định là Strict",
            ),
            "strict",
        ))
}

fn file_command(
    language: CliLanguage,
    name: &'static str,
    about: &'static str,
    default_mode: &'static str,
) -> Command {
    let options_heading = language.pick("Options", "Tùy chọn");
    let usage = match (language, name) {
        (CliLanguage::Vietnamese, "inspect") => "seacad inspect [TÙY_CHỌN] <TỆP>",
        (CliLanguage::Vietnamese, "verify") => "seacad verify [TÙY_CHỌN] <TỆP>",
        (CliLanguage::English, "inspect") => "seacad inspect [OPTIONS] <FILE>",
        (CliLanguage::English, "verify") => "seacad verify [OPTIONS] <FILE>",
        _ => "seacad [OPTIONS] <FILE>",
    };
    Command::new(name)
        .about(about)
        .override_usage(usage)
        .help_template(language.help_template())
        .disable_help_flag(true)
        .arg(
            Arg::new("input")
                .value_name(language.pick("FILE", "TỆP"))
                .required(true)
                .value_parser(value_parser!(PathBuf))
                .help_heading(language.pick("Arguments", "Đối số"))
                .help(language.pick("DXF file to read", "Tệp DXF cần đọc")),
        )
        .arg(
            Arg::new("mode")
                .long("mode")
                .value_name(language.pick("MODE", "CHẾ_ĐỘ"))
                .value_parser(["strict", "compatible"])
                .default_value(default_mode)
                .hide_default_value(true)
                .hide_possible_values(true)
                .help_heading(options_heading)
                .help(language.pick("Framing read mode", "Chế độ đọc framing")),
        )
        .arg(localized_help_arg(language, options_heading))
}

fn localized_help_arg(language: CliLanguage, heading: &'static str) -> Arg {
    Arg::new("help")
        .short('h')
        .long("help")
        .action(ArgAction::Help)
        .help_heading(heading)
        .help(language.pick("Print help", "In trợ giúp"))
}

fn localized_version_arg(language: CliLanguage, heading: &'static str) -> Arg {
    Arg::new("version")
        .short('V')
        .long("version")
        .action(ArgAction::Version)
        .help_heading(heading)
        .help(language.pick("Print version", "In phiên bản"))
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
        DxfPhysicalFormat::Binary => open_binary(report, &source, options),
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

fn open_ascii(report: CliReport, source: &DxfFileSource, options: &CliOptions) -> CliOutcome {
    let cancellation = DxfCancellationToken::default();
    let mut observer = NoopDxfReadObserver;
    let read_options = DxfReadOptions::new(options.read_mode, options.profile);
    let document =
        match DxfAsciiRawDocument::open(source, read_options, &cancellation, &mut observer) {
            Ok(document) => document,
            Err(error) => return identified_core_failure(report, source, options.profile, error),
        };

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
    finish_opened_document(
        report,
        options,
        OpenedDocumentSummary {
            source_id: document.source_id().to_string(),
            groups,
            conformance: ascii_conformance_name(document.conformance()),
            recovered: document.conformance() == DxfAsciiDocumentConformance::Recovered,
            eof_occurrence: document.eof_occurrence(),
            trailing_bytes: document.trailing_span().map_or(0, |span| span.len()),
            diagnostics: document.diagnostics(),
            diagnostics_truncated: document.diagnostics_were_truncated(),
        },
    )
}

fn open_binary(report: CliReport, source: &DxfFileSource, options: &CliOptions) -> CliOutcome {
    let cancellation = DxfCancellationToken::default();
    let mut observer = NoopDxfReadObserver;
    let read_options = DxfReadOptions::new(options.read_mode, options.profile);
    let document =
        match DxfBinaryRawDocument::open(source, read_options, &cancellation, &mut observer) {
            Ok(document) => document,
            Err(error) => return identified_core_failure(report, source, options.profile, error),
        };

    finish_opened_document(
        report,
        options,
        OpenedDocumentSummary {
            source_id: document.source_id().to_string(),
            groups: document.group_count(),
            conformance: binary_conformance_name(document.conformance()),
            recovered: document.conformance() == DxfBinaryDocumentConformance::Recovered,
            eof_occurrence: document.eof_occurrence(),
            trailing_bytes: document.trailing_span().map_or(0, |span| span.len()),
            diagnostics: document.diagnostics(),
            diagnostics_truncated: false,
        },
    )
}

struct OpenedDocumentSummary<'a> {
    source_id: String,
    groups: u64,
    conformance: &'static str,
    recovered: bool,
    eof_occurrence: Option<u64>,
    trailing_bytes: u64,
    diagnostics: &'a [DxfDiagnostic],
    diagnostics_truncated: bool,
}

fn finish_opened_document(
    mut report: CliReport,
    options: &CliOptions,
    summary: OpenedDocumentSummary<'_>,
) -> CliOutcome {
    report.source.id = Some(summary.source_id);
    report.document = Some(DocumentReport {
        conformance: summary.conformance,
        groups: summary.groups,
        eof_occurrence: summary.eof_occurrence,
        trailing_bytes: summary.trailing_bytes,
        diagnostics_truncated: summary.diagnostics_truncated,
    });
    report.diagnostics = summary
        .diagnostics
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

    match (options.action, summary.recovered) {
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
            language: options.language.code(),
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

fn render_text(
    writer: &mut dyn Write,
    report: &CliReport,
    language: CliLanguage,
) -> io::Result<()> {
    writeln!(writer, "SeaCad {}", report.command)?;
    writeln!(
        writer,
        "{}: {}",
        language.pick("Status", "Trạng thái"),
        status_text(language, report.status)
    )?;
    writeln!(
        writer,
        "{}: {}",
        language.pick("Read mode", "Chế độ đọc"),
        read_mode_text(language, report.options.read_mode)
    )?;
    writeln!(
        writer,
        "{}: {}",
        language.pick("Resource profile", "Hồ sơ tài nguyên"),
        profile_text(language, report.options.resource_profile)
    )?;
    writeln!(
        writer,
        "{}: {}",
        language.pick("Physical format", "Định dạng vật lý"),
        physical_text(language, report.format.physical)
    )?;
    if let Some(path) = &report.source.path {
        writeln!(writer, "{}: {path}", language.pick("Path", "Đường dẫn"))?;
    }
    if let Some(bytes) = report.source.bytes {
        writeln!(writer, "{}: {bytes}", language.pick("Bytes", "Số byte"))?;
    }
    if let Some(source_id) = &report.source.id {
        writeln!(
            writer,
            "{}: {source_id}",
            language.pick("Source ID", "ID nguồn")
        )?;
    }
    if let Some(document) = &report.document {
        writeln!(
            writer,
            "{}: {}",
            language.pick("Conformance", "Mức tuân thủ"),
            conformance_text(language, document.conformance)
        )?;
        writeln!(
            writer,
            "{}: {}",
            language.pick("Groups", "Số group"),
            document.groups
        )?;
        let eof = document.eof_occurrence.map_or_else(
            || language.pick("absent", "không có").to_owned(),
            |value| value.to_string(),
        );
        writeln!(
            writer,
            "{}: {eof}",
            language.pick("EOF occurrence", "Vị trí EOF")
        )?;
        writeln!(
            writer,
            "{}: {}",
            language.pick("Trailing bytes", "Byte phía sau EOF"),
            document.trailing_bytes
        )?;
    }
    writeln!(
        writer,
        "{}: {}",
        language.pick("Diagnostics", "Chẩn đoán"),
        report.diagnostics.len()
    )?;
    for diagnostic in &report.diagnostics {
        if let Some(span) = &diagnostic.span {
            writeln!(
                writer,
                "  {} {} [{}, {})",
                diagnostic.code,
                severity_text(language, diagnostic.severity),
                span.start,
                span.end
            )?;
        } else {
            writeln!(
                writer,
                "  {} {}",
                diagnostic.code,
                severity_text(language, diagnostic.severity)
            )?;
        }
    }
    if let Some(error) = &report.error {
        writeln!(
            writer,
            "{} {}: {}",
            language.pick("Error", "Lỗi"),
            error.code,
            error_text(language, &error.code, &error.message)
        )?;
    }
    Ok(())
}

fn status_text(language: CliLanguage, status: &str) -> &str {
    match (language, status) {
        (CliLanguage::English, "ok") => "OK",
        (CliLanguage::English, "verified") => "VERIFIED",
        (CliLanguage::English, "recovered") => "RECOVERED",
        (CliLanguage::English, "not_verified") => "NOT VERIFIED",
        (CliLanguage::English, "invalid") => "INVALID",
        (CliLanguage::English, "unsupported") => "UNSUPPORTED",
        (CliLanguage::Vietnamese, "ok") => "TỐT",
        (CliLanguage::Vietnamese, "verified") => "ĐÃ XÁC MINH",
        (CliLanguage::Vietnamese, "recovered") => "ĐÃ PHỤC HỒI",
        (CliLanguage::Vietnamese, "not_verified") => "CHƯA XÁC MINH",
        (CliLanguage::Vietnamese, "invalid") => "KHÔNG HỢP LỆ",
        (CliLanguage::Vietnamese, "unsupported") => "CHƯA HỖ TRỢ",
        _ => status,
    }
}

fn physical_text(language: CliLanguage, physical: &str) -> &str {
    match (language, physical) {
        (CliLanguage::English, "ascii_candidate") => "ASCII DXF candidate",
        (CliLanguage::English, "binary") => "Binary DXF",
        (CliLanguage::English, "unknown") => "Unknown",
        (CliLanguage::Vietnamese, "ascii_candidate") => "ứng viên DXF ASCII",
        (CliLanguage::Vietnamese, "binary") => "DXF nhị phân",
        (CliLanguage::Vietnamese, "unknown") => "không xác định",
        _ => physical,
    }
}

fn read_mode_text(language: CliLanguage, mode: &str) -> &str {
    match (language, mode) {
        (CliLanguage::Vietnamese, "strict") => "nghiêm ngặt (strict)",
        (CliLanguage::Vietnamese, "compatible") => "tương thích (compatible)",
        _ => mode,
    }
}

fn profile_text(language: CliLanguage, profile: &str) -> &str {
    match (language, profile) {
        (CliLanguage::Vietnamese, "safe") => "an toàn (safe)",
        (CliLanguage::Vietnamese, "large") => "lớn (large)",
        _ => profile,
    }
}

fn conformance_text(language: CliLanguage, conformance: &str) -> &str {
    match (language, conformance) {
        (CliLanguage::Vietnamese, "strict") => "nghiêm ngặt (strict)",
        (CliLanguage::Vietnamese, "recovered") => "đã phục hồi (recovered)",
        _ => conformance,
    }
}

fn severity_text(language: CliLanguage, severity: &str) -> &str {
    match (language, severity) {
        (CliLanguage::Vietnamese, "info") => "thông tin",
        (CliLanguage::Vietnamese, "warning") => "cảnh báo",
        (CliLanguage::Vietnamese, "error") => "lỗi",
        _ => severity,
    }
}

fn error_text<'a>(language: CliLanguage, code: &str, english: &'a str) -> &'a str {
    if language == CliLanguage::English {
        return english
            .strip_prefix(code)
            .and_then(|message| message.strip_prefix(": "))
            .unwrap_or(english);
    }
    match code {
        CLI_INTERNAL_ERROR => "trạng thái nội bộ của CLI không hợp lệ",
        CLI_UNSUPPORTED_FORMAT => "định dạng DXF này chưa được hỗ trợ",
        CLI_UNKNOWN_FORMAT => "nguồn không phải ứng viên DXF ASCII hoặc Binary đã biết",
        CLI_RECOVERED_NOT_VERIFIED => {
            "framing phục hồi chỉ được kiểm tra hoặc sao chép nguyên trạng"
        }
        CLI_OUTPUT_ERROR => "không thể ghi đầu ra CLI",
        "DXF-E0001" => "thao tác vào/ra thất bại",
        "DXF-E0002" => "thao tác đã bị hủy",
        "DXF-E0101" => "nguồn vượt giới hạn số byte",
        "DXF-E0102" => "nguồn vượt giới hạn số record",
        "DXF-E0103" => "giá trị vượt giới hạn số byte",
        "DXF-E0105" => "offset byte bị tràn số nguyên",
        "DXF-E0201" => "group code ASCII không hợp lệ",
        "DXF-E0202" => "group ASCII không có dòng giá trị",
        "DXF-E0203" => "tài liệu ASCII không có marker 0/EOF kết thúc",
        "DXF-E0204" => "tài liệu ASCII nghiêm ngặt có dữ liệu sau EOF",
        "DXF-E0210" => "sentinel Binary DXF không hợp lệ",
        "DXF-E0211" => "group code Binary DXF bị cắt ngắn",
        "DXF-E0212" => "group code Binary DXF không hợp lệ",
        "DXF-E0213" => "group code Binary DXF chưa có wire family được công bố",
        "DXF-E0214" => "giá trị Binary DXF bị cắt ngắn",
        "DXF-E0215" => "chuỗi Binary DXF không có byte NUL kết thúc",
        "DXF-E0216" => "Binary DXF không có record mở đầu 0/SECTION chuẩn",
        "DXF-E0217" => "Binary DXF không có đúng một HEADER $ACADVER được hỗ trợ",
        "DXF-E0218" => "encoding Binary DXF không khớp dialect đã khai báo",
        "DXF-E0219" => "tài liệu Binary DXF không có marker 0/EOF kết thúc",
        "DXF-E0220" => "tài liệu Binary DXF nghiêm ngặt có dữ liệu sau EOF",
        "DXF-E0301" => "định danh nguồn đã thay đổi",
        "DXF-E0302" => "độ dài đầu ra Verbatim không khớp",
        "DXF-E0303" => "định danh đầu ra Verbatim không khớp",
        _ => english,
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

const fn ascii_conformance_name(conformance: DxfAsciiDocumentConformance) -> &'static str {
    match conformance {
        DxfAsciiDocumentConformance::Strict => "strict",
        DxfAsciiDocumentConformance::Recovered => "recovered",
        _ => "unknown",
    }
}

const fn binary_conformance_name(conformance: DxfBinaryDocumentConformance) -> &'static str {
    match conformance {
        DxfBinaryDocumentConformance::Strict => "strict",
        DxfBinaryDocumentConformance::Recovered => "recovered",
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
    fn vietnamese_help_and_usage_errors_are_localized() -> Result<(), Box<dyn Error>> {
        let (help_exit, help, help_error) = invoke(["seacad", "--lang", "vi", "--help"])?;
        assert_eq!(help_exit, 0);
        assert!(help_error.is_empty());
        assert!(help.contains("Cách dùng:"));
        assert!(help.contains("Lệnh:"));
        assert!(help.contains("Tùy chọn:"));
        assert!(help.contains("Kiểm tra và xác minh DXF không làm mất dữ liệu"));
        assert!(!help.contains("Usage:"));
        assert!(!help.contains("[OPTIONS]"));
        assert!(!help.contains("possible values"));

        let (missing_exit, _, missing_help) = invoke(["seacad", "--lang", "vi"])?;
        assert_eq!(missing_exit, 2);
        assert!(missing_help.contains("Cách dùng:"));
        assert!(missing_help.contains("Lệnh:"));
        assert!(!missing_help.contains("Usage:"));

        let (subcommand_exit, subcommand_help, _) =
            invoke(["seacad", "inspect", "--lang=vi", "--help"])?;
        assert_eq!(subcommand_exit, 0);
        assert!(subcommand_help.contains("Đối số:"));
        assert!(subcommand_help.contains("Tệp DXF cần đọc"));
        assert!(subcommand_help.contains("--mode <CHẾ_ĐỘ>"));
        assert!(!subcommand_help.contains("[default:"));
        assert!(!subcommand_help.contains("possible values"));

        let (usage_exit, _, usage_error) = invoke(["seacad", "--lang", "vi", "unknown"])?;
        assert_eq!(usage_exit, 2);
        assert!(usage_error.contains("Lỗi: không nhận ra lệnh"));
        assert!(usage_error.contains("Cách dùng:"));
        assert!(!usage_error.contains("error:"));
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
    fn vietnamese_human_output_preserves_codes_and_redaction() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let valid = directory.write("ten-rieng.dxf", STRICT)?;
        let valid_text = path_text(&valid)?;
        let success = invoke_file("inspect", &valid, &["--lang", "vi"])?;
        assert_eq!(success.0, 0);
        assert!(success.2.is_empty());
        assert!(!success.1.contains(valid_text));
        assert!(success.1.contains("Trạng thái: TỐT"));
        assert!(success.1.contains("Chế độ đọc: tương thích (compatible)"));
        assert!(success.1.contains("Số group: 4"));

        let recovered = directory.write("thieu-eof.dxf", b"0\nSECTION\n0\nENDSEC\n")?;
        let failure = invoke_file(
            "verify",
            &recovered,
            &["--mode", "compatible", "--lang", "vi"],
        )?;
        assert_eq!(failure.0, 1);
        assert!(failure.1.is_empty());
        assert!(failure.2.contains("Trạng thái: CHƯA XÁC MINH"));
        assert!(failure.2.contains("DXF-W0203 cảnh báo"));
        assert!(failure.2.contains("Lỗi CLI-E0004:"));
        assert!(!failure.2.contains("Recovered framing"));
        Ok(())
    }

    #[test]
    fn json_v1_changes_only_declared_language_for_localized_success() -> Result<(), Box<dyn Error>>
    {
        let directory = TestDirectory::new()?;
        let input = directory.write("stable-json.dxf", STRICT)?;
        let english = invoke_file("inspect", &input, &["--json"])?;
        let vietnamese = invoke_file("inspect", &input, &["--json", "--lang", "vi"])?;
        assert_eq!(english.0, 0);
        assert_eq!(vietnamese.0, 0);

        let english_report: Value = serde_json::from_str(&english.1)?;
        let mut vietnamese_report: Value = serde_json::from_str(&vietnamese.1)?;
        assert_eq!(english_report["options"]["language"], "en");
        assert_eq!(vietnamese_report["options"]["language"], "vi");
        assert_eq!(vietnamese_report["status"], "ok");
        vietnamese_report["options"]["language"] = Value::String("en".to_owned());
        assert_eq!(vietnamese_report, english_report);

        let recovered = directory.write("json-error.dxf", b"0\nSECTION\n0\nENDSEC\n")?;
        let failed = invoke_file(
            "verify",
            &recovered,
            &["--mode", "compatible", "--json", "--lang", "vi"],
        )?;
        assert_eq!(failed.0, 1);
        let failed_report: Value = serde_json::from_str(&failed.1)?;
        assert_eq!(failed_report["status"], "not_verified");
        assert_eq!(
            failed_report["error"]["code"],
            super::CLI_RECOVERED_NOT_VERIFIED
        );
        assert_eq!(
            failed_report["error"]["message"],
            "Recovered framing is inspect/verbatim-only and is not verified"
        );
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
    fn binary_inspect_and_verify_use_unchanged_json_v1_shape() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let binary = binary_fixture(true);
        let binary_path = directory.write("binary-secret.dxf", &binary)?;
        let binary_result = invoke_file("inspect", &binary_path, &["--json"])?;
        assert_eq!(binary_result.0, 0);
        let binary_report: Value = serde_json::from_str(&binary_result.1)?;
        assert_eq!(binary_report["schema_version"], "v1");
        assert_eq!(binary_report["status"], "ok");
        assert_eq!(binary_report["format"]["physical"], "binary");
        assert_eq!(binary_report["document"]["conformance"], "strict");
        assert_eq!(binary_report["document"]["groups"], 6);
        assert_eq!(binary_report["document"]["eof_occurrence"], 5);
        assert_eq!(binary_report["document"]["trailing_bytes"], 0);
        assert_eq!(binary_report["source"]["path"], Value::Null);
        assert!(binary_report["source"]["id"].is_string());
        assert_eq!(binary_report["error"], Value::Null);

        let verified = invoke_file("verify", &binary_path, &["--json"])?;
        assert_eq!(verified.0, 0);
        let report: Value = serde_json::from_str(&verified.1)?;
        assert_eq!(report["status"], "verified");
        assert_eq!(report["options"]["read_mode"], "strict");
        assert_eq!(report["format"]["physical"], "binary");
        Ok(())
    }

    #[test]
    fn binary_recovery_errors_and_unknown_input_keep_stable_codes() -> Result<(), Box<dyn Error>> {
        let directory = TestDirectory::new()?;
        let missing = directory.write("binary-missing.dxf", &binary_fixture(false))?;
        let inspected = invoke_file("inspect", &missing, &["--json"])?;
        assert_eq!(inspected.0, 0);
        let report: Value = serde_json::from_str(&inspected.1)?;
        assert_eq!(report["status"], "recovered");
        assert_eq!(report["diagnostics"][0]["code"], "DXF-W0210");

        let verified = invoke_file("verify", &missing, &["--mode", "compatible", "--json"])?;
        assert_eq!(verified.0, 1);
        let report: Value = serde_json::from_str(&verified.1)?;
        assert_eq!(report["status"], "not_verified");
        assert_eq!(report["error"]["code"], super::CLI_RECOVERED_NOT_VERIFIED);

        let mut trailing = binary_fixture(true);
        trailing.extend_from_slice(b"tail");
        let trailing_path = directory.write("binary-tail.dxf", &trailing)?;
        let inspected = invoke_file("inspect", &trailing_path, &["--json"])?;
        assert_eq!(inspected.0, 0);
        let report: Value = serde_json::from_str(&inspected.1)?;
        assert_eq!(report["status"], "recovered");
        assert_eq!(report["document"]["trailing_bytes"], 4);
        assert_eq!(report["diagnostics"][0]["code"], "DXF-W0211");

        let mut malformed = seacad_dxf_core::DXF_BINARY_SENTINEL.to_vec();
        malformed.extend_from_slice(b"payload");
        let malformed_path = directory.write("binary-invalid-secret.dxf", &malformed)?;
        let malformed_text = path_text(&malformed_path)?;
        let failed = invoke_file("inspect", &malformed_path, &["--json"])?;
        assert_eq!(failed.0, 1);
        assert!(!failed.1.contains(malformed_text));
        let report: Value = serde_json::from_str(&failed.1)?;
        assert_eq!(report["status"], "invalid");
        assert_eq!(report["format"]["physical"], "binary");
        assert_eq!(report["error"]["code"], "DXF-E0216");
        assert!(report["source"]["id"].is_string());

        let human = invoke_file("inspect", &malformed_path, &["--lang", "vi"])?;
        assert_eq!(human.0, 1);
        assert!(human.2.contains("DXF-E0216"));
        assert!(human.2.contains("record mở đầu 0/SECTION chuẩn"));
        assert!(!human.2.contains("canonical opening"));

        let unknown_path = directory.write("unknown-secret.dxf", b"")?;
        let unknown_result = invoke_file("inspect", &unknown_path, &["--json"])?;
        assert_eq!(unknown_result.0, 1);
        let unknown_report: Value = serde_json::from_str(&unknown_result.1)?;
        assert_eq!(unknown_report["status"], "invalid");
        assert_eq!(unknown_report["error"]["code"], super::CLI_UNKNOWN_FORMAT);
        Ok(())
    }

    fn binary_fixture(include_eof: bool) -> Vec<u8> {
        let mut bytes = seacad_dxf_core::DXF_BINARY_SENTINEL.to_vec();
        binary_pair(&mut bytes, 0, b"SECTION\0");
        binary_pair(&mut bytes, 2, b"HEADER\0");
        binary_pair(&mut bytes, 9, b"$ACADVER\0");
        binary_pair(&mut bytes, 1, b"AC1032\0");
        binary_pair(&mut bytes, 0, b"ENDSEC\0");
        if include_eof {
            binary_pair(&mut bytes, 0, b"EOF\0");
        }
        bytes
    }

    fn binary_pair(bytes: &mut Vec<u8>, code: i16, value: &[u8]) {
        bytes.extend_from_slice(&code.to_le_bytes());
        bytes.extend_from_slice(value);
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
