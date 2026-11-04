# Story S03: In the window: no refusal offline, and the changes that wait

**Feature:** `F020-offline-changes`
**Status:** In progress: built and tested; to check in the app
**Depends on:** S01

## Goal

Without the NAS, lists, stars, hearts, hiding and stacks are changed as with it, and the footer
says what waits.

## Acceptance criteria

- No *… wijzigen kan alleen met de NAS* any more; only a Mac with no lists or marks at all (never
  connected) is told to connect once first.
- The footer shows *3 wijzigingen wachten op de NAS* while the outbox isn't empty, and nothing when
  it is.
- Once the `Lists` share is there, the window asks for the outbox to be sent, and reads the lists
  and marks again; a send that fails is logged and tried again with other mounts, never in a loop.
- **Tests**: `pnpm test` for the refusals and the footer's text.

## Progress log

### [2026-11-04] Written and built

- `lists.ts` `changeBlocked` and `marks.ts` `marksBlocked` refuse only without any source;
  `waitingText` for the footer.
- `App.tsx`: the count (`outbox_count`) after every change of the lists or marks; `outbox_send`
  when the `Lists` share is there and something waits, once per mounts; the line in the footer.
- `pnpm test` 154 passed (1 new, 2 changed), `tsc` and `vite build` clean.
