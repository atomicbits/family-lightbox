# Story S08: Staying connected: reconnecting quietly, and syncing once the NAS is there

**Feature:** `F006-nas-connection`
**Status:** In progress: built and tested; to check on Lynn's Mac after a release
**Depends on:** S03, S05, feature 7/S05

## Goal

When Lynn opens her laptop, the app finds the NAS again and brings in what is new, without anyone
clicking *Verbinden*. Today, on her Mac (v0.5.0), the NAS stayed unconnected until *Verbinden* was
clicked, and the sync failed in red (*gedeelde map 'Fotobibliotheek' is niet verbonden*) and didn't
start again after connecting.

## Background

- The app connects quietly (without a dialog, with the Keychain's account) once, at start. A Mac
  that sleeps unmounts the shares; back in the app, the window's focus only looks at the mounts
  again (every 30 s at most), it doesn't connect. At start, too, a single try fails when the
  network isn't up yet, as after opening the lid.
- The sync runs once per app process, at start (feature 7/S05): when the NAS wasn't there, it
  fails and doesn't run again, also not after *Verbinden*.

## Acceptance criteria

- **Reconnecting**: while a share of this Mac's layout is missing, the app connects quietly at
  start, when the window comes back (focus), when the network comes back (`online`), and every
  minute meanwhile; never with a dialog, which stays for *Verbinden*.
- **Syncing when the NAS is there**: the sync runs as soon as `Library` is connected and no sync
  has settled it since the app started (done, waiting for the confirmation, or cancelled): at start, after reconnecting quietly, and after
  *Verbinden*. Once one has finished, coming back from sleep doesn't sync again (as now: not on
  focus, not on a timer).
- The sync after a reconnect counts the previews as the one at start does (feature 7/S07, only
  where something can be new), not in full.
- A sync that failed is tried again only when the mounts changed (a reconnect, *Verbinden*, a
  look on focus), never in a loop.
- **Tests**: `pnpm test` for when to connect and when to sync (missing shares, a sync done or not,
  one running); `cargo test` for the sync at start and the later quiet ones choosing their scope.

## How

- `src/lib/nas.ts`: `shouldConnect` and `shouldSync`, pure.
- `App.tsx`: one effect starts the sync when `Library` is connected and a sync is still due; the
  focus, `online` and a one-minute timer connect quietly while something is missing.
- `lib.rs`: `sync_start` with `at_launch: Some(false)` is a quiet later sync: the scope of feature
  7/S07, without the once-per-process rule of the sync at start.

## Files affected

- `src/App.tsx`, `src/lib/nas.ts`, `src-tauri/src/lib.rs`, and their tests.

## Progress log

### [2026-11-03] Written

From Peter, opening Lynn's laptop with v0.5.0.

### [2026-11-03] Built

- `nas.ts`: `shouldConnect` (mounts known, a share missing, no connect running), `shouldSync`
  (`Library` connected, nothing settled, none running) and `syncSettled`.
- `App.tsx`: the start only connects; one effect starts the sync whenever the mounts change and a
  sync is due, the first as the sync at start, later ones quietly (`at_launch: false`), and a
  failed one only again with other mounts. Focus (at most every 30 s), `online` and a one-minute
  timer connect quietly while a share is missing, else look at the mounts as before. The lists
  are read again when `Library` comes up.
- `lib.rs`: `sync_start` gives a quiet later sync the scope of the sync at start (feature 7/S07)
  without its once-per-process rule.
- `pnpm test` 153 passed (3 new), `tsc` and `vite build` clean; `cargo test` 198 passed, `cargo
  clippy --all-targets` clean.
- Not yet known: whether the quiet connect on Lynn's Mac can use the Keychain's account at all
  (it needs the password saved there); if not, only *Verbinden* with its dialog connects.
