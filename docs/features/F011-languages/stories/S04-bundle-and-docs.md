# Story S04: The bundle's languages, the docs and the conventions

**Feature:** `F011-languages`
**Status:** Done (2026-11-05)
**Depends on:** S02, S03

## Acceptance criteria

- The bundle declares English and Dutch, so macOS's own dialogs about the app follow the user.
- `CLAUDE.md` and `docs/features/README.md` say how a UI text and a core message are written.

## Progress log

### [2026-11-05] Done

- `src-tauri/Info.plist` (merged by the Tauri CLI): `CFBundleDevelopmentRegion` `en`,
  `CFBundleLocalizations` `en`, `nl`; `infoplist/<lang>.lproj/InfoPlist.strings` as resources,
  each file named (the glob `infoplist/**` wasn't found by the bundler). Checked in a debug
  bundle: `en.lproj`, `nl.lproj` in `Resources`, the localizations in `Info.plist`.
- The convention in `CLAUDE.md` (*Language*) and `docs/features/README.md`.
