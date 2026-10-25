# FamilyLightbox

Browse and select photos in the family photo library. Tauri 2 (Rust) with a React frontend;
macOS and Windows. The app is a view on the files managed by the `media` CLI (data contract:
manifests, previews, lists) and holds no truth of its own.

First version: months and sources from the local index, a grid of previews, one photo large,
reveal the original in the Finder or open it with the default application.

## Development

    pnpm install
    pnpm tauri dev          # app with hot reload
    pnpm test               # frontend tests (vitest)
    cd src-tauri && cargo test

The app reads `~/.config/media/index.sqlite` (created by `./media index`), `~/Pictures/Previews`
and `~/Pictures/LibraryLocal`; other locations via `MEDIA_CONFIG`, `MEDIA_PREVIEWS`, `MEDIA_LIBRARY`.

## How work is done

Features and stories live in [`docs/features/`](docs/features/README.md): a story is written
before it is implemented. See [`CLAUDE.md`](CLAUDE.md).
