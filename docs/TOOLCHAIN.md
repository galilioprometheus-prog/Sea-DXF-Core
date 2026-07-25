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
