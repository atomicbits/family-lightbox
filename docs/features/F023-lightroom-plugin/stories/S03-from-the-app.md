# Story S03: From the app

**Feature:** `F023-lightroom-plugin`
**Status:** In progress: built and tested; to try in the app
**Depends on:** S02, feature 9

## Acceptance criteria

- The app carries the plug-in (its three files built into the app) and installs it from its
  settings, tab *Algemeen*, section *Lightroom*: *Plugin installeren*, or *Plugin bijwerken* when
  the installed one differs from the app's; *geïnstalleerd* when they are the same. Into
  `~/Library/Application Support/Adobe/Lightroom/Modules/FamilyLightbox.lrplugin`, replaced as a
  whole; Lightroom needs a restart or *Plug-in opnieuw laden* afterwards, which the settings say.
- **Tests**: `cargo test` for the status (none, the same, another) and the installation.
- On a list: *Naar Lightroom* fetches its originals (feature 9) and says to click *Lijsten
  bijwerken* in Lightroom.

## Progress log

### [2026-11-05] Built

- `lrplugin.rs`: the plug-in's three files built into the app (`include_str!`); `status` (none,
  the app's, another) by comparing the files; `install` through a temporary folder, replacing the
  installed one as a whole. Commands `lightroom_plugin_status` and `lightroom_plugin_install`.
- The settings, tab *Algemeen*: the section *Lightroom* with its state, *Plugin installeren* or
  *Plugin bijwerken*, and what to do in Lightroom afterwards.
- A list's menu: *Naar Lightroom…* fetches its originals (with the plan of feature 9) and the
  message afterwards says to click *Lijsten bijwerken* in Lightroom; with all of them here it says
  so at once.
- `cargo test` 216 passed (1 new), `pnpm test` 163 passed (1 new), `tsc`, `vite build` and
  `cargo clippy --all-targets` clean.
