# Feature 14: Hiding photos

**Status:** In design
**Depends on:** Feature 13 (the marks files, the right-click menu, the message with undo)
**Folder:** `F014-hide-photos`

## Goal

Among the iPhone photos are many practical ones: an information board, a price in a shop, a plant
we might buy. Peter and Lynn hide them, so browsing shows the photos that matter. Hiding is the
opposite of stars and hearts, and shared like them: a photo one of us hides is hidden for both.
Hidden photos are not shown, unless asked for one month or one year at a time, until the app
restarts.

Filtering, with an option to show hidden photos, comes later in its own feature.

## Background

- **Stars and hearts** (feature 13) are events in `Lists/marks/<device>.jsonl`, merged the same
  way on every Mac, read again within 15 seconds when the other Mac changes something. Hiding
  fits there: it is shared, per photo, and the last one wins.
- **The right-click menu** (F003/S05) acts on the picked photos or the one under the mouse; the
  **message with undo** (F013/S04) follows every change.
- **The grid** builds its months from the index: the month list's counts come from `months()`,
  the tiles from `items(month)`. Hidden photos have to drop out of both.

## Scope

### In scope

- **Hiding**: right-click on one photo or on the selection, *Verbergen*. Only from the menu: no
  key, no button.
- **Shared**: a `hide` or `unhide` event per photo in the marks files (data contract 5.5); the
  last one decides, whoever wrote it. The other Mac follows within 15 seconds.
- **Not shown**: a hidden photo is left out of the grid and out of the counts in the month list
  and the year rows.
- **Showing them for a while**: right-click on a month or a year in the month list, *Verborgen
  foto's tonen (12)*; they appear in their place, dimmed, with a mark. The same menu, *Verborgen
  foto's weer verbergen*, hides them again. Only on this Mac and only until the app restarts.
- **Unhiding**: right-click on a hidden photo that is shown, *Niet meer verbergen*.
- **The message with undo**, for hiding and unhiding, like stars and hearts: in the message, with
  ↶ and ⌘Z.

### Out of scope

- **Filters**, including "show hidden photos" everywhere: a later feature.
- **Deleting** photos: hiding never removes a file, on the laptop or the NAS.
- **A key** for hiding.

## Acceptance criteria

- Peter picks 20 practical photos in a month and hides them with the right-click menu; they
  disappear from the grid, the month's count drops by 20, and the message offers undo, which
  brings them back.
- On Lynn's Mac, with the window open, they disappear within 15 seconds.
- Right-click on that month, *Verborgen foto's tonen (20)*: they appear dimmed in their place;
  right-click on one of them, *Niet meer verbergen*: it stays, no longer dimmed, for both of us.
- After restarting the app, the shown hidden photos are hidden again.
- `cargo test` covers hide and unhide in the merge and the counts per month without the hidden
  photos; `pnpm test` covers the grid without them, showing a month or a year, and the texts.

## Stories (proposal)

| ID | Title | Status | Depends on |
|---|---|---|---|
| [S01](stories/S01-the-data.md) | The data: `hide` and `unhide` in the marks files and the contract, merged, written, read | Planned | — |
| [S02](stories/S02-not-shown.md) | Not shown: the grid and the counts without hidden photos | Planned | S01 |
| [S03](stories/S03-hiding.md) | Hiding and unhiding from the right-click menu, with the message and undo | Planned | S02 |
| [S04](stories/S04-showing-for-a-while.md) | Showing hidden photos per month or year from the month list, dimmed, until a restart | Planned | S03 |

## Open points

1. **The data.** Proposal: the ops `hide` and `unhide` in the same marks files as stars and
   hearts, so no new files and the same watch. The alternative, files of their own, buys nothing.
2. **Hidden photos in a list.** Proposal: a list shows all its photos, hidden ones too, dimmed
   with the mark: putting a photo in a list is a deliberate choice, and a list shouldn't seem to
   lose photos. The alternative is hiding them there too, with "2 verborgen" in the header.
3. **Stars and hearts on a hidden photo.** Proposal: they stay; hiding changes nothing else, and
   unhiding brings the photo back as it was.
4. **The counts.** Proposal: the month list and the year rows count only the visible photos; the
   tooltip says how many are hidden ("312 foto's, 20 verborgen"). A month where everything is
   hidden stays in the list with 0, so it can be shown.
5. **The menu on a month or year.** Proposal: *Verborgen foto's tonen (20)*, disabled when there
   are none; *Verborgen foto's weer verbergen* when they are shown. A shown month gets a small
   mark in the month list, so it is clear why dimmed photos appear.
6. **When.** Proposal: after F013 is checked, and before or with Lynn's first install, since
   hiding is most useful when both of us go through the iPhone photos.
