# M7.1b source-anchored raw handle projection

M7.1b projects one already indexed raw group occurrence into a handle-valued
view. It composes the M7.1a numeric group-code registry with the existing exact
hexadecimal parser and format-neutral raw-document spans. The result remains a
local interpretation of source bytes, not a resolved object or graph edge.

## Normative boundary

Autodesk's numerical group-code reference defines handle identity, arbitrary,
soft/hard pointer, soft/hard owner, plot-style, and extended-data handle codes:

<https://help.autodesk.com/cloudhelp/2024/ENU/AutoCAD-DXF/files/GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9.htm>

Autodesk states separately that `1005` XDATA handles have soft-pointer behavior:

<https://help.autodesk.com/cloudhelp/2021/ENU/AutoCAD-DXF/files/GUID-8243079C-B44F-493A-BAF7-1D11A6E6C78C.htm>

Those sources justify classifying and parsing a handle-valued group. They do
not, by themselves, prove record boundaries, target existence, graph topology,
dictionary membership, reactor scope, or edit-time translation behavior.

## Public contract

`DxfRawDocumentView`, `DxfAsciiRawDocument`, and `DxfBinaryRawDocument` expose
`raw_handle_at(occurrence, cancellation)`. `DxfRawHandleLookup` distinguishes:

- `MissingOccurrence` when the bounded raw index has no such occurrence;
- `NotHandleGroup` with the unchanged `DxfRawGroup` metadata when its numeric
  code is outside the M7.1a registry;
- `Handle` with a `DxfRawHandleValue` when the code is handle-valued.

`DxfRawHandleValue` retains document source identity, exact group metadata,
the M7.1a `DxfHandleGroupClass`, and either the parsed `DxfHandle` or an exact
`DxfHandleParseIssue`. `read_raw_spelling` revalidates source identity and the
stored group occurrence/span before reading the original bytes.

The shared parser uses a fixed 16-byte stack buffer, checks cancellation before
and after source I/O, rejects empty or overlong payloads, preserves invalid
digit offsets, performs no allocation, and never normalizes letter case or
leading zeroes. The HEADER handle directory now reuses this same internal
primitive instead of retaining a second copy.

## Explicit non-claims

The projection does not identify which entity or object owns a group, pair a
group with a record marker, create a global handle index, resolve a target,
validate null or dangling handles, infer a back-reference, or implement
dictionaries, reactors, XDATA containers, cloning, INSERT, XREF, editing, or
writing. These remain later M7 work.

## Antigravity handoff

Before implementation, Antigravity Agent Hub task 19 performed a read-only
inventory at HEAD `edd6dff6eedfc2853ae2357b8cdefb6847d91ef1`. It located the
format-neutral group metadata and adapters, source/span reads, cancellation
patterns, existing fixed-buffer HEADER parser, and relevant tests. All eight
raw-document-filtered tests passed, Git state remained unchanged, and the
worker wrote no files. Codex independently reproduced the material inventory
and tests before accepting the task.

## Verification scope

Dedicated public integration tests cover ASCII and Binary projection for all
six handle classes, exact mixed-case and leading-zero spelling, typed empty,
overlong, and invalid-digit failures, missing and non-handle groups, pre-R13
Binary XDATA escape parity, pre- and post-read cancellation, source-identity
mismatch, and public `Copy + Send + Sync` traits.

The handwritten production diff adds 131 lines and removes 19. Its 127-line
new module is intentionally below the usual 200-line production target because
it composes existing raw-document and handle contracts and removes the duplicate
HEADER parsing primitive. The dedicated integration test adds 282 physical
lines. No manifest, dependency, lockfile, schema, generated file, fixture,
writer, or CLI contract changes.

## Reviewed implementation artifacts

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/raw_handle.rs` | `4171bfed01fd4811c968db3cfb69037d48c2a5ec74bb633910fbca3de72fe79f` |
| `crates/seacad-dxf-core/src/header_handle.rs` | `f1f0896b57a567437d1b45870042de1df838b338d6b8cffb7b7375cb396ef75d` |
| `crates/seacad-dxf-core/src/lib.rs` | `d094ce9f0c5f792cfdf7b3920de4417170c7c1c2ad3fd06187c90c8e4239d1e2` |
| `crates/seacad-dxf-core/tests/raw_handle_tests.rs` | `825592e0c827cd9bfa01bd3597eb40b9d4f159895b69285e7b223c2fa321a18e` |
| `docs/IMPLEMENTATION_PLAN.md` | `90b05a8fa83a2bae3f4b435376f532ee66231af7728a038048afa2e5ed8935b8` |
| `docs/SUPPORT_MATRIX.md` | `23942b540bb6888218ed96e771d154f6bb567672c79a82939b9ce7c24a910949` |

## Required gates

Rust 1.97.1 passed:

- `cargo deny --locked check` (advisories, bans, licenses, and sources);
- `cargo +1.97.1 fmt --all -- --check`;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`;
- `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings`;
- `cargo +1.97.1 test --workspace`: 263 passed, comprising 204 core unit
  tests, four HEADER handle tests, 12 numeric tests, four text tests, four raw
  handle projection tests, 16 CLI tests, six corpus-receipt tests, and 13
  schema-generator tests;
- `git diff --check`.

The focused raw-handle integration command passed four tests with zero
failures. The reviewed production diff contains no added `unsafe`, `panic!`,
`unwrap`, `expect`, `todo!`, or `unimplemented!` use.
