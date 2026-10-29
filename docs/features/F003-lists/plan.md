# Feature 3: Lists: create and fill lists together

**Status:** In progress (plan accepted 2026-10-29; S01–S03 and S05 done; S04: the other Mac's changes to build)
**Depends on:** Feature 6 (the device is registered, `Lists` is connected), feature 8 (the index, with the hash of every photo)
**Folder:** `F003-lists`

## Goal

Peter and Lynn put the good photos of a year into shared lists, each around a subject (a
birthday, the summer holiday, a ski trip), several per year, each on their own Mac. Both add and
remove, also each other's choices, and each sees what the other did, with who and when. This is
what the whole project is for: the lists are the shared output (requirements F10–F13).

The app writes and reads the lists exactly as data contract 5 describes, so the CLI's `media thin`
(which keeps the originals of photos in lists) and later the Lightroom plug-in read the same lists.

## Background

- **Data contract 5** sets everything down already:
  - `Lists/devices.json`: the devices that may write; the app registers its own (F006/S04, done);
  - `Lists/events/<device>.jsonl`: one file per device, written only by that device, only by
    appending, one event per line (`create`, `add`, `remove`, `rename`, `archive`, `unarchive`);
  - merging: all events of registered devices, sorted by time, then device, then line; the
    result is the same on every Mac, and two Macs can write at the same time without
    coordination;
  - `Lists/<year>/<slug>.json`: a summary per list for people and the Lightroom plug-in, written
    by whoever merges last; never the source.
- **A photo is its hash** (architecture D6): a list holds hashes, with the last known path as a
  hint. Moving, renaming or thinning an original never breaks a list. The app's index has the
  hash of every item.
- **Today on the NAS**: `Lists/devices.json` with `mbp-peter`; no events yet.
- **Who reads lists already**: `media thin` merges the events (contract 5.3) to keep the
  originals of photos in lists. Nothing writes events yet.
- **The grid** (feature 1) shows months with a single selection (arrow keys, click) and a
  lightbox; there is no multiple selection yet.

## Scope

### In scope

- **Reading the lists**: the app merges the events by data contract 5.3, from the `Lists` share
  when it is connected, otherwise from the local copy `~/Pictures/Lists/` (data contract 7), which
  the sync of feature 7 keeps current. Without the NAS the lists can be looked at, not changed.
- **Writing**: every change is one event appended to this Mac's own file on the NAS, in one
  `write` followed by `fsync` (data contract 5.2). Without the NAS the app refuses and says so.
- **Creating a list**: a name and a year; the id is `<year>/<slug>`, the slug made from the name
  (lowercase, without accents, hyphens) and made unique within the year.
- **Renaming, archiving and unarchiving** a list. Archived lists are hidden unless asked for.
  Nothing is deleted, as in the contract.
- **Adding and removing** photos and videos, by anyone, also the other's choices: the one
  selected in the grid or the lightbox, and a range of them (see S05).
- **Seeing the lists**: per year in the sidebar, each with its count; a list opened as a grid of
  its photos across months, with who added each one and when.
- **Seeing the other's changes**: while the app is open at home, changes from the other Mac show
  up within about 15 seconds, without restarting.
- **In the grid**: a photo that is in the active list carries a mark.
- **Summaries** (`Lists/<year>/<slug>.json`): written after a change, for the lists that
  changed, atomically.

### Out of scope

- **Originals of lists on the laptop** (requirements B6, architecture D7): `pull` pass 3 brings
  them in the CLI; in the app this belongs with opening originals from the NAS (feature 9) or
  Peter's workflow (feature 10).
- **The Lightroom plug-in** (architecture D9) and **exports** (requirements F12): their own
  features later. They read the summaries this feature writes.
- **Personal or private lists**: by design there is one shared set (requirements, decision 1).
- **Notes** on events (`note`): the contract allows them; the UI leaves them out for now.
- **Working away from home**: not a requirement (data contract 5.2).

## Acceptance criteria

- Lynn creates "Herfstwandeling" for 2026 on her Mac and adds twenty photos; Peter sees the
  list with those twenty, marked as added by Lynn, within 15 seconds on his Mac, removes two and
  adds three; Lynn sees the result on hers.
- The events on the NAS follow data contract 5.2 to the letter (fields, one line per event,
  `dev` equal to the file name), and `media thin` on Peter's Mac sees the same photos in lists
  as the app.
- Two Macs adding and removing in the same list at the same moment end with the same state on
  both (merging by 5.3).
- A broken or half-written last line in someone's event file is ignored, not fatal; an event
  file of a device not in `devices.json` is ignored.
- Without the NAS the lists are shown from the local copy and every change is refused with a
  clear message; nothing is written locally.
- The summaries on the NAS match the merged state.
- `cargo test` covers the merge (the contract's examples, implicit `create`, duplicates, order on
  equal times, unregistered devices, broken lines), writing an event, the slug, and the
  summaries; `pnpm test` covers the list texts and the marks.

## Stories

| ID | Title | Status | Depends on |
|---|---|---|---|
| [S01](stories/S01-read-and-merge.md) | Read and merge the lists (contract 5.3), from the NAS or the local copy; the sync of feature 7 copies `Lists` | Done | — |
| [S02](stories/S02-write-events.md) | Write events to this Mac's own file on the NAS (create, add, remove, rename, archive); refuse without the NAS; the summaries | Done | S01 |
| [S03](stories/S03-lists-in-the-app.md) | Lists in the app: per year in the sidebar, create, rename, archive; the active list, adding and removing the selected photo, the marks | Done | S02 |
| [S04](stories/S04-a-list-and-the-other-mac.md) | A list as a grid, with who added what and when; the other Mac's changes within 15 seconds | In progress: the grid built | S03 |
| [S05](stories/S05-several-at-once.md) | Selecting several photos (shift-click, ⌘-click) and adding or removing them together | Done | S03 |

## Decisions

Peter asked on 2026-10-29 to start implementing the stories; the proposals below are taken as
accepted.


1. **How a photo goes into a list.** Proposal: an *active list*, chosen in the sidebar; then one
   key (`+`, or `=` without shift) or a button in the lightbox puts the selected photo in it, and
   the same again takes it out. That is one handling per photo (requirement F10), also while
   going through a weekend with the arrow keys. The alternative, a menu per photo with all lists,
   costs a click and a choice every time.
2. **Without the NAS.** Proposal: look, but don't change (as the contract says). The alternative,
   writing locally and sending later, the contract rules out on purpose: the event files would
   need merging across copies.
3. **Seeing the other's changes.** Proposal: while the window has focus, the app looks at the
   sizes of the event files on the NAS every 15 seconds, and reads only what was appended.
   Cheap over SMB, and the 15 seconds are the acceptance criterion.
4. **The summaries.** Proposal: the app writes them, a few seconds after the last change, only
   for the lists that changed, so the Lightroom plug-in and people looking on the NAS have them.
   The contract says "whoever merges last"; both Macs may write the same file, which is fine
   because it is derived and written atomically.
5. **The year of a new list.** Proposal: the year of the month on screen, changeable when
   creating. A list may hold photos of other years (a New Year's party); the year is where it is
   filed.
6. **Several at once (S05).** Proposal: as a separate story after the first three, so Lynn can
   start with one at a time. If she'd use the app mostly to go through a whole weekend at once,
   S05 moves up.
7. **When.** Proposal: after F012 is closed and v0.3.0 is out, so Lynn's first install
   (F004/S05) and the lists can come close together.
