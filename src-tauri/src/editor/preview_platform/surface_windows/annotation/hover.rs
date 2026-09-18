// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The halo that grows under the arrow the pointer is resting on.
//!
//! The twin of `annotation_update_hover` in
//! `recording_preview_surface_macos+annotation_chrome.m`. That side drives the
//! pulse from the main queue, which already runs a display-rate timer for the
//! ruler's hover; DirectComposition has no such timer, so the pulse runs on a
//! short-lived thread that stops as soon as it is no longer the current one.
//!
//! Only the progress and the picture's on-screen width cross the facade. The
//! width the halo grows to lives above it, in
//! `screenshot_preview::annotation::hover_width_points`, so both backends
//! grow the same halo from the same numbers.
//!
//! As everywhere in this module, the report happens with no state held: the
//! callback comes straight back in to re-present.

use std::sync::Arc;
use std::time::{Duration, Instant};

use super::*;

/// How long the halo takes to grow, matching the ruler's own hover pulse.
const HOVER_DURATION: Duration = Duration::from_millis(160);

/// One frame of the pulse, near enough a display interval.
const HOVER_FRAME: Duration = Duration::from_millis(16);

/// What the callback is told: the arrow under the pointer (or -1), how far
/// through the pulse it is, and how wide the layer's picture is drawn.
#[derive(Clone, Copy)]
struct Report {
  index: i32,
  progress: f64,
  image_points: f64,
}

fn report_for(state: &SurfaceState) -> Option<Report> {
  let index = state.annotation.hovered;
  // Letting go needs no picture to measure against: the width is zero and
  // everything above the facade reads a negative index as "no halo". Asking
  // for an image here would strand the halo whenever the selected layer has
  // none of its own - a keyboard shortcut is drawn, not placed, so it has no
  // source rectangle at all.
  if index < 0 {
    return Some(Report {
      index,
      progress: 0.0,
      image_points: 0.0,
    });
  }
  let progress = state
    .annotation
    .hover_started
    .map_or(0.0, |started| started.elapsed().as_secs_f64())
    / HOVER_DURATION.as_secs_f64();
  // The halo is measured against the picture the hovered annotation is drawn
  // in, which is its own layer's - not the selected one's. A shortcut or
  // another layer can hold the selection while the pointer rests on an arrow.
  let image = item_image_frame(state, index)?;
  Some(Report {
    index,
    progress: progress.clamp(0.0, 1.0),
    image_points: image.width,
  })
}

/// MUST be called with no surface state held.
fn report(inner: &SurfaceInner, report: Report) {
  if let Ok(mut callbacks) = inner.callbacks.lock() {
    if let Some(callback) = callbacks.annotation_hover.as_mut() {
      callback(report.index, report.progress, report.image_points);
    }
  }
}

/// Runs the pulse until it finishes or another arrow takes the halo. Nothing
/// is integrated across frames: each report is derived from the clock, so a
/// dropped frame changes nothing about where the halo ends up.
fn pulse(inner: Arc<SurfaceInner>, revision: u64) {
  std::thread::spawn(move || loop {
    std::thread::sleep(HOVER_FRAME);
    let Some((sample, elapsed)) = ({
      let Ok(state) = inner.state.lock() else {
        return;
      };
      if state.annotation.hover_revision != revision || state.annotation.hovered < 0 {
        return;
      }
      let elapsed = state
        .annotation
        .hover_started
        .map_or(HOVER_DURATION, |started| started.elapsed());
      report_for(&state).map(|sample| (sample, elapsed))
    }) else {
      return;
    };
    report(&inner, sample);
    if elapsed >= HOVER_DURATION {
      return;
    }
  });
}

/// Re-measures the halo against the picture's current size. The width above
/// the facade is in canvas pixels, so a zoom that redraws the picture at a
/// new size strands the halo at the size the pointer arrived to until it is
/// measured again.
///
/// This is called with the surface state held, so the report rides a
/// short-lived thread the way the pulse does. One in flight is enough: a
/// zoom drag moves the transform on every sample, and they all want the
/// same single report of where the halo has ended up.
pub(crate) fn refresh(inner: &Arc<SurfaceInner>, state: &mut SurfaceState) {
  if state.annotation.hovered < 0 || state.annotation.hover_refreshing {
    return;
  }
  state.annotation.hover_refreshing = true;
  let inner = Arc::clone(inner);
  std::thread::spawn(move || {
    std::thread::sleep(HOVER_FRAME);
    let Some(sample) = ({
      let Ok(mut state) = inner.state.lock() else {
        return;
      };
      state.annotation.hover_refreshing = false;
      if state.annotation.hovered < 0 {
        return;
      }
      report_for(&state)
    }) else {
      return;
    };
    report(&inner, sample);
  });
}

/// The arrow the pointer rests on, if the halo has any say. A grip belongs to
/// the arrow it was drawn for, so resting on one halos that whole arrow,
/// exactly as resting on its shaft does.
fn hovered_at(state: &SurfaceState, point: (f64, f64), dragging: bool) -> i32 {
  if dragging || state.annotation.mode == MODE_NONE {
    return -1;
  }
  if handle_at_point(state, point).is_some() {
    return state.annotation.selected;
  }
  shaft_at_point(state, point).map_or(-1, |index| index as i32)
}

/// Moves the halo to whatever is under the pointer. A press is never a hover:
/// the halo goes out before anything moves.
///
/// An ordinary move over the same arrow reports nothing at all - only a change
/// of arrow starts a pulse, and only the pulse re-presents.
pub(crate) fn update(inner: &Arc<SurfaceInner>, point: (f64, f64), dragging: bool) {
  let Some((sample, revision, pulsing)) = ({
    let Ok(mut state) = inner.state.lock() else {
      return;
    };
    let hovered = hovered_at(&state, point, dragging);
    if hovered == state.annotation.hovered {
      return;
    }
    state.annotation.hovered = hovered;
    state.annotation.hover_started = Some(Instant::now());
    state.annotation.hover_revision += 1;
    let revision = state.annotation.hover_revision;
    report_for(&state).map(|sample| (sample, revision, hovered >= 0))
  }) else {
    return;
  };
  report(inner, sample);
  if pulsing {
    pulse(Arc::clone(inner), revision);
  }
}
