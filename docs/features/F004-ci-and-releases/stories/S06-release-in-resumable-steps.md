# Story S06: A release in steps that can be retried, with the stapling made robust

**Feature:** `F004-ci-and-releases`
**Status:** Planned
**Depends on:** S03, S04

## Goal

A release that fails on Apple's side can be finished from where it stopped, without building,
signing or notarizing again what already succeeded; and stapling the ticket no longer depends
on one attempt hidden inside the build (Peter, 2026-10-30).

## Background

On 2026-10-30 Apple's ticket service (`api.apple-cloudkit.com`) dropped connections from the
`macos-15` runners for hours. The dry runs for 0.4.0 failed twice on `stapler validate` (fixed by
retrying), and the release of 0.4.3 failed twice in a way retrying can't fix:

- `tauri-action` signs the app, notarizes it (Apple answered *Accepted*), staples the ticket and
  then builds the updater payload (`.app.tar.gz` and its `.sig`) and the `.dmg` from that app;
- its stapling failed without failing the step, so the app, the payload and the `.dmg` had no
  ticket, and the checks after it (`spctl`, `stapler validate`) asked Apple online and were
  refused;
- a re-run of the failed job builds, signs and notarizes everything again (about 7 minutes and a
  new notarization), and meets the same service.

Today everything happens in one job; re-running it starts from scratch.

## Acceptance criteria

- **Jobs that hand over artifacts**, so *Re-run failed jobs* on GitHub starts at the step that
  failed and uses what the earlier jobs left:
  1. **build**: the leak check, the version check, `tauri build` that signs the app with the
     Developer ID, without notarizing (the Apple ID left out); the signed `.app` is uploaded as
     an artifact;
  2. **notarize**: the app zipped and submitted with `notarytool submit`, the submission id
     written to the job summary and kept as an artifact; waiting with `notarytool wait`; a re-run
     that finds an id from an earlier attempt for the same zip (same SHA-256) waits for that one
     instead of submitting again;
  3. **staple**: `stapler staple` and `stapler validate` on the app, up to five times with a
     growing pause; the stapled app as an artifact. The job fails when the ticket can't be
     stapled, so nothing unstapled goes further;
  4. **package**: from the stapled app, the updater payload (`.app.tar.gz`, signed with the
     update key through `tauri signer sign`) and the `.dmg` (built with `hdiutil`, signed,
     notarized as in 2 with its id kept, stapled as in 3); all checks of today (`codesign`,
     `spctl`, the app inside the payload, the `.dmg`); the release files as an artifact;
  5. **publish**: as today, from the release files.
- **A dry run** goes through 1–4 and publishes nothing, as today.
- **Every network call to Apple** (submit, wait, staple, validate, `spctl`) is retried with a
  pause; a real refusal (*Invalid* notarization, a bad signature) fails at once, without retries.
- **The runbook** (`docs/releasing.md`) says what to do when a release stops: which job failed,
  and that *Re-run failed jobs* continues from there; a tag is never moved or recreated for it.
- **Checked**: a dry run, then a re-run of only the staple job of that run reuses the notarized
  app (no new submission in Apple's history); then the next real release.

## How

- The jobs pass files through `actions/upload-artifact` and `download-artifact` within one
  workflow run; GitHub keeps the artifacts of jobs that succeeded when failed jobs are re-run.
- Notarizing outside `tauri-action`: the build gets no `APPLE_ID`, so Tauri only signs.
  `notarytool` takes the zip of the app, as Tauri does now.
- The updater signature: `pnpm tauri signer sign` with `TAURI_SIGNING_PRIVATE_KEY` and its
  password, on the payload made from the stapled app, so `latest.json` refers to that file.
- The `.dmg`: `hdiutil create` from the stapled app with an Applications link, as Tauri's, so the
  app inside the `.dmg` carries its ticket too.

## Files affected

- `.github/workflows/release.yml`, `docs/releasing.md`, maybe a small script in `scripts/` for
  "submit or reuse, then wait".

## Progress log

### [2026-10-30] Written

After the failed release of 0.4.3; that release is retried as it stands while this story waits.
