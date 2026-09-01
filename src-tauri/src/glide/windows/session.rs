// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows session state around the shared Rust runtime and captured HWND.

use std::{
  sync::atomic::{AtomicU64, Ordering},
  time::{Duration, Instant},
};

use tauri::AppHandle;
use windows::Win32::{
  Foundation::POINT,
  UI::WindowsAndMessaging::{GetCursorPos, SetCursorPos},
};

use super::{cursor, native_settings, target::WindowTarget};
use crate::glide::{
  begin_physical,
  core::{GlideDetectorOptions, GlideEffects, GlideRuntime, GlideSample},
  events, finish,
  fit::{emit_fit, FitRect, GlideFitEvent},
  icon::spawn_icon_lookup,
  region_rect::PlacedRegion,
};

const TRACKPAD_IDLE_TIMEOUT: Duration = Duration::from_millis(180);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum InputKind {
  Mouse,
  Trackpad,
}

struct Session {
  anchor: POINT,
  id: u64,
  input: InputKind,
  last_input: Instant,
  moved: bool,
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
  let mut anchor = POINT::default();
  if unsafe { GetCursorPos(&mut anchor) }.is_err() {
    return false;
  }
  let Some((target, pid)) = WindowTarget::at(anchor) else {
    return false;
  };
  let id = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);
  if let Err(error) = begin_physical(app, id, anchor.x, anchor.y) {
    eprintln!("Could not present Glide: {error}");
    return false;
  }
  spawn_icon_lookup(app, id, pid);
  let rest_ms = native_settings::snapshot().rest_ms;
  let session = Session {
    anchor,
    id,
    input,
    last_input: Instant::now(),
    moved: false,
    revealed: false,
    runtime: GlideRuntime::new(GlideDetectorOptions {
      rest_ms,
      ..GlideDetectorOptions::default()
    }),
    runtime_clock: Instant::now(),
    target,
  };
  if let Ok(mut state) = STATE.lock() {
    *state = Some(session);
    true
  } else {
    finish(app, f64::from(anchor.x), f64::from(anchor.y), true);
    false
  }
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
  let (result, timed_out) = STATE
    .lock()
    .ok()
    .and_then(|mut state| {
      let session = state.as_mut()?;
      let timestamp = session.runtime_clock.elapsed().as_secs_f64() * 1_000.0;
      let effects = session.runtime.settle(timestamp);
      let result = effects.ready.then_some((session.id, effects));
      let timed_out = session.input == InputKind::Trackpad
        && session.last_input.elapsed() >= TRACKPAD_IDLE_TIMEOUT;
      Some((result, timed_out))
    })
    .unwrap_or((None, false));
  apply(app, result);
  if timed_out {
    end(app, false);
  }
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
  let landing = if !cancelled && session.moved && settings.cursor_follows {
    session.target.landing(session.anchor)
  } else {
    session.anchor
  };
  if minimize {
    session.target.minimize();
  }
  let _ = unsafe { SetCursorPos(landing.x, landing.y) };
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

pub(super) fn reveal(app: &AppHandle, session_id: u64) -> Result<(), String> {
  let reveal = {
    let mut state = STATE
      .lock()
      .map_err(|_| "The Glide session state is unavailable".to_owned())?;
    let session = session_mut(&mut state, session_id)?;
    if session.revealed {
      false
    } else {
      session.revealed = true;
      true
    }
  };
  if reveal {
    cursor::hide_cursor();
    crate::windows::show_glide_preview(app).map_err(|error| error.to_string())?;
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
  let placement = target.place(region, native_settings::snapshot().window_gap)?;
  emit_fit(
    app,
    GlideFitEvent {
      session_id,
      fits: placement.fits,
      actual: FitRect {
        x: placement.actual.x,
        y: placement.actual.y,
        width: placement.actual.width,
        height: placement.actual.height,
      },
    },
  )
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
