# Feature 13: Ratings and hearts, with undo

**Status:** In progress (plan accepted 2026-10-30; S01 done, S02 and S03 built)
**Depends on:** Feature 3 (lists: the shared event files on the NAS, the selection of several photos, the 15-second watch)
**Folder:** `F013-ratings-and-hearts`

## Goal

Peter and Lynn give photos a rating of one to five stars, shared by everyone, and each puts a
heart on the photos they love: a heart belongs to the person, and everyone who can see the photo
sees it. Both are always visible on the tiles, scale with them, and follow the other Mac within
seconds. Every such change can be undone, from a message that explains what happened, from a
button that is always there, and with ⌘Z (Ctrl-Z on Windows).

Filtering on stars and hearts comes later, in its own feature; this one makes them and shows
them.

## Background

- **Lists** (feature 3) already solve the hard part: every device appends events to its own file
  on the NAS, every reader merges them the same way, and two Macs can write at the same moment
  without coordination (data contract 5). The app checks the files every 15 seconds while its
  window has focus, and reads again when one changed.
- **A photo is its hash** (architecture D6): stars and hearts stay with the photo when it is
  moved, renamed or thinned, and a photo that is in two places has the same stars and hearts in
  both.
- **The selection** (F003/S05): one photo, or several with shift and ⌘, also across months; the
  grid and the list view work the same way.
- **The tiles** show today: the "video" badge bottom left and the "NAS" badge top right (the
  original is only on the NAS).
- **`media thin`** reads `Lists/events/*.jsonl` and treats every line with a `list` field as a
  list event; ratings and hearts must not go into those files.

## Scope

### In scope

- **Stars**: 1 to 5 per photo, shared: whoever sets them last wins, as in a list. Removing the
  rating is possible (back to no stars).
- **Hearts**: per person (the device's `owner`), on or off per photo; a photo shows how many
  people put one on it.
- **Where they are kept**: new event files on the `Lists` share, one per device, appended only,
  merged like the lists (sorted by time, then device, then line). A new section in data
  contract 5. Without the NAS: look, don't change, as for lists.
- **Setting them**: with one or more photos selected, the header shows five stars and a heart,
  with how many photos they apply to ("3 foto's"). Also in the lightbox, and with keys.
- **On the tiles**: stars bottom right, the heart top right with a small number when more than
  one person put one on; both scale with the tile size. The "NAS" badge moves to the top left.
- **The other Mac**: changes show within 15 seconds while the window has focus, like the lists.
- **Undo**:
  - a message at the bottom after every change ("3 foto's: 4 sterren", "hartje weg bij 1
    foto"), with *Ongedaan maken*, disappearing after a few seconds;
  - an undo button that is always in the header, greyed out when there is nothing to undo;
  - ⌘Z (Ctrl-Z on Windows) does the same.

### Out of scope

- **Filtering** on stars or hearts: a later feature.
- **Lightroom's own ratings** in the `.xmp` sidecars: not read and not written.
- **Redo** (⇧⌘Z).
- **Comments** or other reactions.

## Acceptance criteria

- Lynn gives 12 photos 4 stars and hearts 3 of them; on Peter's Mac, with the window open, the
  stars and hearts appear within 15 seconds. Peter hearts one of those 3: it shows a heart with
  "2".
- With 12 photos selected, the header says it applies to 12 photos; one click on the third star
  gives all 12 three stars; the message says so and *Ongedaan maken* puts each of the 12 back to
  what it had before, which may differ per photo.
- ⌘Z after closing the message does the same; with nothing left to undo the button is grey and
  ⌘Z does nothing.
- Stars and hearts are visible on the tiles at the smallest and the largest tile size, in the
  library and in a list.
- The new events follow the new section of the data contract; `media thin` still reads the lists
  as before.
- `cargo test` covers the merge (last rating wins, hearts per person, removing, order on equal
  times) and the writing; `pnpm test` covers the header texts, the messages and the undo stack.

## Stories

| ID | Title | Status | Depends on |
|---|---|---|---|
| [S01](stories/S01-the-data.md) | The data: the contract section, writing and merging rating and heart events, reading them with the lists and the 15-second watch | Done | — |
| [S02](stories/S02-showing-them.md) | Showing them: stars and hearts on the tiles and in the lightbox, scaled with the tile | In progress: built, to look at with real marks | S01 |
| [S03](stories/S03-setting-them.md) | Setting them: the stars and heart in the header for the selection, keys, the lightbox | In progress: built, to try in the app | S02 |
| [S04](stories/S04-undo.md) | Undo: the message with *Ongedaan maken*, the button in the header, ⌘Z; list adds and removes too | Planned | S03 |

## Decisions

Peter accepted proposals 1–9 on 2026-10-30. Point 10, whether Lynn's first install comes before
or after this feature, is his call when the time comes.


1. **Where the events go.** Proposal: their own files, `Lists/marks/<device>.jsonl`, next to
   `Lists/events/`, on the same share (both of you can write there), with ops `rate`
   (`stars` 0–5, 0 removes the rating) and `heart` / `unheart`. The alternative, new ops in the
   list event files, would mislead `media thin`, which takes every line with a `list` field as a
   list event.
2. **Removing a rating.** Proposal: clicking the star the photo already has removes the rating;
   `0` on the keyboard does the same. With several photos that differ, a click sets them all.
3. **Stars on a tile.** Proposal: only the filled stars ("★★★" for 3), small, on a dark
   translucent strip, so an unrated photo shows nothing and the tiles stay calm. The alternative
   is always five stars, filled and empty.
4. **The heart on a tile.** Proposal: red when you put one on it yourself, white when only the
   other did; the number appears from 2 on. The tooltip names who.
5. **Keys.** Proposal: `1`–`5` set the stars and `0` removes them, `H` toggles your heart, for the
   selection, in the grid and the lightbox.
6. **The selection with mixed values.** Proposal: the header shows the stars all selected photos
   share, or a dimmed "–" when they differ; the heart is filled when you put one on all of them,
   half-filled when on some. A click on the heart puts yours on all of them, or takes it off all
   when it was on all.
7. **What undo covers.** Proposal: every change made in this window since it opened, step by
   step, newest first: stars, hearts, and also adding to and removing from lists (cheap now, and
   just as easy to do by accident when dragging). Not creating, renaming or archiving lists.
   The history is gone when the app restarts.
8. **Undo after the other Mac changed the same photo.** Proposal: undo only puts back photos
   whose value is still the one you set; the others are left alone and the message says so ("1
   foto intussen door Lynn gewijzigd, niet teruggezet").
9. **Where the undo button goes.** Proposal: in the header, at the right, next to the settings
   button, as ↶, with a tooltip saying what it would undo ("Ongedaan maken: 4 sterren op 12
   foto's").
10. **When.** Proposal: before Lynn's first install (F004/S05)? Or install first, so she can
    already make lists, and ratings follow with an update (which tests the updater once more)?
