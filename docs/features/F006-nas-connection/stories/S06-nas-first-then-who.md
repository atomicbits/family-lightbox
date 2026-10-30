# Story S06: First start without built-in names: the NAS first, then who you are

**Feature:** `F006-nas-connection`
**Status:** Built (2026-10-30); to check on Lynn's Mac with v0.4.0
**Depends on:** S01–S04

## Goal

The app holds no names of this family: no people, no NAS. A new user, here or in another family,
first connects the app to their NAS, signing in with their own NAS account, and the app proposes
who they are from that account and from the devices the NAS already knows (Peter, 2026-10-30:
the welcome screen offered `Lynn` / `Peter` / someone else, which an open-source app can't do).

## Acceptance criteria

- **No names in the code**: no default NAS host (it was `raven`) and no owners (`lynn`, `peter`)
  outside tests and comments. The share names `Library`, `Previews`, `Lists`, `Exports` stay as
  defaults, because the data contract names them.
- **An existing installation keeps working** without asking anything: with no settings file, the
  app takes the NAS host once from the shares that are mounted (a mount of the `Lists` share
  that holds `devices.json`), or else from the CLI's `state.json` (`nas.library` as
  `host:path`), and writes it to its settings file. `device.json` is never touched (S01).
- **Step 1, the NAS** (shown when this Mac has no device yet):
  - the NAS's name or address, empty unless found as above; the share names behind a small
    *andere namen* toggle;
  - *Verbinden* saves the settings and connects the missing shares with the macOS sign-in
    dialog (S03): the user signs in with their own NAS account;
  - when `Lists` is connected the app goes to step 2; otherwise it shows why (the attempts of
    S03), and the user can try again or continue without the NAS.
- **Step 2, who you are**:
  - the owner proposed from the NAS user the `Lists` share is mounted with (`//lynn@nas/Lists`
    → `lynn`, a workgroup prefix dropped), made to fit the owner rule; empty without the NAS;
  - the people the NAS knows already, from `Lists/devices.json`, each with their devices; a
    click takes that name (a second laptop of the same person);
  - the device name proposed from the Mac's own name (`MBP Lynn` → `mbp-lynn`, made to fit the
    device rule); refused at once when `devices.json` has that name for another owner;
  - *Bewaar* writes `device.json`; the check of S04 then registers the device as today.
- **Without a NAS host** the indicator says the NAS isn't set up and connecting asks for it in
  the settings, instead of trying an empty name.
- **Tests**: `cargo test` for adopting the host (from a mount, from `state.json`, not when a
  settings file exists), the proposals (user name, computer name, uniqueness, a workgroup
  prefix), reading the known devices; `pnpm test` for the texts and the step logic.

## How

- `config.rs`: `Settings::default()` with an empty host; `adopt_host(dir, mounts)` called once
  at start when there is no settings file.
- `nas.rs`: `known_devices(lists)`; `setup_proposal(...)` gives the owner, the device name and
  the known devices; a command `nas_whoami`.
- `connect_nas` refuses an empty host with a clear message.
- `DeviceSetup.tsx` becomes the two-step welcome; it gets the `nas` hook to connect.

## Files affected

- `src-tauri/src/config.rs`, `src-tauri/src/nas.rs`, `src-tauri/src/lib.rs`,
  `src/components/DeviceSetup.tsx`, `src/lib/device.ts`, `src/lib/api.ts`, `src/App.tsx`,
  `src/lib/nas.ts`, `src-tauri/src/mounts.rs` (`NasMounts.host`), `src-tauri/examples/setup-check.rs`.

## Progress log

### [2026-10-30] Written

### [2026-10-30] Built

- `config.rs`: `Settings::default()` has no host; `adopt_host` (a mounted `Lists` share with
  `devices.json`, else the CLI's `nas.library`), called once in the app's `setup`.
- `nas.rs`: `known_devices`, `owner_from_user`, `device_slug`, `propose_device` (a number when
  the NAS has the name for someone else), `computer_name` (`scutil --get ComputerName`).
- `lib.rs`: `nas_whoami`; `connect_nas` refuses an empty host. `mounts.rs`: `NasMounts.host`,
  so the indicator says *NAS niet ingesteld* and offers no connecting.
- `DeviceSetup.tsx`: the two steps; step 1 is skipped when the NAS is set up and `Lists` is
  mounted already. `device.ts`: `proposeDeviceName` (`mbp-<owner>`) gone; `takenBy`, `people`.
- Tests: `cargo test` 165 (adopting the host, the proposals, the known devices; the old tests
  that relied on the built-in host now name it), `pnpm test` 118, clippy, tsc and lint clean.
  Outside tests and comments no names of people or of the NAS are left in the code.
- On Peter's Mac, read-only (`examples/setup-check`, the settings in a temporary folder): the host
  `raven` is adopted from the mounted `Lists`; the proposal is `peter` from the NAS user and
  `mbp-peter-si` from the Mac's name, with `mbp-peter` known. At the app's next start Peter's
  Mac writes its settings file once with `raven`; `device.json` stays as it is.
