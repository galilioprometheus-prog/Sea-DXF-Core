# M5.2c Binary DXF replay/CLI contract

Status: frozen 2026-07-27

M5.2c closes M5 by adding verified unchanged Binary replay and exposing the
M5.2 raw document through `seacad inspect` and `seacad verify`. It does not add
semantic edits, canonical writing, ASCII/Binary conversion, or version
conversion.

## Verified Binary verbatim replay

`DxfBinaryRawDocument::write_verbatim_to_new_file` uses the same reviewed
implementation and `DxfVerbatimWriteReceipt` as ASCII:

- destination creation uses create-new and never overwrites any file;
- the complete original source is copied in bounded 64 KiB chunks, including
  a Compatible opaque tail;
- SHA-256 is recomputed while rereading the source and must equal the document
  `SourceId`, enforcing the stable-source precondition;
- the flushed and synced destination is reopened, length checked, SHA-256
  checked, and removed on any failure;
- cancellation/progress cover copy plus verification and incomplete output is
  removed.

No Binary group is re-encoded. Byte identity is the only accepted successful
receipt for an unchanged document.

## CLI behavior

The existing commands and defaults do not change:

```text
seacad inspect FILE [--mode strict|compatible] [--lang en|vi] [--json] [--large] [--show-path]
seacad verify FILE [--mode strict|compatible] [--lang en|vi] [--json] [--large] [--show-path]
```

Physical `binary` now opens through `DxfBinaryRawDocument`. A Strict Binary
document returns `ok` for inspect or `verified` for verify. A Compatible
Binary recovery returns `recovered` for inspect, while verify returns
`not_verified` with `CLI-E0004`. Fatal Binary framing/envelope failures return
`invalid` with the exact stable `DXF-E...` code. Unknown input remains
`CLI-E0003`; the generic future-format fallback remains `CLI-E0002`.

English and Vietnamese human help now say ASCII/Binary DXF. All Binary fatal
codes currently reachable from the CLI have explicit Vietnamese human text.
Paths remain hidden unless `--show-path` is supplied.

## JSON v1 compatibility

No top-level or nested key is added, removed, renamed, reordered by contract,
or made optional. For Binary:

- `format.physical` is `binary`;
- the existing `document` object reports conformance, group count, EOF
  occurrence, trailing byte count, and diagnostic truncation;
- diagnostics retain stable code/severity/span objects;
- JSON keys, status values, physical/conformance/severity values, codes, and
  English error messages are never translated.

Section/dialect reports remain core APIs and are not added to JSON v1 in this
checkpoint. A future expanded report requires a new schema version or a
separately reviewed backward-compatible extension.

## Deferred boundary

M5.2c is verified unchanged replay, not a writer that edits or re-encodes.
Text policy integration for Binary, semantic values, geometry, transactions,
PreservePatch, canonical writers, and same-version ASCII/Binary conversion
remain M6-M12 work.
