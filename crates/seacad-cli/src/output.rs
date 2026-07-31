//! Stable machine-readable and localized human CLI output.

use std::io::{self, Write};

use crate::{locale::CliLanguage, report::CliReport};

pub(super) const CLI_INTERNAL_ERROR: &str = "CLI-E0001";
pub(super) const CLI_UNSUPPORTED_FORMAT: &str = "CLI-E0002";
pub(super) const CLI_UNKNOWN_FORMAT: &str = "CLI-E0003";
pub(super) const CLI_RECOVERED_NOT_VERIFIED: &str = "CLI-E0004";
pub(super) const CLI_OUTPUT_ERROR: &str = "CLI-E0005";

pub(super) fn render_json(writer: &mut dyn Write, report: &CliReport) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut *writer, report).map_err(io::Error::other)?;
    writeln!(writer)
}

pub(super) fn render_text(
    writer: &mut dyn Write,
    report: &CliReport,
    language: CliLanguage,
) -> io::Result<()> {
    let text = language.catalog();
    writeln!(writer, "SeaCad {}", report.command)?;
    writeln!(
        writer,
        "{}: {}",
        text.status_label,
        status_text(language, report.status)
    )?;
    writeln!(
        writer,
        "{}: {}",
        text.read_mode_label,
        read_mode_text(language, report.options.read_mode)
    )?;
    writeln!(
        writer,
        "{}: {}",
        text.resource_profile_label,
        profile_text(language, report.options.resource_profile)
    )?;
    writeln!(
        writer,
        "{}: {}",
        text.physical_format_label,
        physical_text(language, report.format.physical)
    )?;
    if let Some(path) = &report.source.path {
        writeln!(writer, "{}: {path}", text.path_label)?;
    }
    if let Some(bytes) = report.source.bytes {
        writeln!(writer, "{}: {bytes}", text.bytes_label)?;
    }
    if let Some(source_id) = &report.source.id {
        writeln!(writer, "{}: {source_id}", text.source_id_label)?;
    }
    if let Some(document) = &report.document {
        writeln!(
            writer,
            "{}: {}",
            text.conformance_label,
            conformance_text(language, document.conformance)
        )?;
        writeln!(writer, "{}: {}", text.groups_label, document.groups)?;
        let eof = document
            .eof_occurrence
            .map_or_else(|| text.absent.to_owned(), |value| value.to_string());
        writeln!(writer, "{}: {eof}", text.eof_occurrence_label)?;
        writeln!(
            writer,
            "{}: {}",
            text.trailing_bytes_label, document.trailing_bytes
        )?;
    }
    writeln!(
        writer,
        "{}: {}",
        text.diagnostics_label,
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
            text.error_label,
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

#[cfg(test)]
mod tests {
    use super::{
        CLI_INTERNAL_ERROR, CLI_OUTPUT_ERROR, CLI_RECOVERED_NOT_VERIFIED, CLI_UNKNOWN_FORMAT,
        CLI_UNSUPPORTED_FORMAT, conformance_text, error_text, physical_text, profile_text,
        read_mode_text, severity_text, status_text,
    };
    use crate::locale::CliLanguage;

    #[test]
    fn localized_output_mappings_are_complete_and_stable() {
        let statuses = [
            ("ok", "OK", "TỐT"),
            ("verified", "VERIFIED", "ĐÃ XÁC MINH"),
            ("recovered", "RECOVERED", "ĐÃ PHỤC HỒI"),
            ("not_verified", "NOT VERIFIED", "CHƯA XÁC MINH"),
            ("invalid", "INVALID", "KHÔNG HỢP LỆ"),
            ("unsupported", "UNSUPPORTED", "CHƯA HỖ TRỢ"),
        ];
        for (value, english, vietnamese) in statuses {
            assert_eq!(status_text(CliLanguage::English, value), english);
            assert_eq!(status_text(CliLanguage::Vietnamese, value), vietnamese);
        }

        let physical_formats = [
            (
                "ascii_candidate",
                "ASCII DXF candidate",
                "ứng viên DXF ASCII",
            ),
            ("binary", "Binary DXF", "DXF nhị phân"),
            ("unknown", "Unknown", "không xác định"),
        ];
        for (value, english, vietnamese) in physical_formats {
            assert_eq!(physical_text(CliLanguage::English, value), english);
            assert_eq!(physical_text(CliLanguage::Vietnamese, value), vietnamese);
        }

        let vietnamese_mappings = [
            (
                read_mode_text(CliLanguage::Vietnamese, "strict"),
                "nghiêm ngặt (strict)",
            ),
            (
                read_mode_text(CliLanguage::Vietnamese, "compatible"),
                "tương thích (compatible)",
            ),
            (
                profile_text(CliLanguage::Vietnamese, "safe"),
                "an toàn (safe)",
            ),
            (
                profile_text(CliLanguage::Vietnamese, "large"),
                "lớn (large)",
            ),
            (
                conformance_text(CliLanguage::Vietnamese, "strict"),
                "nghiêm ngặt (strict)",
            ),
            (
                conformance_text(CliLanguage::Vietnamese, "recovered"),
                "đã phục hồi (recovered)",
            ),
            (severity_text(CliLanguage::Vietnamese, "info"), "thông tin"),
            (
                severity_text(CliLanguage::Vietnamese, "warning"),
                "cảnh báo",
            ),
            (severity_text(CliLanguage::Vietnamese, "error"), "lỗi"),
        ];
        for (actual, expected) in vietnamese_mappings {
            assert_eq!(actual, expected);
        }
        assert_eq!(read_mode_text(CliLanguage::English, "strict"), "strict");
    }

    #[test]
    fn localized_error_mappings_are_complete_and_stable() {
        let translations = [
            (CLI_INTERNAL_ERROR, "trạng thái nội bộ của CLI không hợp lệ"),
            (CLI_UNSUPPORTED_FORMAT, "định dạng DXF này chưa được hỗ trợ"),
            (
                CLI_UNKNOWN_FORMAT,
                "nguồn không phải ứng viên DXF ASCII hoặc Binary đã biết",
            ),
            (
                CLI_RECOVERED_NOT_VERIFIED,
                "framing phục hồi chỉ được kiểm tra hoặc sao chép nguyên trạng",
            ),
            (CLI_OUTPUT_ERROR, "không thể ghi đầu ra CLI"),
            ("DXF-E0001", "thao tác vào/ra thất bại"),
            ("DXF-E0002", "thao tác đã bị hủy"),
            ("DXF-E0101", "nguồn vượt giới hạn số byte"),
            ("DXF-E0102", "nguồn vượt giới hạn số record"),
            ("DXF-E0103", "giá trị vượt giới hạn số byte"),
            ("DXF-E0105", "offset byte bị tràn số nguyên"),
            ("DXF-E0201", "group code ASCII không hợp lệ"),
            ("DXF-E0202", "group ASCII không có dòng giá trị"),
            ("DXF-E0203", "tài liệu ASCII không có marker 0/EOF kết thúc"),
            ("DXF-E0204", "tài liệu ASCII nghiêm ngặt có dữ liệu sau EOF"),
            ("DXF-E0210", "sentinel Binary DXF không hợp lệ"),
            ("DXF-E0211", "group code Binary DXF bị cắt ngắn"),
            ("DXF-E0212", "group code Binary DXF không hợp lệ"),
            (
                "DXF-E0213",
                "group code Binary DXF chưa có wire family được công bố",
            ),
            ("DXF-E0214", "giá trị Binary DXF bị cắt ngắn"),
            ("DXF-E0215", "chuỗi Binary DXF không có byte NUL kết thúc"),
            (
                "DXF-E0216",
                "Binary DXF không có record mở đầu 0/SECTION chuẩn",
            ),
            (
                "DXF-E0217",
                "Binary DXF không có đúng một HEADER $ACADVER được hỗ trợ",
            ),
            (
                "DXF-E0218",
                "encoding Binary DXF không khớp dialect đã khai báo",
            ),
            (
                "DXF-E0219",
                "tài liệu Binary DXF không có marker 0/EOF kết thúc",
            ),
            (
                "DXF-E0220",
                "tài liệu Binary DXF nghiêm ngặt có dữ liệu sau EOF",
            ),
            ("DXF-E0301", "định danh nguồn đã thay đổi"),
            ("DXF-E0302", "độ dài đầu ra Verbatim không khớp"),
            ("DXF-E0303", "định danh đầu ra Verbatim không khớp"),
        ];
        for (code, expected) in translations {
            assert_eq!(
                error_text(CliLanguage::Vietnamese, code, "English fallback"),
                expected
            );
        }
        assert_eq!(
            error_text(CliLanguage::English, "DXF-E0001", "DXF-E0001: I/O failed"),
            "I/O failed"
        );
        assert_eq!(
            error_text(CliLanguage::Vietnamese, "DXF-E9999", "English fallback"),
            "English fallback"
        );
    }
}
