# Story S03: This library's file, the settings, the share names retired

**Feature:** `F019-nas-layout`
**Status:** Planned
**Depends on:** S01

## Goal

This library's two Macs read its layout from the NAS, and the settings show the layout instead of
four share names to edit.

## Acceptance criteria

- `familylightbox.json` written in the root of `Lists`, describing the four shares as they are
  (each `path` `""`), with Peter's go; both Macs read it (their home share is empty, so `Lists`).
- The settings' NAS section: the NAS name, the home share, and the layout read (per location its
  share and path, and the root it resolves to), with where it came from (the NAS, the copy kept,
  or the Mac's own settings); the four share-name fields go.
- A Mac whose settings name other shares than the file: the file wins, and the settings say so.
- **Tests**: the texts of where the layout came from; the settings without share names.

## Files affected

- `src/components/Settings.tsx`, `src/lib/api.ts`, `src-tauri/src/lib.rs`.

## Progress log

### [2026-10-31] Written
