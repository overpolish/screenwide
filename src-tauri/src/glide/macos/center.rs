// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use cidre::cg;
use core_graphics::geometry::CGPoint;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Monitor};

use super::native_settings;
use super::own_window::own_window_at;
use super::titlebar::{ax_window_at, AxHit};
use super::tween::{glide_frame, WindowTarget};
use crate::glide::core::GlideFrame;

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

  center_captured(
    &target,
    frame,
    (position.x, position.y),
    (size.width, size.height),
    point,
  );
}

pub(super) fn center_captured(
  target: &WindowTarget,
  frame: cg::Rect,
  work_position: (f64, f64),
  work_size: (f64, f64),
  anchor: CGPoint,
) {
  crate::glide::session::center::<super::session::MacGlide>(
    target,
    glide_frame(frame),
    GlideFrame {
      x: work_position.0,
      y: work_position.1,
      width: work_size.0,
      height: work_size.1,
    },
    (anchor.x, anchor.y),
    native_settings::snapshot().cursor_follows,
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
