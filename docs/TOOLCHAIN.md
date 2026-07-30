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

The supplemental jobs pin `actions/checkout` v4.2.2 commit
`11bd71901bbe5b1630ceea73d27597364c9af683`, whose action manifest uses the
Node20 runtime. The baseline matrix and dependency-policy workflow retain the
reviewed v6.0.2 Node24 pin. This split is required because all three
supplemental hosted runners repeatedly failed to resolve the v6 action before
checkout while the baseline jobs resolved the same v6 SHA successfully.

A daily schedule and manual dispatch run the full formatting, generated-schema,
Clippy, and workspace-test gate on all six native platforms. The three new
push/pull-request jobs remain staged rather than required until twenty
consecutive nightly six-platform runs pass. A runner outage or preview-runner
failure is recorded as a failed evidence run and is never converted into a
passing Core 1.0 receipt.

## Q2.2 offline corpus receipt harness

Q2.2 adds the workspace binary `seacad-corpus-receipt`. It uses the existing
Rust, `serde`, `serde_json`, and DXF core dependencies; no package or external
tool is added.

Run it against an offline corpus with:

```text
cargo +1.97.1 run --locked -p seacad-cli --bin seacad-corpus-receipt -- \
  corpus/offline-manifest.json <offline-corpus-root>
```

The committed manifest contains only public policy and aggregate bounds.
Private inputs remain outside Git, normally under the ignored
`corpus-private/` directory. Receipt JSON records OS/architecture and aggregate
counts only; it omits paths, filenames, source identities, and per-file hashes.
The harness uses strict framing and the existing `Safe` resource profile.

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

## M13.1a release evidence

M13.1a adds the internal `seacad-release-evidence` binary. It invokes the
pinned Cargo metadata command with `--locked`, joins every registry component
to its exact `Cargo.lock` checksum, verifies that every third-party
package/version is listed in `THIRD_PARTY_NOTICES.md`, and renders a
host-independent CycloneDX 1.6 document at `release/sbom.cdx.json`.

Regenerate only after an intentional locked dependency review:

```text
cargo +1.97.1 run --locked -p seacad-schema-gen \
  --bin seacad-release-evidence -- --write
```

Required local and six-native CI gate:

```text
cargo +1.97.1 run --locked -p seacad-schema-gen \
  --bin seacad-release-evidence -- --check
```

The SBOM intentionally omits timestamps, local registry paths, and host
identifiers. The M13.1a checkpoint alone did not package standalone license
files; M13.1b below closes that scoped gap. Neither checkpoint closes the
corpus/native release receipts.

## M13.1b legal bundle

The same `seacad-release-evidence` command also generates and checks
`release/legal/`. It copies the project `LICENSE`, `NOTICE`, and
`THIRD_PARTY_NOTICES.md`, then preserves every regular root license artifact
from each locked crates.io source tree. Accepted filenames begin with
`LICENSE`, `LICENCE`, `COPYING`, `UNLICENSE`, or `NOTICE`.
Project text is normalized to the repository's canonical LF representation;
crate license artifacts remain byte-exact and are protected from Git text
normalization.

`release/legal/manifest.json` binds each raw file to its byte count and SHA-256,
and binds each package directory to name, version, declared SPDX expression,
exact crate checksum, and `Cargo.lock` identity. `--check` also walks the
committed directory and rejects missing, changed, unexpected, symlinked, or
non-regular entries. The M13.1a `--write` and `--check` commands therefore
cover both SBOM and legal bundle; no additional tool is required.

## M13.2a corpus release gate

Q2.2's `corpus/offline-manifest.json` and v1 receipt remain available for
bounded exploratory corpus verification. They do not assert release scale.

M13.2a adds `corpus/release-manifest.json` and receipt v2. Run the same
path-redacted harness with the release policy:

```text
cargo +1.97.1 run --locked -p seacad-cli \
  --bin seacad-corpus-receipt -- \
  corpus/release-manifest.json <offline-corpus-root>
```

The committed release policy requires at least 1,000 verified DXF files and
10 GiB, with zero invalid files. Independent traversal ceilings are 2,000
files, 20 GiB, 20,000 entries, and depth 32. Exit `0` and
`status="verified"` require all three threshold Booleans to be true. A
completed scan below either minimum emits a redacted `failed` receipt and exits
`1`; it does not add a failure code because no individual file failed.

## M13.2b six-native artifact staging

The `seacad-release-packager` binary assembles one create-new directory for an
exact reviewed target. It accepts an already-built native CLI, an empty output
path whose parent exists, and a 40-character lowercase commit SHA. Example:

```text
mkdir dist
cargo +1.97.1 run --locked -p seacad-schema-gen \
  --bin seacad-release-packager -- \
  --target x86_64-pc-windows-msvc \
  --binary target/x86_64-pc-windows-msvc/release/seacad.exe \
  --output dist/seacad-dxf-core-0.0.0-x86_64-pc-windows-msvc \
  --commit 0123456789abcdef0123456789abcdef01234567
```

The output includes the executable, `README.md`, `sbom.cdx.json`, the complete
`legal/` tree, and `RELEASE_RECEIPT.json`. The receipt binds every payload
file—not itself—to exact bytes and SHA-256. The packager removes only the newly
created output directory if assembly fails and never overwrites an existing
destination.

`.github/workflows/release-artifacts.yml` is manual-dispatch only and grants
`contents: read`. It builds on the same six native runner/target pairs, runs
the workspace tests and release-evidence checks, assembles the directory, and
retains it for 14 days using `actions/upload-artifact` v4.6.2 pinned to commit
`ea165f8d65b6e75b540449e92b4886f43607fa02`. It does not create a tag,
GitHub Release, signature, or permanent archive.

## M13.2c six-native receipt aggregation

After all package jobs succeed, the same workflow downloads artifacts with
`actions/download-artifact` v4.3.0 pinned to commit
`d3f86a106a0bac45b974a628896c90dbdf5c8093`. The download action creates one
directory per artifact under `matrix/`.

The `seacad-release-receipts` binary requires exactly those six package
directories and writes one create-new aggregate:

```text
cargo +1.97.1 run --locked -p seacad-schema-gen \
  --bin seacad-release-receipts -- \
  --root matrix \
  --output matrix/SIX_NATIVE_RECEIPT.json \
  --commit 0123456789abcdef0123456789abcdef01234567
```

It validates every per-target receipt and payload hash, compares all
non-executable payloads with the current checkout, and binds the six receipt
hashes plus total files/bytes to the commit. The aggregate is uploaded for 14
days. The workflow remains manual and read-only and does not publish a release.

The GitHub workflow uses `EmbarkStudios/cargo-deny-action` v2.1.1 pinned to
commit `3c6349835b2b7b196a839186cb8b78e02f7b5f25`. Its checkout step uses
`actions/checkout` v6.0.2 pinned to commit
`de0fac2e4500dabe0009e67214ff5f5447ce83dd`, with credential persistence
disabled.
