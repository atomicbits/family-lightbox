# Feature 4: CI and signed releases (macOS first)

**Status:** In design
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
- **What already exists at Atomic Bits**: the private repo `atomicbits/wikai` releases a Tauri 2
  app the same way this one needs to, in `.github/workflows/release.yml`, with its runbook in
  `SIGNING.md` and `RELEASES.md`:
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
  keeps them (password manager), or moving them to organization secrets (open point 1).
- **This repo is public.** GitHub-hosted runners, macOS included, cost nothing for a public repo,
  so the 10× macOS billing that made `wikai` turn its CI off doesn't apply. Releases and their
  downloads are public too. Secrets are never given to workflows from forks.

## Scope

### In scope

- **CI on every push and pull request**, on a macOS runner (the app has macOS-only code):
  `cargo test`, `cargo clippy --all-targets -D warnings`, `pnpm test`, `tsc` and `pnpm build`.
- **A signed and notarized macOS release** when a `v*.*.*` tag is pushed, reusing Atomic Bits'
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

- A push to `main` runs CI, and a failing test or a clippy warning makes it red.
- Pushing a tag `vX.Y.Z` produces a GitHub Release with a `.dmg` for which, on a clean Mac,
  `spctl --assess --type execute FamilyLightbox.app` says `accepted source=Notarized Developer ID`
  and `codesign -dvv` shows Atomic Bits' team ID. Opening it gives no Gatekeeper warning.
- No secret value appears in the repo, in the logs or in the runbook.
- An installed version finds a newer release, offers *Bijwerken*, and after a restart runs the
  new version. A release whose update signature doesn't match is refused.
- On Lynn's Mac, installed from the release: the device setup, connecting to the NAS with her
  account, the first sync with its confirmation, the years before [private] skipped without an error,
  the index built, and the grid browsing. The results go into the stories of features 6–8.

## Stories

| ID | Title | Status | Depends on |
|---|---|---|---|
| S01 | CI on every push and pull request (macOS runner) | Planned | — |
| S02 | Signed and notarized macOS release on a version tag, with the version script and the runbook | Planned | S01 |
| S03 | Updates: the Tauri updater with its own key, `latest.json` on GitHub Releases, *Bijwerken* in the app | Planned | S02 |
| S04 | First install on Lynn's Mac, and the checks left by features 6–8 | Planned | S02 |

## Open points

1. **Where the Apple secrets live.** My proposal is repository secrets on `family-lightbox` with
   the same values as on `wikai`, set by Peter with `gh secret set` from his password manager.
   The alternative is organization secrets shared by both repos, so a renewal happens in one
   place. Whether a Free organization can give organization secrets to a private repo like
   `wikai` is to be checked first; if not, that option falls away.
2. **Lynn's Mac: Apple Silicon or Intel?** Apple Silicon only means `aarch64-apple-darwin`, as
   `wikai` builds. With an Intel Mac, a universal build (`universal-apple-darwin`) is needed,
   which takes about twice as long.
3. **Public downloads.** A public repo means anyone can download the `.dmg`. The app contains no
   private data (the home-network details are kept out, as agreed), so I see no problem, but it
   is worth saying out loud.
4. **A separate update key**, not `wikai`'s: losing or leaking one then never touches the other
   product. The new key is backed up the same way as `wikai`'s (`SIGNING.md`, "Bus factor").
5. **Updates in this feature or later.** Without S03, every new version means Peter installing a
   `.dmg` on Lynn's Mac by hand. I propose keeping S03 in, after S04 if Lynn's tests are more
   urgent.
