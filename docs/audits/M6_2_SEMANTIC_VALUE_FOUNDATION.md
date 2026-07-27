# M6.2 semantic value foundation

M6.2 introduces the public value-state contract required before typed document
semantics. A semantic field is exactly one of `Explicit`, `Defaulted`, `Absent`,
or `Invalid`; absence is never represented as an invalid or fabricated value.

Every state carries the exact document `SourceId` and its generated schema
namespace/field identity. Explicit values always carry a bounded raw group
occurrence and byte span. Invalid values carry raw evidence when it exists;
missing required data remains invalid without inventing a span. Defaulted and
absent values have no raw provenance.

The per-value field provenance is compact: normative topic IDs, evidence rows,
and source receipts remain in the generated schema registry instead of being
duplicated in every lazy semantic cache entry. The schema namespace and field
ID provide the lookup key for that metadata.

Generic mapping and borrowed views preserve state and provenance. Custom Debug
output reports only state and provenance; it does not require `T: Debug` or
`I: Debug` and never prints a semantic value or invalidity payload.

This checkpoint does not evaluate HEADER fields, add a cache, change raw
parsing, or make a new semantic support claim. Those require later evidence and
fixtures.
