# Story S01: The contract (1.7)

**Feature:** `F025-lightroom-edits`
**Status:** Done (2026-11-05)
**Depends on:** —

## Acceptance criteria

- Section 4: a photo edited in Lightroom has *renditions* next to its preview:
  `<path>.lr.jpg` (long side 2 048 px) and `<path>.lr.full.jpg` (full size); a virtual copy
  `<path>.lr-<copy>.jpg` and `.lr-<copy>.full.jpg`, `<copy>` the copy's name as a slug. Derived,
  made only by Lightroom (FamilyLightbox's plug-in), replaced when the edit changes; the time of the
  file is the time of the edit. The `.full.jpg` ones stay on the NAS (not copied to a Mac, not to
  Drive, like all of `Previews`).
- Section 5.5: a mark `variant` per photo (`""` the plain original, `lr`, `lr-<copy>`), the last
  one decides; without one, the variant edited last shows.
- Section 12: the row for 1.7. A reader that doesn't know renditions finds no `<path>.jpg` among
  them, so it isn't misled; it only sees more files in `Previews`.
- The previews' readers (sync, the previews check) skip `.lr` files where they look for
  `<path>.jpg` and `<path>.t.jpg`.

## Progress log

### [2026-11-05] Written

### [2026-11-05] Done

- Data contract 1.7: renditions in section 4 (names, the slug of a copy, JPEG quality 90, the
  time of the edit, replaced and removed, moved with a rename, the full ones only on the NAS);
  `variant` in 5.5 with its merge rule (the last one; else the latest rendition, else the plain
  original); the row in section 12.
- `pull::is_preview` leaves out `is_full_rendition` (`.lr.full.jpg`, `.lr-<copy>.full.jpg`), so
  the previews sync doesn't bring them to a Mac; the marks reader already skips an unknown `op`, so
  today's apps read 1.7 unchanged.
- The CLI's `media pull` excludes them too (LibraryLocal, a fix with its test).
- `cargo test` pull 12 passed (renditions in `is_preview`).
