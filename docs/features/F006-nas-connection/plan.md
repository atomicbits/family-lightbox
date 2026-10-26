# Feature 6: Connect to the NAS

**Status:** In progress (S01–S04 built; checks in the running app and on Lynn's Mac owed)
**Depends on:** Feature 1
**Folder:** `F006-nas-connection`

## Goal

On Lynn's Mac, with nothing installed but the app, FamilyLightbox finds the shares on the NAS
`raven`, connects them when they aren't connected yet, and knows which device it is, so that the
features that follow (7 sync, 8 index, 9 originals, 3 lists) have a NAS to read from and a
registered device to write as. Without the NAS the app keeps working on what is local.

## Background

- The shares are `Library`, `Previews`, `Lists` and `Exports` on `raven` (data contract section 2).
  Lynn is an ordinary DSM user without SSH, so the app only uses file sharing. It reads `Library`
  and `Previews` and writes `Lists`, and `Exports` later.
- Lynn's account is denied the year folders before [private] on the NAS itself (ACL). The app doesn't
  filter anything; it skips what it can't read.
- The mount point is not a reliable name: macOS mounts a second `Library` as `/Volumes/Library-1`,
  and a leftover folder in `/Volumes` can look like a share. Peter's Mac has `Library` mounted
  over **AFP** today, not SMB. The app therefore identifies a share by what is mounted (server and
  share name), not by the folder name, and accepts both `smbfs` and `afpfs`. It mounts new
  connections over SMB only, because Apple has deprecated AFP.
- Laptop-local state lives in `~/.config/media/` (data contract section 7). `device.json` is part of
  the contract; the CLI on Peter's Mac already has one (`mbp-peter`). `Lists/devices.json` doesn't
  exist on the NAS yet. The CLI never built `media device register`, so the app is the first to write it.

## Scope

### In scope (macOS)

- **Settings**: NAS host (default `raven`) and the four share names, in
  `~/.config/media/familylightbox.json`. Defaults work as is; a settings screen only needs to
  show and change them.
- **Device identity**: on first start, if `~/.config/media/device.json` is missing, ask for the
  owner (`peter` or `lynn`, or a new short name) and propose `mbp-<owner>`, checked against
  `^[a-z][a-z0-9-]{1,31}$`. If the file exists, use it as is, so Peter's Mac stays `mbp-peter`.
- **Finding mounted shares**: list the mounts natively (`getmntinfo`), keep `smbfs` and `afpfs`, and
  match the server (`raven`, `raven.local`, `raven._smb._tcp.local`, `raven._afpovertcp._tcp.local`,
  case-insensitive) and the share name from the mount source, whatever the mount point is called.
- **Connecting**: for a share that isn't mounted, mount `smb://<host>/<share>` with NetFS
  (`NetFSMountURLSync`, with the macOS dialog allowed). The user signs in once in the standard
  dialog and can let the Keychain remember it. The app never stores or sees a password. This runs
  off the UI thread.
- **Checking access**: per share, whether it is reachable, readable, and (for `Lists`) writable,
  using a temporary file that is removed again.
- **Device registration** in `Lists/devices.json` (data contract 5.1): add this device if it is
  missing, with `owner`, `label` and `since`. The app writes a temporary file in `Lists/`, renames it
  over the original, and reads it back to confirm the entry is there. It never removes or changes
  another device. A name that is already registered to a different owner is refused with a clear
  message; the same name with the same owner is accepted (a reinstall).
- **Status in the UI**: a small indicator: connected, not connected (with a *Connect* button),
  or NAS unreachable (working on local data). Hovering or clicking shows each share and its mount
  point.

### Out of scope

- Windows (later, once the macOS path works for Lynn).
- Reading manifests or previews from the NAS (feature 7), the index (8), opening originals (9),
  writing list events (3).
- SSH and rsync: Peter's CLI keeps using them; the app doesn't.
- Mounting at login or keeping shares mounted in the background: the app connects when it starts
  or when asked.

## Acceptance criteria

- On a Mac without the shares mounted, *Connect* shows the macOS sign-in dialog once; afterwards
  all four shares are mounted and the indicator says connected. Tested on Lynn's Mac with her
  DSM account.
- With `Library` already mounted over AFP (Peter's Mac today), the app recognises it and doesn't
  mount it a second time. A share mounted as `/Volumes/Library-1` is recognised as well.
- A folder in `/Volumes` that isn't a mount is never taken for a share.
- With the NAS off, the app starts, says the NAS is unreachable, and browsing local previews works
  as before.
- `~/.config/media/device.json` exists after the first start; on Peter's Mac it is left untouched.
- `Lists/devices.json` contains this device after registration and still contains every device it
  contained before. A second registration of the same device changes nothing.
- Unit tests cover: parsing mount sources (SMB and AFP forms, URL-encoded names, `-1` suffixes),
  validating device names, merging into `devices.json` (add, same owner, other owner refused,
  retired device kept). `cargo test` and `pnpm test` pass.

## Stories

| ID | Title | Status | Depends on |
|---|---|---|---|
| [S01](stories/S01-settings-and-device-identity.md) | Settings and device identity (`familylightbox.json`, `device.json`, first-start question) | In progress: built and tested, in-app check owed | — |
| [S02](stories/S02-find-mounted-shares.md) | Find mounted shares by server and share name (`getmntinfo`, SMB and AFP) | In progress: built and tested, in-app check owed | S01 |
| [S03](stories/S03-connect-missing-shares.md) | Connect missing shares with NetFS and the macOS sign-in dialog | In progress: works on Peter's Mac; Lynn's Mac and NAS-offline owed | S02 |
| [S04](stories/S04-check-access-and-register-device.md) | Check access per share and register the device in `Lists/devices.json` | In progress: works on Peter's Mac; Lynn's Mac owed | S02 |
| [S05](stories/S05-connection-indicator.md) | Connection indicator and *Verbinden* in the UI | Planned | S03, S04 |

## Contract changes

Data contract 5.1 says devices are added "with `media device register`". That becomes "by
FamilyLightbox (or a CLI command doing the same)", and section 7 gains `familylightbox.json`. Both
are small text changes in `~/Pictures/LibraryLocal/docs/datacontract.md`, made with S01 and S04;
the contract itself moves into this repo with feature 7.

## Open points

- SMB is on in DSM (port 445 answers, `smbd` runs, protocols NT1 to SMB3; checked 2026-10-25).
  Whether Lynn's account can sign in over SMB and write `Lists` is not proven yet; Lynn's own SMB
  test, which is still open, settles it.
- Label and owner for Lynn's device: proposed `mbp-lynn`, owner `lynn`, label "MacBook Pro Lynn"
  (as in the contract's example).
