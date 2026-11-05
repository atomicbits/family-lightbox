# Story S03: Messages from the Rust core

**Feature:** `F011-languages`
**Status:** Planned: a decision to revisit with Peter
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
- `cargo test` passes; a test per module for one message in English.

## Progress log

### [2026-11-05] Written
