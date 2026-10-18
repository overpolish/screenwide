// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A live session's detector samples, and what their effects do: reveal the
//! preview, hand steps to the monitor picker, carry the window to a region.

use tauri::AppHandle;

use super::{with_session, with_session_id, GlidePlatform};
use crate::glide::core::tween::{animate_to, GlideWindow};
use crate::glide::core::tween_settle::FitContext;
use crate::glide::core::{frames_match, GlideEffects, GlideSample};
use crate::glide::events;
use crate::glide::region_rect::PlacedRegion;

/// One detector sample of the gesture's travel.
pub(crate) fn update<P: GlidePlatform>(app: &AppHandle, delta_x: f64, delta_y: f64, thirds: bool) {
  let result = with_session::<P, _>(|session| {
    P::on_sample(&mut session.extra, delta_x != 0.0 || delta_y != 0.0);
    let timestamp = session.timestamp();
    let effects = session.runtime.update(GlideSample {
      delta_x,
      delta_y,
      thirds,
      timestamp,
    });
    (session.id, effects)
  });
  apply::<P>(app, result);
}

pub(crate) fn set_thirds<P: GlidePlatform>(app: &AppHandle, thirds: bool) {
  let result = with_session::<P, _>(|session| (session.id, session.runtime.set_thirds(thirds)));
  apply::<P>(app, result);
}

/// Resting input produces no samples, so the platform's poll completes the
/// detector's rest gate.
pub(crate) fn settle<P: GlidePlatform>(app: &AppHandle) {
  let result = with_session::<P, _>(|session| {
    let effects = session.runtime.settle(session.timestamp());
    (effects.ready || effects.detection.changed || effects.monitor_step.is_some())
      .then_some((session.id, effects))
  })
  .flatten();
  apply::<P>(app, result);
}

pub(super) fn finish_opening<P: GlidePlatform>(app: &AppHandle) {
  let result = with_session::<P, _>(|session| {
    (
      session.id,
      session.runtime.finish_opening(session.timestamp()),
    )
  });
  apply::<P>(app, result);
}

fn apply<P: GlidePlatform>(app: &AppHandle, result: Option<(u64, GlideEffects)>) {
  let Some((id, effects)) = result else {
    return;
  };
  events::detection(app, effects.detection);
  // Readiness is shared with the preview pulse; a sample that immediately
  // starts another fold must not tick while the detector is settling again.
  if effects.ready {
    P::on_ready(app);
  }
  if P::monitor_effects(app, id, effects.monitor_step, effects.ready) {
    return;
  }
  if effects.reveal {
    if let Err(error) = reveal::<P>(app, id) {
      eprintln!("Could not reveal Glide: {error}");
    }
  }
  if let Some(region) = effects.move_to {
    if let Err(error) = region_moved::<P>(app, id, &region) {
      eprintln!("Could not move the Glide window: {error}");
    }
  }
}

/// Shows the preview once the detector has a destination to draw. Until then
/// a session is invisible, so a stray scroll over a titlebar neither flashes
/// an empty window nor takes the pointer away.
pub(crate) fn reveal<P: GlidePlatform>(app: &AppHandle, id: u64) -> Result<(), String> {
  let input = with_session_id::<P, _>(id, |session| {
    if session.revealed {
      return None;
    }
    session.revealed = true;
    P::hide_pointer();
    Some(session.input)
  })?;
  match input {
    Some(input) => P::show_preview(app, id, input),
    None => Ok(()),
  }
}

/// Moves the session's window into a region of the work area it opened on.
/// This is every transition of a live gesture, not the end of one: the window
/// follows the hand, and the lift has nothing left to place. Everything the
/// animation needs leaves the lock with the destination, since the window
/// calls it makes must never run while the input hooks wait on that lock.
pub(crate) fn region_moved<P: GlidePlatform>(
  app: &AppHandle,
  id: u64,
  region: &PlacedRegion,
) -> Result<(), String> {
  let (target, destination, fit) = with_session_id::<P, _>(id, |session| {
    session.moved = true;
    let (destination, gravity) = P::region_frame(session, region);
    session.returned_to_origin = frames_match(session.original_frame, destination, 1.0);
    let fit = FitContext {
      app: app.clone(),
      session_id: id,
      gravity,
      work: session.work,
    };
    (session.target.duplicate(), destination, fit)
  })?;
  animate_to(target, destination, Some(fit));
  Ok(())
}
