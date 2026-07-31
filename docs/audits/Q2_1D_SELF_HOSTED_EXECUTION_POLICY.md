# Q2.1d Self-Hosted Execution Policy

## Scope

Manual self-hosted run `30640593874` selected Windows PowerShell 5.1 and
reached the provisioned runner, but Windows' `Restricted` execution policy
blocked the Actions-generated temporary script before toolchain verification.

Run evidence:
<https://github.com/seaflower205/SeaCad/actions/runs/30640593874>.

Q2.1d supplies `-ExecutionPolicy Bypass` only to each workflow shell process.
It does not change LocalMachine, CurrentUser, Group Policy, registry, or the
Scheduled Task configuration.

## Verification contract

- The failed run completed setup and checkout on `seacad-win-x64`.
- The failure occurred before any project command executed.
- No GitHub-hosted minute was consumed.
- A corrected manual dispatch must pass end to end before operational success
  is claimed.

## Nonclaims

This process-local shell correction does not weaken repository permissions,
enable automatic triggers, add a dependency, or add release evidence.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `.github/workflows/ci.yml` | 52 | `b3a75b77d020423e8e55beee0fb56d8083fa1fb455ade3f32b9b6cbe23129689` |
| `docs/TOOLCHAIN.md` | 267 | `a44e79298e80e3bb230b06428b8a9da54fcdead142d57b3381be82dc3778df30` |
