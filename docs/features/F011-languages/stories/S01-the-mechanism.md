# Story S01: The mechanism, the choice, the first screens

**Feature:** `F011-languages`
**Status:** Built: to try in the app
**Depends on:** —

## Acceptance criteria

- `src/i18n/en.ts` is the reference catalog; `nl.ts` has the type of `en`, so a missing Dutch
  text fails `tsc` (decision 3). A text with numbers or names is a function.
- `src/i18n/index.ts`: the language in use, `t()` for the catalog, `pickLanguage(setting,
  navigator.languages)` (Dutch when the first preferred language is `nl…`, else English;
  the setting overrides), numbers (`count`, decimals) and month names in that language.
- A context at the root re-renders the app when the language changes, without a restart.
- The setting `language` (`""` system, `en`, `nl`) in the app's settings, chosen in
  *Settings → General*.
- Moved in this story: `count`, `gb`, the month names (`months.ts`), the welcome
  (`DeviceSetup.tsx`) and the language setting itself. The rest follows in S02 and S03.
- The existing tests run in Dutch (a vitest setup file); new tests for English.

## Files affected

- `src/i18n/*`, `src/lib/sync.ts`, `src/lib/months.ts`, `src/lib/device.ts`,
  `src/components/DeviceSetup.tsx`, `src/components/Settings.tsx`, `src/main.tsx`,
  `src-tauri/src/config.rs`, `vite.config.ts`.

## Progress log

### [2026-11-05] Written

### [2026-11-05] Built

- `src/i18n/en.ts` (the reference, `Catalog = typeof en`) and `nl.ts` (`: Catalog`); texts with
  numbers or names are functions. Namespaces so far: `months`, `language`, `device`, `welcome`.
- `src/i18n/index.ts`: `pickLanguage`, `setLanguage`, `t()`, `count` (`124,837` / `124 837`),
  `decimal` (`1.3` / `1,3`), `monthName`, `LanguageContext`, `LANGUAGE_EVENT`.
- `main.tsx`: a `Root` keeps the language: the Mac's at first (`navigator.languages`), then the
  setting's; a change re-renders the whole window (no component is memoized).
- `Settings.language` in Rust (`""`, `en`, `nl`, others refused on save); *Instellingen →
  Algemeen → Taal* saves it on its own and switches at once.
- Moved: `count`, `gb` (`sync.ts`), month names (`months.ts`), `device.ts`'s rules and texts, the
  whole welcome (`DeviceSetup.tsx`), the language setting. The rest of *Settings* and everything
  else stays Dutch until S02; Rust's messages until S03.
- `src/test-setup.ts` runs every test in Dutch; `i18n.test.ts` tests the choice and English.
- `pnpm test` 173 (3 new), `cargo test` 229, `tsc`, `vite build`, clippy clean.
