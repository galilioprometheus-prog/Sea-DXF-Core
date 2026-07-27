# M5.1a Binary DXF wire-registry receipt

Status: local gates passed 2026-07-27; checkpoint CI receipt is the annotated
tag and GitHub Actions run.

Base checkpoint:

- commit: `02f7ddb77811e0954fcc00ba9188ceb66beccd34`;
- tag: `m4.3c3-text-control-tokenizer`.

## Accepted result

M5.1a adds an allocation-free decoder for the explicit pre-R13 one-byte/XDATA
escape and R13-and-later signed 16-bit little-endian group-code headers. A
successful result returns the typed group code and exact consumed wire width;
truncation and invalid encodings return stable source-anchored fatal errors.

The same module freezes every documented Binary DXF value family, including
two-byte group 280 values, one-byte group 290 Boolean values, XDATA group 1004
binary chunks, 64-bit integers, 32-bit long values, strings/handles, and all
reserved gaps. It does not claim source streaming or Binary document opening.

Production scope is 150 lines before `#[cfg(test)]` in the new module, 43 error
contract/test lines, and five workspace module/export lines: 198 production
lines total. No dependency, `Cargo.toml`, or `Cargo.lock` change is present.

## Normative and oracle evidence

The committed audit records Autodesk's current Binary DXF, group-value type,
and XDATA references, plus the conflicting historical pre-R14 wording.
SeaCad follows the current pre-R13 rule and does not conceal the conflict.

AutoCAD Core Console oracle:

- version: `26.0.60.0.0`;
- executable SHA-256:
  `fc6b59c29fea759d556083cf43d9de3657974ef0bf57962d49a19892f729fa77`;
- `/readonly`, `/safemode`, unique isolated profiles;
- known-valid AC1009 input and new-output-only Binary SAVEAS paths.

AutoCAD-generated external fixtures establish one-byte AC1009 group codes,
two-byte AC1015/AC1032 group codes, two-byte group 280 values, and one-byte
group 290 values. Their three hashes and all script/runner hashes are frozen
in the audit. These behavioral fixtures remain outside the repository.

## Verification

- Seven focused wire tests pass.
- AutoCAD boundary byte sequences decode to group 0 with wire widths one and
  two respectively.
- Pre-R13 byte 255 accepts only the documented 1000-1071 XDATA range.
- Signed -5 through 1071 domain boundaries, all family boundaries, reserved
  gaps, special groups 999/1004/1005, exact error spans, and stable codes are
  tested.
- `cargo test -p seacad-dxf-core binary_wire`: 7 passed.
- `cargo test --workspace`: 155 passed (144 core, 11 CLI).
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
| `crates/seacad-dxf-core/src/binary_wire.rs` | `860e22d170dccd9f8d2543e4d7500fb9a90512934a0e7f3511d844f5383bddfc` |
| `crates/seacad-dxf-core/src/error.rs` | `90fc2315192ff5944de8454b967eefb496a8a69c1a7ed655bef856e0b82107cb` |
| `crates/seacad-dxf-core/src/lib.rs` | `5244c8e36f0b56c4b65bd63a64938432e115a170ec65305d038c5ddfa3f4d12e` |
| `docs/M5_1A_BINARY_WIRE_REGISTRY_CONTRACT.md` | `276907b53e467487db27af93db51f51547664baa47119d24d5ab86b830d9c1f6` |
| `docs/audits/M5_1A_AUTODESK_BINARY_REFERENCE.md` | `83c81d9442d012f2511ab841023fa81d7aad53ac4b0a33a855427a6cf7334aff` |
| `docs/IMPLEMENTATION_PLAN.md` | `5945677ccd19a156c9fcbe11dfe069eb9dd29779323ecb6d0b8bdd86bbc0b25b` |
| `docs/SUPPORT_MATRIX.md` | `696e8d3f984fb98277b3938a9a4d6777dfa93555955ec1752e29c9815537b5c2` |

The receipt does not hash itself. The annotated checkpoint tag binds this
receipt and every listed artifact to the final commit.
