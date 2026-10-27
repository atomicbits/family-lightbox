//! The main window opens at the size and place it had (F001/S02). `tauri-plugin-window-state`
//! saves and restores it; this module adds what the plugin doesn't do: a window saved on a larger
//! screen is made to fit the screen it opens on.

use tauri::{PhysicalPosition, PhysicalSize, WebviewWindow};
use tauri_plugin_window_state::{StateFlags, WindowExt};

/// What is saved and restored. Not VISIBLE: the window stays hidden until it fits.
pub fn flags() -> StateFlags {
    StateFlags::all().difference(StateFlags::VISIBLE)
}

/// The window's minimum, in logical points, as `minWidth` and `minHeight` in `tauri.conf.json`.
const MIN: (f64, f64) = (800.0, 500.0);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

fn fit_axis(pos: i64, len: i64, start: i64, avail: i64, min: i64) -> (i64, i64) {
    let len = len.min(avail).max(min);
    let pos = if len >= avail { start } else { pos.clamp(start, start + avail - len) };
    (pos, len)
}

/// The window inside the work area: no larger than it, no smaller than `min`, and moved back
/// inside when it sticks out. A screen smaller than the minimum gets the minimum, at its corner.
pub fn fit(window: Rect, area: Rect, min: (i64, i64)) -> Rect {
    let (x, width) = fit_axis(window.x, window.width, area.x, area.width, min.0);
    let (y, height) = fit_axis(window.y, window.height, area.y, area.height, min.1);
    Rect { x, y, width, height }
}

/// Restores the saved state, fits the window to its monitor, and only then shows it.
pub fn restore(window: &WebviewWindow) {
    if let Err(e) = restore_and_fit(window) {
        eprintln!("window state: {e}");
    }
    let _ = window.show();
}

fn restore_and_fit(window: &WebviewWindow) -> tauri::Result<()> {
    window.restore_state(flags())?;
    if window.is_fullscreen()? || window.is_maximized()? {
        return Ok(());
    }
    let Some(monitor) = window.current_monitor()?.or(window.primary_monitor()?) else {
        return Ok(());
    };
    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    let (pos, outer, inner) = (window.outer_position()?, window.outer_size()?, window.inner_size()?);
    let current = Rect { x: pos.x.into(), y: pos.y.into(), width: outer.width.into(), height: outer.height.into() };
    let fitted = fit(
        current,
        Rect {
            x: area.position.x.into(),
            y: area.position.y.into(),
            width: area.size.width.into(),
            height: area.size.height.into(),
        },
        ((MIN.0 * scale) as i64, (MIN.1 * scale) as i64),
    );
    if fitted != current {
        // set_size is the inner size: keep the title bar's share out of it
        let (dw, dh) = (outer.width.saturating_sub(inner.width), outer.height.saturating_sub(inner.height));
        window.set_size(PhysicalSize::new(fitted.width as u32 - dw, fitted.height as u32 - dh))?;
        window.set_position(PhysicalPosition::new(fitted.x as i32, fitted.y as i32))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAPTOP: Rect = Rect { x: 0, y: 50, width: 3024, height: 1800 };
    const MIN_PX: (i64, i64) = (1600, 1000);

    fn r(x: i64, y: i64, width: i64, height: i64) -> Rect {
        Rect { x, y, width, height }
    }

    #[test]
    fn a_window_that_fits_stays_as_it_is() {
        let w = r(200, 120, 2560, 1640);
        assert_eq!(fit(w, LAPTOP, MIN_PX), w);
    }

    #[test]
    fn too_big_for_a_smaller_screen_is_made_to_fit() {
        // saved on a 5K monitor, opened on the laptop alone
        assert_eq!(fit(r(400, 200, 4800, 2600), LAPTOP, MIN_PX), LAPTOP);
    }

    #[test]
    fn partly_off_the_screen_is_moved_back_inside() {
        assert_eq!(fit(r(2000, 1200, 1800, 1200), LAPTOP, MIN_PX), r(1224, 650, 1800, 1200));
        assert_eq!(fit(r(-300, 0, 1800, 1200), LAPTOP, MIN_PX), r(0, 50, 1800, 1200));
    }

    #[test]
    fn never_below_the_minimum_even_on_a_tiny_screen() {
        assert_eq!(fit(r(10, 60, 900, 700), LAPTOP, MIN_PX), r(10, 60, 1600, 1000));
        let tiny = r(0, 25, 1280, 775);
        assert_eq!(fit(r(100, 100, 1400, 900), tiny, MIN_PX), r(0, 25, 1600, 1000));
    }
}
