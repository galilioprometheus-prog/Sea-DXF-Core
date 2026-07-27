# M4.3a `encoding_rs` candidate dependency review

Status: conditionally approved for proposal at M4.3b; not added at M4.3a

Review date: 2026-07-27

## Package identity

| Field | Reviewed value |
| --- | --- |
| crate | `encoding_rs` |
| exact version | `0.8.35` |
| publisher/source | Henri Sivonen; crates.io; `hsivonen/encoding_rs` |
| package archive SHA-256 | `75030f3c4f45dafd7586dd6780965a8c7e8e285a5ecb86713e63a79c5b2766f3` |
| declared Rust minimum | 1.36 |
| license expression | `(Apache-2.0 OR MIT) AND BSD-3-Clause` |

The archive was fetched only into Cargo's package cache by
`cargo info encoding_rs@0.8.35`. It is not vendored, forked, or committed.

## License and provenance

The crate's `COPYRIGHT` states that non-generated code is available under
Apache-2.0 or MIT and data/code generated from WHATWG Encoding Standard data is
BSD-3-Clause. Test portions may be public-domain/CC0. If M4.3b adds the crate,
SeaCad must ship the relevant copyright and all three applicable license
notices in `THIRD_PARTY_NOTICES.md`.

## Features and dependency surface

- Default feature: `alloc`.
- Required runtime dependency: `cfg-if = 1.0`; SeaCad already resolves
  `cfg-if 1.0.4` through `sha2`.
- Optional: serde, SIMD workaround, and multiple fast/less-slow encode tables.
- No build script and no runtime filesystem, network, process, or platform API.

The permitted proposal is:

```text
encoding_rs = { version = "=0.8.35", default-features = false, features = ["alloc"] }
```

SIMD, serde, and fast encoding features remain disabled unless separately
audited. M4.3b decoding must use APIs that report malformed input without
replacement.

## Safety and security review

The crate uses substantial internal `unsafe` for pointer/SIMD-oriented
performance. SeaCad production code remains `#![forbid(unsafe_code)]`, but that
attribute cannot audit a dependency. Therefore adoption requires focused
malformed-input, boundary, cross-platform, and fuzz evidence around every API
SeaCad calls.

GitHub Advisory API query for `encoding_rs@0.8.35` and RustSec advisory-db code
search both returned zero matches on 2026-07-27. This is a point-in-time result,
not a permanent safety guarantee, and must be rerun at dependency addition and
release.

## Capability gap and decision

`encoding_rs` implements the WHATWG Encoding Standard and explicitly does not
cover every DOS/OEM single-byte encoding. It also must not be used through
replacement-producing convenience paths when exactness matters.

Decision: do not add it in M4.3a. M4.3b may add it for a documented subset only
after SeaCad freezes the `$DWGCODEPAGE` mapping matrix and defines an explicit
unsupported path for every unprovided codepage. It cannot be the basis of a
"100% codepage" claim by itself.
