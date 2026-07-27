# M4.2 ASCII structure index contract

Status: frozen 2026-07-27

This contract covers eager section accounting and group-code 0 boundaries for
lossless ASCII raw documents. It does not interpret section payloads, record
types, text encodings, handles, or geometry.

## Public model

`DxfAsciiRawDocument::structure_index()` returns one immutable
`DxfAsciiStructureIndex` carrying the same `DxfSourceId` as the raw document.
It exposes:

- total parsed group count;
- counts inside and outside section ranges;
- ordered, non-overlapping `DxfAsciiSection` envelopes;
- one compact occurrence for every numeric group code 0;
- bounded, source-anchored structure diagnostics.

For every opened document:

```text
inside_section_group_count + outside_section_group_count = total_group_count
sum(section.group_range length) = inside_section_group_count
```

Parsed groups end at the framed EOF boundary. Preserved trailing source bytes
after EOF remain represented by the raw document's existing trailing span and
are not falsely counted as parsed groups.

## Section representation

Known exact names are HEADER, CLASSES, TABLES, BLOCKS, ENTITIES, OBJECTS, and
THUMBNAILIMAGE. Any other exact group-code 2 value is `Unknown`, retains its
byte span, and is not an error.

Each section has:

- a half-open group range including `SECTION`, its name candidate, payload, and
  `ENDSEC` when present;
- a half-open content range excluding the marker, name candidate, and closing
  `ENDSEC`;
- an exact name classification and optional name byte span;
- closure `Closed`, `Interrupted`, or `Unclosed`.

The section's slice position is its stable ordinal. Its marker occurrence is
the start of its group range. For a non-missing name, the name candidate is the
next occurrence. For a closed section, the `ENDSEC` occurrence is the final
occurrence in its group range.

An exact new `SECTION` interrupts any currently open section before starting a
new non-overlapping range. An exact orphan `ENDSEC` remains outside sections.
Framed EOF or input end closes accounting without adding EOF to the open
section.

## Compact group-zero index

The index stores one `u32` occurrence (four bytes) per numeric group code 0 and
does not copy its value. `zero_group_range(n)` begins at that occurrence and
ends at the next indexed group code 0, or at total group count for the final
entry. The original type/separator bytes remain available through the raw
group's value span.

The index is built while borrowed groups pass through the existing single read
and SHA-256 scan. It performs no second source scan, no random I/O, no text
decoding, and no payload allocation. Section metadata is tested to remain at
most 72 bytes per section.

## Stable diagnostics

| Code | Severity | Meaning |
| --- | --- | --- |
| `DXF-E0410` | Error | SECTION has no immediate group-code 2 name |
| `DXF-E0411` | Error | a new SECTION interrupted an open section |
| `DXF-E0412` | Error | ENDSEC appeared without an open section |
| `DXF-E0413` | Error | a section reached framed EOF/input end without ENDSEC |

Index diagnostics are independently bounded by the selected resource profile.
When the bound is reached, the final retained item becomes the existing
`DXF-W0001` truncation diagnostic. Structure errors do not make lossless raw
open fail or alter raw framing conformance.

## Deliberate M4.2 boundaries

- No section order, uniqueness, or version-presence rules.
- No interpretation of group-code 0 type strings.
- No codepage, Unicode, or escape view.
- No CLI/JSON v1 schema expansion.
- No Binary DXF structure index yet.
- No new dependency.
