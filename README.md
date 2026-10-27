# FamilyLightbox

Browse and select photos in the family photo library. Tauri 2 (Rust) with a React frontend;
macOS and Windows. The app is a view on the files managed by the `media` CLI (data contract:
manifests, previews, lists) and holds no truth of its own.

First version: months and sources from the local index, a grid of previews, one photo large,
reveal the original in the Finder or open it with the default application.

## Development

    sh scripts/setup-dev.sh # once per clone: the leak check before every push
    pnpm install
    pnpm tauri dev          # app with hot reload
    pnpm test               # frontend tests (vitest)
    cd src-tauri && cargo test
    python3 scripts/test_leak_check.py

The app builds its own index, `~/.config/media/familylightbox-index.sqlite`, from the manifests in
`~/Pictures/LibraryLocal`, and reads the previews in `~/Pictures/Previews`; other locations via
`MEDIA_CONFIG`, `MEDIA_PREVIEWS`, `MEDIA_LIBRARY`.

**This repo is public.** Every push goes through `scripts/leak-check.py` (a `pre-push` hook, set
up by `scripts/setup-dev.sh`): it refuses a push that adds a secret (found by `gitleaks`) or a
private term (from a list kept outside the repo, `~/.config/family-lightbox/leak-terms.txt`).
`scripts/leak-check.py --tree` and `--history` check the whole tree or history. Never push with
`--no-verify`.

CI (`.github/workflows/ci.yml`, F004/S02) runs the tests, clippy and the leak check over the whole
history on every push to `main` and every pull request, on macOS. It gets the list of private
terms from the repository secret `LEAK_TERMS`; after a change to the list, set it again with
`gh secret set LEAK_TERMS < ~/.config/family-lightbox/leak-terms.txt`.

## How work is done

Features and stories live in [`docs/features/`](docs/features/README.md): a story is written
before it is implemented. See [`CLAUDE.md`](CLAUDE.md).
