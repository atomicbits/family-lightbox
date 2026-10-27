# Story S03: Signed and notarized macOS release on a version tag

**Feature:** `F004-ci-and-releases`
**Status:** In progress
**Depends on:** S02

## Goal

Pushing a tag `vX.Y.Z` gives a GitHub Release with a `.dmg` that Lynn can open on her Mac without
any Gatekeeper warning: signed with Atomic Bits' Developer ID and notarized by Apple, for Apple
Silicon. A dry run builds and signs the same way without publishing anything, so the pipeline can
be tested before the first real release.

## Acceptance criteria

- **The workflow** `.github/workflows/release.yml`:
  - runs on a pushed tag `v*.*.*`, and by hand with `dry_run` (default on for a manual run);
  - one build job on `macos-14`, target `aarch64-apple-darwin`, with `tauri-apps/tauri-action`
    building **without uploading** (as `wikai` does since its upload failures), with the six
    `APPLE_*` secrets in the environment;
  - checks before anything is published:
    - the tag equals the version in `tauri.conf.json`, `Cargo.toml` and `package.json`;
    - the leak check over the whole history (S01);
    - on the built app: `codesign --verify --deep --strict`, `spctl --assess --type execute`
      saying `source=Notarized Developer ID`, and `xcrun stapler validate`;
    - exactly one `.dmg`;
  - a signed build is required for a tag. Without the `APPLE_*` secrets a tag fails; a dry run
    then builds unsigned with a warning, so the rest of the pipeline can still be tested;
  - the `.dmg` leaves the build job as a workflow artifact; a separate job publishes it on a
    GitHub Release named after the tag. That job can be rerun on its own if GitHub's upload
    fails. A dry run publishes nothing;
  - permissions: `contents: read` for the build, `contents: write` only for the publish job.
- **The bundle**: `publisher` and `copyright` "Atomic Bits", category Photography, a short
  description, and no person's name.
- **The version** in three places, kept equal by `scripts/bump-version.py X.Y.Z`: it sets
  `tauri.conf.json`, `Cargo.toml` and `package.json`, refreshes `Cargo.lock`, and makes the commit
  `Release vX.Y.Z` with the tag. It refuses a version that isn't higher, or a dirty tree. It
  doesn't push: the runbook does that, through the leak check.
- **The runbook** `docs/releasing.md`: the secret names (never values) and where they come from
  (the Developer ID of Atomic Bits, as for `wikai`), the steps of a release, how to verify one on
  a Mac, the Apple agreement that must be accepted before notarization works, and what to do when
  a job fails.
- **By hand**:
  - a dry run without the Apple secrets: the build is green and unsigned;
  - after Peter sets the `APPLE_*` secrets: a dry run that signs and notarizes, checked on the
    artifact with `spctl` on Peter's Mac;
  - the first release `v0.1.0`, after Peter's go, downloaded on a Mac and opened without a
    warning.

## How

- `.github/workflows/release.yml`, after `wikai`'s `release.yml` (macOS job only, no R2, no
  Worker; GitHub Releases is where Lynn downloads, and S04's updater reads from).
- `scripts/bump-version.py` (Python 3, standard library).
- `src-tauri/tauri.conf.json`: the bundle fields.
- The Apple secrets are set by Peter: an agent may not write to a secret store. The values are
  the ones `wikai` uses; the `.p12` can be exported again from his Keychain.
- Deliberately not done here: the update signature and `latest.json` (S04); Windows.

## Files affected

- `.github/workflows/release.yml`, `scripts/bump-version.py`, `docs/releasing.md` (new).
- `src-tauri/tauri.conf.json`, `README.md`.

## Progress log

### [2026-10-27] Written

### [2026-10-27] Built

- `.github/workflows/release.yml`: the build job (version check, signing secrets check, leak check,
  `tauri-action` without upload, the `.dmg`, the signature and notarization checks, the artifact)
  and the publish job (only for a pushed tag; creates the release or adds to it on a rerun,
  uploads with up to five tries). Without the `APPLE_*` secrets, the unsigned dry run uses a build
  step without any signing environment, so empty variables can't confuse the signer.
- `scripts/bump-version.py`, tested on a throwaway clone: 0.1.0 → 0.1.1 changed exactly
  `package.json`, `Cargo.toml`, `Cargo.lock` and `tauri.conf.json`, committed "Release v0.1.1" and
  tagged it; a lower version, the same version and a dirty tree were refused.
- `docs/releasing.md`; the bundle fields in `tauri.conf.json`; a pointer in the README.
- **Unsigned dry run** on GitHub: green in 4 min 43 s, the publish job skipped. The artifact
  `FamilyLightbox_0.1.0_aarch64.dmg` (3.0 MB) mounts, holds `FamilyLightbox.app` (arm64) and the
  Applications link; `spctl` rejects that unsigned app, as it should.
- Waiting on Peter: the six `APPLE_*` secrets. Then a signed dry run, and with Peter's go the
  first release `v0.1.0`.

### [2026-10-27] Signed dry runs

- Peter set the six `APPLE_*` secrets. The first signed dry run signed the app, then notarization
  failed with `401 Invalid credentials`: the Apple ID and the app-specific password didn't belong
  together. With a new app-specific password under the company Apple ID the second run was green
  in 8 min 39 s.
- Checked on Peter's Mac: `spctl` accepts the app as "Notarized Developer ID", `codesign --verify
  --deep --strict` passes, the authority is Atomic BITS bvba's Developer ID with a timestamp, the
  ticket is stapled, the binary is arm64. But the `.dmg` itself was only signed ("Unnotarized
  Developer ID"): tauri-action notarizes the app, not the disk image around it.
- Added a step that notarizes and staples the `.dmg` too, and checks it with `spctl --type open`,
  since Lynn downloads it through a browser and it arrives quarantined. The runbook now names the
  identity as it is in the certificate and explains the 401.

### [2026-10-27] Released v0.1.0

- The dry run with the `.dmg` notarization was green in 3 min 41 s; Apple accepted the `.dmg`. With
  a quarantine flag set as a browser would, `spctl` accepted both the `.dmg` and the app inside as
  "Notarized Developer ID", and both tickets were stapled.
- The CI run on the same commit got no macOS runner within 15 minutes and was cancelled without
  running a step; its rerun was green.
- With Peter's go: tag `v0.1.0` on d4c76d7, pushed through the leak check. The release run was
  green in 4 min 9 s and published https://github.com/atomicbits/family-lightbox/releases/tag/v0.1.0
  with `FamilyLightbox_0.1.0_aarch64.dmg` (3.0 MB).
- Downloaded from that public URL without logging in, and quarantined: `spctl` accepts the `.dmg`
  and the app as notarized, `codesign --verify --deep --strict` passes, the tickets are stapled,
  the bundle says 0.1.0.
- Still to do: open it by double-click on a Mac and see no Gatekeeper warning (Peter; on Lynn's Mac
  in S05).
