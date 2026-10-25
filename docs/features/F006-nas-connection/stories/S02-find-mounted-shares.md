# Story S02: Find mounted shares by server and share name

**Feature:** `F006-nas-connection`
**Status:** Planned
**Depends on:** S01

## Goal

The app knows, for each of the four shares from the settings (S01), whether it is mounted on
this Mac, and where. It finds a share by what is actually mounted (server and share name), not
by the folder name in `/Volumes`. A share mounted as `/Volumes/Library-1` is found, and a
leftover folder `/Volumes/Library` that isn't a mount is never taken for a share. This is
what S03 needs to decide what to connect, and what S04 needs to check access.

## Acceptance criteria

- **Mount list from the system**: the app reads the current mounts natively with
  `getmntinfo(MNT_NOWAIT)`, so it doesn't call `mount` or another program. With `MNT_NOWAIT`, a NAS
  that has gone away doesn't block the call. Only mounts of type `smbfs` and `afpfs` are kept.
  Only what `getmntinfo` returns can count as a share, so a plain folder in `/Volumes` never does.
- **Parsing the mount source** (`f_mntfromname`) into user, server and share:
  - `//peter@raven._smb._tcp.local/Library`, `//peter@raven._afpovertcp._tcp.local/Library`,
    `//lynn@raven/Lists`, `//raven/Previews`, `//WORKGROUP;lynn@raven.local/Exports`.
  - Share names are percent-decoded (`My%20Share` → `My Share`).
  - Anything that doesn't have the form `//[user@]server/share` is skipped, not an error.
- **Matching the server**: the configured host and the mount's server are compared
  case-insensitively, after removing a trailing `.local`, `._smb._tcp.local` or
  `._afpovertcp._tcp.local`. So `raven`, `RAVEN`, `raven.local` and both Bonjour forms all
  match host `raven` (and host `raven.local`). An IP address matches only itself.
- **Matching the share**: case-insensitive equality with the configured share name, because SMB
  share names are case-insensitive.
- **Several mounts of the same share** (for example AFP and SMB at the same time): all are
  reported. The one the app uses is the SMB mount if there is one, otherwise the first in the list.
- **Result per share**: `library`, `previews`, `lists` and `exports` each have zero or more mounts
  `{mount_point, fs_type, server, user}`, with the chosen one marked. Mounts of other shares on
  the same NAS (`TimeMachinePeter`, `Datasets`) are left out.
- **Command** `nas_mounts` returns that result for the current settings.
- **Settings panel**: under *NAS*, a read-only line per share: the mount point and type
  (`/Volumes/Library · AFP`) or "niet verbonden". It refreshes when the panel opens. Connecting
  is S03.
- **Other platforms**: on anything but macOS the mount list is empty and every share is "not
  connected". Windows comes later.
- **Tests**:
  - `cargo test` with no real mounts involved: parsing all the source forms above, plus
    `//raven` (no share), `map auto_home` and an empty string (all skipped); server matching
    (`raven` against `raven`, `RAVEN`, `raven.local`, `raven._smb._tcp.local`,
    `raven._afpovertcp._tcp.local` match; `raven2`, `ravenx.local`, `192.168.1.10` don't, and the
    IP matches itself); choosing among mounts (`Library` over AFP and as `Library-1` over SMB → the
    SMB one is chosen, both listed); type filter (`apfs`, `nfs` and `smbfs` given, only `smbfs`
    kept); share name match ignoring case.
  - On macOS, one test calls the real `getmntinfo` and checks it returns at least `/`, so the FFI
    itself is exercised.
  - Existing tests still pass.

## How

- New module `src-tauri/src/mounts.rs`:
  - `struct RawMount { from: String, on: PathBuf, fs_type: String }`, and
    `fn system_mounts() -> Vec<RawMount>`. On macOS this goes through `libc::getmntinfo` with
    `MNT_NOWAIT` and the C strings from `statfs`. Elsewhere it returns an empty `Vec`.
  - `fn parse_source(from: &str) -> Option<Source { user: Option<String>, server: String, share: String }>`,
    which is pure.
  - `fn same_server(configured: &str, mounted: &str) -> bool`, which is pure.
  - `fn find_shares(raw: &[RawMount], settings: &Settings) -> NasMounts`, which is pure: it filters
    on type, parses, matches and chooses. Everything except `system_mounts` can be tested with
    hand-made lists.
- Dependency: `libc` (the Rust bindings to the system C library; no external program).
  `percent-encoding` is already there.
- `lib.rs`: command `nas_mounts`, which loads the settings, then calls `system_mounts` and
  `find_shares`.
- Frontend: types and call in `api.ts`, and the read-only lines in `Settings.tsx`.
- Deliberately not done here: mounting (S03), checking that a found share is readable or
  writable (S04), the indicator in the header (S05). A found mount isn't opened or read in this
  story, so a slow or dead NAS can't make the panel hang.

## Files affected

- `src-tauri/src/mounts.rs` (new), `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml` (`libc`).
- `src/lib/api.ts`, `src/components/Settings.tsx`.

## Progress log

### [2026-10-25] Written
