# Features and stories

The working index for FamilyLightbox: which features exist, which stories belong to them and how
far along they are. The *why* and the overall design of the photo workflow live outside this repo,
in the `media` CLI's documentation (in Dutch): `~/Pictures/LibraryLocal/docs/`
(`requirements-gewenste-situatie.md`, `architectuur-voorstel.md`). The data contract that this app
and the CLI follow is in this repo: [`../data-contract.md`](../data-contract.md).

## The model in one line

**A feature describes _what_ the app does; a story describes _how_ that is (or was) realized.**
Changing or extending a feature means adding a new story that describes how the change will be
built. Code is never silently changed against a design that isn't written down.

**This is a hard rule: before any non-trivial code is written or changed, there is a story under
the right feature describing how.** Write the story first, then implement against it. Exceptions
are listed at the bottom.

## Language

Everything in this repo is written in **English**: README, `CLAUDE.md`, features, stories, commit
messages and code comments. (The `media` CLI and its docs in `~/Pictures/LibraryLocal` are in
Dutch and follow the same process there.)

## Layout and naming

```
docs/features/
  README.md                         this file: process and index
  F001-<slug>/
    plan.md                         goal, scope, acceptance criteria, list of stories
    stories/
      S01-<slug>.md                 one file per story
```

- **Feature folder**: `F<NNN>-<slug>`, e.g. `F001-browse-months`. Numbers are stable and never
  reused; a dropped feature keeps its number and the next one gets a new number.
- **Story file**: `S<NN>-<slug>.md`. IDs are stable within a feature; stories are not renumbered
  when one is dropped.
- **A feature folder is not a dumping ground.** If something is too small for a story, fold it
  into an existing story or just do it.
- **Needs from the CLI.** When a story needs a change to the `media` CLI or the data contract,
  that change gets its own story in `~/Pictures/LibraryLocal/docs/features/` (in Dutch); the
  story here links to it and depends on it.

## Status

| Status | Meaning |
|---|---|
| **Planned** | described, not started |
| **In design** | feature folder exists, `plan.md` being written |
| **In progress** | being built |
| **Done** | built, tested, meets its acceptance criteria |
| **Blocked** | waiting on something else (say what) |
| **Deferred** | deliberately later |
| **Dropped** | no longer needed (say why) |

**Status lives in three places (the story, `plan.md`, the table below), and the code is the only
source of truth.** A status line is a *claim* about the code. When it disagrees with the code,
believe the code and fix the line before continuing. Prefer recording what is genuinely still
open over a bare **Done**: "built and unit-tested, not yet checked in the running app" is worth
more than a tick.

## Process

1. Pick the next feature from the table below, or add a new row.
2. **Verify the status** in the story and in the code it names. Statuses drift.
3. If the feature folder doesn't exist yet, create it and write `plan.md` first. Get Peter's
   review before writing out the stories.
4. Write the story (template below) **before** implementing it.
5. Implement one story at a time. If something fits no existing story, propose a new one rather
   than quietly widening the one in progress.
6. Keep the documents current while working: the progress log in the story; on completion the
   status in the story, in `plan.md` and in the table below, and the README if usage changed.

### When the work doesn't fit this structure

Small bug fixes, doc tweaks and throwaway experiments don't need a feature or story. Rule of
thumb: under an hour and no change to the architecture or to what the app reads or writes, just
do it. If it's bigger or changes the shape of the system, put it in the right document first.

## Template `plan.md`

```markdown
# Feature NNN: <title>

**Status:** In design
**Depends on:** —
**Folder:** `FNNN-<slug>`

## Goal
What Peter or Lynn can do afterwards that they can't today.

## Scope
### In scope
### Out of scope

## Acceptance criteria
Concrete and checkable.

## Stories
| ID | Title | Status | Depends on |
|---|---|---|---|
```

## Template story

```markdown
# Story SNN: <title>

**Feature:** `FNNN-<slug>`
**Status:** Planned
**Depends on:** —

## Goal

## Acceptance criteria
- concrete, checkable (a test, a command, an observable result)

## How
The approach: which modules, which rules from the data contract, what is deliberately not done.

## Files affected
- `src-tauri/src/…` — …
- `src/…` — …

## Progress log
### [YYYY-MM-DD] …
```

## Direction

Architecture decision D13 (2026-10-25, in `~/Pictures/LibraryLocal/docs/architectuur-voorstel.md`):
**everything evolves into this app, and on Lynn's laptop the app is the only thing that runs.** No
scripts, Python, exiftool, ffmpeg or rsync there. What Lynn needs (features 3 and 6–9) is built
natively in Rust first; then Peter's workflow moves over from the `media` CLI part by part
(feature 10). The CLI and the app follow the same data contract during the transition; a part
is removed from the CLI once the app does it on real data. What runs on the NAS itself
(`media-drive-push.py`, `media-manifest-nas.py`) stays there.

Lynn uses macOS, so the native features are built for macOS first; Windows follows. The data
contract moved into this repo, in English, with feature 7 (2026-10-26):
[`../data-contract.md`](../data-contract.md). LibraryLocal stays for Peter's manual CLI work.

## Index

Feature 1 was built before this process. Its `plan.md` was written afterwards for completeness and
has no stories; any change to it starts with a new story. Rows without a link are planned and have
no folder yet.

| # | Feature | Status | Depends on |
|---|---|---|---|
| 1 | [Browse the library](F001-browse-library/plan.md) | Done | CLI `index`, `previews` |
| 2 | Filter by source (camera, Peter's or Lynn's iPhone) | Planned | 1 |
| 3 | [Lists](F003-lists/plan.md): create and fill lists together, native (data contract section 5) | In progress | 6, 8 |
| 4 | [CI and signed releases](F004-ci-and-releases/plan.md), macOS first (reusing Atomic Bits' signing from `wikai`) | In progress (S01–S03 done; S04 built, v0.2.0 released) | 1 |
| 5 | Start imports from the app via the CLI's `--json` output | Dropped: imports move into the app natively (D13), see 10 | — |
| 6 | [Connect to the NAS](F006-nas-connection/plan.md): find or mount the shares (macOS first), register the device | Done on Peter's Mac; Lynn's Mac deferred | 1 |
| 7 | [Sync manifests and previews from the NAS](F007-sync-from-nas/plan.md) (replaces passes 1–2 of `media pull`; moves the data contract here) | In progress | 6 |
| 8 | [Native index](F008-native-index/plan.md): build the app's own index from the manifests (replaces `media index` for the app) | In progress | 7 |
| 9 | Open originals that are not on this laptop, from the NAS | Planned | 6 |
| 10 | Peter's workflow in the app, one feature per part when it is picked up: import, push, previews, doctor, thin, Drive upload status | Planned | 8 |
| 11 | [Languages](F011-languages/plan.md): English by default, Dutch for us; every UI text in both | Planned (after 4) | — |
| 12 | [Make missing previews](F012-make-missing-previews/plan.md) in the app, natively, from the local copy or the NAS, also onto the NAS (the first part of 10) | In progress (S01–S03 done, S04 built) | 6, 7, 8 |
