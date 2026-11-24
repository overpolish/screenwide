// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The platform overlay fades in as it is turned on, out as it is turned
//! off, and from one phone or app to another, as the rest of the chrome
//! changes over (`chrome_fade`). A picture still being drawn for the next
//! overlay leaves the one shown standing until it can fade in, and a canvas
//! of another shape takes the overlay away at once: faded over the new
//! canvas, the old overlay would be stretched across it.

use std::sync::Arc;
use std::time::{Duration, Instant};

use super::overlay_raster::RasteredOverlay;
use super::selection_draw::draw_selection;
use super::state::SurfaceInner;

/// How long a fade takes: the chrome's own change-over.
const FADE: Duration = Duration::from_millis(200);
/// One frame of a fade, near enough a display interval.
const FRAME: Duration = Duration::from_millis(16);

/// What the chrome would show of the overlay now.
pub(crate) enum Wanted {
  /// Nothing: no overlay is chosen.
  Off,
  /// Not on this canvas, which is not the shape it was chosen for.
  Elsewhere,
  /// The overlay chosen, its picture still being drawn.
  Pending,
  Ready(Arc<RasteredOverlay>),
}

struct Leaving {
  picture: Arc<RasteredOverlay>,
  /// How opaque it was as it began to leave.
  from: f32,
  since: Instant,
}

#[derive(Default)]
pub(crate) struct OverlayFade {
  shown: Option<Arc<RasteredOverlay>>,
  /// When the one shown began to fade in; none once it is whole.
  shown_since: Option<Instant>,
  leaving: Option<Leaving>,
  ticking: bool,
}

/// How far through a fade begun at `since`, eased in and out as the
/// chrome's fades are.
fn blended(since: Instant, now: Instant) -> f32 {
  let t = (now.saturating_duration_since(since).as_secs_f32() / FADE.as_secs_f32()).clamp(0.0, 1.0);
  t * t * (3.0 - 2.0 * t)
}

impl OverlayFade {
  fn shown_opacity(&self, now: Instant) -> f32 {
    self.shown_since.map_or(1.0, |since| blended(since, now))
  }

  fn is_settled(&self) -> bool {
    self.shown_since.is_none() && self.leaving.is_none()
  }

  /// Takes in what is wanted at `now` and answers what to draw, back to
  /// front, each with its opacity.
  fn step(
    &mut self,
    wanted: Wanted,
    now: Instant,
    reduces_motion: bool,
  ) -> Vec<(Arc<RasteredOverlay>, f32)> {
    match wanted {
      Wanted::Elsewhere => {
        self.shown = None;
        self.shown_since = None;
        self.leaving = None;
      }
      Wanted::Pending => {}
      Wanted::Off => self.leave(now),
      Wanted::Ready(picture) => match &self.shown {
        // The same overlay drawn at another size, as the canvas zooms.
        Some(shown) if shown.request == picture.request => self.shown = Some(picture),
        _ => {
          self.leave(now);
          self.shown = Some(picture);
          self.shown_since = Some(now);
        }
      },
    }
    if reduces_motion {
      self.shown_since = None;
      self.leaving = None;
    }
    if self
      .shown_since
      .is_some_and(|since| blended(since, now) >= 1.0)
    {
      self.shown_since = None;
    }
    if self
      .leaving
      .as_ref()
      .is_some_and(|leaving| blended(leaving.since, now) >= 1.0)
    {
      self.leaving = None;
    }
    let leaving = self.leaving.as_ref().map(|leaving| {
      (
        Arc::clone(&leaving.picture),
        leaving.from * (1.0 - blended(leaving.since, now)),
      )
    });
    let shown = self
      .shown
      .as_ref()
      .map(|shown| (Arc::clone(shown), self.shown_opacity(now)));
    leaving.into_iter().chain(shown).collect()
  }

  /// The one shown starts fading out from where it stands.
  fn leave(&mut self, now: Instant) {
    if let Some(picture) = self.shown.take() {
      let from = self.shown_opacity(now);
      self.leaving = Some(Leaving {
        picture,
        from,
        since: now,
      });
    }
    self.shown_since = None;
  }
}

/// What to draw of the overlay now, `wanted` taken in, with a redraw kept
/// going on a thread of its own until any fade is done.
pub(crate) fn overlays_now(
  inner: &SurfaceInner,
  wanted: Wanted,
) -> Vec<(Arc<RasteredOverlay>, f32)> {
  let Ok(mut fade) = inner.overlay_fade.lock() else {
    return Vec::new();
  };
  let drawn = fade.step(wanted, Instant::now(), inner.host.reduces_motion());
  if !fade.is_settled() && !fade.ticking {
    if let Some(inner) = inner.this.upgrade() {
      fade.ticking = true;
      std::thread::spawn(move || redraw_until_settled(&inner));
    }
  }
  drawn
}

fn redraw_until_settled(inner: &Arc<SurfaceInner>) {
  loop {
    std::thread::sleep(FRAME);
    {
      let Ok(state) = inner.state.lock() else {
        return;
      };
      draw_selection(inner, &state);
    }
    // A present from this thread waits on a transaction nothing here
    // commits otherwise.
    inner.host.commit(false);
    let Ok(mut fade) = inner.overlay_fade.lock() else {
      return;
    };
    if fade.is_settled() {
      fade.ticking = false;
      return;
    }
  }
}

#[cfg(test)]
mod tests {
  use screenwide_platform_overlays::{OverlayPicture, OverlayRequest, PlatformOverlay};

  use super::*;

  fn picture(platform: PlatformOverlay) -> Arc<RasteredOverlay> {
    Arc::new(RasteredOverlay {
      request: OverlayRequest {
        platform: Some(platform),
        phone: None,
      },
      pixels: (9, 16),
      picture: OverlayPicture {
        size: (9, 16),
        origin: (0.0, 0.0),
        pixels: vec![0; 9 * 16 * 4],
      },
      place: [0.0, 0.0, 1.0, 1.0],
    })
  }

  fn opacities(drawn: &[(Arc<RasteredOverlay>, f32)]) -> Vec<f32> {
    drawn.iter().map(|(_, opacity)| *opacity).collect()
  }

  #[test]
  fn another_overlay_crossfades_from_the_one_shown() {
    let mut fade = OverlayFade::default();
    let start = Instant::now();
    fade.step(
      Wanted::Ready(picture(PlatformOverlay::Tiktok)),
      start,
      false,
    );
    let settled = start + FADE;
    assert_eq!(
      opacities(&fade.step(Wanted::Pending, settled, false)),
      [1.0]
    );
    // Still drawing the next one: the one shown stands.
    assert_eq!(
      opacities(&fade.step(Wanted::Pending, settled, false)),
      [1.0]
    );
    let next = Wanted::Ready(picture(PlatformOverlay::InstagramReels));
    let midway = fade.step(next, settled, false);
    assert_eq!(opacities(&midway), [1.0, 0.0]);
    let halfway = fade.step(Wanted::Pending, settled + FADE / 2, false);
    let [leaving, coming] = opacities(&halfway)[..] else {
      panic!("two overlays while they cross");
    };
    assert!((leaving + coming - 1.0).abs() < 0.01);
    assert_eq!(
      opacities(&fade.step(Wanted::Pending, settled + FADE, false)),
      [1.0]
    );
    assert!(fade.is_settled());
  }

  #[test]
  fn turned_off_it_fades_out_and_another_shape_drops_it_at_once() {
    let mut fade = OverlayFade::default();
    let start = Instant::now();
    fade.step(
      Wanted::Ready(picture(PlatformOverlay::Tiktok)),
      start,
      false,
    );
    let off = fade.step(Wanted::Off, start + FADE, false);
    assert_eq!(opacities(&off), [1.0]);
    assert!(fade.step(Wanted::Off, start + FADE * 2, false).is_empty());
    fade.step(
      Wanted::Ready(picture(PlatformOverlay::Tiktok)),
      start,
      false,
    );
    assert!(fade.step(Wanted::Elsewhere, start, false).is_empty());
  }
}
