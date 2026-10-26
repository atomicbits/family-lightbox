# Story S01: Move the data contract into this repo, in English

**Feature:** `F007-sync-from-nas`
**Status:** Planned
**Depends on:** —

## Goal

The data contract (folders, manifests, previews, lists, exports, local state, transport, what
"safe" means, the rules for `thin`) lives in this repo as `docs/data-contract.md`, in English,
because the app is becoming the implementation everything moves towards (D13). From now on the
contract is changed here and the `media` CLI follows it. The Dutch file in LibraryLocal becomes a
pointer.

## Acceptance criteria

- **`docs/data-contract.md`**: a faithful English translation of
  `~/Pictures/LibraryLocal/docs/datacontract.md` as of the day of the move, with:
  - **the same section numbering** (1–12, and 3.1–3.5, 5.1–5.4), so references such as "data
    contract 5.1" in code, stories and the CLI stay valid;
  - **the RFC keywords** `MOET`, `MAG NIET`, `MAG`, `ZOU MOETEN` rendered as `MUST`, `MUST NOT`,
    `MAY`, `SHOULD`, with the same meaning;
  - **examples unchanged**: JSON, JSONL, paths, folder names and list names such as
    `2026/herfstwandeling` or `MM.<naam>` are data, not prose, so they stay as they are;
  - **the change log** translated, in the same order, plus a new entry for the move (the content
    of the contract doesn't change, so `contract_version` stays `1.2`).
- **Public repo, operational details**: this repo is public, so the English contract keeps the
  rules and leaves out the operational details of Peter's network, which go to a private note in
  LibraryLocal (`docs/nas-inrichting.md`, Dutch). Specifically:
  - the `~/.ssh/config` block, the key file name and the host alias go to the note. The contract
    keeps that the CLI's bulk transport uses SSH with a key and must work non-interactively;
  - the exact `[private]` command and which account is denied which years go to the note. The
    contract keeps that access per user is restricted on the NAS itself (per year folder, with
    ACLs), never in the app, and that unreadable year folders are skipped;
  - the name of Peter's Drive mirror share (`[private]`) and its Cloud Sync
    exclusions go to the note. The contract keeps that a separate mirror exists and excludes
    `Library`, `Lists` and `Exports`;
  - the account names `peter` and `lynn`, the device names `mbp-peter` and `mbp-lynn`, and the NAS
    name `raven` stay. They are already in this repo's docs, and the examples need concrete
    names.
  Nothing that a component needs to follow the contract is lost; every moved detail is in the
  note, and the contract says such a note exists outside the repo.
- **The Dutch file** `~/Pictures/LibraryLocal/docs/datacontract.md` is replaced by a short Dutch
  pointer: where the contract now lives, that its section numbers are unchanged, that the
  operational details are in `nas-inrichting.md`, and that the last Dutch version is kept as
  `datacontract-nl-2026-10-26.md` for reference, not maintained.
- **References** updated:
  - in this repo: `CLAUDE.md`, `README.md`, `docs/features/README.md` and the feature plans and
    stories that point to the Dutch file;
  - in LibraryLocal: `CLAUDE.md`, `README.md`, `docs/features/README.md` and the CLI feature plans,
    and the architecture proposal where it names the contract's location.
  The CLI's code comments ("datacontract sectie 10") stay as they are, because the section numbers
  didn't change.
- **Check**: a script compares the two versions section by section. Every heading number in the
  Dutch file exists in the English one, the number of table rows per section and the code blocks
  are the same, and every code block is byte-identical apart from the moved operational details.
  It runs once, and its result goes in the progress log. It is not kept in the repo.

## How

- Translate section by section into `docs/data-contract.md`, then run the comparison.
- Write `nas-inrichting.md` in LibraryLocal with the moved details, verbatim from the Dutch
  contract (it is private).
- Keep the old Dutch text as `datacontract-nl-2026-10-26.md`, then replace `datacontract.md` with
  the pointer.
- Then update the references.
- Deliberately not done here: any change to the rules themselves. Corrections found while
  translating are listed in the progress log as proposals, not applied.

## Files affected

- This repo: `docs/data-contract.md` (new), `CLAUDE.md`, `README.md`, `docs/features/README.md`,
  `docs/features/F00*/…` where they point to the contract.
- LibraryLocal (not a git repo): `docs/datacontract.md` (pointer), `docs/datacontract-nl-2026-10-26.md`
  (new), `docs/nas-inrichting.md` (new), `CLAUDE.md`, `README.md`, `docs/features/README.md`,
  `docs/features/F00*/plan.md`, `docs/architectuur-voorstel.md`.

## Progress log

### [2026-10-26] Written
