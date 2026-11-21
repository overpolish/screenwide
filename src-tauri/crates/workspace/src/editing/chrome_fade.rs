// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How the selection chrome changes from one thing to another, so nothing
//! pops in, out or across.
//!
//! The chrome is keyed by what it is drawn round: the chosen layer, the
//! chosen annotations, the objects picked in the camera, or nothing. While
//! the key holds, the chrome follows its subject frame for frame. When it
//! changes, the chrome drawn of the old subject either glides onto the new
//! one, where both are drawn alike (a layer's box onto another's, one arrow's
//! grips onto another's), or crossfades into it, where they differ or the
//! pointer is pressing, since a glide would trail the press. The gizmo's
//! rings fade in and out on their own and glide between layers
//! ([`rings`]). The engine has no timer of its own, so the change-over runs
//! on a short-lived thread that redraws the chrome until it is done, as the
//! hover halo's pulse does.

mod rings;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::{Duration, Instant};

use screenwide_osc::gpu::pipeline::FADED_OUT;
use screenwide_osc::gpu::vertices::{self as osc_gpu, RenderConstants, Vertex};

use super::*;

pub(super) use rings::add_rings;

/// How long a change-over takes: the crop's own ease, near enough.
const CHANGE: Duration = Duration::from_millis(200);

/// One frame of a change-over, near enough a display interval.
const FRAME: Duration = Duration::from_millis(16);

/// How far through a change-over begun at `since` is at `now`, linearly.
fn progress(since: Instant, now: Instant) -> f32 {
  (now.saturating_duration_since(since).as_secs_f32() / CHANGE.as_secs_f32()).clamp(0.0, 1.0)
}

/// How far a glide has gone: eased out, so the chrome leaves at once and
/// settles onto its new subject.
fn eased(since: Instant, now: Instant) -> f32 {
  1.0 - (1.0 - progress(since, now)).powi(3)
}

/// How far a fade has gone: eased in and out. Eased out as a glide is,
/// chrome coming in showed most of itself in the first frame or two, which
/// read as popping in (user report: a highlight chosen from the Screen mode).
fn blended(since: Instant, now: Instant) -> f32 {
  let t = progress(since, now);
  t * t * (3.0 - 2.0 * t)
}

enum Change {
  /// The chrome drawn before, onto the chrome now, vertex for vertex.
  Glide { from: Vec<Vertex>, since: Instant },
  /// The chrome presented before, fading out under the chrome now.
  Crossfade {
    leaving: Vec<Vertex>,
    since: Instant,
  },
}

impl Change {
  fn since(&self) -> Instant {
    match self {
      Change::Glide { since, .. } | Change::Crossfade { since, .. } => *since,
    }
  }
}

#[derive(Default)]
pub(crate) struct ChromeFade {
  /// What the chrome was last drawn round; `None` before its first draw,
  /// which shows at once.
  key: Option<u64>,
  /// The chrome of its subject as last drawn, before any change-over, and
  /// so as presented once a change-over is done.
  drawn: Vec<Vertex>,
  /// The chrome as last presented while a change-over runs, without its
  /// rings.
  presented: Vec<Vertex>,
  change: Option<Change>,
  rings: rings::RingFade,
  /// A thread is redrawing the chrome until the change-over is done.
  ticking: bool,
}

impl ChromeFade {
  /// Whether the last draw finished every change-over, which only a draw
  /// does, so the last frame of one is always presented.
  fn is_settled(&self) -> bool {
    self.change.is_none() && self.rings.is_settled()
  }

  /// The chrome as last presented, taken to change over from.
  fn take_presented(&mut self) -> Vec<Vertex> {
    if self.change.is_some() {
      std::mem::take(&mut self.presented)
    } else {
      std::mem::take(&mut self.drawn)
    }
  }

  /// The chrome to present now in place of `vertices`, the chrome of what
  /// `key` names, and of `rings`, its gizmo. `held` is a press under way;
  /// `glides` answers, if asked, whether a change may glide at all, which
  /// it may not where the person asked the system for less motion.
  pub(super) fn ease(
    &mut self,
    key: u64,
    vertices: Vec<Vertex>,
    rings: Option<turn::Rings>,
    held: bool,
    glides: impl FnOnce() -> bool,
    now: Instant,
  ) -> (Vec<Vertex>, Option<turn::Rings>) {
    let changed = self.key.is_some_and(|last| last != key);
    let glides = changed && glides();
    if changed {
      let alike = glides
        && self.change.is_none()
        && !held
        && !self.drawn.is_empty()
        && self.drawn.len() == vertices.len()
        && self
          .drawn
          .iter()
          .zip(&vertices)
          .all(|(from, to)| from.kind == to.kind);
      let from = self.take_presented();
      self.change = Some(if alike {
        Change::Glide { from, since: now }
      } else {
        Change::Crossfade {
          leaving: from,
          since: now,
        }
      });
    }
    self.key = Some(key);
    // A press caught up in a glide takes the chrome at once: what has glided
    // so far fades out under it rather than trailing behind the pointer.
    if held {
      if let Some(Change::Glide { since, .. }) = self.change {
        self.change = Some(Change::Crossfade {
          leaving: std::mem::take(&mut self.presented),
          since,
        });
      }
    }
    self.drawn.clone_from(&vertices);
    let rings = self.rings.ease(rings, changed, glides && !held, now);
    let running = self
      .change
      .as_ref()
      .is_some_and(|change| progress(change.since(), now) < 1.0);
    let presented = match &self.change {
      Some(Change::Glide { from, since }) if running && from.len() == vertices.len() => {
        let t = eased(*since, now);
        from
          .iter()
          .zip(&vertices)
          .map(|(from, to)| between(*from, *to, t))
          .collect()
      }
      Some(Change::Crossfade { leaving, since }) if running => {
        let t = blended(*since, now);
        leaving
          .iter()
          .map(|vertex| faded(*vertex, 1.0 - t))
          .chain(vertices.iter().map(|vertex| faded(*vertex, t)))
          .collect()
      }
      _ => {
        self.change = None;
        self.presented.clear();
        return (vertices, rings);
      }
    };
    self.presented.clone_from(&presented);
    (presented, rings)
  }
}

/// `vertex` showing `shown` as much as it did.
fn faded(vertex: Vertex, shown: f32) -> Vertex {
  let was = 1.0 - vertex.fade as f32 / FADED_OUT as f32;
  Vertex {
    fade: ((1.0 - was * shown.clamp(0.0, 1.0)) * FADED_OUT as f32).round() as u32,
    ..vertex
  }
}

/// `t` of the way from `from` to `to`.
fn between(from: Vertex, to: Vertex, t: f32) -> Vertex {
  let lerp = |a: [f32; 2], b: [f32; 2]| [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t];
  Vertex {
    position: lerp(from.position, to.position),
    uv: lerp(from.uv, to.uv),
    ..to
  }
}

/// What the chrome is drawn round: the objects picked in the camera, the
/// annotations chosen on a layer, or the chosen layer, as the chrome shows
/// whichever comes first; nothing at all, else. The crop's window keeps the
/// layer's key, since it takes over from the select tool's chrome by its own
/// ease.
pub(super) fn key(state: &SurfaceState, objects: bool, annotations: bool, frame: bool) -> u64 {
  let mut hasher = DefaultHasher::new();
  let layer = state
    .selection
    .map(|selection| (selection.pane_index, selection.layer_id));
  if objects {
    1_u8.hash(&mut hasher);
    layer.hash(&mut hasher);
  } else if annotations {
    2_u8.hash(&mut hasher);
    layer.hash(&mut hasher);
    state.annotation.selected.hash(&mut hasher);
    for member in &state.annotation.group {
      (member.layer_id, member.kind, member.member).hash(&mut hasher);
    }
  } else if frame {
    3_u8.hash(&mut hasher);
    layer.hash(&mut hasher);
  }
  hasher.finish()
}

/// Whether a press is under way on the workspace: a layer or its crop moved
/// or resized, a crop or frame drawn, a turn or an annotation dragged.
pub(super) fn is_held(state: &SurfaceState) -> bool {
  state.gesture.is_some()
    || state.crop_draw.is_some()
    || state.frame_resize.is_some()
    || state.turn.drag.is_some()
    || state.turn.image.is_some()
    || annotation::is_dragging(state)
}

/// Redraws the chrome every frame until its change-over is done, on a
/// thread of its own; one at a time. MUST be called with the surface state
/// and `fade` held, as `draw_selection` holds them.
pub(super) fn keep_drawing(inner: &SurfaceInner, fade: &mut ChromeFade) {
  if fade.ticking || fade.is_settled() {
    return;
  }
  let Some(inner) = inner.this.upgrade() else {
    return;
  };
  fade.ticking = true;
  std::thread::spawn(move || redraw_until_done(&inner));
}

/// A draw held back by a layout or batch past every change-over's end gives
/// way here, and the next draw the engine makes finishes it.
const GIVE_UP: Duration = Duration::from_secs(1);

fn redraw_until_done(inner: &Arc<SurfaceInner>) {
  let started = Instant::now();
  loop {
    std::thread::sleep(FRAME);
    let Ok(state) = inner.state.lock() else {
      return;
    };
    draw_selection(inner, &state);
    let Ok(mut fade) = inner.chrome_fade.lock() else {
      return;
    };
    if fade.is_settled() || started.elapsed() > GIVE_UP {
      fade.ticking = false;
      return;
    }
  }
}

#[cfg(test)]
mod tests;
