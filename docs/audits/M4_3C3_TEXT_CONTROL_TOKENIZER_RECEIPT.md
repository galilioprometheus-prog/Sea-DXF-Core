# M4.3c3 documented text-control tokenizer receipt

Status: local gates passed 2026-07-27; checkpoint CI receipt is the annotated
tag and GitHub Actions run.

Base checkpoint:

- commit: `33b283a5bf25f2332c2375e125c8caaa269e1a9e`;
- tag: `m4.3c2b-johab-decoder`.

## Accepted result

M4.3c3 adds an allocation-free cursor that tokenizes the publicly documented
MTEXT and percent-control families without interpreting formatting values.
Every successful token carries an exact half-open span in decoded UTF-8 and
parameter tokens carry an additional exact, unparsed payload span.

The caller must explicitly select `MText` or `SingleLineText`; SeaCad does not
infer entity semantics from string contents. Unknown, extension-like, and
insufficiently documented sequences remain exact `Text`. Recognized malformed
MTEXT structure returns a typed terminal error while the raw document remains
open and unchanged.

Production scope is 471 lines before `#[cfg(test)]` in the new module plus five
workspace export/module lines. This 476-line production change stays within
the checkpoint review limit. No dependency, `Cargo.toml`, or `Cargo.lock`
change is present.

## Normative and oracle evidence

The committed audit records Autodesk's AutoCAD 2027 alternate-editor format
reference, percent-control reference, MTEXT DXF storage reference, and
formatting examples.

AutoCAD Core Console oracle:

- version: `26.0.60.0.0`;
- executable SHA-256:
  `fc6b59c29fea759d556083cf43d9de3657974ef0bf57962d49a19892f729fa77`;
- `/readonly`, `/safemode`, unique isolated profiles;
- known-valid base plus in-memory MTEXT construction;
- accepted family/context inputs and UTF-16 stdout are frozen by four SHA-256
  receipts in the audit.

The oracle confirms all claimed MTEXT backslash families, eight-level nesting
from Autodesk's normative reference, escaped characters, stacked text,
parameter effects, paragraph break, and context-dependent percent behavior.
It also proves that MTEXT accepts `%%c`, `%%d`, and `%%p` case-insensitively
while preserving the remaining TEXT-only percent controls literally.

One direct hand-authored AC1032 attempt terminated AutoCAD before evidence was
produced. It is disclosed and excluded; no claim or fixture derives from it.

## Verification

- 11 focused tokenizer tests pass.
- The documented-family fixture accounts for every decoded UTF-8 byte exactly
  once and includes every claimed token family.
- The three-stage composition test proves CIF conversion does not consume
  neighboring MTEXT controls.
- Every parameter introducer is tested for typed missing-semicolon failure.
- Valid eight-level, ninth-level overflow, unmatched close, and unclosed block
  cases are tested.
- MTEXT versus single-line percent registries, case behavior, three-digit
  payload spans, unknown controls, UTF-8 text, empty input, and retry after
  terminal completion are tested.
- 380 printable-ASCII introducer/context combinations prove every step either
  advances, completes, or returns a typed terminal error.
- `cargo test -p seacad-dxf-core`: 137 passed.
- `cargo test --workspace`: 148 passed.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- Production forbidden-construct scan: zero unsafe blocks/functions or
  `panic!`, `unwrap()`, `expect()`, `todo!`, and `unimplemented!` hits;
  crate-level `#![forbid(unsafe_code)]` remains active.
- `git diff --check`: passed.

The GitHub Actions matrix must pass on Windows x64, macOS ARM64, and Linux x64
before the checkpoint is reported complete.

## Artifact hashes

| Artifact | SHA-256 |
| --- | --- |
| `crates/seacad-dxf-core/src/lib.rs` | `3ef38c28037d0ea9a5c843ee514eaf430c8f224bd5f062aa68ede7dc7c88bacd` |
| `crates/seacad-dxf-core/src/text_control.rs` | `862c504ebf583acd678725569e235d8d8c68c34bdf1a2ede3506581d5c44773e` |
| `docs/IMPLEMENTATION_PLAN.md` | `366c525242e14817d9f67efdb9fe94a9852985bfb6e907dafb283855370f691c` |
| `docs/SUPPORT_MATRIX.md` | `d2d181cdcd1feac1bb689213e1a2cf9eee99734ac55c3dc81f4210683dcd8d32` |
| `docs/M4_3C3_TEXT_CONTROL_TOKENIZER_CONTRACT.md` | `2f8e9a2062376e37d7adbec21f4a6eec954905defc225db1638e7b3dbd14e5c0` |
| `docs/audits/M4_3C3_AUTODESK_TEXT_CONTROL_REFERENCE.md` | `d7eeb12f710213a4443f2cd032892ebb679f40ad7184c6ce419bc5588d1dd0d8` |

The receipt does not hash itself. The annotated checkpoint tag binds this
receipt and every listed artifact to the final commit.
