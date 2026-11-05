// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Placing a reference on macOS: its window round the picture (`fit`), the
//! picture's view in it, where the island stands for the page, and where the
//! window takes clicks.

use objc2::MainThreadMarker;
use objc2_foundation::{NSPoint, NSRect, NSSize};
use objc2_quartz_core::CALayer;
use screenwide_osc::geometry::{Point, Rect};
use tauri::Emitter;

use super::super::commands::ReferenceLayout;
use super::super::drawing::view_size;
use super::super::fit::{fit, SHADOW_ROOM};
use super::desktop::{appkit_rect, desktop_rect, room_at};
use super::draw::{draw, transaction};
use super::{Native, LAYOUT_EVENT};

pub(super) fn layout(native: &Native) -> ReferenceLayout {
  ReferenceLayout {
    island_x: native.fit.island.origin.x,
    island_y: native.fit.island.origin.y,
    island_above: native.fit.island_above,
  }
}

pub(super) fn picture_on_desktop(mtm: MainThreadMarker, native: &Native) -> Rect {
  let window = desktop_rect(mtm, native.window.frame());
  Rect {
    origin: Point {
      x: window.origin.x + native.fit.picture.origin.x,
      y: window.origin.y + native.fit.picture.origin.y,
    },
    size: native.stage.size,
  }
}

/// `rect`, in the window's points from its top left, in its content view's
/// own, which are not flipped: their origin is the window's foot.
fn in_host(native: &Native, rect: Rect) -> NSRect {
  let y = if native.host.isFlipped() {
    rect.origin.y
  } else {
    native.host.bounds().size.height - rect.origin.y - rect.size.height
  };
  NSRect::new(
    NSPoint::new(rect.origin.x, y),
    NSSize::new(rect.size.width, rect.size.height),
  )
}

/// Places the window round the picture at `picture` on the desktop, the view
/// over the picture and its shadow's room, and draws. Tells the page where
/// its island stands once that moves, and the window where its parts are.
pub(super) fn place(mtm: MainThreadMarker, native: &mut Native, picture: Rect) {
  let middle = Point {
    x: picture.origin.x + picture.size.width / 2.0,
    y: picture.origin.y + picture.size.height / 2.0,
  };
  let island = native.fit.island.origin;
  native.fit = fit(picture, native.stage.island, room_at(mtm, middle));
  native
    .window
    .setFrame_display(appkit_rect(mtm, native.fit.window), true);
  let view = view_size(picture.size);
  let frame = in_host(
    native,
    Rect::from_xywh(
      native.fit.picture.origin.x - SHADOW_ROOM,
      native.fit.picture.origin.y - SHADOW_ROOM,
      view.width,
      view.height,
    ),
  );
  native.view.setFrame(frame);
  let bounds = NSRect::new(NSPoint::new(0.0, 0.0), frame.size);
  transaction(|| {
    let scale = native.window.backingScaleFactor();
    for layer in [&native.picture_layer, &native.chrome_layer] {
      let layer: &CALayer = layer;
      layer.setFrame(bounds);
      layer.setContentsScale(scale);
    }
  });
  draw(mtm, native);
  if native.fit.island.origin != island {
    let _ = native
      .app
      .emit_to(native.label.as_str(), LAYOUT_EVENT, layout(native));
  }
  refresh_hit_area(native);
}

/// Lets the window take clicks on the picture and the island alone.
pub(super) fn refresh_hit_area(native: &Native) {
  crate::app_windows::options::hit_area::set_hit_area(
    native.app.clone(),
    native.label.clone(),
    Some(native.fit.hit_rects()),
  );
}
