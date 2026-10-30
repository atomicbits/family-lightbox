# Story S05: Choosing a stack's color

**Feature:** `F015-stacks`
**Status:** In progress: built and tested; to try in the app
**Depends on:** S04

## Goal

Peter asked, once stacks worked, to give a stack a color himself. The right-click menu on a photo
of a stack shows the eight colors; a click gives the stack that color, for everyone.

## Acceptance criteria

- **The menu** on a photo of one stack: a row of the eight colors (blauw, amber, groen, roze,
  paars, turkoois, oranje, grijs), the current one marked; a click sets it.
- **Shared**: `{"op":"color","stack":"<id>","color":"groen"}` in the marks files (data contract
  5.5); per stack the last one decides; a stack without one keeps the color made from its id.
  The event has no `photo`.
- **The message with undo**: "reeks groen gemaakt"; undo puts the color before back.
- **Tests**: `cargo test` for the merge and an unknown color; `pnpm test` for the color chosen
  over the one from the id, and undo.

## Files affected

- `src-tauri/src/marks.rs`, `src-tauri/src/lib.rs`, `src/lib/marks.ts`, `src/lib/stacks.ts`,
  `src/lib/undo.ts`, `src/components/PhotoMenu.tsx`, `src/components/Grid.tsx`, `src/App.tsx`,
  `docs/data-contract.md`.

## Progress log

### [2026-10-30] Written

### [2026-10-30] Built

- `marks.rs`: `Op::Color` (an event without a photo), `COLORS`, `Marks.stack_colors`,
  `Change::Color`, the command `marks_color`; an unknown color is refused and ignored when read.
- UI: `PALETTE` with the names, `stackColor(id, chosen)`, a row of eight swatches in the menu
  on photos of one stack (the current one ringed), a `color` undo step that puts the color
  before back unless someone changed it since.
- One Rust test (the last color wins, an unknown one refused and ignored), the chosen color over
  the one from the id in `pnpm test`; 130 Rust and 96 vitest tests, clippy, `tsc` and
  `pnpm build` clean. Data contract 5.5 extended.
