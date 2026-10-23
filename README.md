# FamilyLightbox

Bladeren en selecteren in de gezinsfotobibliotheek. Tauri 2 (Rust) met een React-frontend;
macOS en Windows. De app is een weergave op de bestanden die de `media`-CLI beheert
(datacontract: manifesten, previews, lijsten) en heeft geen eigen waarheid.

Eerste versie: maanden en bronnen uit de lokale index, raster op de previews, één foto groot,
origineel tonen in de Finder of openen met het standaardprogramma.

## Ontwikkelen

    pnpm install
    pnpm tauri dev          # app met hot reload
    pnpm test               # frontend-tests (vitest)
    cd src-tauri && cargo test

De app leest `~/.config/media/index.sqlite` (gemaakt met `./media index`), `~/Pictures/Previews`
en `~/Pictures/LibraryLocal`; andere plekken via `MEDIA_CONFIG`, `MEDIA_PREVIEWS`, `MEDIA_LIBRARY`.
