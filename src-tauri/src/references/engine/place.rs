// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! Placing a reference's window round its picture, and drawing the picture
//! and its chrome (`references::drawing` says what they hold).

use screenwide_osc::geometry::{Point, Rect};
use tauri::Emitter;

use super::super::commands::ReferenceLayout;
use super::super::drawing::{chrome_vertices, constants, picture_vertices, view_round};
use super::super::fit::{fit, Fit};
use super::hold::{frame, middle, release};
use super::{Native, PlatformHost, ReferenceHost, LAYOUT_EVENT};

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

/// Places the window round the picture at `picture` on the desktop
/// (`hold` says how it keeps still where the view spans it), and draws.
pub(super) fn place(native: &mut Native, picture: Rect) {
  let fitted = fit_round(native, picture);
  let fitted = frame(native, fitted, picture);
  show(native, picture, fitted);
}

/// Lets go of a window held through a resize or a glide, fitting it round
/// the picture where it now stands.
pub(super) fn release_hold(native: &mut Native) {
  if native.held.is_none() {
    return;
  }
  let picture = picture_on_desktop(native);
  let fitted = fit_round(native, picture);
  let fitted = release(native, fitted);
  show(native, picture, fitted);
}

fn fit_round(native: &Native, picture: Rect) -> Fit {
  fit(
    picture,
    native.stage.island,
    native.host.room_at(middle(picture)),
  )
}

/// Puts the window at `fitted`, the view over the picture and its shadow's
/// room, or over the whole window where the view spans it, and draws. Tells
/// the page where its island stands once that moves, and the window where
/// its parts are.
fn show(native: &mut Native, picture: Rect, fitted: Fit) {
  let island = native.fit.island.origin;
  native.fit = fitted;
  let window = fitted.window;
  let view = if PlatformHost::VIEW_SPANS_WINDOW {
    // Only ever grows, and at once to all a resize on this screen could
    // need: a layer resized while it shows is blank until it is drawn
    // again, so it is resized while nothing moves rather than mid-drag.
    let screen = native.host.screen_at(middle(picture));
    let reach = fit_round(native, screen.union(picture)).window.size;
    Rect::from_xywh(
      0.0,
      0.0,
      window.size.width.max(reach.width).max(native.view.width),
      window.size.height.max(reach.height).max(native.view.height),
    )
  } else {
    view_round(fitted.picture)
  };
  // The picture lands on whole pixels where it stands on the desktop,
  // however the window's and the view's corners round.
  let scale = native.host.scale();
  let pixel = |points: f64| (points * scale).round();
  native.stage.in_view = Rect {
    origin: Point {
      x: (pixel(picture.origin.x) - pixel(window.origin.x + view.origin.x)) / scale,
      y: (pixel(picture.origin.y) - pixel(window.origin.y + view.origin.y)) / scale,
    },
    size: picture.size,
  };
  native.view = view.size;
  native.host.set_window_frame(window);
  native.host.set_view_frame(view, fitted.picture);
  if native.drawn != Some((native.view, native.stage.in_view, scale)) {
    draw(native);
  }
  native.host.commit_placement();
  if fitted.island.origin != island {
    let _ = native
      .app
      .emit_to(native.label.as_str(), LAYOUT_EVENT, layout(native));
  }
  crate::app_windows::options::hit_area::set_hit_area(
    native.app.clone(),
    native.label.clone(),
    Some(fitted.hit_rects()),
  );
}

/// Draws the picture and its chrome where they stand in the view.
pub(super) fn draw(native: &mut Native) {
  let scale = native.host.scale();
  let (view, picture) = (native.view, native.stage.in_view);
  let pixels = (
    (view.width * scale).ceil().max(1.0) as u32,
    (view.height * scale).ceil().max(1.0) as u32,
  );
  let constants = constants(native.host.light_appearance(), scale, native.see_through);
  let handles = native.stage.hovered || native.stage.resizing();
  native.drawn = Some((view, picture, scale));
  let drawn = native
    .picture
    .present(pixels, &picture_vertices(view, picture), &constants)
    .and_then(|()| {
      native.chrome.present(
        pixels,
        &chrome_vertices(view, picture, scale, handles),
        &constants,
      )
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
