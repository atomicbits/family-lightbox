# Story S04: Updates: the Tauri updater with its own key, `latest.json` on GitHub Releases, *Bijwerken* in the app

**Feature:** `F004-ci-and-releases`
**Status:** In progress
**Depends on:** S03

## Goal

Once Lynn has installed FamilyLightbox, later versions reach her without a new download: at start
the app sees that a newer release exists and offers *Bijwerken* in the header. One click
downloads it, checks its signature, installs it and restarts the app. The app never updates
without being asked, and never in the middle of a sync.

## Acceptance criteria

- **The key**: an update-signing key pair of this app's own (Peter's decision 4), not `wikai`'s.
  - The public key goes in `tauri.conf.json` (`plugins.updater.pubkey`). It is public by nature.
  - The private key and its password go in the repository secrets `TAURI_SIGNING_PRIVATE_KEY` and
    `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, set by Peter.
  - A backup of the private key is kept outside the repo. Where it is kept is written down in
    LibraryLocal, not here.
  - The runbook says what losing the key means: installed apps can no longer be updated and have
    to be reinstalled by hand from a new key's release.
- **The release** (`release.yml`):
  - builds the updater payload with `bundle.createUpdaterArtifacts`: `FamilyLightbox.app.tar.gz`
    and its `.sig`;
  - a tag fails without the two `TAURI_SIGNING_*` secrets, the same as without the Apple secrets.
    A dry run without them builds without the updater payload and warns;
  - checks that the `.app` inside the `.tar.gz` is the notarized one: `spctl` and `stapler
    validate` after unpacking it;
  - the collect step expects exactly one `.dmg`, one `.app.tar.gz` and one `.app.tar.gz.sig`;
  - the publish job builds `latest.json` from the `.sig`, with
    `scripts/build-latest-json.py` (Python 3, standard library, with tests):
    - `version`, `pub_date`, and a short `notes`;
    - `platforms.darwin-aarch64`, holding the `.sig`'s contents as `signature` and the release
      download URL of the `.tar.gz` as `url`;
  - uploads `latest.json` last, after the `.dmg`, the `.tar.gz` and the `.sig`. An app that looks
    while a release is being published then never sees a `latest.json` whose payload isn't there
    yet.
- **The endpoint**: one, `https://github.com/atomicbits/family-lightbox/releases/latest/download/latest.json`.
  GitHub's "latest" is the newest non-draft, non-prerelease release, which is exactly what should
  reach Lynn. There is no R2 and no Worker.
- **In the app**: the updater lives in Rust (D13), with `tauri-plugin-updater` through commands,
  like the rest of the app. There is no JavaScript updater plugin and no extra capability.
  - **Commands**:
    - `update_check` returns either nothing or `{ version, notes }`;
    - `update_install` downloads, checks the signature and installs. It then restarts the app
      itself, and refuses while a sync is running.
  - **Events**: `update-progress` carries the bytes downloaded and the total.
  - **At start**: the app checks once, a few seconds after the window opens. It doesn't check in
    a dev build.
  - **No network or no release**: if the check fails, nothing is shown. The reason is only
    logged, and Lynn doesn't see an error for an update she didn't ask for.
  - **The header**:
    - with a newer version: `Versie X.Y.Z beschikbaar` and a *Bijwerken* button;
    - while downloading: `Bijwerken… NN %`;
    - during a sync, the button is disabled, with the reason as its tooltip: `na de synchronisatie`.
  - **Settings, a section *Over*:**
    - the installed version;
    - *Controleren op updates*, which reports `Je hebt de nieuwste versie` or the version found;
    - the error of the last check.
  - **A wrong signature** is refused by the updater. Nothing is installed, and the app shows
    `Update geweigerd: de handtekening klopt niet`. It doesn't offer the same version again
    until the app has been restarted.
- **The runbook** `docs/releasing.md`: the two secrets, generating the key, the backup (only
  that one exists and that losing it is final, not where it is), and how an update reaches an
  installed app.
- **By hand**:
  - generate the key; Peter sets the secrets;
  - release `v0.2.0` with the updater, and install it from the `.dmg`;
  - release `v0.2.1`; the installed `v0.2.0` offers *Bijwerken*, and after the click and the
    restart it runs `v0.2.1`;
  - a refused signature: a debug build pointed at a local `latest.json` whose `signature`
    belongs to another file. The update is refused and nothing changes. The override of the
    endpoint exists only in debug builds (`FAMILYLIGHTBOX_UPDATE_ENDPOINT`), so a release can't
    be pointed elsewhere.
  - Each of the two releases is public, so each needs Peter's go.

## How

- **Rust**:
  - `tauri-plugin-updater = "2"`, registered in `lib.rs`;
  - a new module `update.rs` with the two commands and the state: the update found, whether a
    download is running, and whether one was refused;
  - the restart via `AppHandle::restart()`, so no `tauri-plugin-process` is needed;
  - the check for a running sync asks the `runner` state, as `sync_start` does.
- **`tauri.conf.json`**:
  - `bundle.createUpdaterArtifacts: true`;
  - `plugins.updater` with the `pubkey` and the endpoint.
- **The unsigned dry run** turns the updater payload off with `tauri build --config` (an inline
  JSON override), so the build doesn't ask for a missing private key.
- **The frontend**:
  - `src/lib/update.ts`, holding the state and the texts (with vitest tests);
  - a small `UpdateLine` in the header;
  - the *Over* section in `Settings.tsx`.
- **`scripts/build-latest-json.py`** with `scripts/test_build_latest_json.py`. CI runs the tests,
  like those of the leak check.
- **The key** is generated by Peter with `pnpm tauri signer generate -w <path>`, which asks for
  the password itself. The public half is copied into `tauri.conf.json`, and the agent commits it.
- **Deliberately not done here**:
  - checking periodically while the app stays open;
  - release notes beyond one line;
  - channels (beta, stable);
  - Windows;
  - the installed `v0.1.0` on Peter's Mac. It has no updater, so it is replaced by hand with
    `v0.2.0`.

## Files affected

- `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `src-tauri/src/update.rs` (new),
  `src-tauri/tauri.conf.json`.
- `src/lib/update.ts` and `update.test.ts`, `src/components/UpdateLine.tsx` (new), `src/App.tsx`,
  `src/components/Settings.tsx`.
- `.github/workflows/release.yml`, `.github/workflows/ci.yml`.
- `scripts/build-latest-json.py`, `scripts/test_build_latest_json.py` (new).
- `docs/releasing.md`, `README.md`.

## Progress log

### [2026-10-27] Written

### [2026-10-27] Built

- `tauri-plugin-updater` 2.14 in Rust, `update.rs` with `update_check` and `update_install`.
  The updater refuses `update_install` during a sync, and `sync_start` refuses while an update
  installs. A failed signature check (minisign, base64, or the signed version) counts as
  refused.
- `src/lib/update.ts` (state, header line, texts; tests), `UpdateLine` in the header, *Over* in
  the settings. The app checks five seconds after start, not in a dev build.
- `tauri.conf.json`: `createUpdaterArtifacts` and `plugins.updater` with the public key and the
  endpoint. `tauri.no-updater.json` turns the payload off for a dry run without the update key.
  It is a file, not inline JSON, because tauri-action splits `args` like a shell and would strip
  the quotes.
- `release.yml`: the update key required for a tag; the `.tar.gz` and `.sig` collected and
  counted; the app inside the `.tar.gz` checked with `spctl` and `stapler`. The publish job
  builds `latest.json` with `scripts/build-latest-json.py` (tests in CI and before publishing)
  and uploads it last.
- Peter generated the key pair, set `TAURI_SIGNING_PRIVATE_KEY` and its password, and keeps the
  backup outside the repo (written down in LibraryLocal).
- All checks green locally: 51 vitest tests, 73 cargo tests, clippy, the build, and both script
  suites. The dev build starts with the plugin loaded.

### [2026-10-27] Checked: the dry run and a refused signature

- Dry run 40573511167 with the update key: green in 5 min 28 s. The artifact holds
  `FamilyLightbox.app.tar.gz` (4.0 MB), its `.sig` and the `.dmg`. The `.sig`'s key id
  (`4ecf23162f692e45`) is the public key's in `tauri.conf.json`, and the app inside the `.tar.gz`
  passed the workflow's notarization check. `build-latest-json.py` turns the files into a valid
  `latest.json`.
- A refused signature, by hand: a fake `9.9.9` payload signed with a throwaway key, served with
  its `latest.json` from `127.0.0.1`, and a debug build pointed there with
  `FAMILYLIGHTBOX_UPDATE_ENDPOINT`. *Controleren op updates* found 9.9.9. *Bijwerken* first waited
  for the sync the app starts by itself ("na de synchronisatie"), as designed. After that, the
  app downloaded the payload and showed "Update geweigerd: de handtekening klopt niet", installed
  nothing and kept running.
- Next, with Peter's go: release `v0.2.0`, install it by hand over `v0.1.0`, then release `v0.2.1`
  and update through the app.
