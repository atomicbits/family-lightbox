# Story S02: The app reads its own index and keeps it current

**Feature:** `F008-native-index`
**Status:** In progress: built and tested, index built by the running app on real data; the reload, the focus check and *Index bijwerken* in the app still to check
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

### [2026-10-27] Built

- `index.rs`:
  - `fingerprint(library)`: a hash over every manifest's path, size and modification time. A
    synced manifest keeps the NAS's time, which can be older than the index, so comparing with
    the index file's time wouldn't catch it;
  - `IndexRunner`: one build at a time on its own thread. A request during a build sets a flag
    and builds once more afterwards; several requests queue one build. `stale()` compares the
    fingerprint taken at the start of the last good build with the current one, and is true
    without an index file;
  - `IndexState` (running, the last outcome, the error, the number of builds) and `IndexSink`.
- `lib.rs`:
  - `Paths::index()` is `familylightbox-index.sqlite`; the CLI's `index.sqlite` is no longer read;
  - the commands `index_rebuild(only_if_stale)`, which returns whether it asked for a build, and
    `index_status`; the event `index-done`;
  - the sync's `EventSink` asks for a rebuild when a sync copied manifests or previews.
- `db.rs`: without an index the message no longer points at `./media index`.
- Frontend:
  - `lib/indexer.ts` (not `index.ts`, which would read as the folder's entry point): `useIndex`,
    and the pure `indexLine`, `outcomeText` and `reselect`;
  - `App.tsx`:
    - at start it shows the index there is and rebuilds in the background; without one, the grid
      says "index wordt gemaakt…" and "index bijwerken…" shows in the header;
    - after every finished build: months again, an empty month cache, the selection by path (or
      the same position, or nothing when its month is gone), and a jump to the month that was at
      the top when it moved. Answers to requests from before the rebuild are dropped;
    - the focus check (shared with the NAS check, at most every 30 s) rebuilds only when stale;
    - the header shows the index line instead of "geen index: draai ./media index";
  - the sync line has no `./media index` hint any more;
  - *Index bijwerken* in the settings panel, with the last result ("124 837 items uit 364
    manifesten · 1,3 s") and broken manifests with their reasons.
- `cargo test` 68 passed (3 new), `pnpm test` 44 passed (7 new), `tsc`, `pnpm build` and `cargo
  clippy --all-targets` clean.
- Different from the story:
  - the rebuild after a sync is asked from `lib.rs` (the sync's event sink), so `runner.rs` is
    unchanged;
  - one command `index_rebuild(only_if_stale)` instead of a separate `index_stale`;
  - the stale check remembers the fingerprint in memory. The build at start sets it, so the first
    focus check compares with that.
- **In the app**: Peter's `pnpm tauri dev` was running and restarted with this code. It built
  `~/.config/media/familylightbox-index.sqlite` (53 MB) at start. A second instance with an empty
  `MEDIA_CONFIG` and no device built the index from the real working copy (124 837 items, 364
  folders) without the CLI. In dev, React's StrictMode runs the start effect twice; the second
  request is queued and builds once more, as intended.
- Still to check by hand in the app: the reload keeping the selection and the top month, a
  touched manifest rebuilding on focus, and *Index bijwerken*.
