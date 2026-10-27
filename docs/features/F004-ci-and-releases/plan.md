# Feature 4: CI and signed releases (macOS first)

**Status:** In progress (S01 and S02 done)
**Depends on:** Feature 1
**Folder:** `F004-ci-and-releases`

## Goal

Lynn installs FamilyLightbox on her Mac from a signed, notarized download, without warnings from
macOS and without Peter building it on her machine, and later versions reach her on their own.
Every push is checked by CI, so what reaches her has passed the tests. With this, the checks that
features 6, 7 and 8 left for Lynn's Mac can finally be done.

## Background

- **Today** the app only runs from source, with `pnpm tauri dev` on Peter's Mac. There is no CI
  and no release; `tauri.conf.json` is at version 0.1.0 with identifier
  `io.atomicbits.familylightbox`.
- **What already exists at Atomic Bits**: the private repo `atomicbits/wikai` (checked out at
  `~/Books`) releases a Tauri 2 app the same way this one needs to, in
  `.github/workflows/release.yml`, with its runbook in `SIGNING.md` and `RELEASES.md` and the
  background in `docs/features/F029-code-signing/` and `F028-auto-update/`:
  - **macOS**: an Apple Developer ID Application certificate of **Atomic Bits**. `tauri-action`
    imports the `.p12` into a temporary keychain, signs the `.app`, notarizes it with
    `notarytool`, staples the ticket and rebuilds the `.dmg`. It activates when the six
    `APPLE_*` secrets are present (`APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`,
    `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`);
  - **Windows**: Azure Trusted Signing through `bundle.windows.signCommand` during bundling, with
    `artifact-signing-cli` and six `AZURE_*` secrets;
  - **updates**: the Tauri updater with an Ed25519 key of its own, a `latest.json` built from
    the `.sig` files, and publishing on Cloudflare R2.
  Lessons in that workflow worth keeping: build without uploading, then assemble and publish in
  separate jobs that can be retried on their own; check the number of bundle files; a dry-run
  input; accept Apple's revised agreements before a release, or notarization fails while
  everything looks healthy.
- **The secrets are on the `wikai` repo, not on the organization**, and GitHub never shows a
  secret's value again. Reusing them means setting the same values on this repo from where Peter
  keeps them (password manager), or moving them to organization secrets (open point 1). The
  certificate and its private key are in Peter's Keychain (`wikai` F029), so the `.p12` can be
  exported again if the backup isn't at hand: Keychain Access → "Developer ID Application: Atomic
  Bits (…)" → Export, then `base64 -i <file>.p12 | gh secret set APPLE_CERTIFICATE`.
- **This repo is public.** GitHub-hosted runners, macOS included, cost nothing for a public repo,
  so the 10× macOS billing that made `wikai` turn its CI off doesn't apply. Releases and their
  downloads are public too. Secrets are never given to workflows from forks.

## Scope

### In scope

- **A leak check before every push** (Peter, 2026-10-27): because the repo is public, nothing
  is pushed before a check over what is being pushed finds no secrets (keys, tokens, passwords,
  certificates) and none of our private terms (personal names beyond the agreed ones, e-mail
  addresses, home-network details). It runs as a git `pre-push` hook, it is required for an
  agent doing a push, and CI repeats it as a second line. The list of private terms lives outside
  the repo, because listing them in the repo would leak them. What already leaked is cleaned up.
- **CI on every push and pull request**, on a macOS runner (the app has macOS-only code):
  `cargo test`, `cargo clippy --all-targets -D warnings`, `pnpm test`, `tsc`, `pnpm build`, and
  the leak check.
- **A signed and notarized macOS release**, Apple Silicon only (`aarch64-apple-darwin`; Lynn's Mac
  is Apple Silicon), when a `v*.*.*` tag is pushed, reusing Atomic Bits'
  Developer ID and the shape of `wikai`'s `release.yml`: a `.dmg`, on a GitHub Release of this
  repo. One script bumps the version in `tauri.conf.json`, `Cargo.toml` and `package.json`.
- **Updates on their own**: the Tauri updater with a key of this app's own, `latest.json` on the
  latest GitHub Release (public, so no R2 or Worker is needed). The app checks at start and offers
  *Bijwerken* in the header; it never updates without being asked.
- **A runbook**: `docs/releasing.md` with the secret names (never values), the steps of a
  release, how to verify one (`spctl`, `codesign`), and the agreements to accept. What is private
  (where the key backups live) stays out of this public repo.
- **The first install on Lynn's Mac**, and the checks features 6, 7 and 8 left for it.

### Out of scope

- **Windows** releases. The Azure Trusted Signing of `wikai` can be reused the same way later; it
  gets its own story once Windows is picked up.
- **Linux.**
- **The App Store** and its sandbox. A Developer ID app outside the store is what's needed.
- **Release notes in the app**, and update channels (beta, stable).

## Acceptance criteria

- A push that adds a secret or a private term is refused before anything leaves the Mac, also
  when an agent pushes; the message says which file and line, without repeating a secret. A push
  that removes one goes through. The current tree contains none of them.
- A push to `main` runs CI, and a failing test or a clippy warning makes it red.
- Pushing a tag `vX.Y.Z` produces a GitHub Release with a `.dmg` for which, on a clean Mac,
  `spctl --assess --type execute FamilyLightbox.app` says `accepted source=Notarized Developer ID`
  and `codesign -dvv` shows Atomic Bits' team ID. Opening it gives no Gatekeeper warning.
- No secret value appears in the repo, in the logs or in the runbook.
- An installed version finds a newer release, offers *Bijwerken*, and after a restart runs the
  new version. A release whose update signature doesn't match is refused.
- On Lynn's Mac, installed from the release: the device setup, connecting to the NAS with her
  account, the first sync with its confirmation, the years her account may not read skipped without an error,
  the index built, and the grid browsing. The results go into the stories of features 6–8.

## Stories

| ID | Title | Status | Depends on |
|---|---|---|---|
| [S01](stories/S01-leak-check.md) | Leak check before every push: secrets and private terms, git hook, required for agents, current tree cleaned | Done | — |
| [S02](stories/S02-ci.md) | CI on every push and pull request (macOS runner), including the leak check | Done | S01 |
| [S03](stories/S03-signed-release.md) | Signed and notarized macOS release on a version tag, with the version script and the runbook | In progress | S02 |
| S04 | Updates: the Tauri updater with its own key, `latest.json` on GitHub Releases, *Bijwerken* in the app | Planned | S03 |
| S05 | First install on Lynn's Mac, and the checks left by features 6–8 | Planned | S03 |

## Decisions and open points

Peter's answers on 2026-10-27:

1. **Apple secrets as repository secrets** on `family-lightbox`, with the same values as on
   `wikai`. On condition that every push is checked for leaks first (S01).
2. **Apple Silicon only**: `aarch64-apple-darwin`, no universal build.
3. **Public downloads are fine**, with the same condition: a pre-push leak check that an agent
   must run when it pushes (S01).
4. **A separate Tauri update key** for this app, not `wikai`'s.
5. **Updates are part of this feature** (S04).

Decided with S01 (details in LibraryLocal F010/S01): which names may appear, the author of new
commits, and rewriting the public history, which was done on 2026-10-27.
