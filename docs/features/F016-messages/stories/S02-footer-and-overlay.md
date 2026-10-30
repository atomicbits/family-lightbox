# Story S02: The ⚠ in the footer, the overlay, and lines that fade

**Feature:** `F016-messages`
**Status:** In progress: built and tested; to try in the app
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

### [2026-10-30] Built

- `messages.ts`: `useMessages` listens to the core's `message` events (its own and the UI's,
  since `log_message` now reports through the core too), keeps at most 500, newest first, and
  counts the unseen ones; `pushMessage`, `timeLabel`.
- The footer shows ⚠ with the number unseen in amber (grey once seen, while there are messages);
  a click opens `MessagesOverlay` (time, source, text, *Wissen*, the log's path); also *Meldingen*
  under *Over* in the settings.
- Fading: the sync line fades after 8 s for done, cancelled and failures (`fades`), and stays for
  too little disk space and the previews confirmation; the amber index line fades after 8 s.
  The list's messages faded already; the update line, a wrong `device.json` and the previews
  line stay.
- `pnpm test`: the list's limit, the time label, which sync lines fade; 103 tests. `tsc`,
  `pnpm build` and clippy clean.

### [2026-10-30] Fix: the time test failed on the CI

The test of `timeLabel` wrote its moments in Belgian time (`+01:00`); the CI runs in UTC, so
the label said 20:15 and every run since this story failed. The test now builds its moments in
local time. The same weakness in the lists test (`addedText`, a day off east of UTC+12) is fixed
the same way. Both suites checked under UTC, Brussels, Auckland, Kiritimati, Los Angeles and
Pago Pago.

### [2026-10-30] Copying the messages

On Lynn's Mac the text in *Meldingen* couldn't be copied: the app turns text selection off
everywhere (`index.css`), and right-click is off too. The list is selectable now (`select-text`),
so ⌘C works, and *Kopiëren* puts all messages on the clipboard as the log's lines, oldest first
(`messagesText`), to send on.
