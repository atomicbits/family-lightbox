# Feature 11: Languages: English by default, Dutch for us

**Status:** Planned (plan accepted 2026-10-27; after feature 4)
**Depends on:** —
**Folder:** `F011-languages`

## Goal

FamilyLightbox speaks English by default, so it is usable by anyone who installs it from the
public releases, and Dutch on our Macs. Every text the app shows comes in both languages:
buttons, headers, the settings, the sync and index lines, the confirmations, the month names and
the error messages from the Rust core. Nothing in the data is translated: folder names, file
names and list names stay as they are.

## Background

- **Today every UI text is Dutch**, written straight into the components (`App.tsx`, `Settings.tsx`,
  `SyncLine.tsx`, `DeviceSetup.tsx`, …), into the helpers that build texts (`src/lib/sync.ts`,
  `indexer.ts`, `nas.ts`, `update.ts`, `months.ts` with its own month names), and into the Rust core.
  The Rust core returns messages such as "nog geen index: …", "bijwerken mislukt: …" or
  "onbekende synchronisatie" as plain strings. About 30 such places are in `config.rs`, `lib.rs`,
  `runner.rs`, `nas.rs`, `pull.rs`, `update.rs` and `db.rs`.
- **The convention so far** was that the code and docs are English and the UI texts Dutch. This
  feature replaces that: the UI has an English text and a Dutch one for every message.
- **Tests compare texts**: the vitest tests of `sync.ts`, `indexer.ts`, `update.ts` and `months.ts`
  assert Dutch strings, and some Rust tests assert messages. They have to follow.
- **macOS** tells the app the user's preferred languages (System Settings → General → Language &
  Region), and the webview exposes them as `navigator.languages`. Tauri's own menu and system
  dialogs follow the system already.

## Scope

### In scope

- **Two languages**: English (the default) and Dutch. Adding a third later means adding one
  catalog, nothing else.
- **The choice**: the app follows the Mac's preferred language. Dutch when the first preferred
  language is Dutch (`nl`, `nl-BE`, `nl-NL`), English otherwise. A setting *Language* in the
  settings overrides it: *System*, *English*, *Nederlands*. It is kept per Mac, next to the other
  app settings, and switches without a restart.
- **All texts the app shows**, from the frontend and from the Rust core, in both languages. That
  includes texts with numbers, plurals and names in them: "3 items" / "1 item", "319 previews to
  copy (1.2 GB)", "Version 0.2.1 available".
- **Dates and numbers** in the chosen language: month names ("July 2026" / "juli 2026"), and the
  thousands separator and decimal comma ("124,837" / "124 837", "1.3 s" / "1,3 s").
- **Complete by construction**: a text missing in one language is a build error, not a blank or
  a key on screen.
- **The macOS bundle** declares both languages, so Finder and the system's own dialogs about the
  app match.
- **The docs**: `CLAUDE.md` and `docs/features/README.md` describe the new convention (every UI
  text in both catalogs), and the stories of features 6–8 keep their Dutch quotes as history.

### Out of scope

- **Translating data**: folder names ("2012/scans …"), file names, list names, device and owner
  names.
- **The `media` CLI** in LibraryLocal: it stays Dutch.
- **More languages** than these two, and right-to-left scripts.
- **Translating the release notes** and the GitHub release page.

## Acceptance criteria

- On a Mac whose first language is English (or anything but Dutch), every screen of the app is in
  English, error messages from the Rust core included. On a Mac whose first language is Dutch,
  everything is in Dutch, as today.
- *Settings → Language* switches between System, English and Nederlands at once, and the choice
  survives a restart.
- Month names, counts and sizes follow the chosen language.
- Removing a text from one catalog makes `pnpm build` (or `cargo build`) fail.
- No UI text is left written straight into a component or a Rust message.
- The existing tests pass in both languages where they test texts.

## Stories (proposal)

| ID | Title | Status | Depends on |
|---|---|---|---|
| S01 | The mechanism: catalogs for English and Dutch, the language choice (system, setting), dates and numbers; the first screens moved | Planned | — |
| S02 | All frontend texts moved to the catalogs | Planned | S01 |
| S03 | Messages from the Rust core: codes with parameters, translated in the UI | Planned | S01 |
| S04 | The bundle's languages, the docs and the conventions | Planned | S02, S03 |

## Decisions

Peter accepted all five proposals on 2026-10-27.

1. **How the language is chosen.** Proposal: follow the Mac, with a setting to override. The
   alternative is a setting only, English unless set. Following the Mac means Lynn and Peter see
   Dutch without doing anything.
2. **Messages from Rust.** Proposal: the Rust core returns a code with parameters
   (`{ code: "index_missing", path: "…" }`), and the frontend translates it, so there is one place
   for every text. The alternative, a small catalog on the Rust side, keeps the commands as they
   are, but then the language has to be passed into Rust, and the texts live in two places.
3. **A library or our own.** Proposal: our own small typed module (`src/i18n/en.ts`, `nl.ts`),
   with English as the reference type, so a missing Dutch text is a `tsc` error. Plurals and
   numbers go through the browser's `Intl.PluralRules` and `Intl.NumberFormat`. A library such
   as i18next adds a dependency and its key strings aren't type-checked. The app has a few hundred
   texts, which our own module handles.
4. **Which English.** Proposal: British or American doesn't matter much for these texts; we take
   American spelling ("color", "canceled") because it's what macOS uses.
5. **When.** Before the first install on Lynn's Mac (F004/S05) or after it? It doesn't block her:
   her Mac is Dutch, so she sees Dutch either way. Proposal: after F004, before the app is
   announced anywhere public.
