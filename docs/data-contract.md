# Data contract

*Version 1.2, 17 October 2026. This document sets down what the data shared by the CLI (`media`), the desktop app (FamilyLightbox) and the Lightroom plug-in looks like: folders, file names, manifests, previews, lists and local state. Everything stated here is binding for every component; anything not stated here is up to each component. Changes raise the version number and get a line in section 12.*

*Moved into this repo, translated from Dutch, on 2026-10-26 (FamilyLightbox F007/S01). The section numbers are unchanged, so references such as "data contract 5.1" in code and documents stay valid. Operational details of the home network that a component doesn't need to follow the contract (SSH configuration, which account is denied which years, the name of the Drive mirror share) are kept in a private note outside this repo, `docs/nas-inrichting.md` in LibraryLocal.*

The words **MUST**, **MUST NOT**, **SHOULD** and **MAY** are meant as in RFC 2119.

## 1. General conventions

| Topic | Rule |
|---|---|
| Text encoding | UTF-8, without BOM. JSON per RFC 8259, JSON Lines one object per line with `\n` as separator. |
| Timestamps | ISO 8601 with a time zone offset, at least second precision: `2026-10-20T19:42:11+02:00`. UTC is written as `+00:00`, not `Z`. |
| Dates | `YYYY-MM-DD`. Months `YYYY/MM` with a leading zero. |
| Paths | Relative to the root of the share, with `/` as separator, without a leading `/`, never `.` or `..` as a component. Case-sensitive: `IMG_0955.JPG` and `img_0955.jpg` are different files, even though macOS doesn't allow both in one folder. |
| Content hash | SHA-256 of the complete file content, written as `sha256:` followed by 64 lowercase hexadecimal characters. This is the identity of an original. Chosen over blake2b because the NAS can recompute it itself with `sha256sum`. |
| Device name | `^[a-z][a-z0-9-]{1,31}$`, unique within `Lists/devices.json`. Examples: `mbp-peter`, `mbp-lynn`. |
| List id | `YYYY/<slug>` with slug `^[a-z0-9][a-z0-9-]{0,63}$`, derived from the display name (lowercase, accents stripped, everything that isn't a letter or digit becomes `-`, no double or trailing `-`). `2026/herfstwandeling`. |
| Writing | Every file that is written as a whole (manifest, summary, devices) MUST be written atomically: to a temporary file in the same folder, `fsync`, then rename. A reader never sees half a file. |
| Exclusions | These names are ignored by every component and never copied or put in a manifest: `@eaDir`, `#recycle`, `#snapshot`, `.DS_Store`, `._*`, `Thumbs.db`, `.TemporaryItems`, `.Spotlight-V100`, `.fseventsd`, `*.tmp`, `*.partial`. |
| Deleting | No component deletes anything on the NAS or on Drive, except what this contract explicitly names as a deletion step (none). Deleting locally is only done by `media thin` under the rules of section 10. |
| Overwriting | An existing original MUST NOT be overwritten, not even by a file with the same name. See 3.3. |

## 2. Shares on the NAS

| Share | Path on the NAS | Content | Who writes |
|---|---|---|---|
| `Library` | `/volume1/Library` | Originals and month manifests | laptops via `media push` |
| `Previews` | `/volume1/Previews` | Derived JPEGs, same path structure | laptops via `media previews push`, and FamilyLightbox when it makes a missing preview |
| `Lists` | `/volume1/Lists` | Devices, event logs, summaries | app and CLI, directly |
| `Exports` | `/volume1/Exports` | Output of collections | laptops via `media push` |

The NAS itself writes nowhere, except snapshots (invisible to this contract) and the `@eaDir` folders that DSM creates and that everyone ignores. A scheduled rclone task on the NAS (`media-drive-push.py`, see 8) reads `Library`, `Lists` and `Exports` and copies them to Google Drive under `Pictures/Library`, `Pictures/Lists`, `Pictures/Exports`; `Previews` doesn't go to Drive.

**Access per user.** `peter` and `lynn` have full rights on the NAS to `Library`, `Previews`, `Lists` and `Exports`. A separate share that mirrors Google Drive belongs to `peter` alone. What someone may not see is restricted on the NAS itself, never in the app: year folders in `Library` and `Previews` can carry an explicit ACL denial for a user, inherited by everything below them. A component that can't read a year folder skips it without an error. A new user gets the same denial if they should have the same restricted view. Files on the NAS must be readable by the group `users` (mode at least `0444`); otherwise rsync with `-a` copies the laptop's mode `0400` and only the owner can see them.

Each laptop has its own NAS user (`peter`, `lynn`) with read and write rights on all four shares. DSM 7 only lets members of the group `administrators` in over SSH, and an ordinary user gets no shell there. Therefore:

- `peter` is a member of `administrators` and uses rsync over SSH (key, no password). This account also does the one-off copy of the existing library and the checking of hashes on the NAS.
- `lynn` is an ordinary DSM user without SSH. Only FamilyLightbox runs on her laptop (architecture D13); it writes lists over SMB and only needs to push when she imports from a camera herself; then `push` goes through the SMB mount (section 8). No component MAY require an administrator account for ordinary use.
- Setting up the NAS (shares, snapshots, Cloud Sync) is done in DSM's web interface and is outside this contract.

## 3. Library

### 3.1 Path rules

```
<year>/<month>/<source>/<day>/<file>                   standard
<year>/<month>/<source>/<photos|videos>/<day>/<file>   sources with split (EOSR)
<year>/<month>/<source>/<file>                         iPhone sources (historical, no day folder)
<year>/<subfolder>/.manifest.json                      manifest of that year subfolder
```

- The folder directly below the year is called the **year subfolder**. From 2022 on it is always `<month>` = `MM`. Before 2022 the old layout exists: `MM.<name>` or `MM-DD.<name>` (for example `2012/07.Trouw`, `2007/04-07.Within Temptation`); the month is then the first two digits. Such folders are historical and read-only: nothing writes in them, nothing renames them, and below such a folder no source or day rules apply (`source` is `null` in the manifest). A year subfolder that matches neither form (for example `2012/scans Peter`, `2019/02.Ski.fcpbundle`) does get a manifest, with `month` `null`, and is on the clean-up list of phase 4.
- `<year>` is `YYYY`, `<month>` is `MM`, `<day>` is `YYYY-MM-DD` and MUST fall within that month.
- `<source>` matches `^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$`. Known sources: `EOSR` (split), `Hero13`, `Max2`, `Insta360-X5`, `flowtrack`, `DVD`, `extra`, and the iPhone sources `iPhone-photos`, `iPhone-videos`, `iPhone-photos-<Name>`, `iPhone-videos-<Name>`. A new source is simply a new name; only `split` and leaving out the day folder are deviations named in the configuration of `media`.
- Year and month come from the capture date (section 3.4), not from the import date.
- File names stay as the device gave them. Nothing renames an original, except the conflict rule in 3.3.

### 3.2 What an original is

Every file under a source that isn't excluded (section 1) and isn't a manifest. An original has a `kind`:

| kind | Extensions (case-insensitive) |
|---|---|
| `photo` | jpg jpeg heic heif png gif tif tiff webp bmp dng cr2 cr3 crw raf arw nef orf rw2 pef srw psd |
| `video` | mp4 mov m4v avi 3gp 3g2 mkv mts m2ts webm mpg mpeg wmv flv 360 insv lrv lrf |
| `sidecar` | xmp gpx thm sav json aae (belong to the file with the same stem in the same folder) |
| `other` | everything that's left; is put in the manifest, gets no preview |

Originals are never modified. Lightroom's editing metadata is in an `.xmp` sidecar, never in the original.

### 3.3 Name conflicts

If a file with the same name but a different hash already exists at the target path, the new file gets the suffix `~` plus the first eight hexadecimal characters of its hash before the extension: `IMG_0955~3f9a1c2e.JPG`. The same name with the same hash is not a conflict but a duplicate, and is skipped.

### 3.4 Capture date

In this order, the first that succeeds:

1. EXIF or QuickTime: `DateTimeOriginal`, `CreateDate`, `MediaCreateDate`, `TrackCreateDate`, `CreationDate` (exiftool on the laptop; on the NAS its own reader that reads the same from JPEG, TIFF-based RAW, CR3, HEIC and MP4/MOV, and falls through to step 2 for a format it doesn't know).
2. A date in the file name: `YYYY-MM-DD`, `YYYYMMDD` or `YYYY_MM_DD`, with a plausibility check (2000 up to the current year plus one).
3. The file's modification time.

Between step 1 and 2 one more rule applies: a file without its own date takes the date of the file with the same stem in the same folder, if that one has one (the clip of a Live Photo follows its photo this way). The manifest records which source was used (`taken_from`: `exif`, `partner`, `name` or `mtime`). EXIF times are clock times without a zone; the manifest writes them with the zone of the device that made the manifest.

### 3.5 The manifest per year subfolder

`<year>/<subfolder>/.manifest.json` describes every original in that year subfolder as it is on the NAS. For an ordinary month that is `<year>/<month>/.manifest.json`, called the month manifest. It is the only source for "this file is safely on the NAS".

```json
{
  "contract_version": "1.2",
  "folder": "2026/07",
  "month": "2026/07",
  "generated_at": "2026-10-21T22:14:03+02:00",
  "generated_by": "mbp-peter",
  "files": [
    {
      "path": "2026/07/EOSR/photos/2026-07-30/CO7A9123.CR3",
      "size": 25873419,
      "hash": "sha256:9f3a…",
      "kind": "photo",
      "source": "EOSR",
      "taken": "2026-07-30T16:58:12+02:00",
      "taken_from": "exif",
      "mtime": "2026-07-30T16:58:14+02:00",
      "sidecars": ["2026/07/EOSR/photos/2026-07-30/CO7A9123.xmp"]
    }
  ]
}
```

Rules:

- `files` is sorted by `path` (bytewise), so two manifests of the same content are identical.
- `folder` is the year subfolder (`<year>/<subfolder>`); `month` is `YYYY/MM` derived from the subfolder, or `null` if that isn't possible. For an ordinary month both are equal. A manifest without `folder` (versions 1.0 and 1.1) counts as `folder` = `month`.
- `path` is relative to the root of `Library` and so begins with `<year>/<subfolder>/`.
- `generated_by` is a device name (section 7) or `raven` if the NAS made the manifest itself (`media manifest nas`).
- `taken` is `null` if no source gave a date (then `taken_from` is `"none"` and the modification time decided the month folder).
- `sidecars` holds the sidecars of this file; sidecars don't appear as their own entry in `files`. A sidecar without a partner (same stem in the same folder) does get its own entry with `kind` `sidecar`, so it doesn't drop out of sight.
- `mtime` is informative; `size` and `hash` are normative.
- The manifest is rewritten by the device that last pushes to that year subfolder or makes its manifest, as a merge of the existing manifest on the NAS and the files known locally: per `path` the entry with the most recent `generated_at` origin wins; an entry for a path that no longer exists on the NAS is removed, never the file.
- A file only counts as "on the NAS" when (a) it is in the manifest, (b) the file exists on the NAS with the same `size`, and (c) for `media thin` in addition the hash has been recomputed in place or this device's rsync transfer with checksum succeeded (section 10).

## 4. Previews

Every `photo` and `video` has two derivatives in `Previews`, at the same relative path as the original, with the complete original file name plus a suffix:

```
Previews/2026/07/EOSR/photos/2026-07-30/CO7A9123.CR3.jpg      preview, long side 2048 px
Previews/2026/07/EOSR/photos/2026-07-30/CO7A9123.CR3.t.jpg    thumbnail, long side 400 px
Previews/2026/07/EOSR/videos/2026-07-30/CO7A9140.MP4.jpg      poster of a video
Previews/2026/07/EOSR/videos/2026-07-30/CO7A9140.MP4.t.jpg
```

The original extension stays in the name so that `IMG_1.HEIC` and `IMG_1.JPG` never get the same preview.

| Property | Rule |
|---|---|
| Format | JPEG, quality 85, sRGB, progressive not required. |
| Size | Long side 2048 and 400 px respectively; never upscale (a smaller original keeps its own size). |
| Orientation | Applied: the preview is upright, without an orientation tag. |
| Metadata | Only `DateTimeOriginal` (with `OffsetTimeOriginal`), equal to `taken` from the manifest, and `ColorSpace=sRGB`; no GPS, no camera settings, no orientation tag. |
| Source for RAW | The embedded JPEG from the RAW file, scaled, where it is at least as large as the preview (exiftool `-PreviewImage` or `-JpgFromRaw` in the CLI, ImageIO's thumbnail in the app); otherwise the RAW decoded by the system. No RAW development beyond that: no own settings or edits. |
| Source for video | The frame at 10 % of the duration, or at 1 s if the video is shorter than 10 s (half the duration under 2 s); the first frame if that one can't be read. Upright by the track's rotation. |
| Refreshing | A preview is made again when it is missing or when the hash of the original in the manifest changed. Existing previews are never rewritten for no reason. |
| Completeness | Every laptop SHOULD have the complete `Previews` tree locally; the app MUST work when previews are missing (it shows a placeholder). |

Previews are derived and can be rebuilt; they have no manifest and don't go to Drive.

They are made on a laptop, by the CLI (`media previews`) or by FamilyLightbox (feature 12, natively with ImageIO and AVFoundation), not on the NAS: DSM's ImageMagick and ffmpeg don't read CR3, HEIC or HEVC and don't write JPEG. The CLI uses `sips` (macOS, hardware decoding), `exiftool` (embedded JPEG from RAW, metadata) and `ffmpeg` (video frame). Which preview belongs to which hash is recorded in the local index (`~/.config/media/index.sqlite`, table `previews`, not part of the contract): a preview that already exists without an index entry is adopted, not made again.

## 5. Lists

### 5.1 Devices: `Lists/devices.json`

```json
{
  "contract_version": "1.0",
  "devices": [
    {"name": "mbp-peter", "owner": "peter", "label": "MacBook Pro Peter", "since": "2026-10-20T18:00:00+02:00"},
    {"name": "mbp-lynn",  "owner": "lynn",  "label": "MacBook Pro Lynn",  "since": "2026-10-25T10:00:00+02:00"}
  ]
}
```

- A device MUST be listed here before it may write events. FamilyLightbox adds its own device as soon as `Lists` is writable (a CLI command doing the same MAY as well): read, merge, write a temporary file `devices.json.tmp-<device>` in `Lists/`, check that `devices.json` didn't change in the meantime (otherwise merge again, at most three times), rename, and read back that its own device is in it. Other devices and unknown fields stay unchanged. The same name with a different `owner`, or a device with `retired`, is refused; a broken file is not touched.
- `label` is free text for people (FamilyLightbox uses the local host name without `.local`); `since` is the moment of registration in ISO 8601 with the local offset.
- `owner` is a short name of the person; the same `owner` may appear on several devices.
- A device is never removed; a device taken out of use gets `"retired": "<timestamp>"` and its event file keeps counting.

### 5.2 Events: `Lists/events/<device name>.jsonl`

One file per device, written only by that device, only by appending at the end. One event per line:

```json
{"v":1,"ts":"2026-10-20T19:42:11+02:00","dev":"mbp-lynn","who":"lynn","op":"create","list":"2026/herfstwandeling","name":"Herfstwandeling"}
{"v":1,"ts":"2026-10-20T19:42:30+02:00","dev":"mbp-lynn","who":"lynn","op":"add","list":"2026/herfstwandeling","photo":"sha256:9f3a…","path":"2026/10/EOSR/photos/2026-10-18/CO7A9410.CR3"}
{"v":1,"ts":"2026-10-20T21:10:40+02:00","dev":"mbp-peter","who":"peter","op":"remove","list":"2026/herfstwandeling","photo":"sha256:9f3a…"}
{"v":1,"ts":"2026-11-02T09:00:00+01:00","dev":"mbp-peter","who":"peter","op":"rename","list":"2026/herfstwandeling","name":"Herfstwandeling Ardennen"}
{"v":1,"ts":"2027-01-05T20:00:00+01:00","dev":"mbp-peter","who":"peter","op":"archive","list":"2026/herfstwandeling"}
```

| Field | Required | Meaning |
|---|---|---|
| `v` | yes | Version of the event format, now `1`. |
| `ts` | yes | Moment of the action on the device. |
| `dev` | yes | Device name; MUST equal the file name without `.jsonl`. A reader ignores lines where it doesn't. |
| `who` | yes | `owner` of the device at that moment. |
| `op` | yes | `create`, `add`, `remove`, `rename`, `archive`, `unarchive`. |
| `list` | yes | List id `YYYY/<slug>`. |
| `name` | with `create`, `rename` | Display name. |
| `photo` | with `add`, `remove` | Content hash of the original. |
| `path` | with `add` | Last known path in `Library`, as a readable hint; MAY be outdated. |
| `note` | no | Free text, at most 500 characters. |

Writing rules:

- A line MUST be written in one `write` call, followed by `fsync`, and end with `\n`. A reader ignores a last line without `\n` or with invalid JSON (half written).
- An `add` for a list that has no `create` counts as an implicit `create` with the slug as name.
- A `remove` of a photo that isn't in the list and an `add` of a photo that is already in it are allowed and have no effect; they stay in the history.
- The writer preferably writes directly to the file on the NAS. If the NAS is unreachable, it refuses (working away from home is not a requirement) and says so.

### 5.3 Merging

The state of all lists follows deterministically from all `events/*.jsonl` of registered devices:

1. Read all lines, discard invalid ones (see above).
2. Sort by `ts`, for equal `ts` by `dev`, for equal `dev` by line order in the file.
3. Process in that order: `create` makes the list (if it already exists, it works as `rename`); `add` puts the photo in it with `who` and `ts` as `added_by`/`added_at`; `remove` takes it out; `rename` changes the name; `archive`/`unarchive` set the flag.

The result is the same on every device. Two devices can therefore write at the same time without coordination.

### 5.4 Summaries: `Lists/<year>/<slug>.json`

Derived from 5.3, written by whoever merges last, atomically. Meant for people, for the Lightroom plug-in and as a quick starting point for the app.

```json
{
  "contract_version": "1.0",
  "list": "2026/herfstwandeling",
  "name": "Herfstwandeling Ardennen",
  "year": 2026,
  "archived": false,
  "created_by": "lynn",
  "created_at": "2026-10-20T19:42:11+02:00",
  "updated_at": "2026-11-02T09:00:00+01:00",
  "updated_by": "mbp-peter",
  "count": 23,
  "photos": [
    {"photo": "sha256:c71e…", "path": "2026/10/EOSR/photos/2026-10-18/CO7A9412.CR3", "added_by": "lynn", "added_at": "2026-10-20T19:43:02+02:00"}
  ]
}
```

- `photos` is sorted by `path`, then by `photo`.
- The slug of a new list (FamilyLightbox F003/S02): its name in lowercase, accents removed, every run of characters other than `a–z` and `0–9` one `-`, at most 60 characters, `lijst` when nothing is left; `-2`, `-3`, … when `<year>/<slug>` exists already. The id never changes after `create`; `rename` only changes the name.
- A summary is written through a temporary name `<slug>.json.tmp-<device>` and a rename.
- `path` is the path from the most recent manifest in which the hash appears; if the hash is in no manifest, the hint from the `add` event.
- A summary is never the source: when in doubt, merge again from the events.

### 5.5 Marks: stars and hearts, `Lists/marks/<device name>.jsonl`

Stars are shared: a photo has at most one rating, 1 to 5, and whoever rates it last sets it. A heart belongs to a person (`who`): each person's heart is on or off per photo, and everyone sees whose hearts are on it. Hiding is shared like stars: a photo is hidden for everyone, and whoever hides or unhides it last decides; nothing is deleted. A stack groups near-identical photos; a photo is in at most one stack, and the last `stack` or `unstack` of it decides. All refer to the photo by its hash, like lists (FamilyLightbox features 13 and 14).

One file per device, next to `events/`, written only by that device, only by appending, with the writing rules of 5.2 (one `write` and `fsync` per change, every line ending in `\n`, a reader ignores a half-written or invalid line and lines whose `dev` isn't the file's device). Only devices in `devices.json` count. They are separate from the list events on purpose: a reader of `events/` takes every line with a `list` field as a list event.

```json
{"v":1,"ts":"2026-10-30T10:15:02+01:00","dev":"mbp-lynn","who":"lynn","op":"rate","photo":"sha256:9f3a…","stars":4}
{"v":1,"ts":"2026-10-30T10:15:40+01:00","dev":"mbp-lynn","who":"lynn","op":"heart","photo":"sha256:9f3a…"}
{"v":1,"ts":"2026-10-30T21:02:11+01:00","dev":"mbp-peter","who":"peter","op":"rate","photo":"sha256:9f3a…","stars":0}
```

| Field | Required | Meaning |
|---|---|---|
| `v`, `ts`, `dev`, `who` | yes | As in 5.2. |
| `op` | yes | `rate`, `heart`, `unheart`, `hide`, `unhide`, `stack`, `unstack`. |
| `photo` | yes | Content hash of the original. |
| `stars` | with `rate` | 0 to 5; 0 removes the rating. |
| `stack` | with `stack` | The stack's id, made by the device that makes the stack: its name and the moment, `mbp-peter-20261030T211502123`. |
| `path` | no | Last known path in `Library`, as a hint. |

Merging: all lines of registered devices, sorted as in 5.3 (moment of `ts`, then `dev`, then line). For each photo the last `rate` gives its stars, with its `who` and `ts`; for each photo and `who` the last `heart` or `unheart` decides whether that person's heart is on it; for each photo the last `hide` or `unhide`, whoever wrote it, decides whether it is hidden; for each photo the last `stack` or `unstack` decides its stack. A stack is the photos whose last event names it; a stack of fewer than two photos counts as none. An `unheart` without a heart, or a rating of 0 without a rating, changes nothing.

## 6. Exports

```
Exports/<year>/<year>-<month>-<name>/            freely organised, mostly JPEGs
Exports/<year>/<year>-<month>-<name>/export.json  optional
```

`export.json`, if present:

```json
{"contract_version": "1.0", "name": "Kerstkaartje 2026", "from_lists": ["2026/herfstwandeling"], "lightroom_collection": "2026 Kerstkaartje", "exported_at": "2026-12-01T20:00:00+01:00", "by": "peter"}
```

Exports are end products: never rewritten automatically, but pushed and synced to Drive.

## 7. Local state on a laptop

```
~/.config/media/device.json      {"name": "mbp-peter", "owner": "peter"}  — fixed, independent of the host name
~/.config/media/state.json       last push/pull per month, last merge, NAS address
~/.config/media/index.sqlite     derived index of manifests, previews and lists; can be rebuilt with `media index`
~/.config/media/familylightbox-index.sqlite  the app's own index, the same tables as `index.sqlite`; only the app writes it
~/.config/media/familylightbox-previews.sqlite  which hash each preview was made from, and what failed; only the app writes it, not part of the contract
~/.config/media/familylightbox.json  the app's settings (NAS name, names of the shares); not part of the contract
~/.config/media/familylightbox-state.json  the app's own state (for example when the first previews pass was confirmed); not part of the contract
~/Pictures/LibraryLocal/         working copy of Library (partial)
~/Pictures/Previews/             copy of Previews (complete)
~/Pictures/Lists/                read copy of Lists; writing happens on the NAS
~/Pictures/Exports/              own exports, get pushed
```

- The working copy of `Library` follows the same path rules as the NAS; a path means the same file in both places.
- `index.sqlite` is not part of the contract (each component may build its own index), but MUST be rebuildable from the contract files without loss of information.
  The CLI's index (`media index`) has a table `folders` (folder, month, contract_version, generated_at, generated_by, n_files) and a table `items` (path, folder, month, size, hash, taken, taken_from, kind, source, sidecars as a JSON list, `local` = the original is local, `preview` = the preview is local), with indexes on month, hash and taken. The app may read it; only the CLI writes it.
  FamilyLightbox builds its own index from the same manifests and files into `familylightbox-index.sqlite`, with the same `items` and `folders` tables and indexes, and writes no other file's index. Neither component writes the other's file.
- `device.json` is created once (by the CLI or by FamilyLightbox at first start) and afterwards rewritten by no component.
- The existing `LibraryLocal/.media-state.json` is taken over into `state.json` at the first start.

## 8. Transport

| Flow | Mechanism |
|---|---|
| Laptop → NAS, bulk (`push`), with SSH | `rsync -a --partial --relative --chmod=Fgo+r,Dgo+rx` (the group must be able to read, section 2) with the exclusions of section 1, over SSH to `raven:/volume1/<share>/` (an SSH host alias, see below), per month (`./2026/07`), in two passes: first all originals with `--ignore-existing` (never overwrite, section 1), then only `*.xmp` without that flag (Lightroom sidecars may change). No `--delete`. Then read back the sizes on the NAS and write the manifest. |
| Laptop → NAS, bulk (`push`), without SSH | The same rsync to the SMB mount `/Volumes/<share>/` instead of `raven:`, with `--whole-file` (delta transfer makes no sense over SMB). Slower, but the same result and the same manifest. The CLI chooses this path if the target is a plain path instead of `host:path`. |
| NAS → laptop, lists (FamilyLightbox F003) | After the manifests, the app's sync copies `devices.json`, `events/*.jsonl`, `marks/*.jsonl` and the summaries `<year>/*.json` from `Lists` to `~/Pictures/Lists`, newer or changed files only, never deleting. The copy is for reading without the NAS; changes are written only on the NAS (5.2). |
| Laptop → NAS, previews by the app (FamilyLightbox F012) | Each preview the app makes is copied to the `Previews` share right away, thumbnail first, each through a temporary name `<name>.<process id>.tmp` and a rename, so another Mac never reads a half-written file. Before making one, the app looks on the share: one that is there already is copied instead. Nothing is deleted. |
| Laptop → NAS, previews (`previews push`) | `rsync -a --partial --relative --prune-empty-dirs`, only `*.jpg`, without `--ignore-existing` (a preview may be renewed when the original changed), no `--delete`, to `raven:/volume1/Previews/` per month. |
| NAS → laptop, bulk (`pull`) | The same rsync, in the other direction, in three passes per readable year folder (a year folder this user may not read is skipped silently): (1) all `.manifest.json`, also of years without originals, with `--update`; (2) all `*.jpg` from `Previews`, with `--update` (a newer local preview stays); (3) originals of the last 18 months with `--ignore-existing`, without camera video (`mp4 360 insv lrv thm`); later also the originals from lists. Beforehand a dry run with `--stats`: at least 20 GB must stay free after the pull. Then `media index`. |
| One original on demand | The SMB share, mounted as `/Volumes/Library`; the app opens the path directly. |
| Lists | Directly on the SMB share `/Volumes/Lists` (read and write), small files. |
| NAS → Drive | `rclone copy` per share, run on the NAS by `media-drive-push.py` (Task Scheduler, daily; by hand via `./media drive push`). Never delete on the Drive side (`copy`, not `sync`); never overwrite originals (`--ignore-existing`); only `*.xmp` and `.manifest.json` are updated; filter for `Library`: skip the extensions `lrv thm` (D11); skip `@eaDir`, `#recycle`, `#snapshot`. Cloud Sync can't do this: the existing mirror task on the Drive root rules out any other task on the same connection. |

The target is in `state.json` as `nas.library` (`raven:/volume1/Library` or a mount path), not in code. Both sides need rsync 3.1 or later; the NAS has 3.1.2 (hence no xxh3 checksums: md5 is enough), macOS only ships openrsync, so the laptop uses Homebrew's rsync. On DSM the rsync service must be on in addition to SSH, otherwise `rsync --server` refuses with "Permission denied" even though ssh logs in. For SSH the CLI only uses the host alias `raven` and leaves user, key and port to the SSH configuration (`~/.ssh/config`; the actual entry is in the private NAS note).

A `push` over SSH MUST succeed without interaction (`ssh -o BatchMode=yes raven true`); otherwise the CLI falls back to the SMB mount or stops with a clear message.

## 9. What "safe" means

An original is **on the NAS** if it meets 3.5. It is **on Drive** if it exists on Drive at `Pictures/Library/<path>` with the same `size` (to be established through the Drive mount or, as `media thin` does, with `rclone lsjson` on the NAS; Drive allows two files with the same name, one with the right size is enough), or if it is a file that, according to the filter, doesn't need to go to Drive. It is **safe** if it is on the NAS and on Drive. Since 2026-10-25 this applies to all photo and video; only `lrv` and `thm` (derived proxies and thumbnails) don't need to go to Drive.

## 10. `media thin`: rules for deleting locally

A local original MAY only be deleted if all of the following hold:

1. It is safe according to section 9, where the hash on the NAS has been recomputed for this file (`ssh raven sha256sum`, only possible from a device with SSH) or this device's push with checksum succeeded and the manifest hasn't changed since. A device without SSH MUST, after a push over SMB, read back the hash of the file on the mount before it counts as succeeded with checksum.
2. It is a `video` from a non-iPhone source, or it is older than 18 months (by `taken`, otherwise `mtime`).
3. The hash appears in no list that isn't archived.
4. The call was not a dry run, and the user has seen the dry run (the CLI requires `--yes` after a dry run in the same session, the app shows the list beforehand).

Sidecars follow their original and must have the same hash on the NAS. Previews and manifests always stay.

The dry run (`media thin`) writes its outcome as a plan to `~/.config/media/thin-plan.json`; `media thin --yes` only carries out that plan, refuses a plan older than 12 hours, reads the lists again and skips every file whose size or `mtime` changed since the dry run. "18 months" is the same calendar day 18 months back (31 August → 28 February). If the lists can't be read, `thin` deletes nothing. Every deletion is logged in `~/.config/media/thin.log` with path, hash, size and moment.

## 11. Example of one photo through the whole contract

```
original    Library/2026/10/EOSR/photos/2026-10-18/CO7A9410.CR3        sha256:9f3a…  25.9 MB
sidecar     Library/2026/10/EOSR/photos/2026-10-18/CO7A9410.xmp
manifest    Library/2026/10/.manifest.json                              entry with path, size, hash, taken
preview     Previews/2026/10/EOSR/photos/2026-10-18/CO7A9410.CR3.jpg    2048 px
thumbnail   Previews/2026/10/EOSR/photos/2026-10-18/CO7A9410.CR3.t.jpg  400 px
event       Lists/events/mbp-lynn.jsonl                                 add  sha256:9f3a…  2026/herfstwandeling
summary     Lists/2026/herfstwandeling.json                             photos[] contains sha256:9f3a…
Lightroom   collection "Lijsten / 2026 Herfstwandeling"                 points to ~/Pictures/LibraryLocal/2026/10/…/CO7A9410.CR3
export      Exports/2026/2026-12-Kerstkaartje/CO7A9410.jpg
Drive       Pictures/Library/2026/10/EOSR/photos/2026-10-18/CO7A9410.CR3  via rclone on the NAS
```

## 12. Changes

| Version | Date | What |
|---|---|---|
| 1.2 (addition) | 2026-10-30 | Section 5.5: `stack` and `unstack`, a photo's stack decided by its last event (FamilyLightbox F015/S01). |
| 1.2 (addition) | 2026-10-30 | Section 5.5: `hide` and `unhide`, shared per photo, the last one decides (FamilyLightbox F014/S01). |
| 1.2 (addition) | 2026-10-30 | Section 5.5: stars and hearts as events in `Lists/marks/<device>.jsonl`, merged like the lists; the app's sync copies them too (FamilyLightbox F013/S01). List events unchanged. |
| 1.2 (addition) | 2026-10-29 | Section 5.4: how FamilyLightbox makes the slug of a new list, and the temporary name of a summary (F003/S02). Event format unchanged. |
| 1.2 (addition) | 2026-10-29 | Section 8: the app's sync copies `Lists` to the local copy, for reading without the NAS (F003/S01). Section 5 unchanged. |
| 1.2 (addition) | 2026-10-28 | Sections 1 and 8: FamilyLightbox writes the previews it makes to the `Previews` share, through a temporary name and a rename (F012/S03). Manifest format unchanged. |
| 1.2 (addition) | 2026-10-28 | Section 7: `familylightbox-previews.sqlite`, the app's own record of which hash each preview was made from (F012/S02). Not part of the contract. |
| 1.2 (addition) | 2026-10-28 | Section 4: FamilyLightbox makes previews too (F012/S01, ImageIO and AVFoundation). The RAW rule allows the decoded RAW where the embedded JPEG is smaller than the preview; the video rule names the CLI's behavior under 2 s and the first-frame fallback. Manifest format unchanged. |
| 1.2 (addition) | 2026-10-27 | Section 7: `familylightbox-index.sqlite`, the app's own index with the CLI's `items` and `folders` tables, written only by the app (F008/S01). Not part of the contract. |
| 1.2 (addition) | 2026-10-26 | Section 7: `familylightbox-state.json`, the app's own state, kept apart from its settings (F007/S04). Not part of the contract. |
| 1.2 (addition) | 2026-10-26 | Moved into the FamilyLightbox repo and translated into English (F007/S01). Section numbers unchanged. Operational details of the home network moved to a private note in LibraryLocal (`docs/nas-inrichting.md`); the rules they support stay here (sections 2 and 8). Content of the rules unchanged, `contract_version` stays 1.2. |
| 1.0 | 2026-10-15 | First version. Hash is sha256 (the architecture document said blake2b; changed). |
| 1.2 (addition) | 2026-10-20 | `pull` made concrete (section 8: three passes, readable years, 18 months, space check) and the CLI's index tables described (section 7). |
| 1.2 (addition) | 2026-10-25 | Section 5.1: devices are registered by FamilyLightbox (F006/S04), with the way of writing and the refusal rules; `label` and `since` described. First `devices.json` on raven: `mbp-peter`. |
| 1.2 (addition) | 2026-10-25 | Section 7: `familylightbox.json` (the app's settings, not contract) and the rule that `device.json` is created once and never rewritten (FamilyLightbox F006/S01). |
| 1.2 (addition) | 2026-10-25 | Architecture D13: only FamilyLightbox runs on Lynn's laptop, and it implements everything she needs natively according to this contract; the CLI is a transition. The contract itself doesn't change. With FamilyLightbox feature 7 this document moves into that repo, in English; then the Dutch file becomes a pointer. |
| 1.2 (addition) | 2026-10-25 | D11 revised: camera video (`mp4 360 insv`) goes to Drive too; only `lrv thm` don't (sections 8 and 9). So `thin` only deletes camera video once it is on Drive too. `thin --videos` limits the dry run to video. Manifest format unchanged. |
| 1.2 (addition) | 2026-10-23 | `media thin` built (section 10): plan from the dry run, `--yes` within 12 hours, hash on the NAS with `sha256sum`, Drive check with `rclone lsjson` on the NAS (section 9), lists read again on execution. Manifest format unchanged. |
| 1.2 (addition) | 2026-10-20 | Access per user (section 2): restriction on the NAS through an ACL denial per year folder, files readable by the group. |
| 1.2 (addition) | 2026-10-19 | Previews (section 4) made concrete after the first implementation of `media previews`: made on the laptop with sips/exiftool/ffmpeg, metadata limited to `DateTimeOriginal`/`OffsetTimeOriginal` (= `taken`) and `ColorSpace`, transport `previews push` (section 8). Manifest format unchanged. |
| 1.2 (addition) | 2026-10-18 | Transport NAS → Drive is `rclone copy` on the NAS (`media-drive-push.py`, daily via the Task Scheduler) instead of Cloud Sync: the existing Cloud Sync mirror task on the Drive root allows no second task or connection (sections 2 and 8). Manifest format unchanged, `contract_version` stays 1.2. |
| 1.2 | 2026-10-17 | Year subfolder instead of month as the unit: historical folder names `MM.<naam>` and `MM-DD.<naam>` (2000–2021) allowed as read-only, a manifest per year subfolder with the new field `folder` and `month` possibly `null`; `generated_by` may be `raven`. Capture date on the NAS without exiftool: its own reader for EXIF in JPEG/TIFF-RAW, CR3, HEIC and QuickTime/MP4 (section 3.4). |
| 1.1 | 2026-10-16 | Only `peter` has SSH (DSM 7 limits SSH to `administrators`); `lynn` is an ordinary user, `push` without SSH goes through the SMB mount. The host alias `raven` in the SSH configuration set down. The rsync agreements made concrete after the first implementation of `media push`: two passes (originals `--ignore-existing`, `*.xmp` updated), no `--checksum-choice` (the NAS has rsync 3.1.2), the rsync service on DSM required. The values of `taken_from` named; orphan sidecars get their own entry. |
