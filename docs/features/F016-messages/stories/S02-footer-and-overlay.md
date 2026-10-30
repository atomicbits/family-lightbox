# Story S02: The ⚠ in the footer, the overlay, and lines that fade

**Feature:** `F016-messages`
**Status:** Planned
**Depends on:** S01

## Goal

Messages gather in a list behind a ⚠ in the footer; the footer's lines say briefly what happened
and fade; what asks for an action stays.

## Acceptance criteria

- **The list** in the UI: the messages of this session (from the core's events and the UI's own),
  newest first, at most 500; which are unseen.
- **The ⚠** in the footer with the number unseen, only when there are any; a click opens the
  overlay and marks them seen.
- **The overlay**: time, source and text per message, *Wissen*, the log file's path; Esc or a
  click outside closes it; also from the settings.
- **Fading**: a failed sync, a failed write or an index problem shows a short text for 8 seconds;
  the previews confirmation, too little disk space, a wrong `device.json` and an update stay.
- **Tests** (`pnpm test`): the list with its limit, unseen, which lines fade.

## Files affected

- `src/lib/messages.ts` (new), `src/components/MessagesOverlay.tsx` (new), `src/App.tsx`,
  `src/lib/sync.ts`, `src/lib/indexer.ts`, `src/components/Settings.tsx`.

## Progress log

### [2026-10-30] Written
