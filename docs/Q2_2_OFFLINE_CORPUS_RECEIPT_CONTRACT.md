# Q2.2 offline corpus receipt contract

## Purpose

Q2.2 supplies a cross-platform, offline harness for exercising the strict DXF
core against a private corpus without publishing corpus bytes or identifying
metadata. It is evidence infrastructure only and does not increase format,
dialect, semantic, performance, or large-file support claims.

The committed `corpus/offline-manifest.json` contains policy and resource
bounds only. It contains no input path, filename, file size, content hash,
source identity, expected per-file outcome, customer identity, or private DXF
bytes.

## Invocation

Keep private inputs under the ignored `corpus-private/` directory or another
offline location, then run:

```text
cargo run --locked -p seacad-cli --bin seacad-corpus-receipt -- \
  corpus/offline-manifest.json <offline-corpus-root>
```

The process exits `0` only when at least one selected file is present and every
selected file passes strict framing verification. A completed scan with no
selected files or any invalid file emits a receipt with `status="failed"` and
exits `1`. Invalid invocation exits `2`. Manifest, filesystem, symlink, and
resource-bound failures exit `1` without echoing an input path.

## Selection and bounds

Version 1 selects regular files whose extension is `.dxf`, case-insensitively.
Directories are traversed without following symlinks. Every canonicalized
entry must remain under the canonical corpus root.

The manifest fixes:

- strict parsing with the existing `Safe` resource profile;
- maximum traversal depth, observed entries, selected files, and aggregate
  selected bytes;
- 1,000 selected files and 10 GiB total as harness ceilings, not achieved
  evidence or support claims.

Every individual source remains subject to the core's `Safe` limits. Any
arithmetic overflow or configured limit breach fails closed.

## Redacted receipt

The stable JSON contract is `seacad-offline-corpus-receipt/v1`. It contains:

- operating system and architecture;
- the public manifest contract, corpus ID, privacy flags, and limits;
- aggregate entry, directory, selected-file, ignored-entry, byte, physical
  format, verified, and invalid counts;
- aggregate failure counts keyed only by stable core or harness error code.

The receipt deliberately excludes paths, filenames, source IDs, per-file
hashes, per-file sizes, per-file outcomes, timestamps, usernames, hostnames,
and environment data. The harness never copies, modifies, or writes alongside
corpus inputs.

Q2.2 does not provide a cryptographic corpus commitment. Final native receipt,
corpus scale, and release evidence remain owned by M13.
