# Story S02: The plug-in in Lightroom

**Feature:** `F023-lightroom-plugin`
**Status:** Done (2026-11-05)
**Depends on:** S01

## Acceptance criteria

- `FamilyLightbox.lrplugin` (Lua, Lightroom Classic SDK 10+): *Bibliotheek → Plug-in-extra's →
  Lijsten bijwerken* reads the file of S01 and keeps the collection set *FamilyLightbox*:
  - per list a collection *<year> <name>*, archived ones in the set *Gearchiveerd*; a list is
    found again by its id (kept in the plug-in's preferences), so a renamed list renames its
    collection;
  - its photos: those here, found in the catalog by path, or added to the catalog where they are;
    photos no longer in the list leave it; those not here are counted;
  - nothing outside *FamilyLightbox* is touched; in the end a message: *12 lijsten bijgewerkt;
    40 originelen nog niet hier: haal ze op in FamilyLightbox*, and a log file.
- By hand in Lightroom Classic.

## Progress log

### [2026-11-05] Built

`lightroom/FamilyLightbox.lrplugin`: `Info.lua` (the menu item *Lijsten bijwerken* under
*Bibliotheek → Plug-in-extra's*), `Json.lua` (a small decoder, Lightroom's Lua has none) and
`UpdateLists.lua`: the photos looked up by path outside the write access, then in one write
access the set *FamilyLightbox* with *Gearchiveerd*, originals here but not in the catalog added,
per list its collection found by id in the plug-in's preferences (made again when removed by
hand), renamed, moved to or out of *Gearchiveerd*, photos added and removed; a message with the
counts and the originals not here; a log in `~/Documents/LrClassicLogs/FamilyLightbox.log`.
Installed by hand in Lightroom's modules folder on Peter's Mac to try; no Lua here to test it
outside Lightroom.

### [2026-11-05] Fix: a new collection can't be read in the write access that made it

- Lightroom: *Can't get collection information after creating collection inside the same
  withWriteAccessDo function*. The plug-in now works in four steps: outside a write access the
  collections known by id and the photos not in the catalog; a first write access that only makes
  the sets, the missing collections and adds the photos; outside again the new collections found
  back by name in their set, their ids kept, and what to add, remove, rename or move; a second
  write access that does it.

### [2026-11-05] Fix: `LrTasks.pcall`

- The collections came, but empty: every `catalog:addPhoto` failed with *AgEventLoop.yieldToScheduler
  called when yielding is not allowed* (Lightroom's console), as it ran inside a plain `pcall`,
  which doesn't allow Lightroom to yield. All `pcall`s are `LrTasks.pcall` now.

### [2026-11-05] It works; what it is doing, said

- Peter's run: *2 lijsten, 1 bijgewerkt. 569 originelen aan de catalogus toegevoegd. 1456
  originelen nog niet op deze Mac*; *2026 Aosta* shows its 602 photos in Lightroom. Adding took a
  few minutes: Lightroom looks up each new photo's place (`AddressLookup` in its console).
- The progress bar now says its step: *de lijsten lezen*, *toevoegen aan de catalogus: 120 van
  569*, *de collecties vergelijken*, *de collecties bijwerken*.
