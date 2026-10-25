# Feature 1: Browse the library

**Status:** Done — unit-tested (5 cargo, 3 vitest); not yet used by Lynn, and no signed release yet (see F004)
**Depends on:** the `media` CLI's `index` and `previews` (LibraryLocal F006, F007)
**Folder:** `F001-browse-library`

Built before the features/stories process; this description was written afterwards for
completeness. The design it implements is in `~/Pictures/LibraryLocal/docs/datacontract.md`,
sections 4 (previews) and 7 (local state on a laptop).

## Goal

Peter or Lynn can open the app and browse the whole family library month by month on fast local
previews, look at one photo large, and get to the original when they need it.

## What exists

- **Rust core** (`src-tauri/src/lib.rs`, `db.rs`): opens `~/.config/media/index.sqlite` read-only
  and exposes the commands `status`, `months`, `items(month)`, `reveal(path)` and
  `open_original(path)`. Previews are served through a `preview://` protocol from
  `~/Pictures/Previews`; paths are checked so nothing outside the previews or library folder can be
  served or opened. Locations can be overridden with `MEDIA_CONFIG`, `MEDIA_PREVIEWS`,
  `MEDIA_LIBRARY`.
- **Month list** (`MonthList.tsx`): months grouped by year with photo, video and preview counts;
  folders without a month (for example `2012/scans Peter`) are shown by their name.
- **Grid** (`Grid.tsx`): a virtualized grid of thumbnails (`@tanstack/react-virtual`), sorted by
  capture time, sidecars left out. Arrow keys move the selection, space or Enter opens it.
- **Lightbox** (`Lightbox.tsx`): one photo large; arrow keys step, Escape or space closes.
- **Originals**: reveal in the Finder/Explorer, or open with the default application (via
  `tauri-plugin-opener`), when the original is on this laptop.
- Name, identifier `io.atomicbits.familylightbox`, public repo `atomicbits/family-lightbox`
  (2026-10-23).

## Acceptance criteria (as realized)

- Months count photos, videos and previews, and group month-less folders by folder name (cargo test).
- Items are sorted by capture time and skip sidecars (cargo test).
- A missing index gives a helpful error that says to run `./media index` (cargo test).
- Month labels are Dutch month names; a folder without a month shows its name (vitest).

## Still open

- Filter by source (camera, iPhone of Peter or Lynn): planned as F002.
- Originals that are not on this laptop (after `thin`) can't be opened yet; the app could fall back
  to the NAS mount.
- Code comments are still in Dutch; new code is commented in English.

## Stories

None: built before this process. Any change to this feature starts with a new story in `stories/`.
