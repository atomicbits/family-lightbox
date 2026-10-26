# Story S03: Connect missing shares with NetFS and the macOS sign-in dialog

**Feature:** `F006-nas-connection`
**Status:** Done on Peter's Mac; Lynn's Mac (one sign-in) and a NAS that is really off (soft mount) deferred
**Depends on:** S02

## Goal

When a share isn't mounted, the app can connect it itself. On Lynn's Mac, *Verbinden* is enough:
macOS shows its own sign-in dialog once, Lynn signs in with her DSM account (and can let the
Keychain remember it), and the shares that were missing are then mounted over SMB. The app never
asks for, sees or stores a password.

## Acceptance criteria

- **Only what is missing**: the app connects only the shares for which S02 found no mount at all.
  A share that is already mounted, over SMB or AFP and under any mount point, is left alone. On
  Peter's Mac today that means `Library` (AFP) stays as it is and `Previews`, `Lists` and `Exports`
  are connected over SMB.
- **How**: `NetFSMountURLSync` from the system framework NetFS, with:
  - the URL `smb://<host>/<share>`, the share name percent-encoded;
  - no mount path, so macOS chooses `/Volumes/<share>` and adds `-1` if that is taken;
  - no user and no password;
  - the open option `UIOption = AllowUI`, so macOS shows its sign-in dialog only when it has no
    stored credentials;
  - the mount option `SoftMount = true`, so a NAS that disappears gives read errors instead of
    hanging the app or the Finder.
  It never uses AFP, and it never calls `mount_smbfs`, `open smb://` or another program.
- **Off the UI thread**: the call blocks while the dialog is open, so it runs in a background
  task. The window stays responsive; *Verbinden* is disabled and shows that it is busy until all
  shares are done.
- **In order, with one sign-in**: the missing shares are connected one after another in the
  order `Lists`, `Library`, `Previews`, `Exports` (`Lists` first because writing lists needs it).
  macOS reuses the signed-in session for the next shares on the same server. That this means one
  dialog per connect is checked on Lynn's Mac; if it turns out not to hold, that is written down
  here, and S03 stays done.
- **Outcome per share**, in Dutch in the panel:
  - mounted → the mount point (from a new S02 lookup afterwards, not from NetFS's answer alone);
  - already mounted (NetFS returns `EEXIST`) → treated as success;
  - cancelled in the dialog (`ECANCELED`, or `-128` user cancelled) → "geannuleerd". Then the
    remaining shares are not attempted, so the user isn't asked again right away;
  - wrong account or password (`EAUTH`) → "aanmelden mislukt";
  - share doesn't exist (`ENOENT`) → "gedeelde map '<name>' bestaat niet op <host>";
  - NAS unreachable (`EHOSTUNREACH`, `ETIMEDOUT`, `ENETUNREACH`, `EHOSTDOWN`) → "<host> is niet
    bereikbaar". Then the remaining shares are not attempted either;
  - any other code → "verbinden mislukt (code N)". The remaining shares are still attempted.
- **Panel**: *Verbinden* sits next to "Gevonden op deze Mac". It is enabled when at least one
  share is not connected. After a run, the lines show the new state and the outcome per share.
  The indicator in the header and connecting at start-up are S05.
- **Tests**:
  - `cargo test`, without NetFS or a NAS:
    - building the URL (`raven` + `Library` → `smb://raven/Library`; `raven.local` +
      `My Share` → `smb://raven.local/My%20Share`);
    - choosing what to connect from a `NasMounts` (none mounted → all four in the order above;
      `Library` over AFP → the other three; all mounted → nothing);
    - mapping each return code above to its outcome;
    - the sequence logic against a fake mounter: cancel stops, unreachable stops, an
      unknown error continues, `EEXIST` counts as success.
  - **By hand**, recorded in the progress log:
    - on Peter's Mac: *Verbinden* mounts `Previews`, `Lists`, `Exports` over SMB, and `Library`
      stays on AFP;
    - on Lynn's Mac with her DSM account: the dialog appears, signing in mounts all four, and a
      second *Verbinden* with everything mounted does nothing;
    - with the NAS unreachable: the message above, and no hang.
  - Existing tests still pass.

## How

- New module `src-tauri/src/connect.rs`:
  - `fn share_url(host, share) -> String` and `fn to_connect(&NasMounts, &Settings) ->
    Vec<(ShareKey, String)>`, both pure.
  - `enum Outcome { Mounted, AlreadyMounted, Cancelled, AuthFailed, NoSuchShare, Unreachable,
    Failed(i32) }`, with `fn outcome(code: i32) -> Outcome` (pure) and a Dutch message per
    variant.
  - `trait Mounter { fn mount(&self, url: &str) -> i32 }`. The real `NetFsMounter` (macOS only)
    wraps `NetFSMountURLSync` and builds the `CFURL` and both option dictionaries with the
    `core-foundation` crate. Link with `#[link(name = "NetFS", kind = "framework")]`; there is no
    bindgen and no extra program.
  - `fn connect_all(m: &dyn Mounter, list) -> Vec<(ShareKey, Outcome)>`, with the stop rules
    above. This is what the fake mounter tests.
- `lib.rs`: an `async` command `connect_nas` that runs `connect_all` in
  `tauri::async_runtime::spawn_blocking`, then repeats the S02 lookup and returns the outcomes
  together with the fresh `NasMounts`. On other platforms it returns "not supported" without
  trying anything.
- Dependencies: `core-foundation` (Rust bindings to Apple's CoreFoundation; Tauri already pulls
  it in on macOS) and the system framework NetFS, which ships with macOS.
- Frontend: `nas.connect()` in `api.ts`; button, busy state and outcomes in `Settings.tsx`.
- Deliberately not done here:
  - disconnecting;
  - connecting at app start or keeping shares connected (S05 decides what happens at start);
  - checking that a connected share is readable and `Lists` writable (S04);
  - Windows.

## Files affected

- `src-tauri/src/connect.rs` (new), `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`
  (`core-foundation`).
- `src/lib/api.ts`, `src/components/Settings.tsx`.

## Progress log

### [2026-10-25] Written

### [2026-10-25] Built

- `src-tauri/src/connect.rs`: `share_url`, `to_connect` (`Lists` first), `outcome` for the NetFS
  return codes, `connect_all` with the stop rules, and `NetFsMounter`. That one calls
  `NetFSMountURLSync` with `UIOption = AllowUI` and `SoftMount = true` and a null mount path, user
  and password, and links the NetFS framework directly. The return codes are written out as
  Darwin values in the module, so it also compiles where `libc` has other numbers.
- `lib.rs`: async command `connect_nas`, which runs on a blocking thread and repeats the S02
  lookup afterwards. `libc` and `core-foundation` are now macOS-only dependencies.
- Settings panel: *Verbinden* next to "Gevonden op deze Mac". It is disabled while busy, while all
  shares are mounted, or while there are unsaved settings. Each share shows its outcome when it
  isn't plain "verbonden".
- `cargo test` 24 passed (5 new), `pnpm test` 9 passed (1 new), `pnpm build` and `cargo clippy
  --all-targets` clean.
- **By hand on Peter's Mac**, with a throwaway example running the same code (removed
  afterwards):
  - `Lists`, `Previews` and `Exports` were mounted over SMB 3.1.1 at `/Volumes/<share>`, without a
    dialog, because the Keychain already had the account;
  - `Library` stayed on AFP;
  - a second run found nothing to connect.
- Not verified: whether `SoftMount` took effect. Neither `mount` nor `smbutil statshares` shows it.
  The unreachable-NAS test will show it.
- Still to do by hand: Lynn's Mac with her DSM account (dialog once?), the NAS unreachable, and
  the button in the running app.

### [2026-10-26] Checked in the running app

- Peter looked at the app on his Mac: the NAS indicator shows "NAS verbonden" and the app works as expected. The checks on Lynn's Mac are deferred until she has a build (feature 4).
