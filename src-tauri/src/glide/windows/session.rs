// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows session state around the shared Rust runtime and captured HWND.

use std::{
  sync::atomic::{AtomicU64, Ordering},
  time::Instant,
};

use tauri::AppHandle;
use windows::Win32::{Foundation::POINT, UI::WindowsAndMessaging::GetCursorPos};

use super::{
  cursor,
  input_kind::InputKind,
  native_settings,
  target::WindowTarget,
  tween::{self, FitContext},
};
use crate::glide::{
  begin_physical,
  core::{GlideDetectorOptions, GlideEffects, GlideRuntime, GlideSample},
  events, finish,
  icon::spawn_icon_lookup,
  region_rect::PlacedRegion,
};

struct Session {
  anchor: POINT,
  id: u64,
  input: InputKind,
  last_input: Instant,
  moved: bool,
  pointer_travel: f64,
  revealed: bool,
  runtime: GlideRuntime,
  runtime_clock: Instant,
  target: WindowTarget,
}

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);
static STATE: std::sync::Mutex<Option<Session>> = std::sync::Mutex::new(None);

pub(super) fn begin(app: &AppHandle, input: InputKind) -> bool {
  if !native_settings::snapshot().enabled {
    return false;
  }
  if active_input().is_some() {
    return false;
  }
  let mut anchor = POINT::default();
  if unsafe { GetCursorPos(&mut anchor) }.is_err() {
    return false;
  }
  let Some((target, pid)) = WindowTarget::at(app, anchor) else {
    return false;
  };
  target.raise();
  let id = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);
  if let Err(error) = begin_physical(app, id, anchor.x, anchor.y) {
    eprintln!("Could not present Glide: {error}");
    return false;
  }
  spawn_icon_lookup(app, id, pid);
  let session = Session {
    anchor,
    id,
    input,
    last_input: Instant::now(),
    moved: false,
    pointer_travel: 0.0,
    revealed: false,
    runtime: GlideRuntime::new(GlideDetectorOptions {
      rest_ms: crate::glide::REST_MS,
      ..GlideDetectorOptions::default()
    }),
    runtime_clock: Instant::now(),
    target,
  };
  if let Ok(mut state) = STATE.lock() {
    if state.is_some() {
      finish(app, f64::from(anchor.x), f64::from(anchor.y), true);
      return false;
    }
    *state = Some(session);
    true
  } else {
    finish(app, f64::from(anchor.x), f64::from(anchor.y), true);
    false
  }
}

pub(super) fn promote_scroll_to_contacts() -> bool {
  STATE.lock().is_ok_and(|mut state| {
    let Some(session) = state.as_mut() else {
      return false;
    };
    if session.input != InputKind::TrackpadScroll {
      return false;
    }
    session.input = InputKind::TrackpadContacts;
    true
  })
}

pub(super) fn update(app: &AppHandle, delta_x: f64, delta_y: f64, thirds: bool) {
  let result = STATE.lock().ok().and_then(|mut state| {
    let session = state.as_mut()?;
    session.last_input = Instant::now();
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

pub(super) fn set_thirds(app: &AppHandle, thirds: bool) {
  let result = STATE.lock().ok().and_then(|mut state| {
    let session = state.as_mut()?;
    Some((session.id, session.runtime.set_thirds(thirds)))
  });
  apply(app, result);
}

pub(super) fn tick(app: &AppHandle) {
  let result = STATE
    .lock()
    .ok()
    .and_then(|mut state| {
      let session = state.as_mut()?;
      let timestamp = session.runtime_clock.elapsed().as_secs_f64() * 1_000.0;
      let effects = session.runtime.settle(timestamp);
      Some(effects.ready.then_some((session.id, effects)))
    })
    .flatten();
  apply(app, result);
}

pub(super) fn end(app: &AppHandle, cancelled: bool) {
  let session = STATE.lock().ok().and_then(|mut state| state.take());
  let Some(session) = session else {
    return;
  };
  let minimize = session.runtime.should_minimize(cancelled);
  if cancelled && session.moved {
    session.target.restore();
  }
  let settings = native_settings::snapshot();
  let landing = if !session.runtime.commits_terminal_action(cancelled)
    && !cancelled
    && session.moved
    && settings.cursor_follows
  {
    session.target.landing(session.anchor)
  } else {
    session.anchor
  };
  if minimize {
    session.target.minimize();
  }
  tween::land_cursor(landing);
  if session.revealed {
    cursor::show_cursor();
  }
  finish(
    app,
    f64::from(session.anchor.x),
    f64::from(session.anchor.y),
    cancelled,
  );
}

pub(super) fn active_input() -> Option<InputKind> {
  STATE
    .lock()
    .ok()
    .and_then(|state| state.as_ref().map(|session| session.input))
}

pub(super) fn anchor() -> Option<POINT> {
  STATE
    .lock()
    .ok()
    .and_then(|state| state.as_ref().map(|session| session.anchor))
}

pub(super) fn pointer_displacement() -> Option<f64> {
  let anchor = STATE.lock().ok().and_then(|state| {
    state
      .as_ref()
      .filter(|session| session.input.is_trackpad())
      .map(|session| session.anchor)
  })?;
  let mut pointer = POINT::default();
  unsafe { GetCursorPos(&mut pointer) }.ok()?;
  Some(f64::from(pointer.x.abs_diff(anchor.x)) + f64::from(pointer.y.abs_diff(anchor.y)))
}

pub(super) fn reveal(app: &AppHandle, session_id: u64) -> Result<(), String> {
  let blocks_hover = {
    let mut state = STATE
      .lock()
      .map_err(|_| "The Glide session state is unavailable".to_owned())?;
    let session = session_mut(&mut state, session_id)?;
    if session.revealed {
      None
    } else {
      session.revealed = true;
      Some(session.input == InputKind::Mouse)
    }
  };
  if let Some(blocks_hover) = blocks_hover {
    cursor::hide_cursor();
    crate::windows::show_glide_preview(app, blocks_hover).map_err(|error| error.to_string())?;
  }
  Ok(())
}

pub(super) fn region_moved(
  app: &AppHandle,
  session_id: u64,
  region: &PlacedRegion,
) -> Result<(), String> {
  let target = {
    let mut state = STATE
      .lock()
      .map_err(|_| "The Glide session state is unavailable".to_owned())?;
    let session = session_mut(&mut state, session_id)?;
    session.moved = true;
    session.target
  };
  let destination = target.destination(region, native_settings::snapshot().window_gap);
  tween::animate_to(
    target,
    destination.frame,
    Some(FitContext {
      app: app.clone(),
      session_id,
      gravity: destination.gravity,
      work: destination.work,
    }),
  );
  Ok(())
}

fn apply(app: &AppHandle, result: Option<(u64, GlideEffects)>) {
  let Some((session_id, effects)) = result else {
    return;
  };
  events::detection(app, effects.detection);
  if effects.reveal {
    if let Err(error) = reveal(app, session_id) {
      eprintln!("Could not reveal Glide: {error}");
    }
  }
  if let Some(region) = effects.move_to {
    if let Err(error) = region_moved(app, session_id, &region) {
      eprintln!("Could not move the Glide window: {error}");
    }
  }
}

fn session_mut(session: &mut Option<Session>, session_id: u64) -> Result<&mut Session, String> {
  session
    .as_mut()
    .filter(|session| session.id == session_id)
    .ok_or_else(|| "The Glide session has already ended".to_owned())
}

pub(super) fn accumulate_pointer_travel(distance: f64) -> f64 {
  STATE
    .lock()
    .ok()
    .and_then(|mut state| {
      state.as_mut().map(|session| {
        session.pointer_travel += distance;
        session.pointer_travel
      })
    })
    .unwrap_or(0.0)
}
