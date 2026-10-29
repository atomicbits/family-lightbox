# Story S02: The window opens at the size and place it had

**Feature:** `F001-browse-library`
**Status:** Done
**Depends on:** —

## Goal

FamilyLightbox opens at the size, and in the place, its window had when it was last closed.
Today it always opens at 1280 × 820. When the screen is now smaller than the saved size (a
laptop without its external monitor), the window is made to fit the screen instead of hanging
over its edges.

## Acceptance criteria

- **What is kept**: the window's size, its position, and whether it was maximized or in full
  screen, per Mac. Tauri's `tauri-plugin-window-state` keeps them: it saves them when the window
  closes and when the app quits (also with ⌘Q).
- **Before an update restarts the app** (F004/S04), the state is saved too.
- **When it opens**:
  - **the monitor**: the plugin puts the window back where it was when that place is still on a
    monitor. Otherwise macOS places it;
  - **the size**: never larger than the work area of the monitor the window is on (the screen
    without the menu bar and the Dock), and never smaller than the minimum of 800 × 500. This is
    the part the plugin doesn't do, so the app adds it;
  - **the position**: a window that would stick out of the work area at the saved position is
    moved back inside.
- **No flash**: the window is shown only after its size is set.
- **No saved state**: the window opens as today, at 1280 × 820, centered.
- **Tests**: the fitting is a pure function (window rectangle, work area, minimum → the rectangle
  to use), with unit tests for:
  - a window that fits as it is;
  - one too big for a smaller screen;
  - one partly off the screen;
  - a screen smaller than the minimum.
- **By hand**:
  - resize and move the window, quit with ⌘Q, open it again: it is where it was;
  - make it larger than the laptop screen on the external monitor, quit, open without the
    external monitor: the window fits.

## How

- `tauri-plugin-window-state = "2"`, with `skip_initial_state("main")`, so the app restores the
  main window itself in `setup`. The steps there:
  - `window.restore_state(StateFlags::all())`;
  - the fitting against `current_monitor()` (its `work_area` and scale factor);
  - then `show()`.
- `tauri.conf.json`: the main window starts with `"visible": false` and `"center": true`.
- `update_install` calls `app.save_window_state(StateFlags::all())` just before `restart()`.
- The fitting is in a small module `window.rs`, as a pure function with its tests.
- Deliberately not done here:
  - more than one window;
  - restoring the scroll position or the selected month, which is the grid's own state.

## Files affected

- `src-tauri/Cargo.toml`, `src-tauri/src/window.rs` (new), `src-tauri/src/lib.rs`,
  `src-tauri/src/update.rs`, `src-tauri/tauri.conf.json`.

## Progress log

### [2026-10-27] Written

### [2026-10-27] Built

- Peter asked to use Tauri's plugin rather than code of our own: `tauri-plugin-window-state` 2.5
  saves and restores the size, position, maximized and full screen. VISIBLE is left out, so the
  window stays hidden until it fits.
- `window.rs`: `fit()` with four tests, and `restore()`, which restores, fits to the work area
  of `current_monitor()` (or the primary), and shows. The plugin skips the main window
  (`skip_initial_state`), so the app restores it itself in `setup`.
- `update_install` saves the state before `restart()`.
- Clippy clean, 77 cargo tests green. The dev build starts and shows its window.

### [2026-10-27] Checked

- Peter, in the dev build: resized and moved the window, quit with ⌘Q, opened it again; it came
  back at the same size and place.
- The smaller screen (a size saved on the external monitor, opened on the laptop alone) is
  covered by the unit tests of `fit()`; it wasn't tried by hand.

### [2026-10-29] Saved shortly after a move or resize, too

- Peter: the size wasn't kept any more. The state file was last written on 2026-10-27: the plugin
  saves when the window closes or the app quits normally, and since then the dev build was
  mostly stopped by `tauri dev` restarting it after a Rust change, or by Ctrl-C, which ends the
  process without either. A crash or a forced quit would lose it the same way.
- `window::on_event` now also saves the state 1.5 s after the last move or resize of the visible
  window.
