# M6.1a schema-tool dependency-scope review

`seacad-schema-gen` is an internal build tool. It reuses exact packages already
present in `Cargo.lock` and reviewed at earlier milestones:

| Package | Version | Tool features | Prior review |
| --- | ---: | --- | --- |
| `serde` | 1.0.229 | `derive`, `std` | M3.5 CLI dependency review |
| `serde_json` | 1.0.151 | `std` | M3.5 CLI dependency review |
| `sha2` | 0.11.0 | default features disabled | M2.3 SHA-256 review |

Purpose is restricted to strict JSON deserialization, deterministic normalized
serialization, and a SHA-256 input receipt. The generator performs no network
access, uses no build script, loads only manifest-listed repository files, and
emits one committed Rust file through a same-directory temporary file and
rename. Unknown JSON fields fail because every input structure uses
`deny_unknown_fields`.

`Cargo.lock` gains only the local `seacad-schema-gen` workspace-package entry;
no new third-party package or transitive dependency is added. The core's
runtime dependency graph is unchanged. Existing licenses and notices in
`THIRD_PARTY_NOTICES.md` already cover the reused packages.
