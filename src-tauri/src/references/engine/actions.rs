// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a reference's island asks of it: room for the island, the picture's
//! opacity, the fade as it closes, and putting the picture back.

use screenwide_osc::geometry::{Point, Rect, Size};
use tauri::Position;

use super::super::commands::ReferenceLayout;
use super::glide::glide_to;
use super::place::{layout, picture_on_desktop, place};
use super::{with, ReferenceHost};

/// How long a reference takes to fade as it closes, in seconds: the
/// `travel` timing the island tucks away over (`src/lib/motion.ts`).
const FADE_SECONDS: f64 = 0.22;

/// Fits the window round an island of `island` by the picture. Answers
/// where the island stands.
pub(crate) fn fit_island(label: &str, island: Size) -> Option<ReferenceLayout> {
  with(label, |native| {
    native.stage.island = island;
    let picture = picture_on_desktop(native);
    place(native, picture);
    layout(native)
  })
}

pub(crate) fn set_opacity(label: &str, opacity: f64) {
  with(label, |native| native.host.set_picture_opacity(opacity));
}

/// Fades the picture and its shadow out as the island tucks away, before
/// the window closes.
pub(crate) fn fade_out(label: &str) {
  with(label, |native| native.host.fade_out(FADE_SECONDS));
}

/// Glides the picture back to `origin` at its true size.
pub(crate) fn to_origin(label: &str, origin: Position) {
  let target = with(label, |native| Rect {
    origin: native.host.desktop_point(origin),
    size: native.stage.true_size,
  });
  if let Some(target) = target {
    glide_to(label, target);
  }
}

/// Glides the picture back to its true size, its middle where it is.
pub(crate) fn reset_size(label: &str) {
  let target = with(label, |native| {
    let shown = picture_on_desktop(native);
    let size = native.stage.true_size;
    Rect {
      origin: Point {
        x: (shown.origin.x + (shown.size.width - size.width) / 2.0).round(),
        y: (shown.origin.y + (shown.size.height - size.height) / 2.0).round(),
      },
      size,
    }
  });
  if let Some(target) = target {
    glide_to(label, target);
  }
}
