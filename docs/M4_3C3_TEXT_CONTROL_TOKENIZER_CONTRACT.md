# M4.3c3 documented text-control tokenizer contract

Status: frozen 2026-07-27

This checkpoint adds allocation-free lexical tokenization after the existing
replacement-free storage and CIF/MIF decode layers. It does not render text,
parse numeric formatting values, classify DXF entities, or mutate raw bytes.

## Pipeline and provenance

The caller explicitly composes:

1. source-anchored storage decode;
2. CIF/MIF control decode;
3. `DxfTextControlCursor` over the resulting valid UTF-8.

Tokenizer offsets are half-open UTF-8 byte spans in layer-three input. Each
token exposes its exact source span and, where present, its unparsed payload
span. Concatenating all successful token source spans reconstructs the input
byte-for-byte. The source-backed receipt from layer one remains authoritative.

## Explicit context

`DxfTextControlContext` is required because Autodesk assigns different
percent-control behavior to `MTEXT` and single-line `TEXT`.

- `MText` recognizes the documented MTEXT backslash grammar and only the
  percent symbol controls that AutoCAD 2027 applies in MTEXT.
- `SingleLineText` recognizes the documented percent-control registry and
  treats MTEXT backslash sequences as ordinary text.

The tokenizer never guesses an entity type from content.

## Documented MTEXT tokens

Exact two-byte controls:

| Raw | Token |
| --- | --- |
| `\O`, `\o` | overline on/off |
| `\L`, `\l` | underline on/off |
| `\K`, `\k` | strikethrough on/off |
| `\~` | nonbreaking space |
| `\\` | escaped backslash |
| `\{`, `\}` | escaped literal braces |
| `\P` | paragraph break |

Exact uppercase parameter introducers consume through the next semicolon and
retain the bytes between the introducer and semicolon as an unparsed payload:
`\A`, `\C`, `\F`, `\H`, `\S`, `\T`, `\Q`, and `\W`.

Unescaped `{` and `}` are block tokens. Nesting through eight levels is valid.
A ninth opening block, unmatched close, or unclosed block is a typed structural
failure. A recognized parameter introducer without a semicolon is a typed
missing-terminator failure.

## Percent controls

Both contexts recognize `%%c`, `%%d`, and `%%p`, case-insensitively, as
diameter, degree, and plus/minus tokens.

Only `SingleLineText` additionally recognizes:

- `%%k`, `%%o`, and `%%u`, case-insensitively, as style toggles;
- `%%%` as a literal percent control;
- `%%nnn` with exactly three decimal digits as a Unicode-decimal token.

The three decimal digits remain an unparsed payload. This checkpoint does not
convert them into a scalar.

## Unknown and malformed boundaries

Unknown, case-different, undocumented, or extension-like sequences are
coalesced into exact `Text` tokens. This includes lowercase parameter
introducers, fields, and `\p` paragraph-format syntax whose complete public
grammar was not established. Unknown data is not an error and is never
discarded.

Only syntax already claimed as documented can fail structurally. After a
failure the cursor is terminal and must be discarded. The raw document and
decoded source remain available; a future semantic view may mark only the
owning record invalid.

## Resource and support boundary

- constant extra memory: one eight-entry block-offset array;
- no heap allocation, normalization, replacement, locale, FFI, or dependency;
- linear scan with no global cache;
- no numeric range validation, style lookup, stack evaluation, glyph shaping,
  layout, field evaluation, or plain-text rendering;
- no MTEXT group-1/group-3 reassembly or entity semantics before M6/M10;
- no CLI/JSON, Binary DXF, writer, edit, or transaction change.
