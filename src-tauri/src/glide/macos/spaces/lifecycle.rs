// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{preview_windows, transport, Input, Session, CLOSING, NEXT_ID, SESSION};
use crate::glide::{
  begin_logical, finish, finish_with_fade,
  icon::spawn_icon_lookup,
  platform::{cursor, native_settings, session},
};
use core_graphics::geometry::CGPoint;
use std::{
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
  time::{Duration, Instant},
};
use tauri::AppHandle;

pub(super) fn begin(app: &AppHandle, input: Input, anchor: CGPoint) -> bool {
  if CLOSING.load(Ordering::Acquire)
    || preview_windows::is_dismissing()
    || cursor::is_cursor_pinned()
  {
    return false;
  }
  let Some((target, _, pid)) = session::target_at(app, anchor) else {
    return false;
  };
  preview_windows::initialize(app);
  target.raise();
  if cursor::pin_cursor(anchor).is_err() {
    return false;
  }
  let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
  if begin_logical(app, id, anchor.x, anchor.y).is_err() {
    cursor::release_cursor(anchor, false);
    return false;
  }
  let now = Instant::now();
  let mut state = SESSION.lock().unwrap_or_else(|error| error.into_inner());
  *state = Some(Session {
    id,
    anchor,
    target,
    input,
    clock: now,
    last_input: now,
    detector: Default::default(),
    snapshot: None,
    selected: None,
    icon_path: None,
    preview_phase: "idle",
    buffered: (0.0, 0.0),
    moving: false,
    ending: false,
    revealed: false,
    cancelled: Arc::new(AtomicBool::new(false)),
  });
  drop(state);
  spawn_icon_lookup(app, id, pid);
  transport::inspect(app, id);
  true
}

pub(super) fn request_end(app: &AppHandle, cancel: bool) {
  if let Ok(mut state) = SESSION.lock() {
    if let Some(session) = state.as_mut() {
      session.ending = true;
      if cancel {
        session.cancelled.store(true, Ordering::Release);
      }
    }
  }
  transport::commit(app);
}

pub(super) fn close(app: &AppHandle) {
  let Some(session) = SESSION.lock().ok().and_then(|mut state| state.take()) else {
    return;
  };
  CLOSING.store(true, Ordering::Release);
  let anchor = session.anchor;
  let cancelled = session.cancelled.load(Ordering::Acquire);
  preview_windows::dismiss(app, cancelled);
  let done = move || {
    preview_windows::after_dismissed(Box::new(|| CLOSING.store(false, Ordering::Release)));
  };
  if cancelled || !session.revealed {
    cursor::release_cursor(anchor, session.revealed);
    finish(app, anchor.x, anchor.y, cancelled);
    if app.run_on_main_thread(done).is_err() {
      CLOSING.store(false, Ordering::Release);
    }
  } else {
    finish_with_fade(
      app,
      anchor.x,
      anchor.y,
      Box::new(move || cursor::release_cursor(anchor, true)),
      Box::new(done),
    );
  }
}

pub(in crate::glide::platform) fn poll(app: &AppHandle) {
  let mut ready = None;
  let mut end = false;
  if let Ok(mut state) = SESSION.lock() {
    if let Some(session) = state.as_mut() {
      let settings = native_settings::snapshot();
      if !settings.enabled || crate::capture_overlays::blocks_glide(app) {
        session.cancelled.store(true, Ordering::Release);
        session.ending = true;
      }
      if session.input == Input::Wheel && session.last_input.elapsed() >= Duration::from_millis(300)
      {
        session.ending = true;
      }
      if !session.ending
        && session
          .detector
          .settle(session.clock.elapsed().as_secs_f64() * 1000.0)
      {
        ready = session
          .snapshot
          .clone()
          .map(|snapshot| (session.id, snapshot, session.selected.clone()));
      }
      end = session.ending && !session.moving;
    }
  }
  if let Some((id, snapshot, selected)) = ready {
    super::presentation::publish(app, id, &snapshot, selected.as_deref(), "ready");
    if native_settings::snapshot().haptics {
      let _ = crate::glide::platform::haptic(app);
    }
  }
  if end {
    transport::commit(app);
  }
}
