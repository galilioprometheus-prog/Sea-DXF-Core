//! Explicit, platform-independent localization for human CLI output.

use std::{
    ffi::{OsStr, OsString},
    io::{self, Write},
};

use clap::{
    Error,
    error::{ContextKind, ErrorKind},
};

const HELP_TEMPLATE_EN: &str =
    "{before-help}{name}\n{about-with-newline}\nUsage: {usage}\n\n{all-args}{after-help}";
const HELP_TEMPLATE_VI: &str =
    "{before-help}{name}\n{about-with-newline}\nCách dùng: {usage}\n\n{all-args}{after-help}";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum CliLanguage {
    #[default]
    English,
    Vietnamese,
}

impl CliLanguage {
    pub(crate) fn detect(args: &[OsString]) -> Self {
        let mut language = Self::English;
        let mut index = 1;
        while let Some(argument) = args.get(index) {
            if argument == OsStr::new("--") {
                break;
            }
            if argument == OsStr::new("--lang") {
                if let Some(value) = args.get(index + 1).and_then(|value| value.to_str())
                    && let Some(selected) = Self::from_code(value)
                {
                    language = selected;
                }
            } else if let Some(value) = argument
                .to_str()
                .and_then(|value| value.strip_prefix("--lang="))
                && let Some(selected) = Self::from_code(value)
            {
                language = selected;
            }
            index += 1;
        }
        language
    }

    pub(crate) fn from_code(code: &str) -> Option<Self> {
        match code {
            "en" => Some(Self::English),
            "vi" => Some(Self::Vietnamese),
            _ => None,
        }
    }

    pub(crate) const fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Vietnamese => "vi",
        }
    }

    pub(crate) const fn pick<'a>(self, english: &'a str, vietnamese: &'a str) -> &'a str {
        match self {
            Self::English => english,
            Self::Vietnamese => vietnamese,
        }
    }

    pub(crate) const fn help_template(self) -> &'static str {
        self.pick(HELP_TEMPLATE_EN, HELP_TEMPLATE_VI)
    }
}

pub(crate) fn write_usage_error(
    writer: &mut dyn Write,
    error: &Error,
    language: CliLanguage,
) -> io::Result<()> {
    if language == CliLanguage::English {
        return write!(writer, "{error}");
    }

    writeln!(writer, "Lỗi: {}", vietnamese_error_kind(error.kind()))?;
    write_context(writer, error, ContextKind::InvalidSubcommand, "Lệnh")?;
    write_context(writer, error, ContextKind::InvalidArg, "Đối số")?;
    write_context(writer, error, ContextKind::InvalidValue, "Giá trị")?;
    write_context(writer, error, ContextKind::ValidValue, "Giá trị hợp lệ")?;
    write_context(
        writer,
        error,
        ContextKind::SuggestedSubcommand,
        "Lệnh gợi ý",
    )?;
    write_context(writer, error, ContextKind::SuggestedArg, "Đối số gợi ý")?;
    write_context(writer, error, ContextKind::SuggestedValue, "Giá trị gợi ý")?;
    if let Some(usage) = error.get(ContextKind::Usage) {
        let usage = usage.to_string();
        let usage = usage.strip_prefix("Usage: ").unwrap_or(usage.as_str());
        writeln!(writer, "\nCách dùng: {usage}")?;
    }
    writeln!(writer, "\nChạy `seacad --lang vi --help` để xem trợ giúp.")
}

fn write_context(
    writer: &mut dyn Write,
    error: &Error,
    kind: ContextKind,
    label: &str,
) -> io::Result<()> {
    if let Some(value) = error.get(kind) {
        writeln!(writer, "{label}: {value}")?;
    }
    Ok(())
}

const fn vietnamese_error_kind(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::InvalidValue => "giá trị đối số không hợp lệ",
        ErrorKind::UnknownArgument => "không nhận ra đối số",
        ErrorKind::InvalidSubcommand => "không nhận ra lệnh",
        ErrorKind::NoEquals => "tùy chọn này yêu cầu dấu bằng",
        ErrorKind::ValueValidation => "giá trị không vượt qua kiểm tra",
        ErrorKind::TooManyValues => "có quá nhiều giá trị",
        ErrorKind::TooFewValues => "có quá ít giá trị",
        ErrorKind::WrongNumberOfValues => "số lượng giá trị không đúng",
        ErrorKind::ArgumentConflict => "các đối số xung đột nhau",
        ErrorKind::MissingRequiredArgument => "thiếu đối số bắt buộc",
        ErrorKind::MissingSubcommand => "thiếu lệnh bắt buộc",
        ErrorKind::InvalidUtf8 => "đối số không phải UTF-8 hợp lệ",
        ErrorKind::Io => "không thể đọc hoặc ghi dữ liệu dòng lệnh",
        ErrorKind::Format => "không thể định dạng thông báo dòng lệnh",
        _ => "dòng lệnh không hợp lệ",
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::CliLanguage;

    #[test]
    fn language_detection_is_explicit_and_defaults_to_english() {
        assert_eq!(
            CliLanguage::detect(&arguments(&["seacad", "inspect", "a.dxf"])),
            CliLanguage::English
        );
        assert_eq!(
            CliLanguage::detect(&arguments(&["seacad", "inspect", "a.dxf", "--lang", "vi"])),
            CliLanguage::Vietnamese
        );
        assert_eq!(
            CliLanguage::detect(&arguments(&["seacad", "--lang=en", "--help"])),
            CliLanguage::English
        );
    }

    #[test]
    fn language_detection_does_not_interpret_values_after_separator() {
        assert_eq!(
            CliLanguage::detect(&arguments(&["seacad", "inspect", "--", "--lang", "vi"])),
            CliLanguage::English
        );
    }

    fn arguments(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }
}
