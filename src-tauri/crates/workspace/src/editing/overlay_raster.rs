// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The platform overlay drawn to pixels away from the chrome's draws. Laying
//! out and rasterizing an app's interface takes tens of milliseconds, the
//! first one longer while the system's fonts are read, which every zoom step
//! would otherwise wait for. So it is drawn on a thread at a few sizes only,
//! each a step of √2 apart, and the chrome scales the nearest picture it has
//! to the canvas on screen until the one the canvas wants arrives and the
//! chrome is drawn again.

use std::sync::{Arc, Weak};

use screenwide_platform_overlays::{OverlayPicture, OverlayRequest};

use super::selection_draw::draw_selection;
use super::state::SurfaceInner;

/// The longest side a picture is drawn at: past it, a canvas zoomed far in
/// has its overlay sampled up, larger than any phone shows it anyway.
const MOST: f64 = 2048.0;
/// The smallest picture drawn, and the first rung of the sizes above it.
const LEAST: f64 = 256.0;

/// An overlay drawn for a canvas of `pixels`, and where it lies from the
/// canvas's corner as a share of the canvas, with its size as one: a
/// phone's bars reach past the canvas.
pub struct RasteredOverlay {
  pub request: OverlayRequest,
  pub pixels: (u32, u32),
  pub picture: OverlayPicture,
  pub place: [f32; 4],
}

#[derive(Default)]
pub(crate) struct OverlayRaster {
  ready: Option<Arc<RasteredOverlay>>,
  /// The drawing under way, and the one wanted after it, if another.
  drawing: Option<(OverlayRequest, (u32, u32))>,
  next: Option<(OverlayRequest, (u32, u32))>,
}

/// The picture to draw over a canvas `shown` pixels on screen: drawn for it
/// at the nearest rung, or, while that one is being drawn, the last one of
/// the same overlay and shape, which the chrome scales. `None` while there
/// is nothing yet of this overlay's shape, which leaves the canvas bare
/// rather than show a picture made for another canvas.
pub(crate) fn picture_for(
  inner: &SurfaceInner,
  request: OverlayRequest,
  shown: (f64, f64),
) -> Option<Arc<RasteredOverlay>> {
  let want = (request, rung(shown)?);
  let mut raster = inner.overlay_raster.lock().ok()?;
  let ready = raster.ready.clone();
  if ready
    .as_ref()
    .is_some_and(|ready| (ready.request, ready.pixels) == want)
  {
    return ready;
  }
  if raster.drawing.is_none() {
    raster.drawing = Some(want);
    start(inner.this.clone(), want);
  } else if raster.drawing != Some(want) {
    raster.next = Some(want);
  }
  ready.filter(|ready| ready.request == request && same_shape(ready.pixels, want.1))
}

/// `shown` scaled to the rung at or above it, keeping its shape.
fn rung(shown: (f64, f64)) -> Option<(u32, u32)> {
  let longest = shown.0.max(shown.1);
  if longest.is_nan() || longest < 8.0 {
    return None;
  }
  let steps = (longest.min(MOST) / LEAST).log2() * 2.0;
  let side = (LEAST * 2f64.powf(steps.ceil().max(0.0) / 2.0)).min(MOST);
  let scale = side / longest;
  Some((
    (shown.0 * scale).round().max(1.0) as u32,
    (shown.1 * scale).round().max(1.0) as u32,
  ))
}

fn same_shape(left: (u32, u32), right: (u32, u32)) -> bool {
  let ratio = |(width, height): (u32, u32)| f64::from(width) / f64::from(height.max(1));
  (ratio(left) / ratio(right) - 1.0).abs() < 0.01
}

fn start(inner: Weak<SurfaceInner>, (request, pixels): (OverlayRequest, (u32, u32))) {
  std::thread::spawn(move || {
    let drawn = screenwide_platform_overlays::draw(request, pixels).map(|picture| {
      let share = |value: f64, whole: u32| (value / f64::from(whole.max(1))) as f32;
      let place = [
        share(picture.origin.0, pixels.0),
        share(picture.origin.1, pixels.1),
        share(f64::from(picture.size.0), pixels.0),
        share(f64::from(picture.size.1), pixels.1),
      ];
      Arc::new(RasteredOverlay {
        request,
        pixels,
        picture,
        place,
      })
    });
    let Some(inner) = inner.upgrade() else {
      return;
    };
    {
      let Ok(mut raster) = inner.overlay_raster.lock() else {
        return;
      };
      if drawn.is_some() {
        raster.ready = drawn;
      }
      raster.drawing = raster.next.take();
      if let Some(next) = raster.drawing {
        start(inner.this.clone(), next);
      }
    }
    // The chrome drawn again with the new picture, as the canvas is now,
    // and committed: a present from this thread waits on a transaction
    // nothing here commits otherwise.
    let Ok(state) = inner.state.lock() else {
      return;
    };
    draw_selection(&inner, &state);
    inner.host.commit(false);
  });
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rungs_are_few_and_never_smaller_than_the_canvas_below_the_ceiling() {
    let mut rungs = std::collections::BTreeSet::new();
    for width in (100..2000).step_by(7) {
      let shown = (f64::from(width), f64::from(width) * 16.0 / 9.0);
      let (rung_width, rung_height) = rung(shown).expect("a rung");
      rungs.insert(rung_height);
      if shown.1 <= MOST {
        assert!(f64::from(rung_height) >= shown.1 - 0.5, "{shown:?}");
        let rung_floor = shown.1.max(LEAST);
        assert!(
          f64::from(rung_height) <= rung_floor * 2f64.sqrt() + 1.0,
          "{shown:?}"
        );
      }
      assert!(same_shape((rung_width, rung_height), (9, 16)));
    }
    // Zooming from a thumbnail to past the ceiling draws a handful of sizes.
    assert!(rungs.len() <= 7, "{rungs:?}");
  }
}
