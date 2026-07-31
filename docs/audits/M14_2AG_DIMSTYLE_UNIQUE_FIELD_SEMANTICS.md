# M14.2ag DIMSTYLE Unique-Field Semantics

## Scope

M14.2ag adds lazy selection for every field in the 68-entry M14.2af DIMSTYLE
registry. A caller requests a record and field; SeaCad returns exact explicit,
absent, or typed invalid state without constructing a 68-value semantic object
or selecting a duplicate occurrence.

Group-70 standard flags receive a typed wrapper with helpers for Autodesk's
three documented bits. Its original signed value and every unknown bit remain
available.

## Normative basis

Autodesk's DIMSTYLE entry reference defines group 70 as standard flags and
assigns bit 16 to externally dependent, bit 32 to resolved external-reference
or dependent, and bit 64 to referenced external-reference state:
<https://help.autodesk.com/cloudhelp/2023/ENU/AutoCAD-DXF/files/GUID-F2FAD36F-0CE3-4943-9DAD-A9BCD2AE81DA.htm>.

The same reference defines each underlying field's text, double, signed-16-bit,
or handle wire family. M14.2af retains that evidence and its cardinality; this
checkpoint only performs unique-occurrence selection and the reviewed flag-bit
projection.

## Contract

- Any registered field can be requested by exact admitted record or raw record
  ordinal. Unknown records return `None` rather than borrowing another record.
- A unique valid occurrence becomes `Explicit` with stable
  `dimstyle.record` field provenance and exact raw group provenance.
- Absence remains `Absent`; no DIMSTYLE default is invented.
- A unique invalid ASCII number or handle remains `Invalid` with its typed
  lexical issue and raw provenance.
- Multiple occurrences remain `Invalid(MultipleValues)` with the exact count
  and first source occurrence as diagnostic provenance; no winner is chosen.
- Standard flags preserve their exact `i16`, expose bits 16/32/64, and return
  every other bit through `unknown_bits()`.
- The semantic directory retains the complete field-card/value/table/raw
  evidence stack, exact source identity, and cancellation behavior.

## Coverage and size

The existing focused DIMSTYLE suite now exercises unique text, duplicate
double, absent application-decoy field, malformed double and handle, standard
flags, unknown bits, missing-record lookup, source identity, cancellation, and
public traits. Its parity loop runs ASCII and Binary for all nine dialects.

The new semantic module is 295 lines. The expanded integration test is 436
lines. Both remain below 500 lines; `lib.rs` changes by five declarative
module/re-export lines only. No user-facing string, localization catalog entry,
dependency, manifest, or lockfile changes.

## Nonclaims

This checkpoint does not supply DIMSTYLE defaults, validate numeric domains or
enum combinations, normalize text/handles, resolve text-style or arrow-block
handles, interpret tolerance strings, construct glyph geometry, edit, or write
DIMSTYLE records.

## Verification

All required local gates passed on 2026-07-31: dependency policy,
generated-schema drift, release-evidence drift, formatting, workspace Clippy
with warnings denied, 715 workspace tests with zero failures or ignored tests,
production forbidden-macro scanning, and `git diff --check`. The focused suite
passed 4/4 tests. No dependency manifest or lockfile changed.

The M14.2af GitHub push was independently inspected with `gh`. Both CI and
Dependency Policy stopped before their first step because GitHub annotated the
account with failed recent payments or an insufficient spending limit. This is
an external billing blocker, not a repository test result; no six-platform CI
success is claimed for this checkpoint until billing is corrected and the
workflows are rerun.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `README.md` | 79 | `51b76741937a76f4cdd0b730f13f76bf73d48fae86a37741a20c3027ee2d9775` |
| `crates/seacad-dxf-core/src/lib.rs` | 806 | `5c1c71128d0c0f9e4cb024b35de41fbd95b2842343804acad25f4818889194a1` |
| `crates/seacad-dxf-core/src/dimstyle_field_semantic.rs` | 295 | `38a80866558f376ffd653ff3dda7d101125703412706a53c7c1800a7215cacdf` |
| `crates/seacad-dxf-core/tests/dimstyle_field_semantic_tests.rs` | 436 | `f3af72f5156f5436cdc1db9a33168af30c0423f2b526ce79764d0135e86e0ceb` |
| `docs/DXF_ENTITY_COMPLETION_PLAN.md` | 295 | `4d7b9d6a17eede4644186d1ef3fae07bdd377c267bb1ea8dc418b28ee3974467` |
| `docs/IMPLEMENTATION_PLAN.md` | 1,656 | `5c8507a9f62b8cfede5f5728a463e5826a516e8e160a1454489a1ce04d05df6b` |
| `docs/SUPPORT_MATRIX.md` | 1,350 | `11000cd2648c92f8fe97bab9dfc38fd6e9546592ad330bcfbb61f0fde47313ab` |
