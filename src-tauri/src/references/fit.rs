// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a reference's window stands round its picture. The window holds
//! the picture and the island by it, with room round both for the shadows
//! they cast, which clicks pass through. Where the island is wider than the
//! picture, it is centred on it as far as the screen leaves room; short of
//! room on one side, it reaches further on the other, keeping the island on
//! the screen. The shadows' room may reach past the screen's edge, as
//! nothing is drawn there that matters. The picture never moves for any of
//! it: the window is placed round where the picture is.

use screenwide_osc::geometry::{Rect, Size};

use crate::app_windows::options::hit_area::IslandRect;

/// The room round the picture that its handles reach into, in points: a
/// handle's disc is about five points across its centre on the edge, with its
/// anti-aliased rim. The picture takes clicks this far round it.
pub(crate) const HANDLE_MARGIN: f64 = 8.0;
/// The room round the picture and the island for their shadows, in points:
/// the islands' own (`ISLAND.shadowRoom`).
pub(crate) const SHADOW_ROOM: f64 = 48.0;
/// The gap between the picture and the island, in points: `--spacing-separate`.
const ISLAND_GAP: f64 = 12.0;

/// A reference's window, and where its picture and island stand in it, in
/// points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Fit {
  /// The window on the desktop, top-left origin.
  pub window: Rect,
  pub picture: Rect,
  pub island: Rect,
  /// Whether the island hangs over the picture rather than under it.
  pub island_above: bool,
}

impl Fit {
  /// Where the window takes clicks: the picture and the handles' reach round
  /// it, and the island. Anywhere else, its shadows' room, lets them through.
  pub(crate) fn hit_rects(&self) -> Vec<IslandRect> {
    let rect = |rect: Rect, reach: f64| IslandRect {
      height: rect.size.height + reach * 2.0,
      width: rect.size.width + reach * 2.0,
      x: rect.origin.x - reach,
      y: rect.origin.y - reach,
    };
    vec![rect(self.picture, HANDLE_MARGIN), rect(self.island, 0.0)]
  }
}

/// The window for a picture at `picture` on the desktop with an island of
/// `island` by it, on a screen whose room for windows is `screen`. The
/// island hangs under the picture, or over it where it would not fit under
/// it and there is more room above.
pub(crate) fn fit(picture: Rect, island: Size, screen: Rect) -> Fit {
  let width = picture.size.width.max(island.width);
  let inset = spare_before(
    width - picture.size.width,
    picture.origin.x - screen.origin.x,
    screen.right() - picture.right(),
  );
  let beside = if island.height > 0.0 {
    ISLAND_GAP + island.height
  } else {
    0.0
  };
  let room_below = screen.bottom() - picture.bottom();
  let room_above = picture.origin.y - screen.origin.y;
  let island_above = room_below < beside && room_above > room_below;
  // The island's room is kept on both sides of the picture, so the picture
  // stands in the same place in the window whichever side the island is on,
  // and the island glides from one side to the other inside the window.
  let picture_top = SHADOW_ROOM + beside;
  Fit {
    window: Rect::from_xywh(
      picture.origin.x - inset - SHADOW_ROOM,
      picture.origin.y - picture_top,
      width + SHADOW_ROOM * 2.0,
      picture.size.height + beside * 2.0 + SHADOW_ROOM * 2.0,
    ),
    picture: Rect {
      origin: screenwide_osc::geometry::Point {
        x: SHADOW_ROOM + inset,
        y: picture_top,
      },
      size: picture.size,
    },
    island: Rect::from_xywh(
      SHADOW_ROOM + (width - island.width) / 2.0,
      if island_above {
        SHADOW_ROOM
      } else {
        picture_top + picture.size.height + ISLAND_GAP
      },
      island.width,
      island.height,
    ),
    island_above,
  }
}

/// How much of `spare` width goes before the picture: half where the screen
/// leaves `before` and `after` either side of it, else as near half as they
/// allow.
fn spare_before(spare: f64, before: f64, after: f64) -> f64 {
  let least = (spare - after).max(0.0);
  let most = spare.min(before);
  if least > most {
    return most.clamp(0.0, spare.max(0.0));
  }
  (spare / 2.0).clamp(least, most)
}

#[cfg(test)]
mod tests {
  use super::*;
  use screenwide_osc::geometry::Point;

  const SCREEN: Rect = Rect {
    origin: Point { x: 0.0, y: 0.0 },
    size: Size {
      width: 1000.0,
      height: 800.0,
    },
  };
  const ISLAND: Size = Size {
    width: 216.0,
    height: 48.0,
  };

  fn picture(x: f64, width: f64) -> Rect {
    Rect::from_xywh(x, 100.0, width, 50.0)
  }

  fn on_desktop(fit: &Fit, rect: Rect) -> Rect {
    Rect::from_xywh(
      fit.window.origin.x + rect.origin.x,
      fit.window.origin.y + rect.origin.y,
      rect.size.width,
      rect.size.height,
    )
  }

  #[test]
  fn a_narrow_picture_has_its_island_centred_under_it() {
    let fit = fit(picture(400.0, 100.0), ISLAND, SCREEN);
    let island = on_desktop(&fit, fit.island);
    assert_eq!(island.origin.x + island.size.width / 2.0, 450.0);
    assert_eq!(island.origin.y, 100.0 + 50.0 + ISLAND_GAP);
  }

  #[test]
  fn the_picture_stays_where_it_is() {
    for x in [0.0, 30.0, 400.0, 880.0, 900.0] {
      let fit = fit(picture(x, 100.0), ISLAND, SCREEN);
      assert_eq!(on_desktop(&fit, fit.picture), picture(x, 100.0));
    }
  }

  #[test]
  fn near_an_edge_the_island_reaches_the_other_way_to_stay_on_screen() {
    let island = |x| {
      let fit = fit(picture(x, 100.0), ISLAND, SCREEN);
      on_desktop(&fit, fit.island)
    };
    assert_eq!(island(30.0).origin.x, 0.0);
    assert_eq!(island(870.0).right(), 1000.0);
    // The shadows' room reaches past the screen's edge.
    assert!(fit(picture(30.0, 100.0), ISLAND, SCREEN).window.origin.x < 0.0);
  }

  #[test]
  fn at_the_screens_foot_the_island_goes_above_the_picture() {
    let picture = Rect::from_xywh(400.0, 760.0, 100.0, 30.0);
    let fit = fit(picture, ISLAND, SCREEN);
    let island = on_desktop(&fit, fit.island);
    assert_eq!(island.bottom() + ISLAND_GAP, picture.origin.y);
    assert_eq!(on_desktop(&fit, fit.picture), picture);
  }

  #[test]
  fn the_picture_keeps_its_place_in_the_window_whichever_side_the_island_is() {
    let below = fit(Rect::from_xywh(400.0, 100.0, 100.0, 30.0), ISLAND, SCREEN);
    let above = fit(Rect::from_xywh(400.0, 760.0, 100.0, 30.0), ISLAND, SCREEN);
    assert!(!below.island_above && above.island_above);
    assert_eq!(below.picture, above.picture);
    assert_eq!(below.window.size, above.window.size);
  }

  #[test]
  fn the_window_leaves_the_shadows_room_round_everything() {
    let fit = fit(picture(100.0, 500.0), ISLAND, SCREEN);
    assert_eq!(fit.picture.origin.x, SHADOW_ROOM);
    assert_eq!(fit.window.size.width, 500.0 + SHADOW_ROOM * 2.0);
    assert_eq!(fit.island.bottom() + SHADOW_ROOM, fit.window.size.height);
  }
}
