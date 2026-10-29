# Feature 15: Stacks (reeksen)

**Status:** In design
**Depends on:** Feature 13 (marks: stars and hearts, the message with undo), feature 14 (the right-click menu on photos, hidden photos)
**Folder:** `F015-stacks`

## Goal

A photo shoot gives dozens of nearly identical photos of the same person or group. Peter and Lynn
put such photos together in a stack (*reeks*), which the grid shows folded: only the photos that
matter (those with a heart, or 4 or 5 stars), or else the first one. A click on the stack's
symbol unfolds it to show all its photos, together, even when other devices' photos were taken
in between; another click folds it again. Folding is only on this Mac and only until the app
restarts; the stacks themselves are shared, like stars and hearts.

## Background

- **Marks** (feature 13, data contract 5.5): stars, hearts and hidden photos are events in
  `Lists/marks/<device>.jsonl`, merged the same way on every Mac and followed within 15 seconds.
  A photo's place in a stack fits there.
- **The grid** shows a month's items by date taken, in rows of tiles, one month after another
  (feature 1); every row is as wide as the window allows.
- **Several devices** fill the same month: Peter's camera, two iPhones. Photos of a shoot can be
  interleaved with the other devices' photos of that moment.
- **The selection** (F003/S05), the **right-click menu** (F014) and **undo** (F013/S04) are in
  place.

## Scope

### In scope

- **Making a stack** from two or more selected photos that aren't in a stack yet.
- **Adding** to a stack: with one or more photos of one stack selected and photos outside any
  stack, right-click, *Aan de reeks toevoegen*. Not possible when photos of two stacks are
  selected.
- **Taking out**: right-click on a photo in a stack, *Uit de reeks halen*; a stack with one photo
  left is no stack any more.
- **Shared**: stacks are kept in the marks files, followed by the other Mac within 15 seconds.
- **Folded by default**: the stack shows the photos with a heart or 4–5 stars, or else its first
  photo (by date taken); together, at the place of the stack's first photo.
- **Unfolded**: all photos of the stack, together, in the order taken; folding and unfolding is
  local and forgotten when the app restarts.
- **Looks**: a colored border and a light background of the same color around the stack's tiles;
  stronger when a photo of the stack is selected, and then the stack symbol (two overlapping
  photos) in that color at the top right of the stack's last shown tile, which folds or unfolds
  it.
- **The message with undo** for making, adding, taking out.

### Out of scope

- **Finding stacks automatically** (by time or by likeness): perhaps later.
- **A stack spanning several months as a block in each**: a stack lives in the month of its first
  photo.
- **Exporting** or handing stacks to Lightroom.
- **Filters**: a later feature.

## Acceptance criteria

- Peter selects 40 photos of a shoot, interleaved with 10 of Lynn's iPhone, and makes a stack;
  the 40 fold to their first photo, at the place of the first, and Lynn's 10 stay where they
  were.
- He gives 3 of them a heart; folded, the stack shows those 3, side by side, with the border.
- Selecting one of them shows the symbol; a click unfolds all 40 together, in the order taken;
  a click folds them again. After a restart the stack is folded.
- With one photo of the stack and 5 photos outside it selected, *Aan de reeks toevoegen* adds the
  5; *Uit de reeks halen* on one takes it out. Each with the message and undo.
- On Lynn's Mac the stack appears folded within 15 seconds.
- `cargo test` covers the merge of stack events; `pnpm test` covers the order of a month with
  stacks folded and unfolded, which photos a folded stack shows, the menu items for a selection,
  and undo.

## Stories (proposal)

| ID | Title | Status | Depends on |
|---|---|---|---|
| [S01](stories/S01-the-data.md) | The data: `stack` and `unstack` in the marks files and the contract, merged, written, read | Planned | — |
| [S02](stories/S02-the-grid.md) | The grid: a month's photos with stacks together, folded or unfolded; which photos a folded stack shows; the counts | Planned | S01 |
| [S03](stories/S03-the-looks.md) | The looks: border, background, the stronger border when selected, the symbol that folds and unfolds | Planned | S02 |
| [S04](stories/S04-making-and-changing.md) | Making, adding to and taking out of a stack from the right-click menu, with the message and undo | Planned | S03 |

## Open points

1. **The data.** Proposal: in the marks files, `{"op":"stack","photo":…,"stack":"<id>"}` puts a
   photo in a stack and `{"op":"unstack","photo":…}` takes it out; per photo the last one
   decides. A stack's id is made by the device that makes it (its name and the moment), so two
   Macs never pick the same one. A stack is the photos whose last event names its id; one photo
   left is no stack.
2. **Making a stack.** Proposal: right-click on two or more selected photos, *Reeks maken*; not
   offered when one of them is in a stack already (then *Aan de reeks toevoegen* applies).
3. **The first photo.** Proposal: the earliest taken; with the same time, the first by path.
4. **Which photos a folded stack shows.** As asked: those with any heart or with 4 or 5 stars, in
   the order taken; none of those: the first photo.
5. **Where the stack stands.** Proposal: at the place of its first photo in that month; a stack
   with photos in two months (a shoot around midnight) stands in the month of its first photo,
   with all its photos.
6. **Counts.** Proposal: the month list keeps counting photos, not tiles: a stack hides nothing,
   it only takes less room. The tooltip mentions the stacks ("312 foto's, 2 reeksen").
7. **In a list.** Proposal: a list shows every photo on its own, without folding; a tile in a
   stack keeps a thin border of the stack's color, so it is recognizable.
8. **Hidden photos in a stack** (feature 14): hidden stays hidden; folded or unfolded, a hidden
   photo is left out unless its month shows hidden photos.
9. **Color.** Proposal: a small palette of soft colors, chosen from the stack's id, so the same
   stack always has the same color and two neighbouring stacks rarely share one.
10. **Selection and keys.** Proposal: the arrow keys go through the shown tiles, folded or not;
    stars, hearts, dragging to a list and hiding act on the selected tiles only, never on the
    photos folded away. Folding and unfolding has no key; the symbol does it (and the menu:
    *Reeks openklappen* / *dichtklappen*).
11. **Dissolving.** Proposal: right-click in a stack also offers *Reeks opheffen*, which takes
    all its photos out at once (with undo).
