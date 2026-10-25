# Story S04: Check access per share and register the device in `Lists/devices.json`

**Feature:** `F006-nas-connection`
**Status:** Planned
**Depends on:** S01, S02 (S03 for connecting first, but S04 also works on shares mounted by hand)

## Goal

For each connected share, the app knows whether it can actually use it: `Library`, `Previews`
and `Exports` readable, and `Lists` readable and writable. Once `Lists` is writable, the app
registers this device in `Lists/devices.json` (data contract 5.1), so that list events can be
written later (feature 3) and the CLI counts them. This is the first time the app writes on the
NAS. It writes exactly two things: a test file in `Lists` that it removes again right away, and
`Lists/devices.json`.

## Acceptance criteria

- **Access check** per share, on the mount S02 chose:
  - *readable*: the root of the mount can be listed. DSM's own entries (`@eaDir`, `#snapshot`,
    `#recycle`) don't count and aren't opened.
  - *writable*, only for `Lists`: the app creates `Lists/.familylightbox-<device>-<random>.tmp`
    with `create_new`, writes one byte, and removes the file again. It never writes a test file
    in `Library`, `Previews` or `Exports`. A file left behind by a crash is removed on the next
    check (same prefix and this device's name, older than an hour).
  - It runs on a blocking thread, so a slow or dead NAS doesn't freeze the window (with the soft
    mount from S03 it ends in an error, not a hang).
  - The result per share is one of: not mounted, readable, readable and writable, or not readable
    (with the system's reason).
- **Registration**, automatic once `Lists` is writable and the device from S01 exists:
  - **Merging** (pure, on the file's JSON, so unknown fields and the order of other devices stay
    as they are):
    - no `devices.json` → a new file `{"contract_version": "1.0", "devices": [this device]}`;
    - this device missing → appended;
    - same `name` and same `owner` → nothing to do, and nothing is written;
    - same `name` with another `owner` → refused: "toestelnaam '<name>' is al van <owner>";
    - same `name` marked `retired` → refused: "dit toestel staat als buiten gebruik in
      devices.json". Taking a device back into use is done by hand;
    - not valid JSON, or no `devices` list → refused with the reason, and the file isn't touched.
  - **The new entry**: `name` and `owner` from `device.json`; `label` from the Mac's local host
    name without `.local` (for example `Lynns-MacBook-Pro`), or "Mac van <owner>" when there is
    none; `since` = now in ISO 8601 with the local offset (`2026-10-26T10:00:00+01:00`), as in the
    contract's example.
  - **Writing**: to `Lists/devices.json.tmp-<device>` in the same folder, then renamed over
    `devices.json`, then read back.
  - **Two devices at once**: the read-back has to contain every device that was there before plus
    this one. If another device wrote in between and an entry is missing, the app merges again
    from the current file and writes again, at most three times. Each device checks its own entry
    after writing, so concurrent registrations converge.
- **Command** `nas_check` returns the access per share and the registration outcome (registered
  now, already registered, refused with reason, or not tried because `Lists` isn't writable or
  there is no device yet). The settings panel calls it when it opens and after *Verbinden*. At
  app start it runs from S05.
- **Panel**: per share, after the mount point: "leesbaar", "leesbaar en schrijfbaar" (`Lists`),
  or the problem. Under *Dit toestel*: "geregistreerd op de NAS", "nu geregistreerd", or the
  reason it was refused.
- **Contract**: in data contract 5.1, "Toevoegen gebeurt met `media device register`" becomes
  "by FamilyLightbox when `Lists` is writable (or a CLI command doing the same): read, merge,
  temporary file, rename, read back". `label` and `since` are described as above. This is a text
  change in `~/Pictures/LibraryLocal/docs/datacontract.md`.
- **CLI still works**: after registration, `./media thin` (droogloop) on Peter's Mac still reads
  the lists without error. With `devices.json` present, the CLI only counts events from
  registered devices (data contract 5.3); there are no events yet, so nothing changes.
- **Tests**:
  - `cargo test` on temporary folders:
    - access: readable folder, a folder with mode `000` (not readable, with the reason),
      `Lists` writable, `Lists` read-only (not writable, and no test file left behind), and a stale
      test file of this device that gets removed;
    - merging: every case above, including that other devices and unknown fields stay
      byte-for-byte equal in content and order;
    - writing: no temporary file left, and the read-back is correct;
    - concurrency: a test hook writes a competing `devices.json` (with another device added)
      between reading and renaming. The result contains both devices after the retry.
  - **By hand**, in the progress log:
    - Peter's Mac registers `mbp-peter`/`peter` in the real `Lists/devices.json`, and a second
      check writes nothing;
    - Lynn's Mac registers `mbp-lynn`/`lynn` and keeps `mbp-peter`;
    - `./media thin` on Peter's Mac still runs.
  - Existing tests still pass.

## How

- New module `src-tauri/src/nas.rs`:
  - `fn check_share(mount: &Path, write_test: Option<&str>) -> Access`.
  - `fn merge_device(existing: Option<&str>, me: &Entry) -> Result<Option<String>, String>`, which
    is pure. `None` means nothing to write. It works on `serde_json::Value` with the
    `preserve_order` feature, so other entries keep their order.
  - `fn register(lists: &Path, device: &Device, label: &str, now: &str, before_rename: &dyn Fn())
    -> RegisterOutcome`, with read, merge, write, rename, read back, and up to three tries.
    `before_rename` is a no-op in the app and the competing write in the test.
  - `fn local_label(owner) -> String` from `libc::gethostname` (macOS), with the fallback above.
- Dependencies:
  - `serde_json` with `preserve_order`;
  - `chrono` (with only the `clock` feature) for `since` with the local offset;
  - `rand` is not needed: the random part of the test file name comes from the time and the
    process id.
- `lib.rs`: `async` command `nas_check` on a blocking thread, using the settings, S02's mounts
  and the device from S01.
- Frontend: `nas.check()` in `api.ts`; the lines in `Settings.tsx`.
- Deliberately not done here:
  - creating `Lists/events/` or writing events (feature 3);
  - writing anything in `Library`, `Previews` or `Exports`;
  - taking a retired device back into use;
  - the header indicator (S05).

## Files affected

- `src-tauri/src/nas.rs` (new), `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml`.
- `src/lib/api.ts`, `src/components/Settings.tsx`.
- `~/Pictures/LibraryLocal/docs/datacontract.md` section 5.1 and its change log (outside this repo).

## Progress log

### [2026-10-25] Written
