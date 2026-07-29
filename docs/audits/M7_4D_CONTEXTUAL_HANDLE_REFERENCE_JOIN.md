# M7.4d contextual handle reference join

M7.4d joins every M7.3b pointer/owner resolution entry to the exact optional
M7.4c application group containing its source occurrence. Numeric handle class,
target resolution, lexical container context, and malformed group state remain
separate evidence.

## Normative boundary

Autodesk's common entity and object contracts place `330` persistent-reactor
handles inside `{ACAD_REACTORS`, `360` extension-dictionary handles inside
`{ACAD_XDICTIONARY`, and ordinary `330` owner-object pointers outside those
control groups:

<https://help.autodesk.com/cloudhelp/2017/ENU/AutoCAD-DXF/files/GUID-3610039E-27D1-4E23-B6D3-7E60B22BB5BD.htm>

<https://help.autodesk.com/cloudhelp/2025/ENU/AutoCAD-DXF/files/GUID-6939D69E-04CB-4F4C-87B2-67BC540FCF58.htm>

The same numeric group-code ranges still carry their independent soft/hard
pointer or owner classifications:

<https://help.autodesk.com/cloudhelp/2018/ENU/AutoCAD-DXF/files/GUID-3F0380A5-1C15-464D-BC66-2C5F094BCFB9.htm>

M7.4d records the join but deliberately does not collapse these dimensions into
a semantic role. That requires record-type and common-contract applicability
work beyond a purely lexical range match.

## Public contract

`DxfRawDocumentView`, `DxfAsciiRawDocument`, and `DxfBinaryRawDocument` expose
`contextual_handle_reference_directory(cancellation)`. The directory owns one
shared M7.3b resolution directory, one shared M7.4c application-group directory,
and exactly one source-order `DxfContextualHandleReferenceEntry` per resolution.

Each entry retains the exact `DxfHandleResolutionEntry`, an optional compact
application-group ordinal, and one `DxfHandleReferenceContext`:

- `OutsideApplicationGroup`;
- `AcadReactors`;
- `AcadXDictionary`;
- `OtherApplicationGroup`.

The optional group ordinal preserves access to the exact M7.4c kind, record,
source/content ranges, controls, and `Closed`, `Interrupted`, or `Unclosed`
state. `entry`, `entry_for_group`, `entries_for_record`, and
`application_group_for_entry` are bounded exact lookups.

Construction scans the two shared directories once and performs a binary
content-range lookup per reference without rereading handle payloads. It checks
both source identities, cancellation throughout the join, allocation failures,
and compact ordinal conversion. Storage remains linear in references and
application-control evidence.

## Explicit non-claims

Context does not change a reference's numeric class or resolution state. M7.4d
does not yet label a reference as a valid persistent reactor, extension
dictionary, ordinary owner pointer, or ownership link. It does not validate
record types, application payloads, one-owner conformance, dictionary
membership, purge behavior, graph topology, cycles, edits, or writes.

## Antigravity evidence

Antigravity task 28 performed a read-only mechanical join inventory at exact
clean M7.4c HEAD `163277482d7e4742feec2a453f8ac366e287adf2` using the
quota-aware Gemini 3.5 Flash (High) recommendation. All six commands exited
zero, the combined resolution/application-group suite passed 6/6, before/after
status stayed clean and identical, and no paths were written. Its anchors
covered source-order resolutions, occurrence keys, application-group content
lookup, source identity checks, cancellation, and document adapters.

Codex independently verified those mechanics, chose and implemented the public
contract, and reproduced the focused and required workspace gates before
accepting task 28. Semantic, normative, support, audit, commit, and tag decisions
remained with Codex.

## Verification scope

Public integration tests cover ASCII/Binary parity across all nine supported
dialects using the documented pre-R13 `1005` escape, all four contexts, exact
optional group ordinals, unclosed-group state preservation, per-record and
per-occurrence lookups, independent numeric class/context evidence for modern
`330` and `360`, unique target state preservation, cancellation, linear source
reads, bounds, compact/copy/thread-safe values, and debug redaction.

## Reviewed artifact receipt

The reviewed production delta is 230 added lines: 225 lines in
`handle_context.rs` and five module/export lines in `lib.rs`. The public
integration test artifact adds 302 lines. There are no manifest, dependency,
lockfile, schema, generated-file, parser-framing, writer, or corpus changes.

SHA-256 at review time:

- `lib.rs`: `0A98CE6968D58DFA5037CF066B8D00AF33F1B12E7410C0D6A53FC6D28A8CF69B`
- `handle_context.rs`: `0F4ADDE148D0B3ED078B48FAC74BD4E6541B7783E7559B7F071AE6C249FE3B3A`
- `handle_context_tests.rs`: `B6BBAE1195BA5738B516C6672027D517D612885EFDBED1FEB189676FA416827F`
- `IMPLEMENTATION_PLAN.md`: `250BD9D9163B32AE5E4E3E96C87E435BDFD7CEDDA957FC631A8CAAF516BA6EC5`
- `SUPPORT_MATRIX.md`: `7EC7BFCE122FDA124BEFB58BBE1596DDDF47CE57A7F715F289A83A6A7E1785D6`

The reviewed production and integration-test artifacts add no `unsafe`, panic
path, `unwrap`, `expect`, `todo`, or `unimplemented` use.

## Required gates

- focused M7.4d contextual-reference suite: 3/3 passed;
- Antigravity substrate suites: 6/6 passed read-only at the clean base;
- `cargo deny --locked check`: passed advisories, bans, licenses, and sources;
- `cargo +1.97.1 fmt --all -- --check`: passed;
- `cargo +1.97.1 run --locked -p seacad-schema-gen -- --check`: passed;
- `cargo +1.97.1 clippy --workspace --all-targets -- -D warnings`: passed;
- `cargo +1.97.1 test --workspace`: 285 tests passed (16 CLI, 6 corpus,
  204 core unit, 3 application-group, 3 contextual-reference,
  3 handle-identity, 3 handle-reference, 3 handle-resolution, 4 HEADER-handle,
  12 numeric, 4 text, 4 ownership-evidence, 4 raw-handle, 3 raw-record, and
  13 schema-generator);
- `git diff --check`: passed.
