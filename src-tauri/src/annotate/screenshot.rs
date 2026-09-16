// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The annotations a screenshot takes with it.
//!
//! The overlay's windows are kept out of every capture, so a still never
//! holds the annotations as pixels. What it gets instead is the annotations
//! themselves, moved into its own pixel space with the same transform a
//! recording uses: the shot is one more source rectangle on the desktop.

use crate::editor::annotations::Annotation;
use crate::recording::cursor::{CursorSource, CursorSourceKind};
use crate::recording::Region;
use crate::screenshots::{CapturedImage, ScreenshotTarget};

/// Where the shot came from, in global logical points, or nothing for a
/// target whose desktop placement the still does not carry: a window shot is
/// the window's own content, wherever it was and whatever covered it.
fn source(target: ScreenshotTarget, image: &CapturedImage) -> Option<CursorSource> {
  let (monitor_id, region, kind) = match target {
    ScreenshotTarget::Screen { monitor_id } => (monitor_id, None, CursorSourceKind::Screen),
    ScreenshotTarget::Region { monitor_id, region }
    | ScreenshotTarget::DesktopRegion { monitor_id, region } => {
      (monitor_id, Some(region), CursorSourceKind::Region)
    }
    ScreenshotTarget::Window { .. } => return None,
  };
  let monitor = xcap::Monitor::all()
    .ok()?
    .into_iter()
    .find(|monitor| monitor.id().ok() == Some(monitor_id))?;
  let monitor_x = f64::from(monitor.x().ok()?);
  let monitor_y = f64::from(monitor.y().ok()?);
  let (x, y, width, height) = match region {
    Some(Region { position, size }) => (
      monitor_x + position.x,
      monitor_y + position.y,
      size.width,
      size.height,
    ),
    None => {
      let scale = f64::from(monitor.scale_factor().ok()?);
      (
        monitor_x,
        monitor_y,
        f64::from(image.width) / scale,
        f64::from(image.height) / scale,
      )
    }
  };
  Some(CursorSource {
    height,
    kind,
    platform_id: monitor_id.to_string(),
    video_height: image.height,
    video_width: image.width,
    width,
    x,
    y,
  })
}

/// The annotations on screen that fall inside a still, in its pixels.
pub(crate) fn annotations_for(target: ScreenshotTarget, image: &CapturedImage) -> Vec<Annotation> {
  let live = super::live_clips::annotations();
  if live.is_empty() {
    return Vec::new();
  }
  let Some(source) = source(target, image) else {
    return Vec::new();
  };
  live
    .iter()
    .filter_map(|annotation| super::geometry::source_annotation(annotation, &source))
    .collect()
}
