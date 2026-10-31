# Feature 19: The NAS layout in one file

**Status:** In progress (plan accepted 2026-10-31)
**Depends on:** Feature 6 (connecting the NAS), feature 7 (the sync), feature 18 (import)
**Folder:** `F019-nas-layout`

## Goal

Where the library, the previews, the lists and the exports are on the NAS is described once, in
one file on the NAS that every Mac reads, instead of four share names set on each Mac. A new user
puts everything in one shared folder with four subfolders: one share, one mount, one sign-in. An
existing setup with several shares, or a library inside a share that is there already (Synology
Photos' `photo`, for example), is described by the same file.

Afterwards this library moves from its four shares to one (Peter, 2026-10-31).

## Background

- **Data contract 2** names four shares, `Library`, `Previews`, `Lists` and `Exports`, each its
  own SMB share on the NAS. The app keeps their names per Mac in `familylightbox.json`
  (`shares`), and mounts each one (F006).
- **Every path inside them is relative to its root**: a manifest's `path` to the root of the
  library, a preview's path to the root of the previews, an event file's to the root of the lists.
  So the sync, the index, the previews, the lists and the import don't care where those roots are;
  only how the app finds them changes.
- **`Lists`** holds only small JSON files; **`Previews`** is derived and doesn't go to Drive;
  **`Library`** and **`Exports`** do. Year folders can be denied to a user by an ACL on the NAS,
  in `Library` and `Previews` (data contract 2); that works the same on subfolders of one share.
- **What reads the shares outside the app**: the CLI (`state.json` → `nas.library`), the NAS
  scripts (`media-manifest-nas.py` with `--root`, `media-drive-push.py` per share), Lightroom
  through the CLI's working copy. They keep working as long as the shares stay where they are.

## Scope

### In scope

- **`familylightbox.json` in the root of the home share**: `contract_version`, a `name` for the
  library, and per location (`library`, `previews`, `lists`, `exports`) a `share` and a `path`
  inside it (`""` for the share's root). Data contract 2 becomes "Locations", version 1.4: without
  the file the four shares of today apply.
- **Per Mac only the NAS and the home share** are kept; everything else is read from the file,
  so all Macs use the same layout.
- **Mounting what the file names**: the shares it names, each once, with one sign-in when they are
  one; the four roots resolved to folders under the mounts.
- **The sync, the index, the previews, the lists, the import and the access check** use the four
  roots instead of the four share names.
- **The welcome** (F006/S06): after connecting the home share, the file is looked for; found, the
  app goes on; not found, it offers to set up a new library there: the four subfolders and the
  file, in one share by default, or pointing at folders that exist already.
- **The settings** show the layout read from the NAS; changing it means changing the file.
- **This installation**: a `familylightbox.json` in `Lists` describing its four shares, so both
  Macs read it; `Lists` is the home share until the move.
- **Moving an existing library to one share** (the last story): the general steps in this repo
  (the file rewritten, the clients follow); what is particular to this NAS (its scripts, Drive,
  snapshots, rights) in a feature of its own in LibraryLocal.

### Out of scope

- **Several libraries** on one NAS, or one Mac using several: one library per Mac.
- **Other protocols** than SMB (NFS, WebDAV) and other NAS brands' specifics: the file is
  generic, connecting stays as F006 does it.
- **Moving the files for the user** in the app: the move is done on the NAS, with its own tools.

## Acceptance criteria

- A new Mac pointed at an empty share sets up a library there with one click: `Library`,
  `Previews`, `Lists`, `Exports` and `familylightbox.json`, and works from it (sync, import,
  lists) with one mount.
- Lynn's and Peter's Macs read this library's layout from the file in `Lists` and behave exactly
  as before; the per-Mac share names are no longer used.
- A file naming another share for one location (the library in `photo/FamilyLightbox`, say) works:
  that share is mounted too, and the root is the folder inside it.
- A broken or missing file is reported in the settings and the messages, and the app keeps
  working with the layout it read last.
- After the move to one share, changing the file is all the Macs need; nothing is set again on
  either Mac.
- `cargo test` covers reading the file (one share, several shares, paths inside a share, broken,
  missing → the defaults), resolving the roots against mounts, and setting up a new library;
  `pnpm test` the welcome's texts.

## Stories

| ID | Title | Status | Depends on |
|---|---|---|---|
| [S01](stories/S01-the-layout-file.md) | The layout file in the contract (1.4), read from the home share; the four roots resolved against the mounts; everything uses the roots | In progress | — |
| [S02](stories/S02-the-welcome.md) | The welcome: the home share, the file looked for, a new library set up in one share | Planned | S01 |
| [S03](stories/S03-this-librarys-file.md) | This library's file in `Lists`; the settings show the layout; the per-Mac share names retired | Planned | S01 |
| [S04](stories/S04-moving-to-one-share.md) | Moving an existing library to one share: the file rewritten, the Macs follow; with LibraryLocal's story for the NAS's scripts, Drive and snapshots | Planned | S03 |

## Decisions

Taken with Peter on 2026-10-31:

1. **One file on the NAS** describes the layout for all clients: `familylightbox.json` in the root
   of the home share.
2. **One share by default** for a new library, with `Library`, `Previews`, `Lists` and `Exports`
   as subfolders; other shares and paths stay possible per location.
3. **This library moves to one share** once the feature works.

Accepted by Peter on 2026-10-31 as proposed:

4. **The home share of this library until the move.** Proposal: `Lists`, which both of you can
   read and write and where `devices.json` is already.
5. **The name of the one share after the move.** Proposal: decided in S04 with the LibraryLocal
   story (it touches the NAS's scripts and the Drive mirror), not here.
6. **When the file and a Mac disagree** (an old Mac with share names kept): the file wins; the
   Mac's own names are only used while no file is found, and the settings say which applies.
