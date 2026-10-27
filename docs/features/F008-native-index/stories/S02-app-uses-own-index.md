# Story S02: The app reads its own index and keeps it current

**Feature:** `F008-native-index`
**Status:** Planned
**Depends on:** S01

## Goal

The grid reads the app's own index from S01, and the app keeps it current by itself: at start,
after a sync, when Peter comes back from the CLI, and by hand. The `./media index` hints go away.

## Acceptance criteria

- **Reading**: `months` and `items` open `familylightbox-index.sqlite`. The CLI's `index.sqlite`
  is no longer read. Without an index, the grid says "index wordt gemaakt…" while it is built.
- **When it is built**, on a blocking thread, one build at a time (a request during a build runs
  once more afterwards):
  - at start, always, before the NAS start of feature 6;
  - after a sync of feature 7 that copied manifests or previews;
  - on window focus (at most every 30 s), when a manifest is newer than the index or the number
    of manifests changed;
  - with *Index bijwerken* in the settings panel.
- **Events**: `index-done` with the result of S01 or the error.
- **The grid after a rebuild**: months are fetched again and the month cache is cleared; the
  selection stays on the same item (by path) when it still exists, otherwise on the same month.
  The scroll position stays on the month at the top.
- **The header**:
  - "index bijwerken…" while a build that started by hand or at start without an index runs;
    quiet rebuilds at focus and after a sync show nothing unless they fail;
  - a failure in amber ("index: 1 manifest onleesbaar: 2016/12" or the error);
  - the sync line no longer says `./media index`.
- **Status**: `status` reports the app's index path and whether it exists; the "geen index: draai
  ./media index" text goes.
- **Tests**:
  - `cargo test`: the focus check (newer manifest, a manifest removed or added, nothing
    changed) and the one-at-a-time rule with a queued second request;
  - `pnpm test`: keeping the selection after a rebuild, and the header texts.
  - Existing tests still pass, typecheck, build and clippy clean.
  - **By hand** on Peter's Mac: start the app with no app index (it appears), import nothing but
    touch a manifest and switch back (it rebuilds), and *Index bijwerken*.

## How

- `index.rs`: `stale(library, index) -> bool`, an `IndexRunner` like the `SyncRunner`, with the
  same `Sink` idea.
- `lib.rs`: `Paths::index()` points at the new file; commands `index_rebuild` and
  `index_stale`. The sync runner asks for a rebuild when it copied something.
- Frontend: `src/lib/index.ts` (`useIndex`), `App.tsx` (reload after `index-done`, keep the
  selection), `Settings.tsx` (*Index bijwerken*), `SyncLine` without the hint.
- Deliberately not done here: updating during a long sync (S03).

## Files affected

- `src-tauri/src/index.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/runner.rs`.
- `src/lib/index.ts` (new), `src/lib/index.test.ts` (new), `src/lib/sync.ts`, `src/App.tsx`,
  `src/components/Settings.tsx`.

## Progress log

### [2026-10-27] Written
