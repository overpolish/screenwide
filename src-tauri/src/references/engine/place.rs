// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Placing a reference's window round its picture, and drawing the picture
//! and its chrome (`references::drawing` says what they hold).

use screenwide_osc::geometry::{Point, Rect};
use tauri::Emitter;

use super::super::commands::ReferenceLayout;
use super::super::drawing::{chrome_vertices, constants, picture_vertices, view_size};
use super::super::fit::{fit, SHADOW_ROOM};
use super::{Native, ReferenceHost, LAYOUT_EVENT};

pub(super) fn layout(native: &Native) -> ReferenceLayout {
  ReferenceLayout {
    island_x: native.fit.island.origin.x,
    island_y: native.fit.island.origin.y,
    island_above: native.fit.island_above,
  }
}

/// Where the picture stands on the desktop.
pub(super) fn picture_on_desktop(native: &Native) -> Rect {
  let window = native.host.window_frame();
  Rect {
    origin: Point {
      x: window.origin.x + native.fit.picture.origin.x,
      y: window.origin.y + native.fit.picture.origin.y,
    },
    size: native.stage.size,
  }
}

/// Places the window round the picture at `picture` on the desktop, the view
/// over the picture and its shadow's room, and draws. Tells the page where
/// its island stands once that moves, and the window where its parts are.
pub(super) fn place(native: &mut Native, picture: Rect) {
  let middle = Point {
    x: picture.origin.x + picture.size.width / 2.0,
    y: picture.origin.y + picture.size.height / 2.0,
  };
  let island = native.fit.island.origin;
  native.fit = fit(picture, native.stage.island, native.host.room_at(middle));
  native.host.set_window_frame(native.fit.window);
  let view = view_size(picture.size);
  native.host.set_view_frame(Rect::from_xywh(
    native.fit.picture.origin.x - SHADOW_ROOM,
    native.fit.picture.origin.y - SHADOW_ROOM,
    view.width,
    view.height,
  ));
  draw(native);
  if native.fit.island.origin != island {
    let _ = native
      .app
      .emit_to(native.label.as_str(), LAYOUT_EVENT, layout(native));
  }
  crate::app_windows::options::hit_area::set_hit_area(
    native.app.clone(),
    native.label.clone(),
    Some(native.fit.hit_rects()),
  );
}

/// Draws the picture and its chrome at their present size.
pub(super) fn draw(native: &mut Native) {
  let scale = native.host.scale();
  let size = native.stage.size;
  let view = view_size(size);
  let pixels = (
    (view.width * scale).ceil().max(1.0) as u32,
    (view.height * scale).ceil().max(1.0) as u32,
  );
  let constants = constants(native.host.light_appearance(), scale);
  let handles = native.stage.hovered || native.stage.resizing();
  let drawn = native
    .picture
    .present(pixels, &picture_vertices(size), &constants)
    .and_then(|()| {
      native
        .chrome
        .present(pixels, &chrome_vertices(size, scale, handles), &constants)
    });
  if let Err(error) = drawn {
    eprintln!("Could not draw a reference: {error}");
  }
}

/// Places the reference `label` again where its picture now stands, once
/// its window has come onto a display of another scale.
#[cfg(target_os = "windows")]
pub(crate) fn rescale(label: &str) {
  super::with(label, |native| {
    native.host.refresh_scale();
    let picture = picture_on_desktop(native);
    place(native, picture);
  });
}
