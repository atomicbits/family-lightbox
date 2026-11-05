# Story S03: Messages from the Rust core

**Feature:** `F011-languages`
**Status:** Built: to try in the app
**Depends on:** S01

## The question

Decision 2 chose codes with parameters from Rust (`{ code: "index_missing", path }`), translated
in the frontend. Counting them (2026-11-05): some 400 messages in 30 modules, most of them
`format!("{path}: {e}")` around an I/O error, many only reaching the messages overlay.

With codes each message exists three times: the code and its parameters in Rust, the English and
the Dutch text in TypeScript, and every `Result<_, String>` changes type.

**Proposal:** both texts next to each other in Rust, through a macro that needs both
(`tr!("nog geen index: {}", "no index yet: {}", path)`), with the language the window chose
passed in once (`set_language`, kept in an atomic). Missing a language is a compile error, as
in the frontend; the commands keep returning `String`; the Rust tests keep asserting Dutch
(the default in tests). It reverses decision 2's reason ("the texts live in two places"): they
would live in two places, but each message in one.

## Acceptance criteria (either way)

- Every message from the core a user can see comes in the chosen language; an OS error's own
  text (`Permission denied`) stays as the system gives it.
- `cargo test` passes in Dutch unchanged; `tr!` tested in both languages, and a few messages in English.

## Progress log

### [2026-11-05] Written

### [2026-11-05] Decided

Peter: the macro in Rust. `src-tauri/src/i18n.rs`: `tr!(nl, en, args…)` (both literals, as
`format!`), `set_language` (a command the window calls whenever its language changes),
`in_english` for tests (per thread). Dutch until the window says otherwise.

### [2026-11-05] Built

- About 185 messages in 28 modules through `tr!`, done by three helpers in their own worktrees and
  taken over one after the other; the Dutch is unchanged, so every existing test passes. Modules
  that only wrap a path and an OS error (`"{}: {e}"`) and the data strings (device labels in
  `devices.json`, stack colors, slugs) stay as they are.
- The labels of the messages' sources (*synchronisatie* / *sync*, …) in both languages.
- The export presets stay a `const` in Dutch; `label` and `what` are serialized through an English
  table (`export::ENGLISH`), with a test that every preset has its entry.
- `pull::gb` writes `29.1` in English; the window recognizes a refused update and a lack of space
  in both languages (`update.ts`, `sync.ts`).
- The window's own lines in *Meldingen* (`browse.log`) in both languages.
- `cargo test` 232 (`tr!` and four messages in English), `pnpm test` 174, clippy, `tsc` clean.
