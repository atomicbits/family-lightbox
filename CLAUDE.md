# FamilyLightbox

Desktop app (Tauri 2 + React, macOS and Windows) for browsing and selecting photos in the family
photo library. It is a view on the files the `media` CLI manages (manifests, previews, lists, as
defined in the data contract) and holds no truth of its own.

## Direction

Everything evolves into this app (architecture decision D13, 2026-10-25). On Lynn's laptop the app
is the only thing that runs, so whatever she needs is built natively in Rust, with no external
tools (no Python, exiftool, ffmpeg or rsync). Peter's workflow then moves over from the `media`
CLI part by part. Until a part has moved, the CLI and the app must both follow the data contract.
See [`docs/features/README.md`](docs/features/README.md#direction).

## Language

Everything in this repo is in **English**: README, this file, `docs/`, features and stories,
commit messages, code comments. The `media` CLI and its documentation in `~/Pictures/LibraryLocal`
are in Dutch. The data contract lives here, in [`docs/data-contract.md`](docs/data-contract.md)
(moved from LibraryLocal on 2026-10-26); contract changes are made here and the CLI follows.

## Work process

**A feature describes _what_ the app does; a story describes _how_.** Before any non-trivial code
is written or changed, there is a story under the right feature in
[`docs/features/`](docs/features/README.md). Write the story first, then implement. Conventions,
statuses, templates and exceptions are in [`docs/features/README.md`](docs/features/README.md).

Read only as deep as the task needs:

1. `~/Pictures/LibraryLocal/docs/requirements-gewenste-situatie.md` and
   `architectuur-voorstel.md`: why the app exists and where it fits (Dutch).
2. [`docs/data-contract.md`](docs/data-contract.md): the files the app reads and writes.
3. `docs/features/F<NNN>-<slug>/plan.md` and `stories/S<NN>-<slug>.md`: the work itself.
4. [`README.md`](README.md): how to run and develop the app.

## This repo is public: the leak check

Nothing personal or secret may reach GitHub (F004/S01):

- **Before every push**, run `python3 scripts/leak-check.py --push`. The `pre-push` hook runs the
  same check (`sh scripts/setup-dev.sh` sets it up), and a Claude Code hook in
  `.claude/settings.json` runs it before any `git push` an agent makes.
- **Never** push with `--no-verify` (or `-n`), and never work around a finding. On a finding,
  stop and report it to Peter.
- **Personal details and home-network details are never written here**, not even in a story
  about keeping them out. They belong in LibraryLocal (private, not in git):
  `~/Pictures/LibraryLocal/docs/features/F010-privacy-app-repo/` and `docs/nas-inrichting.md`.
  Write about them in general terms here ("an account that may not read some years").
- The list of private terms is `~/.config/family-lightbox/leak-terms.txt`, outside the repo.

## Build and test

    sh scripts/setup-dev.sh
    pnpm install
    pnpm tauri dev
    pnpm test
    cd src-tauri && cargo test
    python3 scripts/test_leak_check.py

The app builds and reads its own index (`familylightbox-index.sqlite`, feature 8); it never
modifies originals or previews.
