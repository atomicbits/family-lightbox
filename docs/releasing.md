# Releasing FamilyLightbox

How a release is made, signed and checked (F004/S03). A release is a pushed tag `vX.Y.Z`:
`.github/workflows/release.yml` builds the app for Apple Silicon, signs it with the Developer ID
of **Atomic Bits**, has Apple notarize it, and publishes the `.dmg` on a GitHub Release of this
repo. The repo is public, so the release and its download are public too.

## Secrets

Repository secrets on `atomicbits/family-lightbox`. Only their names are written down anywhere.
An agent may not set them; Peter does, with `gh secret set <NAME> -R atomicbits/family-lightbox`.

| Secret | What | Where it comes from |
|---|---|---|
| `APPLE_CERTIFICATE` | base64 of the Developer ID Application `.p12` | the same certificate `wikai` uses; export it again from the Keychain if needed: Keychain Access → "Developer ID Application: Atomic BITS bvba (…)" → Export, then `base64 -i <file>.p12 \| gh secret set APPLE_CERTIFICATE -R atomicbits/family-lightbox` |
| `APPLE_CERTIFICATE_PASSWORD` | the `.p12`'s export passphrase | chosen at export |
| `APPLE_SIGNING_IDENTITY` | `Developer ID Application: Atomic BITS bvba (<team id>)`, verbatim | Keychain Access |
| `APPLE_ID` | the company Apple ID | as for `wikai` |
| `APPLE_PASSWORD` | an app-specific password for `notarytool` | appleid.apple.com → App-Specific Passwords; one per app is clearer than sharing `wikai`'s |
| `APPLE_TEAM_ID` | the Apple Developer team ID | developer.apple.com → Membership |
| `LEAK_TERMS` | the private terms list of the leak check (F004/S01) | `gh secret set LEAK_TERMS -R atomicbits/family-lightbox < ~/.config/family-lightbox/leak-terms.txt`, again after every change to the list |

Renewals: the Developer ID certificate is valid for five years; the Apple Developer Program is
yearly. Both are shared with `wikai`, whose `SIGNING.md` keeps the dates.

## Before a release

1. **Apple's agreements.** When Apple revises the Developer Program License Agreement, notarization
   fails until it is accepted, while the secrets and the workflow look healthy. Check the
   Apple Developer app (or developer.apple.com/account) for a banner first.
2. **Green CI on `main`** for the commit to release.
3. **A clean working tree** on `main`, up to date with `origin/main`.
4. **Choose the version**: pre-1.0, `0.MINOR.PATCH`. Peter decides.

## Cut a release

```sh
scripts/bump-version.py 0.2.0      # sets the three versions and Cargo.lock, commits, tags v0.2.0
git push origin main v0.2.0        # through the leak check, like every push
gh run watch -R atomicbits/family-lightbox   # the release run
```

The `build` job checks, before anything is published:

- the tag equals the version in `tauri.conf.json`, `Cargo.toml` and `package.json`;
- the leak check over the whole history;
- the app: `codesign --verify --deep --strict`, `spctl` accepting it as a notarized Developer ID
  app, and the stapled ticket;
- exactly one `.dmg`, itself notarized and stapled too (`spctl --type open` accepting it as a
  notarized Developer ID disk image): tauri-action only notarizes the app inside.

The `publish` job then creates the GitHub Release and uploads the `.dmg`.

## A dry run

*Actions → release → Run workflow* (or `gh workflow run release.yml -R atomicbits/family-lightbox`)
runs the same build and checks and publishes nothing; the `.dmg` is a workflow artifact for 14
days. Without the `APPLE_*` secrets a dry run builds unsigned (with a warning); a tag never does.

## Verify a release on a Mac

```sh
spctl --assess --type execute --verbose=4 /Applications/FamilyLightbox.app
#   accepted, source=Notarized Developer ID
codesign -dvv /Applications/FamilyLightbox.app 2>&1 | grep -E 'Authority|TeamIdentifier'
xcrun stapler validate /Applications/FamilyLightbox.app
```

And open it by double-click: no Gatekeeper warning.

## When something fails

- **Notarization rejected**: the run prints a submission id. `xcrun notarytool log <id> --apple-id
  <Apple ID> --team-id <team id> --password <app-specific password>` gives the reason per file.
- **`401 Invalid credentials`** at notarization: `APPLE_ID` and `APPLE_PASSWORD` don't belong
  together. The password must be an app-specific password made under that same Apple ID (the
  company one, member of the team), not the account password. Test both locally with `xcrun
  notarytool store-credentials <profile> --apple-id <Apple ID> --team-id <team id>`, which asks
  for the password and says "Success" when they work; then set both secrets again.
- **Notarization refused for every submission** while nothing changed: an Apple agreement to accept
  (see above).
- **The `publish` job failed on the upload**: rerun only that job (`gh run rerun <id> --failed`).
  It adds to an existing release instead of making a second one.
- **The tag was wrong**: delete the release and the tag (`gh release delete vX.Y.Z --cleanup-tag`),
  fix, and tag again. Never reuse a version that reached Lynn.
