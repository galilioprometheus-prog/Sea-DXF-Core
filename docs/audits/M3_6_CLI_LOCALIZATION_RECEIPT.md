# M3.6 CLI localization receipt

Status: completed 2026-07-27

This receipt records deterministic evidence for explicit English/Vietnamese
human CLI output. It uses one synthetic DXF byte vector and contains no private
CAD data.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-cli/src/main.rs` | `76c3f41b7d810cbf666a2951ea2737e33f08d92e7998bf72d0f146d0102fe069` |
| `crates/seacad-cli/src/locale.rs` | `2cfe3512b929b54ec72e1a15d2c5f6fcb51b25c53771172e50c98a854efa16f1` |
| `docs/CLI_JSON_V1.md` | `9ca594010b98bb97e9477117f5d0f0578904019ad9f814c003f972378ddadf56` |
| `README.md` | `bae3b42fe9a6b354deadf725c849397b95d2742f394dff978668277e71c4a499` |

M3.6 adds no dependency and does not change Cargo.toml or Cargo.lock. No
legacy/external source, translation catalog, or fixture is imported.

## Language contract

| Case | Required result |
| --- | --- |
| no `--lang` | English human output; JSON `options.language` is `en` |
| `--lang en` or `--lang=en` | Explicit English |
| `--lang vi` or `--lang=vi` | Vietnamese human help, usage errors, and reports |
| argument after literal `--` | Never interpreted as a locale option |
| `--lang vi --json` | JSON identifiers/messages remain English; only declared language changes |

Locale selection is independent of operating-system or terminal settings. All
source files are strict UTF-8 without requiring a BOM.

## Native smoke vector

The Windows native binary was run against `strict-ascii`, the 36-byte synthetic
vector with SHA-256
`dc919c8f27c3229e9c4a240278f07819b2ed960642d983ceefc6604750b621f2`.

| Command/case | Exit | Required observation |
| --- | ---: | --- |
| `seacad --lang vi --help` | 0 | Vietnamese usage, command, option headings; no English `Usage:` |
| `seacad --lang vi` | 2 | Vietnamese missing-command error and usage; stdout empty |
| `seacad inspect strict-ascii --lang vi` | 0 | Vietnamese OK/mode/group labels; source path absent |
| `seacad inspect strict-ascii --json --lang vi` | 0 | schema `v1`, status `ok`, language `vi`, four groups, null path |
| `seacad --lang vi unknown` | 2 | Vietnamese command error and usage guidance |

A separate recovered missing-EOF vector verifies Vietnamese diagnostic
severity, `CLI-E0004`, exit 1, and no fallback English CLI message.

## Test coverage and review size

Eleven CLI tests cover the six M3.5 behavior groups plus explicit locale
detection, the `--` separator, root/subcommand/missing-command help, Vietnamese
usage errors, success/failure text, redaction, Unicode labels, and JSON
equivalence. M3.6 adds 379 production lines,
within the repository's 200-500-line micro-milestone target.

## Checkpoint gates

The checkpoint requires Rustfmt, workspace Clippy with warnings denied, every
workspace test, strict UTF-8 validation, forbidden-production-pattern scan,
`git diff --check`, and the three-platform GitHub Actions matrix.
