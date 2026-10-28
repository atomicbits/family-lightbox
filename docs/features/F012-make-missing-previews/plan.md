# Feature 12: Make missing previews in the app

**Status:** In progress (plan accepted 2026-10-28, before feature 3; S01 and S02 done)
**Depends on:** Feature 6 (the NAS shares), feature 7 (sync), feature 8 (the index knows which previews are missing)
**Folder:** `F012-make-missing-previews`

## Goal

When a photo or video has no preview, the app makes one, natively, without the CLI. It does that
for originals on this Mac and for originals that are only on the NAS, and it puts the new previews
on the NAS too, so every other Mac gets them with its next sync. A gap in the previews then heals
itself instead of staying a grey placeholder until someone notices it.

This is the first part of Peter's workflow (feature 10) to move into the app: making previews.
The CLI's `media previews` gets no new features; it stays as it is until this feature replaces it.

## Background

- **The gap found on 2026-10-28.** 85 874 photos and videos of 2000–2021 have no preview
  anywhere (the CLI counted 40 838, because it skips the historical folder names; see S02): not on Peter's Mac and not on the NAS (`Previews` holds 2022–2026 only). All
  their originals are on the NAS and readable over SMB.
- **Why.** `media previews` makes a preview only when the original is in the local working copy
  (architecture decision D4: previews are made on the laptop at import). The old years were never
  imported through the laptop with the new CLI, so they never got previews. The app syncs previews
  from the NAS (feature 7) and finds none for those years.
- **What the contract asks** (data contract 4): JPEG quality 85, sRGB, long side 2048 and 400 px,
  never upscaled, upright without an orientation tag, only `DateTimeOriginal`/`OffsetTimeOriginal`
  (= `taken`) and `ColorSpace`; RAW from its embedded JPEG; video at 10 % of the duration (1 s
  under 10 s); made again when missing or when the hash in the manifest changed.
- **What the app already has.** The index (feature 8) has a `preview` flag per item, so the
  missing ones are one query. The app knows the `Library` and `Previews` mounts (feature 6) and
  copies previews from the NAS (feature 7). The grid shows a placeholder for a missing preview.
- **macOS does the decoding.** ImageIO reads JPEG, HEIC, PNG, TIFF and the RAW formats of our
  cameras (CR2, CR3, DNG) and gives an upright, scaled image or the embedded preview; AVFoundation
  gives a frame of a video (MP4, MOV, HEVC). Both are part of macOS, so Lynn's Mac needs nothing
  extra (D13). The CLI uses `sips`, `exiftool` and `ffmpeg` for the same.

## Scope

### In scope

- **Finding what is missing**: every `photo` and `video` in the index without both a preview and a
  thumbnail locally, and, when the NAS is connected, not on the NAS either (then it is only copied,
  not made).
- **Making them natively** with ImageIO and AVFoundation, by the rules of data contract 4.
- **From where the original is**: the local working copy first, else the `Library` share on the
  NAS. An original that can't be read (not local, NAS not connected, or a year this user may not
  read) is skipped without an error and tried again later.
- **To where**: the local previews folder, and the `Previews` share on the NAS when it is
  connected, written to a temporary name and renamed, so a half-written preview is never seen and
  two Macs making the same preview at the same time don't harm each other.
- **Refreshing on a changed hash**: the app records which hash each preview was made from, in its
  own file, and makes it again when the manifest's hash differs. A preview that exists without a
  record is adopted, as the CLI does.
- **In the background**, after a sync, at a low priority and with a few at a time, so browsing
  stays smooth; the month on screen goes first. A line in the window shows the progress
  ("Previews maken: 1 204 / 85 874"), with a way to pause it. It continues after a restart where
  it left off.
- **The grid** shows a new preview as soon as it is made, without a reload.
- **Failures** (a damaged file, a format macOS can't read) are counted, listed in the settings,
  and not retried on every start, only when the hash changes or by hand.

### Out of scope

- **Making manifests, hashing, importing**: other parts of feature 10.
- **Changing `media previews`** in the CLI. Its local `index.sqlite` table `previews` is not read or
  written; the app keeps its own record.
- **Previews on the NAS itself**: DSM can't decode CR3, HEIC or HEVC (data contract 4).
- **Windows**: ImageIO and AVFoundation are macOS. Windows needs its own decoders later.
- **RAW development**: as today, only the embedded JPEG.

## Acceptance criteria

- On Peter's Mac, the 85 874 missing previews of 2000–2021 are made by the app, appear in the
  grid while it runs, and end up on the NAS; another Mac gets them with its next sync.
- A preview made by the app follows data contract 4: size, quality, sRGB, upright without an
  orientation tag, only the allowed metadata. Checked against previews from `media previews` on the
  same originals (JPEG, HEIC, CR2, CR3, a portrait photo, MP4, MOV), allowing for small pixel
  differences between the two encoders.
- Browsing stays smooth while it runs; pausing stops it within a few seconds; quitting and
  restarting continues where it left off.
- An original whose hash changed in its manifest gets a new preview; an unchanged one is never made
  again.
- A year this user may not read, a disconnected NAS or a damaged file gives no error dialog and
  doesn't stop the rest.
- `cargo test` covers the planning (missing, on the NAS already, hash changed, adopted, skipped)
  on temporary folders, and the making on a small set of real sample files.

## Stories

| ID | Title | Status | Depends on |
|---|---|---|---|
| [S01](stories/S01-make-one-preview.md) | Make one preview and thumbnail natively (ImageIO, AVFoundation) by data contract 4, compared with the CLI's | Done | — |
| [S02](stories/S02-what-is-missing.md) | The plan: what is missing or outdated, the app's own record of hashes, adopting existing previews | Done | S01 |
| [S03](stories/S03-background-job.md) | The background job: from the local copy or the NAS, to the local folder and the NAS, the progress line, pause, resume | Planned | S02 |
| [S04](stories/S04-grid-and-failures.md) | The grid picks up new previews as they are made; failures in the settings | Planned | S03 |

## Decisions

Peter accepted all five proposals on 2026-10-28, with this feature before feature 3, and the
missing previews of today waiting for the app (no run of `media previews` on the NAS mount).


1. **Which Macs make previews.** Proposal: every Mac running the app, Lynn's too. The previews
   land on the NAS, so whichever Mac gets there first does the work for both. The alternative is
   Peter's Mac only, with a setting.
2. **Writing to the NAS.** Proposal: the app writes new previews straight to the `Previews` share
   (both accounts may write there, data contract 1). This changes data contract 1 and 4 ("made on a
   laptop with `media previews`, pushed with `previews push`") to "made by the CLI or the app". The
   alternative, local only, would leave Lynn's Mac without the previews Peter's Mac made.
3. **RAW through ImageIO.** ImageIO's thumbnail of a RAW file uses the embedded JPEG when it is
   large enough, and otherwise decodes the RAW. Proposal: accept both, and loosen "the embedded
   JPEG" in the contract to "the embedded JPEG where it is large enough". Our EOS R and older Canon
   RAWs embed a full-size JPEG, so in practice nothing changes.
4. **The 85 874 of today.** The app will take a few hours for them over the network once S03 is
   in. Proposal: wait for the app rather than run `media previews` on the NAS mount now, so the
   first real run is this feature's. If you'd rather have them sooner, the CLI can do it today
   (`MEDIA_LIBRARY=/Volumes/Library ./media previews`) without any change to it.
5. **Where it fits.** Proposal: right after F003 (lists), or before it if the gap bothers you
   while browsing the old years.
