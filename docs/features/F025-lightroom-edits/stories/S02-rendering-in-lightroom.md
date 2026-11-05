# Story S02: Rendering in Lightroom

**Feature:** `F025-lightroom-edits`
**Status:** Built: to try in Lightroom
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

### [2026-11-05] Built

- `Renditions.lua`, called by *Lijsten bijwerken* after the lists (step 5), with the masters of
  the photos in the lists' collections (a virtual copy in a collection brings its master).
- Edited: `hasDevelopAdjustments` where this Lightroom knows it (through `LrTasks.pcall`),
  otherwise only a crop, with a line in the log; `editCount` isn't used, it counts stars too.
  Videos are left out. Every virtual copy of such a master is a variant `lr-<slug of its copy
  name>` (`lr-copy` without a name, `-2` for a second same slug).
- Rendered with two `LrExportSession`s (2 048 px long edge without enlarging, and full size;
  JPEG 0.9, sRGB, location metadata removed, no sharpening, no re-import) into a work folder,
  then moved to `lightroom-out/<id>.jpg` and `.full.jpg`, the JSON written last through a
  temporary name. The prefs keep `path|variant → lastEditTime`; a variant that was rendered and is
  gone while its master is still in a list gets a JSON with `removed`.
- The dialog at the end says how many versions went to FamilyLightbox and how many were removed.
- The plug-in's version 0.2; `Renditions.lua` installed with the others (`lrplugin.rs`).
- Not checked here: no Lua on this Mac; Lightroom shows a syntax error at the first click.
