// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows session state around the shared Rust runtime and captured HWND.

use std::{
  path::PathBuf,
  sync::atomic::{AtomicU64, Ordering},
  time::Instant,
};

use tauri::AppHandle;
use windows::Win32::{Foundation::POINT, UI::WindowsAndMessaging::GetCursorPos};

use super::monitors;
use super::preview_windows;
use super::{
  cursor,
  input_kind::InputKind,
  native_settings,
  target::WindowTarget,
  tween::{self, FitContext},
};
use crate::glide::{
  begin_physical,
  core::{GlideEffects, GlideRuntime, GlideSample},
  events, finish, finish_with_fade,
  icon::spawn_icon_lookup,
  region_rect::PlacedRegion,
};

#[path = "session/access.rs"]
mod access;
#[path = "session/effects.rs"]
mod effects;
#[path = "session/end.rs"]
mod end;
pub(super) use end::end;

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
  original_frame: crate::glide::core::GlideFrame,
  monitors: Option<monitors::Selection>,
  icon_path: Option<PathBuf>,
  armed: bool,
}

static NEXT_SESSION_ID: AtomicU64 = AtomicU64::new(1);
static STATE: std::sync::Mutex<Option<Session>> = std::sync::Mutex::new(None);
pub(super) fn begin(app: &AppHandle, input: InputKind) -> bool {
  if crate::glide::core::activity::BusyLease::is_busy() {
    return false;
  }
  if super::navigation::cancelled() {
    return false;
  }
  if crate::shortcuts::is_capturing()
    || native_settings::is_down(native_settings::snapshot().spaces_modifier)
  {
    return false;
  }
  if super::spaces::active_input().is_some() || super::spaces::is_closing() {
    return false;
  }
  if crate::capture_overlays::blocks_glide(app) || !native_settings::snapshot().enabled {
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
  let Ok(original_frame) = target.frame() else {
    return false;
  };
  let id = NEXT_SESSION_ID.fetch_add(1, Ordering::Relaxed);
  let settings = native_settings::snapshot();
  if native_settings::is_down(settings.monitors_modifier) && access::monitor_commit_latched() {
    return false;
  }
  crate::glide::core::trace::input(
    "windows-session",
    format!(
      "begin input={input:?} mouse={} monitors={} spaces={}",
      native_settings::is_down(settings.mouse_modifier),
      native_settings::is_down(settings.monitors_modifier),
      native_settings::is_down(settings.spaces_modifier)
    ),
  );
  let monitors = native_settings::is_down(settings.monitors_modifier)
    .then(|| monitors::capture(app, anchor))
    .flatten();
  if native_settings::is_down(settings.monitors_modifier) && monitors.is_none() {
    return false;
  }
  target.raise();
  if let Err(error) = begin_physical(app, id, anchor.x, anchor.y) {
    eprintln!("Could not present Glide: {error}");
    return false;
  }
  let mut session = Session {
    anchor,
    id,
    input,
    last_input: Instant::now(),
    moved: false,
    pointer_travel: 0.0,
    revealed: false,
    runtime: GlideRuntime::default(),
    runtime_clock: Instant::now(),
    target,
    original_frame,
    monitors,
    icon_path: None,
    armed: false,
  };
  if session.monitors.is_some() {
    session.runtime.begin_monitor_navigation();
  }
  if let Ok(mut state) = STATE.lock() {
    if state.is_some() {
      finish(app, f64::from(anchor.x), f64::from(anchor.y), true);
      return false;
    }
    *state = Some(session);
    spawn_icon_lookup(app, id, pid);
    true
  } else {
    finish(app, f64::from(anchor.x), f64::from(anchor.y), true);
    false
  }
}

pub(super) fn arm_monitor(app: &AppHandle) -> bool {
  let Some(id) = STATE
    .lock()
    .ok()
    .and_then(|state| state.as_ref().map(|session| session.id))
  else {
    return false;
  };
  if !STATE.lock().is_ok_and(|mut state| {
    let Some(session) = state.as_mut().filter(|session| session.id == id) else {
      return false;
    };
    session.armed = true;
    true
  }) {
    return false;
  }
  effects::update_monitor(app, id, None, true);
  true
}

pub(super) fn update(app: &AppHandle, delta_x: f64, delta_y: f64, thirds: bool) {
  let result = STATE.lock().ok().and_then(|mut state| {
    let session = state.as_mut()?;
    if delta_x != 0.0 || delta_y != 0.0 {
      session.armed = false;
    }
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
      Some((effects.ready || effects.detection.changed).then_some((session.id, effects)))
    })
    .flatten();
  apply(app, result);
}

pub(super) use access::{
  accumulate_pointer_travel, active_input, anchor, clear_monitor_commit,
  clear_monitor_commit_if_released, monitor_mode, next_id, pointer_displacement,
  promote_armed_to_mouse, promote_scroll_to_contacts, revealed, set_icon, target, target_hwnd,
};
use effects::apply;
