# Story S02: The plug-in in Lightroom

**Feature:** `F023-lightroom-plugin`
**Status:** In progress: built; to try in Lightroom
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
