# M4.3c2b Unicode CP1361 data license review

Status: approved 2026-07-27

## Artifact and purpose

SeaCad derives `johab_decode_le.bin` from Microsoft's CP1361 decode records in
Unicode's `bestfit1361.txt`. The source is a computer data file below
`https://www.unicode.org/Public/`, so Unicode's Terms of Use classify it as a
Unicode Data File.

The source file is not committed. The 131,072-byte derived table is committed
solely to provide deterministic, cross-platform, replacement-free CP1361
decoding. No Unicode-to-codepage best-fit records, comments, names, software,
or parser source are included.

## License

Unicode License v3 grants permission to use, copy, modify, merge, publish,
distribute, and sell Data Files and derivatives, provided the copyright and
permission notice appears with copies or in associated documentation. It also
contains an as-is warranty disclaimer and restriction on promotional use of
copyright-holder names.

SeaCad satisfies these conditions by reproducing the Unicode License v3
copyright, permission notice, disclaimer, and name restriction in
`THIRD_PARTY_NOTICES.md`. The proprietary license explicitly leaves
third-party components under their own licenses.

## Supply-chain boundary

- No Cargo package, feature, build script, network runtime, FFI runtime, or
  platform dependency is added.
- The input SHA-256, canonical record SHA-256, output SHA-256, exact counts,
  and Windows NLS comparison are frozen in the audit tool and receipt.
- Regeneration fails on any drift.
- Release packaging must retain `THIRD_PARTY_NOTICES.md`.

Decision: approved as a milestone-scoped Unicode-3.0 data component.
