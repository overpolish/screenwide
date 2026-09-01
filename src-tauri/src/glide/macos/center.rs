// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use cidre::cg;
use core_graphics::geometry::CGPoint;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Monitor};

use super::own_window::own_window_at;
use super::titlebar::{ax_window_at, AxHit};
use super::tween::{animate_to, WindowTarget};

/// Centers the window under a global point on the monitor that point lies on,
/// keeping the window's size. This runs on the event tap thread, like the
/// titlebar hit test: the Accessibility messaging timeout is already short, and
/// a main-thread hop would stall the tap. The travel is the same tween every
/// glide runs through, and the settable guard the direct set used to make
/// lives in the tween.
pub(super) fn center_window_at(app: &AppHandle, point: CGPoint) {
  let Some((target, frame)) = target_at(app, point) else {
    return;
  };
  let Some((position, size)) = work_area_at(app, point) else {
    eprintln!("Could not find the monitor to center the window on");
    return;
  };

  let (x, y) = centered_origin(position, size, (frame.size.width, frame.size.height));
  animate_to(
    &target,
    cg::Rect {
      origin: cg::Point { x, y },
      size: frame.size,
    },
    // A centering is not a region placement: it keeps the window's own size, so
    // there is no fit to correct and nothing for the preview to hear about.
    None,
  );
}

/// The window under a global point as something the tween can drive, with the
/// frame it was read at. One of Screenwide's own answers first and natively;
/// everything else goes through the Accessibility hit test, which never returns
/// one of ours.
fn target_at(app: &AppHandle, point: CGPoint) -> Option<(WindowTarget, cg::Rect)> {
  match ax_window_at(point) {
    AxHit::Foreign(_, window) => {
      let Some(frame) = window.frame().ok().and_then(|value| value.cg_rect()) else {
        eprintln!("Could not read the window to center");
        return None;
      };
      Some((WindowTarget::new(window), frame))
    }
    AxHit::OwnProcess => {
      own_window_at(app, point).map(|(window, frame)| (WindowTarget::own(window), frame))
    }
    AxHit::Miss => None,
  }
}

/// The logical work area of the monitor holding a global point. macOS reports
/// Accessibility geometry in the same space as Tauri's logical coordinates, so
/// each monitor is converted with its own scale factor.
pub(super) fn work_area_at(
  app: &AppHandle,
  point: CGPoint,
) -> Option<(LogicalPosition<f64>, LogicalSize<f64>)> {
  let monitors = app.available_monitors().ok()?;
  let monitor = monitors
    .into_iter()
    .find(|monitor| contains(monitor, point))
    .or_else(|| app.primary_monitor().ok().flatten())?;

  let scale = monitor.scale_factor();
  let work_area = monitor.work_area();
  Some((
    work_area.position.to_logical(scale),
    work_area.size.to_logical(scale),
  ))
}

fn contains(monitor: &Monitor, point: CGPoint) -> bool {
  let scale = monitor.scale_factor();
  let position: LogicalPosition<f64> = monitor.position().to_logical(scale);
  let size: LogicalSize<f64> = monitor.size().to_logical(scale);

  point.x >= position.x
    && point.x < position.x + size.width
    && point.y >= position.y
    && point.y < position.y + size.height
}

/// The top-left corner that centers a window of `window_size` in a logical work
/// area. A window larger than the work area pins to the work area's origin.
fn centered_origin(
  work_position: LogicalPosition<f64>,
  work_size: LogicalSize<f64>,
  window_size: (f64, f64),
) -> (f64, f64) {
  let origin = crate::windows::centered_logical_position(
    work_position,
    work_size,
    LogicalSize::new(window_size.0, window_size.1),
  );
  (origin.x, origin.y)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn centers_a_window_in_the_work_area_of_its_monitor() {
    assert_eq!(
      centered_origin(
        LogicalPosition::new(-1_440.0, 25.0),
        LogicalSize::new(1_440.0, 875.0),
        (800.0, 475.0),
      ),
      (-1_120.0, 225.0)
    );
  }

  #[test]
  fn pins_a_window_larger_than_the_work_area_to_its_origin() {
    assert_eq!(
      centered_origin(
        LogicalPosition::new(0.0, 25.0),
        LogicalSize::new(1_440.0, 875.0),
        (1_800.0, 1_000.0),
      ),
      (0.0, 25.0)
    );
  }
}
