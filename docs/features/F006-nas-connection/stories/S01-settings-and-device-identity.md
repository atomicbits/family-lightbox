# Story S01: Settings and device identity

**Feature:** `F006-nas-connection`
**Status:** Done on Peter's Mac; first start on Lynn's Mac deferred
**Depends on:** —

## Goal

The app knows which NAS and shares to use, and which device it is. That is the base the rest of
feature 6 builds on: S02 needs the host and share names to recognise mounts, and S04 needs the
device to register it in `Lists/devices.json`. On Lynn's Mac the app asks once who she is. On
Peter's Mac it picks up the existing `mbp-peter` without touching it.

## Acceptance criteria

- **Settings file** `~/.config/media/familylightbox.json` (in the folder from `MEDIA_CONFIG`,
  as for the index):
  - Missing file: the defaults apply (host `raven`; shares `Library`, `Previews`, `Lists`,
    `Exports`) and nothing is written until the user saves.
  - A file with only some fields: the missing fields get their default.
  - A file that isn't valid JSON: the app starts on the defaults, shows the error (path and
    reason) in the settings panel, and does not overwrite the file until the user saves.
  - Saving writes a temporary file in the same folder and renames it over the original; no
    temporary file is left behind. The folder is created if it doesn't exist.
  - Validation on save: the host is not empty and contains no `/`, spaces or a scheme
    (`smb://` is stripped with a clear message, not silently); each share name is not empty and
    contains no `/`. Invalid input is refused with the reason, and the file stays as it was.
- **Device identity** `~/.config/media/device.json` (data contract section 7), format
  `{"name": "...", "owner": "..."}`:
  - If it exists, it is read and **never rewritten**, so the byte content on Peter's Mac is
    identical before and after starting the app. Unknown extra fields are allowed. A file without
    a valid `name` or `owner` is shown as an error in the panel, and the app doesn't replace it on
    its own.
  - If it is missing, a first-start dialog appears over the app. The user picks the owner (`lynn`,
    `peter`, or another short name) and gets the proposed device name `mbp-<owner>`, which can be
    edited. Then the user saves. Until then, browsing works behind the dialog; the dialog can't be
    dismissed without choosing, because writing lists (feature 3) needs it.
  - Name rule: `^[a-z][a-z0-9-]{1,31}$` (data contract 1). Owner rule: `^[a-z][a-z0-9]{1,31}$`, a
    short lowercase name such as `lynn`. The dialog shows why a name is refused while typing. Rust
    checks again on save, and Rust is the authority.
  - Creating the file refuses if it already exists (create-new, no overwrite), so two windows or
    a race can't replace an identity.
- **Settings panel**: a gear button in the header opens a panel that shows the device (name and
  owner, read-only, with the path of `device.json`), the host and the four share names (editable,
  *Save* and *Cancel*), and errors from loading either file. The text in the app is in Dutch, as
  it is today (Peter and Lynn are the users).
- **Contract**: data contract section 7 lists `familylightbox.json` as the app's own settings file
  (not part of the contract, like `state.json`). This is a text change in
  `~/Pictures/LibraryLocal/docs/datacontract.md` until the contract moves here with feature 7.
- **Tests**:
  - `cargo test`: defaults when missing; partial file merged with defaults; invalid JSON reported
    and file untouched; save and load round-trip with no temporary file left; host and share
    validation (good and bad cases, `smb://raven` refused with its message); device name and
    owner validation (good: `mbp-lynn`, `mbp-2`; bad: `Mbp-lynn`, `m`, `-x`, 33 characters, `mbp_lynn`);
    existing `device.json` read and byte-identical afterwards; creating when one exists is refused.
  - `pnpm test`: proposed name from owner; live name validation matching the Rust rules on the
    same cases.
  - Existing tests still pass.

## How

- New module `src-tauri/src/config.rs`:
  - `Settings { nas_host: String, shares: Shares { library, previews, lists, exports } }` with
    `#[serde(default)]` on each field so a partial file fills in. `load(dir) -> (Settings,
    Option<String>)` returns the defaults plus an error string for a broken file instead of
    failing the app. `save(dir, &Settings)` validates, then writes `familylightbox.json.tmp` and
    renames it.
  - `Device { name: String, owner: String }` read with `serde_json::Value` first, so extra fields
    are fine and the file is never re-serialised. `create_device(dir, name, owner)` validates and
    opens the file with `create_new(true)`.
  - The validation functions are hand-written character checks, so the regex crate isn't needed.
- `Paths` in `lib.rs` gets `settings()` and `device()` next to `index()`, all under `config`.
- New commands: `settings` (settings plus load error), `save_settings`, `device` (device, missing,
  or an error), `create_device`. Register them in `invoke_handler`; there is no state to manage,
  because each command reads the files.
- Frontend:
  - `src/lib/api.ts` gets the types and calls.
  - `src/lib/device.ts` gets `proposeDeviceName(owner)` and `deviceNameProblem(name)` /
    `ownerProblem(owner)`, which return `null` or a Dutch reason.
  - `src/components/DeviceSetup.tsx` is the first-start dialog.
  - `src/components/Settings.tsx` is the panel.
  - `App.tsx` loads `device()` at start: missing shows `DeviceSetup`; the gear in the header
    toggles `Settings`.
- Deliberately not done here: anything that touches the NAS. Host and shares are only stored and
  checked for form; recognising and mounting them is S02 and S03, and registering the device is S04.

## Files affected

- `src-tauri/src/config.rs` (new), `src-tauri/src/lib.rs` (paths, commands, handler).
- `src/lib/api.ts`, `src/lib/device.ts` (new), `src/lib/device.test.ts` (new).
- `src/components/DeviceSetup.tsx` (new), `src/components/Settings.tsx` (new), `src/App.tsx`.
- `~/Pictures/LibraryLocal/docs/datacontract.md` section 7 and its change log (outside this repo).

## Progress log

### [2026-10-25] Written

- Story written after Peter approved the feature 6 plan.

### [2026-10-25] Built

- `src-tauri/src/config.rs`: settings with per-field defaults, load that reports a broken file
  instead of failing, atomic save with validation; device identity read without re-serialising,
  created with `create_new` only. Hand-written name rules, no regex crate.
- `lib.rs`: commands `settings`, `save_settings`, `device`, `create_device`.
- Frontend: `lib/device.ts` (same rules for live feedback), `DeviceSetup.tsx` (first start, can't
  be dismissed), `Settings.tsx` (gear in the header; *Bewaar* is also enabled when the file is
  broken, so it can be repaired with the defaults), a header hint when `device.json` is invalid.
  The grid's keyboard shortcuts are off while a dialog is open or an input has focus, so typing a
  space in a name field doesn't open a photo.
- Data contract section 7 (in LibraryLocal): `familylightbox.json` listed; `device.json` is created
  once and never rewritten.
- `cargo test` 14 passed (9 new), `pnpm test` 7 passed (4 new), `pnpm build` and `cargo clippy
  --all-targets` clean.
- Open: a look in the running app (first-start dialog with an empty `MEDIA_CONFIG`; the panel on
  Peter's Mac, where `device.json` must stay byte-identical).

### [2026-10-26] Checked in the running app

- Peter looked at the app on his Mac: the NAS indicator shows "NAS verbonden" and the app works as expected. The checks on Lynn's Mac are deferred until she has a build (feature 4).
