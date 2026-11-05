// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the view spans the window (`VIEW_SPANS_WINDOW`), the window keeps
//! the picture where it stands in it for as long as it can, and its corner
//! moves only where it must, and only as far: the island, drawn by the
//! page, takes a moment to follow a corner that moves, and shows out of
//! place by as much until it does. Moving, the window moves with the
//! picture. Resizing or gliding, it is held still, growing only on the sides
//! the picture needs, and fitted round the picture again once that ends,
//! keeping its corner where the picture leaves room for it.

use screenwide_osc::geometry::{Point, Rect};

use super::super::fit::{fit, Fit};
use super::{Native, PlatformHost, ReferenceHost};

/// How far, in points, a window may fall short of what it should hold and
/// still be taken to hold it: far less than a pixel, and far more than the
/// arithmetic placing it drifts by.
const SLACK: f64 = 0.25;

/// A window held still through a resize or a glide.
#[derive(Clone, Copy)]
pub(super) struct Hold {
  window: Rect,
}

/// Holds the window still from here on, where the view spans it.
pub(super) fn hold(native: &mut Native) {
  if PlatformHost::VIEW_SPANS_WINDOW && native.held.is_none() {
    native.held = Some(Hold {
      window: native.fit.window,
    });
  }
}

/// Holds the window round `also` as well, for a glide that ends there.
pub(super) fn hold_round(native: &mut Native, also: Rect) {
  hold(native);
  if let Some(hold) = native.held.as_mut() {
    hold.window = grown(hold.window, also, also);
  }
}

/// The window `fitted` is placed in for the picture at `picture` on the
/// desktop: the held one, grown where the picture needs it; or, not held,
/// the window moved with the picture where it still holds everything.
pub(super) fn frame(native: &mut Native, mut fitted: Fit, picture: Rect) -> Fit {
  if let Some(hold) = native.held.as_mut() {
    if !encloses(hold.window, fitted.window) {
      let reach = native
        .stage
        .resize_bounds()
        .map_or(fitted.window, |bounds| {
          fit(
            bounds,
            native.stage.island,
            native.host.room_at(middle(bounds)),
          )
          .window
        });
      hold.window = grown(hold.window, fitted.window, reach);
    }
    place_in(&mut fitted, hold.window);
  } else if PlatformHost::VIEW_SPANS_WINDOW {
    let kept = Rect {
      origin: Point {
        x: picture.origin.x - native.fit.picture.origin.x,
        y: picture.origin.y - native.fit.picture.origin.y,
      },
      size: native.fit.window.size,
    };
    if encloses(kept, fitted.window) {
      place_in(&mut fitted, kept);
    }
  }
  fitted
}

/// Lets go of a held window, fitting `fitted` round the picture again, its
/// corner kept where the picture leaves room for it there.
pub(super) fn release(native: &mut Native, mut fitted: Fit) -> Fit {
  if let Some(hold) = native.held.take() {
    let corner = hold.window.origin;
    let window = fitted.window;
    if window.origin.x >= corner.x - SLACK && window.origin.y >= corner.y - SLACK {
      place_in(
        &mut fitted,
        Rect::from_xywh(
          corner.x,
          corner.y,
          window.right() - corner.x,
          window.bottom() - corner.y,
        ),
      );
    }
  }
  fitted
}

pub(super) fn middle(rect: Rect) -> Point {
  Point {
    x: rect.origin.x + rect.size.width / 2.0,
    y: rect.origin.y + rect.size.height / 2.0,
  }
}

/// Puts `fitted`'s parts where they stand on the desktop in `window`.
fn place_in(fitted: &mut Fit, window: Rect) {
  let (dx, dy) = (
    fitted.window.origin.x - window.origin.x,
    fitted.window.origin.y - window.origin.y,
  );
  for part in [&mut fitted.picture, &mut fitted.island] {
    part.origin.x += dx;
    part.origin.y += dy;
  }
  fitted.window = window;
}

/// `held`, grown on each side `needed` reaches past it. The start and the
/// top move the corner, so they go no further than needed; the end and the
/// bottom take all the room as far as `reach` at once, so the window, and
/// the webview in it, resize once rather than at every step.
fn grown(held: Rect, needed: Rect, reach: Rect) -> Rect {
  let left = held.origin.x.min(needed.origin.x);
  let top = held.origin.y.min(needed.origin.y);
  let right = if needed.right() > held.right() + SLACK {
    needed.right().max(reach.right())
  } else {
    held.right()
  };
  let bottom = if needed.bottom() > held.bottom() + SLACK {
    needed.bottom().max(reach.bottom())
  } else {
    held.bottom()
  };
  Rect::from_xywh(left, top, right - left, bottom - top)
}

/// Whether `outer` holds the whole of `inner`, give or take `SLACK`.
fn encloses(outer: Rect, inner: Rect) -> bool {
  inner.origin.x >= outer.origin.x - SLACK
    && inner.origin.y >= outer.origin.y - SLACK
    && inner.right() <= outer.right() + SLACK
    && inner.bottom() <= outer.bottom() + SLACK
}

#[cfg(test)]
mod tests {
  use super::*;

  const REACH: Rect = Rect::from_xywh(0.0, 0.0, 1000.0, 800.0);

  #[test]
  fn a_held_window_takes_the_room_at_its_end_and_foot_at_once() {
    let held = Rect::from_xywh(100.0, 100.0, 200.0, 200.0);
    let grown = grown(held, Rect::from_xywh(100.0, 100.0, 250.0, 260.0), REACH);
    assert_eq!(grown.origin, held.origin);
    assert_eq!((grown.right(), grown.bottom()), (1000.0, 800.0));
  }

  #[test]
  fn a_held_window_moves_its_corner_no_further_than_needed() {
    let held = Rect::from_xywh(100.0, 100.0, 200.0, 200.0);
    let grown = grown(held, Rect::from_xywh(97.0, 100.0, 203.0, 200.0), REACH);
    assert_eq!(grown.origin.x, 97.0);
    assert_eq!(grown.right(), held.right());
  }
}
