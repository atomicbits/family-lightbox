# Story S02: Rendering in Lightroom

**Feature:** `F025-lightroom-edits`
**Status:** Planned
**Depends on:** S01

## Acceptance criteria

- *Lijsten bijwerken* goes on, after the lists, with the photos in the lists' collections that
  have edits (develop adjustments or a crop) and the virtual copies among them.
- Per photo whose last edit is newer than what it rendered before: two JPEGs through
  `LrExportSession` (sRGB, quality 90; 2 048 px long side, and full size), into
  `~/.config/media/lightroom-out/` with a JSON next to each: the original's path in the library,
  the variant (`lr` or `lr-<copy>`), the size, the edit time.
- What it rendered is remembered in the plug-in's prefs by photo and edit time, so a second click
  renders nothing.
- Progress in Lightroom's progress bar, cancellable; the end says how many.

## Files affected

- `lightroom/FamilyLightbox.lrplugin/UpdateLists.lua` (or a `Renditions.lua` it calls).

## Progress log

### [2026-11-05] Written
