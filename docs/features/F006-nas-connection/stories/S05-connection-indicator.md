# Story S05: Connection indicator and *Verbinden* in the UI

**Feature:** `F006-nas-connection`
**Status:** In progress: built and tested; the quiet connect at start checked on Peter's Mac; the indicator in the running app and Lynn's Mac still to check
**Depends on:** S03, S04

## Goal

Lynn sees at a glance whether the app is connected to the NAS, and can connect with one click
from the main window. On start the app connects on its own when macOS already has the account
in the Keychain, but it never pops up a sign-in dialog without being asked. With feature 6
finished, the features that read from the NAS (7, 8, 9, 3) have one place to ask whether the NAS
is there.

## Acceptance criteria

- **Indicator in the header**, right of the existing hints and left of ⚙, one of:
  - **verbonden** (green dot): all four shares mounted, `Library`, `Previews` and `Exports`
    readable, `Lists` readable and writable;
  - **deels verbonden** (amber dot) plus *Verbinden*: at least one share is mounted and at least
    one isn't, or a mounted share has an access problem;
  - **niet verbonden** (grey dot) plus *Verbinden*: no share mounted;
  - **NAS niet bereikbaar** (grey dot) plus *Opnieuw*: the last connect attempt ended in
    "unreachable";
  - **bezig…** while a check or connect runs.
  Clicking the indicator text opens the settings panel, which shows each share's mount point and
  access (S02, S04).
- **At start**:
  1. If there is no device yet, the first-start dialog (S01) comes first; the steps below run
     once it is saved.
  2. Look up the mounts (S02).
  3. If something is missing, connect it **without UI**: the same call as S03 with the open
     option `UIOption = NoUI`. That succeeds when the Keychain has the account and fails quietly
     otherwise. A failure at start isn't shown as an error. The indicator just says "niet
     verbonden" or "deels verbonden" with *Verbinden*.
  4. Run the check (S04), which also registers the device.
  Browsing local previews works during all of this; nothing waits on the NAS.
- ***Verbinden*** in the header does what the panel's button does (S03, with the macOS dialog
  allowed), then runs the check. Both buttons share one state, so a connect started in one shows
  as busy in the other, and a second click while busy does nothing.
- **Staying current**: when the window regains focus, the app looks up the mounts and checks
  again, at most once every 30 seconds. Ejecting a share in the Finder therefore shows up the
  next time Lynn comes back to the app. Closing the settings panel also refreshes the state. There
  is no polling in the background.
- **One source of state**: a React hook `useNas()` holds mounts, check, last attempts and busy,
  plus `refresh()` and `connect()`. The header and the settings panel both use it, so neither
  calls the commands on its own anymore.
- **Commands**: `connect_nas` gets a parameter `allow_ui: bool`. `true` is the button (S03 as
  is), `false` is the start. Nothing else changes in Rust.
- **Tests**:
  - `pnpm test`: a pure `nasSummary(mounts, check, attempts, busy)` gives the state, label and
    whether to offer *Verbinden* or *Opnieuw*, for each state above, including:
    - all mounted but `Lists` not writable → deels verbonden;
    - a last attempt "unreachable" → NAS niet bereikbaar, even though old mounts are still
      listed;
    - busy wins over everything.
  - `cargo test`: the UI option for `allow_ui` true and false (`AllowUI` / `NoUI`).
  - **By hand**, in the progress log:
    - Peter's Mac with everything mounted: start shows "verbonden" without a dialog;
    - eject `Lists` in the Finder, return to the app: "deels verbonden"; *Verbinden* mounts it
      again;
    - Lynn's Mac on first start: the device dialog, no sign-in dialog on its own, then
      *Verbinden* shows the macOS dialog;
    - with the NAS unreachable: "NAS niet bereikbaar" and *Opnieuw*.
  - Existing tests still pass.
- **Feature 6 done**: when the manual checks of S01–S05 have passed on Peter's and Lynn's Macs,
  feature 6 meets its acceptance criteria in `plan.md` and gets the status *Done*.

## How

- Rust:
  - `connect.rs`: `NetFsMounter { allow_ui: bool }` and `fn ui_option(allow_ui) -> &'static str`.
  - `lib.rs`: `connect_nas(allow_ui: bool, …)`.
- Frontend:
  - `src/lib/nas.ts` (new): `nasSummary` (pure) and the `useNas()` hook. The hook is created
    once in `App.tsx` and passed down to the panel.
  - `src/components/NasIndicator.tsx` (new): dot, label and button.
  - `Settings.tsx` uses the hook's state instead of its own calls.
  - `App.tsx`: the start sequence, the focus listener (30 s throttle), and the indicator in the
    header.
- Deliberately not done here:
  - mounting at login;
  - keeping shares connected while the app isn't open;
  - a menu-bar item;
  - notifications;
  - Windows.

## Files affected

- `src-tauri/src/connect.rs`, `src-tauri/src/lib.rs`.
- `src/lib/nas.ts` (new), `src/lib/nas.test.ts` (new), `src/lib/api.ts`,
  `src/components/NasIndicator.tsx` (new), `src/components/Settings.tsx`, `src/App.tsx`.

## Progress log

### [2026-10-26] Written

### [2026-10-26] Built

- `connect.rs`: `NetFsMounter { allow_ui }` and `ui_option` (`AllowUI` / `NoUI`). `connect_nas`
  takes `allow_ui`.
- `src/lib/nas.ts`: pure `nasSummary` and the `useNas()` hook (`refresh`, `connect`, `start`),
  with one shared busy guard so a second click while busy does nothing. A refresh that finds
  everything mounted clears a stale "unreachable".
- `NasIndicator.tsx` in the header (dot, label that opens the panel, *Verbinden* / *Opnieuw*).
  `App.tsx`: the start sequence once the device exists, and a focus listener with a 30 s
  throttle. The settings panel now uses the hook, refreshes when it opens, and refreshes again
  when it closes or after *Bewaar*.
- `cargo test` 31 passed (1 new), `pnpm test` 15 passed (5 new), `tsc`, `pnpm build` and
  `cargo clippy --all-targets` clean.
- **By hand on Peter's Mac**, with a throwaway example calling the mounter with
  `allow_ui: false` (removed afterwards):
  - overnight `Lists` and `Previews` had been unmounted (likely sleep); the quiet connect
    mounted both again without any dialog, from the Keychain;
  - with host `raven-bestaat-niet`: `Lists` unreachable, and the rest skipped right away.
- Still to do by hand: the indicator and focus refresh in the running app, Lynn's first start
  (no sign-in dialog on its own), and the NAS really switched off.
