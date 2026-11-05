# Feature 23: The lists in Lightroom Classic

**Status:** In progress (plan accepted 2026-11-04)
**Depends on:** Feature 3 (lists and their summaries), feature 9 (fetching originals)
**Folder:** `F023-lightroom-plugin`

## Goal

A list made in FamilyLightbox appears in Lightroom Classic as a collection, without making it
again there, so Peter can edit its photos and export them for a photo book, a print or a
calendar with Lightroom's own presets. The lists stay the source (requirement F17, decision D9):
the bridge goes one way.

## Background

- Every list has a summary, `Lists/<year>/<slug>.json`, with each photo's path in the library;
  the sync keeps a copy of `Lists` on every Mac (data contract 5.4, 8). The plug-in reads a file
  the app writes instead (S01), with the paths in the working copy and whether each original is
  here: there is no Lua on this Mac to test a plug-in outside Lightroom, so its logic is in Rust.
- Lightroom finds a photo by its path. The catalog has to point at the working copy first:
  that is a one-off fix of the catalog outside this repo (LibraryLocal). After it, the paths in
  the summaries are the paths in the catalog.

## Scope

### In scope

- **A Lightroom Classic plug-in** (`FamilyLightbox.lrplugin`, Lua, Lightroom's SDK) in this repo,
  installed from the app's settings (*Lightroom-plugin installeren*), which copies it into
  Lightroom's modules folder.
- **On a click**: *Bibliotheek → Plug-in-extra's → Lijsten bijwerken* reads the summaries in the
  local copy of `Lists` and keeps a collection set *FamilyLightbox* with, per list that isn't
  archived, a collection *<year> <name>* (an archived list's collection goes to a set
  *Gearchiveerd*):
  - a photo whose original is here and in the catalog joins its collection;
  - one that is here but not in the catalog is added to the catalog first, where it is;
  - one that isn't on this Mac is counted: *12 originelen nog niet hier: haal ze op in
    FamilyLightbox* (feature 9), and joins after a fetch and another click;
  - a photo removed from the list leaves the collection; a list renamed renames its collection;
  - a collection under *FamilyLightbox* changed by hand in Lightroom is set back at the next click
    (one way); other collections are never touched.
- **From the app**: on a list, *Naar Lightroom* fetches its originals (feature 9) and says to click
  *Lijsten bijwerken* in Lightroom.
- What the plug-in did, in its own log and in a short message after the click.

### Out of scope

- Changes in Lightroom going back to the lists.
- Lightroom's edits shown in the app: feature 25.
- Lightroom (cloud) and other editors.

## Acceptance criteria

- After the catalog fix, *Lijsten bijwerken* makes a collection for each of the lists, with every
  photo whose original is here; the count of the missing ones is right; a second click changes
  nothing; a photo added in the app is in the collection after a fetch and a click.
- The plug-in never writes in the library, the lists or the catalog's other collections.
- Lua tested where it can be outside Lightroom (reading summaries, the plan of changes);
  by hand in Lightroom Classic.

## Stories

| ID | Title | Status | Depends on |
|---|---|---|---|
| [S01](stories/S01-the-file-for-lightroom.md) | The file for Lightroom, written by the app: every list with its photos' paths in the working copy and whether they are here | Done | — |
| [S02](stories/S02-the-plug-in.md) | The plug-in: the collection set, by list id, adding what is here, the message and the log | Done | S01 |
| [S03](stories/S03-from-the-app.md) | From the app: installing the plug-in, *Naar Lightroom* on a list | Planned | S02, feature 9 |
| [S04](stories/S04-stars.md) | Stars and hearts to Lightroom: a heart is 5 stars, FamilyLightbox wins where it has one | Built, to try in Lightroom | S01, S02 |
