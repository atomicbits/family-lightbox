# Feature 20: Changes without the NAS

**Status:** Done (2026-11-05)
**Depends on:** Feature 3 (lists), feature 13 (stars and hearts), feature 14 (hiding), feature 15
(stacks)
**Folder:** `F020-offline-changes`

## Goal

Lists, stars, hearts, hiding and stacks can be changed when the NAS can't be reached: on the
train, on holiday, or while the NAS sleeps or restarts. The change shows at once, as it does now,
and reaches the NAS by itself the next time it is connected. Today the app refuses: *lijsten
wijzigen kan alleen met de NAS*, *sterren en hartjes wijzigen kan alleen met de NAS*.

## Background

- Reading works without the NAS already: the sync copies `Lists` to the local copy (data contract
  section 8), and the app reads the lists and marks from there when the share isn't connected.
- Every change is an event appended to this device's own file on the share
  (`Lists/events/<device>.jsonl`, `Lists/marks/<device>.jsonl`, contract 5.2 and 5.5). The state
  follows from all events, sorted by their moment (5.3). A change made offline can therefore be
  written later with the moment it was made, and merges as if it had been written at once: no
  device has to coordinate.
- The contract says today, in 5.2: *if the NAS is unreachable, it refuses (working away from home
  is not a requirement)*. This feature changes that rule.

## Scope

### In scope

- **An outbox per device**, in the app's local state (`~/.config/media/outbox/`): one file for
  list events and one for marks, in exactly the format of 5.2 and 5.5, written with the same rules
  (one `write` and `fsync` per line). A change is written there when the share isn't connected or
  writing to it fails.
- **Reading with the outbox**: the lists and marks are the merge of the local copy and the outbox,
  so offline changes show, also after a restart.
- **Sending**: when the share is connected (at start, after connecting, on focus), the outbox's
  lines are appended to this device's files on the share, in order and unchanged, and removed from
  the outbox only after they are written and read back. Lines already on the share (an interrupted
  send) are not written twice.
- **A new list made offline** whose id the other Mac took meanwhile for another list (two lists
  `2026/vakantie`) gets the next free id when it is sent (`2026/vakantie-2`); its events in the
  outbox follow it. Without that the two lists would become one.
- **In the window**: no more refusal without the NAS; a line in the footer, *3 wijzigingen wachten
  op de NAS*, while the outbox isn't empty, and a message when sending fails.
- **The data contract**: 5.2 and 5.5 describe the outbox's behaviour (a device MAY write later
  with the original moment); the format of the events doesn't change, so older apps and the CLI
  read them as before.

### Out of scope

- **Importing, making previews, pushing** without the NAS: they write originals and manifests,
  not events; feature 18 keeps them for when the NAS is there.
- **Conflicts beyond the contract's rule**: the last change by its moment wins, as now. A star set
  offline on Monday loses against one set at home on Tuesday, also when it is sent on Wednesday.
- **The Lightroom plug-in** and the CLI writing offline.

## Acceptance criteria

- Without the NAS (the share ejected), Peter makes a list, adds photos, gives stars and a heart,
  hides a photo and makes a stack; all of it shows at once and after a restart of the app.
- Once the share is connected, the outbox is sent without a click; Lynn's Mac then shows the same
  as Peter's, and the outbox is empty.
- An outbox sent half (the app quits, the share goes away midway) is sent again without duplicate
  lines on the share.
- A list made offline under an id the other Mac used meanwhile ends up as its own list.
- With the share connected, changes go straight to the share as now; the outbox is only used when
  writing there fails.
- `cargo test` covers the outbox (writing, merging with the copy, sending, resending, the id taken
  meanwhile); `pnpm test` the footer's texts.

## Stories

| ID | Title | Status | Depends on |
|---|---|---|---|
| [S01](stories/S01-the-outbox.md) | The outbox: writing a change there when the share isn't connected, and reading the lists and marks with it | Done | — |
| [S02](stories/S02-sending.md) | Sending the outbox: on connect, in order, read back, without duplicates; the id of a list taken meanwhile | Done | S01 |
| [S03](stories/S03-in-the-window.md) | In the window: no refusal offline, *wijzigingen wachten op de NAS* in the footer, a message when sending fails | Done | S01 |
| [S04](stories/S04-data-contract.md) | The data contract: writing later with the original moment (5.2, 5.5), version 1.5 | Done | S02 |

## Decisions

Taken with Peter on 2026-11-04, as proposed:

1. **Also when the NAS answers but writing fails**: the change goes to the outbox, and the footer
   says it waits.
2. **The outbox** is in the app's local state (`~/.config/media/outbox/`), not in the local copy
   of `Lists`, which the sync overwrites.
3. **No extra warning** about the other Mac's changes while offline; they come at the next sync.
