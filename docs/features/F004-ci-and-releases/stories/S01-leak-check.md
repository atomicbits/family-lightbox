# Story S01: Leak check before every push

**Feature:** `F004-ci-and-releases`
**Status:** Planned
**Depends on:** —

## Goal

This repo is public, and with S03 it gets the signing secrets in its workflows. Nothing may reach
GitHub that doesn't belong there: no secrets, and none of the family's private details. Before
every push, a check over what is being pushed refuses it when it finds one. The check runs for
Peter's own pushes and is required for an agent that pushes. What already leaked is cleaned up.

## Background

- **The private side of this story** is in LibraryLocal, which is not published:
  `~/Pictures/LibraryLocal/docs/features/F010-privacy-app-repo/` (Dutch). It holds the private
  terms, the leaks already found with their files and lines, and Peter's decisions about names
  and about the history. This story only describes the mechanism and never names a term.
- **What leaked so far**, in general terms: 12 lines in the current tree (in the docs of features
  4, 6 and 7, and in `Cargo.toml`) name home-network details or a personal detail that should
  have stayed in the private note. Some of them are in F007/S01, the very story that was meant
  to keep them out. They have been in the public history since 2026-10-26. The commit author
  identity of every past commit is public too.
- **Tools**: `gitleaks` (Homebrew, 8.30.1) knows hundreds of secret formats (private keys, GitHub
  and cloud tokens, base64 certificates). The private terms need a list of our own.

## Acceptance criteria

- **What is checked**: for every ref being pushed, the commits that aren't on the remote yet:
  - the **added lines** of their diffs. Removed lines don't count, so a push that cleans up a
    leak goes through;
  - the **file names**, and the **commit messages**;
  - for a new branch, the commits that aren't on any remote branch.
- **Secrets**: `gitleaks` over the same commits, with its default rules. A finding names the
  commit, the file and the line, and the rule, never the secret itself.
- **Private terms**: a list of case-insensitive patterns, one per line with `#` comments, at
  `~/.config/family-lightbox/leak-terms.txt`. It is **outside the repo**, because the list is
  exactly what may not leak. The story starts it with:
  the terms of LibraryLocal F010/S01: home-network details from the private NAS note, and
  personal details (names, e-mail addresses). An allow list in the same file (lines starting
  with `allow:`) lets an agreed term through where it is needed, such as the account and device
  names this repo already uses.
- **Fail closed**: without `gitleaks` or without the list, the push is refused with what to
  install or create. There is no "skip" option. `git push --no-verify` is not used (see agents).
- **The git hook**: `.githooks/pre-push`, in the repo, calls `scripts/leak-check.py` with the
  refs git passes it. `scripts/setup-dev.sh` sets `core.hooksPath` to `.githooks` once per clone.
  The README's development section says so.
- **Agents**:
  - `CLAUDE.md` states the rule: an agent runs `scripts/leak-check.py` before every push, never
    passes `--no-verify`, and on a finding stops and reports to Peter instead of working around
    it;
  - a Claude Code hook in `.claude/settings.json` (`PreToolUse` on Bash) runs the check before
    any command containing `git push` and blocks it on a finding, or when `--no-verify` or
    `-n` is in the command. That way it holds even in a clone where `core.hooksPath` isn't set.
- **The whole tree** can be checked with `scripts/leak-check.py --tree` (for the clean-up and for
  CI), and the whole history with `--history`.
- **Clean-up**:
  - the 12 lines (listed in LibraryLocal F010/S01) are rewritten to say what matters without
    the detail: an account that may not read some years, a separate Drive mirror, an ACL tool on
    the NAS. The details stay in the private note;
  - `--tree` finds nothing afterwards;
  - `--history` is run once, and its findings go to LibraryLocal F010/S01, not here. Rewriting
    the history (with `git filter-repo` and a force push of `main`) only happens if Peter asks
    for it;
  - `authors` in `Cargo.toml` and the author identity of new commits follow Peter's decision in
    F010/S01; with that, the check also looks at the author of the commits being pushed.
- **Tests** (`scripts/test_leak_check.py`, on throwaway git repos in a temporary folder):
  - a private key or token in an added line is refused, and the message doesn't contain it;
  - a private term in an added line, in a file name and in a commit message is refused;
  - an allowed term passes; the same term matched by a broader private pattern still passes only
    where the allow line covers it;
  - removing a line with a term passes;
  - a missing list or a missing `gitleaks` refuses;
  - a new branch checks only the commits that aren't on a remote branch;
  - a clean push passes.
- **By hand**: a real push of this story through the hook, and a deliberately bad commit on a
  throwaway branch, refused by both the git hook and the Claude Code hook, then dropped.

## How

- `scripts/leak-check.py` (Python 3, standard library only; dev tooling on Peter's Mac, never on
  Lynn's): reads the pre-push lines from stdin, works out the ranges, runs `git log -p
  --format=…` and `gitleaks git --log-opts=<range> --redact --no-banner`, and prints findings as
  `<commit> <file>:<line>: <rule or "private term">`.
- `.githooks/pre-push`, `scripts/setup-dev.sh`.
- `.claude/settings.json` with the `PreToolUse` hook, which calls the same script (in `--push`
  mode, working out the ranges from the current branch and its upstream).
- `CLAUDE.md` and `README.md`.
- Deliberately not done here: running it in CI (S02, which gets the list as a repository secret);
  a hook on every commit (the push is what makes something public).

## Open point

Which names may appear in this repo, and what happens with the public history, is decided in
LibraryLocal F010/S01. Until then the agreement of F007/S01 holds: the account names, the device
names and the NAS name stay.

## Files affected

- `scripts/leak-check.py`, `scripts/test_leak_check.py`, `scripts/setup-dev.sh` (new).
- `.githooks/pre-push`, `.claude/settings.json` (new).
- `CLAUDE.md`, `README.md`, `src-tauri/Cargo.toml` (authors, per F010/S01).
- The 12 lines in `docs/features/F004…`, `F006…`, `F007…` (clean-up).

## Progress log

### [2026-10-27] Written
