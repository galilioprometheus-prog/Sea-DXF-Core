# M13.2a corpus release gate contract

## Purpose

M13.2a turns the existing offline corpus scanner into an actual Core 1.0
release gate without weakening Q2.2 compatibility or publishing private
evidence. The ceiling-only v1 manifest and receipt remain unchanged. A distinct
v2 manifest supplies release requirements and produces a distinct v2 receipt.

## Release policy

The committed `corpus/release-manifest.json` fixes:

- strict parsing with the `Safe` resource profile;
- case-insensitive regular `.dxf` selection;
- no symlink traversal and canonical-root containment;
- minimum 1,000 verified files;
- minimum 10 GiB selected and verified bytes;
- exactly zero invalid files;
- maximum depth 32;
- maximum 20,000 entries;
- maximum 2,000 selected files;
- maximum 20 GiB selected bytes.

Minimums must be positive and no larger than their configured maxima. Manifest
limits must remain within the hard implementation ceilings. Invalid
requirements fail before corpus traversal with `CORPUS-E0007`.

## Receipt and exit behavior

The stable release receipt contract is
`seacad-offline-corpus-receipt/v2`. It retains all v1 redaction and aggregate
fields and adds:

- the public minimum requirements;
- `verified_files_met`;
- `verified_bytes_met`;
- `zero_invalid_files_met`;
- `release_gate_met`, the conjunction of those three states.

Only `release_gate_met=true` produces `status="verified"` and exit `0`.
Threshold shortfall produces a completed `status="failed"` receipt and exit
`1`. It does not fabricate a per-file failure code. Invalid files remain
aggregated by stable core/harness code and necessarily fail the zero-invalid
threshold.

## Privacy and non-claims

V2 continues to exclude paths, filenames, source identities, per-file hashes,
per-file sizes/outcomes, timestamps, usernames, hostnames, and private bytes.
It does not cryptographically commit to a corpus and receipts with identical
aggregates are intentionally indistinguishable. The committed manifest proves
policy, not achievement. A private v2 receipt, six-native evidence closure,
and final release authorization remain separate M13 evidence.
