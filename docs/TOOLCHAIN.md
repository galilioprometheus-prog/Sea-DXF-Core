# Toolchain Baseline

M0 baseline recorded on 2026-07-26:

- Git 2.55.0
- GitHub CLI 2.96.0
- Git LFS 3.7.1
- Rustup 1.29.0
- Rust/Cargo 1.97.1, host `x86_64-pc-windows-msvc`
- Rust targets: `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`
- Components: rustfmt, Clippy, LLVM tools
- Visual Studio Build Tools 2022 17.14.37
- MSVC x64/x86 and ARM64 tools
- Windows SDK
- AutoCAD 2027 `acad.exe` and `accoreconsole.exe` available as future M3
  behavioral oracles
- MicroStation V8i available for the later DGN program

OpenSpace, ODA File Converter, GUI frameworks, scripting runtimes, Wasmtime,
and extra Cargo QA tools are intentionally not installed by M0.

## Q2.1 native CI matrix

Q2.1 retains the existing full quality gate on every push and pull request:

- Linux x64: `ubuntu-24.04`;
- Windows x64: `windows-2025`;
- macOS ARM64: `macos-15`.

It adds native schema generation, workspace checking, and core smoke tests on:

- Linux ARM64: `ubuntu-24.04-arm`;
- Windows ARM64: `windows-11-arm`;
- macOS x64: `macos-15-intel`.

A daily schedule and manual dispatch run the full formatting, generated-schema,
Clippy, and workspace-test gate on all six native platforms. The three new
push/pull-request jobs remain staged rather than required until twenty
consecutive nightly six-platform runs pass. A runner outage or preview-runner
failure is recorded as a failed evidence run and is never converted into a
passing Core 1.0 receipt.

## Q1 dependency-policy tool

Q1 adds `cargo-deny 0.20.2` as a separately installed development tool. It is
not a workspace or runtime dependency.

Local installation:

```text
cargo install --locked cargo-deny --version 0.20.2
```

Required local gate:

```text
cargo deny --locked check
```

The GitHub workflow uses `EmbarkStudios/cargo-deny-action` v2.1.1 pinned to
commit `3c6349835b2b7b196a839186cb8b78e02f7b5f25`. Its checkout step uses
`actions/checkout` v6.0.2 pinned to commit
`de0fac2e4500dabe0009e67214ff5f5447ce83dd`, with credential persistence
disabled.
