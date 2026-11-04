# Feature 9: Fetching originals from the NAS

**Status:** In design
**Depends on:** Feature 6 (the NAS), feature 8 (the index), feature 3 (lists)
**Folder:** `F009-fetch-originals`

## Goal

The originals behind the previews in the grid come to this Mac on request: one photo, a
selection, or a whole list. They land in the working copy at their own path in the library, so
Lightroom Classic finds them where its catalog expects them (feature 23), and an export from the
app (feature 24) has them. Each Mac keeps only the recent months by itself; everything older is
on the NAS only.

## Scope

### In scope

- **What**: from the grid's right-click menu, *Originelen ophalen* for the photos picked; from a
  list, *Originelen ophalen* for all its photos; and the same command for features 23 and 24.
- **Where**: the working copy, at the original's path (`2019/07/iPhone-photos/IMG_1234.JPG`), with
  its sidecars (`.xmp`, `.AAE`, a video's `.THM` and `.LRV` only if asked), and the month's manifest
  from the NAS, so the index knows them as local.
- **Safely**: copied through a temporary name, the hash checked against the manifest, the time
  kept; never over a local file; a file already here with the right hash is skipped. In the
  background with its progress in the footer (percentage), resumable after a restart, like an
  import; the window may be closed.
- **Staying**: originals in a list that isn't archived stay, as `thin` keeps them already (data
  contract 10); others fetched by hand may be thinned again later as any old month.
- **Space**: the total size first, refused when it doesn't fit with the 20 GB reserve.
- **Opening an original** that isn't here (double-click, *Open in…*) offers to fetch it first.

### Out of scope

- Fetching whole months or years (that is `media pull`'s job, and later its own feature).
- Videos above a size without asking: the plan shows them, a click fetches them.

## Acceptance criteria

- A list of 40 photos from 2014 is fetched; afterwards they are in the working copy at their
  paths with their sidecars, verified, the index marks them local, and Lightroom shows them as
  found (after feature 23's catalog fix).
- An interrupted fetch continues to the same result; a file that changed on the NAS since the
  manifest is reported, not taken.
- `cargo test` for the plan (what is here already, sidecars, space), copying and verifying,
  resuming; `pnpm test` for the texts.

## Stories

| ID | Title | Status | Depends on |
|---|---|---|---|
| S01 | The plan: what to fetch, what is here already, sidecars, the space needed | Planned | — |
| S02 | Fetching: copy, verify, the month's manifest, resumable, in the footer | Planned | S01 |
| S03 | In the window: the right-click menu, a list's command, opening an original that isn't here | Planned | S02 |
