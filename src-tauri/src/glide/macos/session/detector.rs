// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The native session's platform-neutral detector boundary. The event tap
//! supplies normalized deltas; this module owns the clock and emits results.

use tauri::AppHandle;

use super::SharedState;
use crate::glide::{
  core::{GlideEffects, GlideSample},
  events,
};

pub(in crate::glide::platform) fn update(
  app: &AppHandle,
  state: &SharedState,
  delta_x: f64,
  delta_y: f64,
  thirds: bool,
) {
  let result = state.lock().ok().and_then(|mut state| {
    let session = state.session.as_mut()?;
    let effects = session.runtime.update(GlideSample {
      delta_x,
      delta_y,
      thirds,
      timestamp: session.runtime_clock.elapsed().as_secs_f64() * 1_000.0,
    });
    Some((session.id, effects))
  });
  apply(app, result);
}

pub(in crate::glide::platform) fn set_thirds(app: &AppHandle, state: &SharedState, thirds: bool) {
  let result = state.lock().ok().and_then(|mut state| {
    let session = state.session.as_mut()?;
    Some((session.id, session.runtime.set_thirds(thirds)))
  });
  apply(app, result);
}

/// Resting fingers produce no input callback, so the event tap's existing
/// 16-ms run-loop poll completes the detector's rest gate.
pub(in crate::glide::platform) fn settle(app: &AppHandle, state: &SharedState) {
  let result = state.lock().ok().and_then(|mut state| {
    let session = state.session.as_mut()?;
    let timestamp = session.runtime_clock.elapsed().as_secs_f64() * 1_000.0;
    let effects = session.runtime.settle(timestamp);
    (effects.ready || effects.detection.changed || effects.monitor_step.is_some())
      .then_some((session.id, effects))
  });
  apply(app, result);
}

pub(in crate::glide::platform) fn finish_opening(app: &AppHandle, state: &SharedState) {
  let result = state.lock().ok().and_then(|mut state| {
    let session = state.session.as_mut()?;
    let timestamp = session.runtime_clock.elapsed().as_secs_f64() * 1_000.0;
    Some((session.id, session.runtime.finish_opening(timestamp)))
  });
  apply(app, result);
}

fn apply(app: &AppHandle, result: Option<(u64, GlideEffects)>) {
  let Some((session_id, effects)) = result else {
    return;
  };
  events::detection(app, effects.detection);
  // Readiness is shared with the preview pulse; a sample that immediately
  // starts another fold must not tick while the detector is settling again.
  if effects.ready && super::super::native_settings::snapshot().haptics {
    let _ = super::super::haptic(app);
  }
  if super::monitors::update(app, session_id, effects.monitor_step, effects.ready) {
    return;
  }
  if effects.reveal {
    if let Err(error) = super::reveal(app, session_id) {
      eprintln!("Could not reveal Glide: {error}");
    }
  }
  if let Some(region) = effects.move_to {
    if let Err(error) = super::region_moved(app, session_id, &region) {
      eprintln!("Could not move the Glide window: {error}");
    }
  }
}
