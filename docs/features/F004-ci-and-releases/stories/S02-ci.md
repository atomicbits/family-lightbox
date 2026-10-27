# Story S02: CI on every push and pull request (macOS runner), including the leak check

**Feature:** `F004-ci-and-releases`
**Status:** In progress
**Depends on:** S01

## Goal

Every push to `main` and every pull request is built and tested on GitHub, so a broken build or a
failing test shows up before a release is cut (S03). The leak check of S01 runs there too, as a
second line behind the pre-push hook.

## Acceptance criteria

- **When**: on every push to `main`, on every pull request, and by hand (`workflow_dispatch`). A
  newer run on the same branch cancels the older one.
- **Where**: `macos-14` (Apple Silicon), since the app has macOS-only code (mounts, NetFS,
  `statvfs`). GitHub-hosted runners cost nothing for a public repo.
- **Steps**, each failing the run:
  1. `pnpm install --frozen-lockfile`;
  2. `pnpm test` (vitest);
  3. `pnpm build` (`tsc` and `vite build`; the Rust build embeds `dist/`, so this comes first);
  4. `cargo clippy --all-targets -- -D warnings`;
  5. `cargo test`;
  6. `python3 scripts/test_leak_check.py`;
  7. the leak check over the whole history (`scripts/leak-check.py --history`), with `gitleaks`
     from Homebrew and the list of private terms from the repository secret `LEAK_TERMS`.
- **The leak check and forks**: GitHub gives no secrets to a pull request from a fork, so the
  check is skipped there (and says so); the pre-push hook and the check after the merge to `main`
  still hold. On a push to `main`, a missing `LEAK_TERMS` fails the run (fail closed).
- **Nothing is printed** that the check found: the script never prints a term, and GitHub masks
  the secret.
- **Fast enough**: Rust and pnpm caches, so a run with a warm cache takes a few minutes.
- **Permissions**: the workflow only reads (`contents: read`).
- **By hand**: the first run on `main` is green; a run where a test fails goes red (on a
  throwaway branch with a pull request, then closed and deleted).

## How

- `.github/workflows/ci.yml`, after the shape of `wikai`'s `ci.yml` and `release.yml` (pinned
  major versions of `actions/checkout`, `pnpm/action-setup`, `actions/setup-node`,
  `dtolnay/rust-toolchain`, `Swatinem/rust-cache`; `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24`).
- The secret is set once by Peter's agent with `gh secret set LEAK_TERMS <
  ~/.config/family-lightbox/leak-terms.txt`, and again whenever the list changes. The runbook of
  S03 says so; until then the README's development section does.
- Deliberately not done here: building the app bundle and signing (S03); Windows and Linux.

## Files affected

- `.github/workflows/ci.yml` (new).
- `README.md` (development: CI and the secret).

## Progress log

### [2026-10-27] Written

### [2026-10-27] Built

- `.github/workflows/ci.yml` as in the story, and the README's development section.
- First run on `main` (cold cache, 2 min 47 s): every step green up to and including the leak check
  tests; the leak check itself failed with "the repository secret LEAK_TERMS is missing", as
  intended while the secret isn't set (fail closed).
- Setting the secret was refused to the agent by Claude Code's permission check (a write to a
  secret store), so Peter sets it himself, as the README says.
- GitHub notes that `actions/checkout@v4`, `actions/setup-node@v4` and `pnpm/action-setup@v4` still
  ship Node 20 internals; they run forced on Node 24. Harmless; newer majors can follow later.
- Still to do: a green run once `LEAK_TERMS` is set, and the red run from a failing test on a
  throwaway pull request.
