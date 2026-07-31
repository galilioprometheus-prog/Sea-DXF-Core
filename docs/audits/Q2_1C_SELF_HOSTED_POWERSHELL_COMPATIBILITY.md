# Q2.1c Self-Hosted PowerShell Compatibility

## Scope

The first manual self-hosted workflow run reached `seacad-win-x64`, completed
setup and checkout, then failed before any project gate because the workflow
requested `pwsh` while the provisioned machine has Windows PowerShell 5.1.

Run evidence:
<https://github.com/seaflower205/SeaCad/actions/runs/30640437822>.

Q2.1c sets the workflow-wide run shell to `powershell` and removes the
step-local `pwsh` request. No PowerShell 7 package or dependency is added.

## Verification contract

- The failed run consumed no GitHub-hosted minutes.
- Checkout succeeded on the registered self-hosted runner.
- The corrected shell is already provisioned by Windows.
- The full local cold-cache contract passed immediately before the first
  dispatch; a corrected end-to-end dispatch must pass before this checkpoint
  is considered operational evidence.

## Nonclaims

This compatibility correction is Windows x64 diagnostic infrastructure only.
It does not add six-native, corpus, nightly, signature, or release evidence.

The audit file omits its own hash so that the receipt is not self-referential.

| Artifact | Lines | SHA-256 |
|---|---:|---|
| `.github/workflows/ci.yml` | 52 | `f546bb8771b54f6c73e14a8517473be254815935a3e4dafb11179eec3fba8149` |
| `docs/TOOLCHAIN.md` | 266 | `b5a4ec7ff688f4b96a9ad513bb5c16e1f8e007e9bcc9a1c3d0965f5eeb58fa0a` |
