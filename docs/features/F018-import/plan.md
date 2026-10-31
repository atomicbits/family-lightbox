# Feature 18: Import

**Status:** In progress (plan accepted 2026-10-30)
**Depends on:** Feature 6 (the NAS is connected), feature 8 (the index, with the hash of every
original), feature 12 (making previews and putting them on the NAS), LibraryLocal F011 (the
camera in the manifest)
**Folder:** `F018-import`

## Goal

New photos and videos come into the library from the app: from a folder they are dropped in,
from a folder another app keeps in sync (a phone's photos on Google Drive, iCloud, Dropbox), or
from a camera's card. One button takes them all the way: into the right month and source on the
NAS, with their manifest and previews, verified, visible on every Mac. Peter does it on his Mac
instead of `media import` and `media push`; Lynn can read a card on hers with the same result
(requirement F4b, scenario 7), without scripts, exiftool or rsync (D13).

The app is open source: nothing in it assumes this family's folders, phones or cameras. What is
particular to this library is either a setting or a historical rule written down in the data
contract.

## Background

How it is done today, with the `media` CLI on Peter's Mac (in Dutch: LibraryLocal
`docs/huidige-workflow.md` and `README.md`):

| Path | How files arrive | What `media import` does |
|---|---|---|
| Offload folder | One subfolder per source: the Canon's tool writes day folders into `EOSR/`, cards are copied with the Finder into `Hero13/`, `Max2/`, downloads into their own folder. | Copies to `<year>/<month>/<source>/<day>/` (`EOSR` split into `photos/` and `videos/`), checks the size, then moves the original to `_imported/`. |
| Peter's iPhone | A sync app on the phone writes to a Drive folder per year and month. | Copies only, from the last 6 months, to `iPhone-photos` and `iPhone-videos`, without day folders. |
| Lynn's iPhone | The same, into her Drive, shared into Peter's. | The same, to `iPhone-photos-Lynn` and `iPhone-videos-Lynn`. |

Then `push` (copy to the NAS, read back, write the manifest), `previews` and the index are
separate commands. Particular to Peter's setup and kept in code today: the table of source
exceptions (`EOSR` split, one folder renamed, one skipped), the Drive folder names (`iPhone…Sync`),
the place of the offload folder. Weak spots: duplicates are recognized by name and size only; a
name conflict is left alone, while the contract (3.3) says to add `~<hash>`.

What the app can't do yet: read a capture date or a camera from a file, hash, write a manifest,
copy originals to the NAS.

How other photo managers lay out their files (Lightroom Classic's import, Immich's storage
template, PhotoPrism, Synology Photos' mobile backup): a date hierarchy, the camera as metadata,
and in multi-user setups one folder per person or stream above it. That last one is this
library's `<source>`: the way a photo came in (whose phone), which EXIF can't tell, and which
keeps file names unique (two iPhones both make `IMG_0955.JPG`).

## Scope

### In scope

- **Three kinds of source**, all generic:
  - a **drop folder**: each subfolder is a source; after an import verified on the NAS its
    originals move to `_imported/`, or, if the source's setting says so, are deleted;
  - a **synced folder**: kept by another app; the app never writes in it, copies only, and
    remembers what it took, so it doesn't look through years each time;
  - a **card or device**: a mounted volume with `DCIM/`; copied, never erased; the app offers to
    eject it afterwards.
- **One layout rule** for new sources: `<year>/<month>/<source>/<day>/<file>`. The two
  deviations of this library (`EOSR` with `photos/` and `videos/`; the iPhone sources without
  day folders) stay, for those sources only, as historical rules in the data contract, like the
  `MM.<name>` folders before 2022. There are no layout settings.
- **The source's name comes from the library.** For a card, the app looks in the index under
  which source the photos of that camera already are (`GoPro HERO13 Black` → `Hero13`) and
  proposes that; for a camera that is nowhere yet, a name from the model, to confirm or change.
  Once imported, the library remembers it for every Mac. No separate shared file.
- **Settings per source**, kept on this Mac: its place, exclusions (`*-mask.png`), "ignore",
  and for a drop folder "delete from `_imported/` once verified on the NAS" (off by default).
- **One job, in steps, resumable** (requirement N4):
  1. scan and plan: per source what is new, in which months, which dates don't come from the
     file itself, duplicates and conflicts;
  2. *Importeren*, always by a click;
  3. copy into a local staging area through a temporary name, hash, read the date and camera;
  4. duplicates **by hash** against the whole index, so the same photo by two ways (a card and
     Drive) or under another name comes in once; a name conflict gets `~<hash>` (contract 3.3);
  5. previews (feature 12), copy to the NAS, read back, write the month's manifest;
  6. only then the source is marked done, the staging copy removed unless this Mac keeps that
     month locally, and the index rebuilt.
- **The NAS is the target**, not the local folder. Neither Mac has a full working copy: Peter's
  local library keeps the last years and the months he puts there to work on, Lynn's maybe
  nothing. After an import the app keeps locally the months this Mac keeps (a setting, in line
  with the CLI's `pull`: the last 18 months).
- **Native**: the capture date and the camera are read in Rust (a port of the reader in
  `media-manifest-nas.py`, tested against the same files as exiftool), hashing in Rust, copying
  to the NAS over the SMB mount. No exiftool, rsync or Python.
- **Who imports what**: a source belongs to the Mac it is set up on. Peter imports both iPhones
  from their Drive folders on his Mac, as today.

### Out of scope

- **Importing over a cable** from an iPhone or camera (macOS ImageCaptureCore, Windows WPD): the
  most likely path for other users, later, as its own feature.
- **Driving the camera's own software**, or keeping macOS from claiming the camera during a
  download (today's helper scripts for the Canon): outside the app.
- **Converting** (HEIC to JPG, reframing 360° video): the originals stay as the device gave them.
- **Renaming** originals: file names stay as the device gave them, apart from the conflict rule.
- **`thin`**, **`doctor`** and the Drive upload status: other parts of feature 10.
- **Windows**: macOS first, as for the rest of the app; the design doesn't depend on it.

## Acceptance criteria

- Peter imports a Hero13 card's copy from the offload folder and both iPhones from Drive with
  the app; the files land at the same paths as `media import` would put them, each month on the
  NAS gets a manifest that `media` and `media-manifest-nas.py` accept, the previews are on the
  NAS, and the files are in the grid on both Macs after their next sync.
- Lynn reads a GoPro card on her Mac without anything else installed; the app proposes `Hero13`,
  the result is identical to Peter's import, nothing is erased from the card.
- A photo that is already in the library under another name or by another way is not imported a
  second time; a different file with the same name gets `~<hash>`.
- An import interrupted at any step (the app quits, the NAS goes away) leaves no half file on the
  NAS or in the library, and resumes or starts again cleanly; the source is never marked done
  before its files are verified on the NAS.
- A synced folder is never written to; a card is never written to.
- The dates and cameras read natively equal exiftool's on the test files of
  `test-manifest-nas.py`; `cargo test` covers the plan (layout rules, the historical sources,
  duplicates, conflicts), the steps and resuming; `pnpm test` covers the screen's texts.

## Stories

| ID | Title | Status | Depends on |
|---|---|---|---|
| [S01](stories/S01-date-camera-and-hash.md) | Reading the capture date and the camera natively, and hashing (a port of the NAS script's reader, tested against exiftool) | Done | — |
| [S02](stories/S02-manifest-and-push.md) | Writing a month's manifest and copying originals to the NAS over SMB, verified (the app's `push`); the data contract's transport and layout rules | Done | S01 |
| [S03](stories/S03-sources.md) | Sources: the drop folder's subfolders, synced folders, cards; their settings; the name proposed from the library | Built | — |
| [S04](stories/S04-the-plan.md) | The plan and the import screen: what is new per source, months, dates, duplicates by hash, conflicts | Built | S01, S03 |
| [S05](stories/S05-drop-folder.md) | Importing from a drop folder: staging, previews, NAS, verify, `_imported/` (or delete), what stays local, the index; resumable | Planned | S02, S04 |
| [S06](stories/S06-synced-folders.md) | Synced folders: copy only, remember what was taken, the iPhone sources' historical layout | Planned | S05 |
| [S07](stories/S07-cards.md) | Cards and devices: `DCIM/` volumes, never erased, eject afterwards | Planned | S05 |

## Decisions

Taken with Peter on 2026-10-30, while discussing this plan:

1. **The NAS is the target, with a local staging area on every Mac.** Neither Mac keeps a full
   working copy; what stays local afterwards is the months this Mac keeps.
2. **Peter imports both iPhones** from their Drive folders on his Mac, as today; a synced folder
   belongs to one Mac.
3. **Emptying `_imported/`** after verification on the NAS is a setting per source, on or off.
4. **`<year>/<month>/<source>/` stays**, because the source tells which way a photo came in and
   keeps names unique; the camera is metadata (F011) and never a folder.
5. **New sources always get a day folder.** No layout settings; the existing deviations (`EOSR`
   split, iPhone sources without day folders) are historical rules for those sources.
6. **No shared file of sources.** A source's name comes from the library itself (which camera's
   photos are under which source), so it can't drift from what is on the NAS.

Accepted by Peter on 2026-10-30 as proposed:

7. **Remembering what a synced folder gave.** Proposal: per file its path, size and modification
   time with the hash it had, in the app's local state; a file that is no longer in the synced
   folder is forgotten. The alternative, looking only at the last months as the CLI does, misses
   a photo the phone uploads late.
8. **Where the staging area is.** Proposal: inside the local library folder, under
   `.import/<job>/`, so a move to its final local place is a rename, and an interrupted job is
   found again at start.
9. **During the transition.** Proposal: the CLI's `import` and `push` keep working unchanged
   until S06 works on real data; then `media import` says to use the app. Both follow the same
   layout and manifest, so a month imported by one is read by the other.
