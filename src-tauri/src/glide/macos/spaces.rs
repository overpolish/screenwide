// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Spaces uses the same titlebar resolver and cursor lifecycle as resizing,
//! with its own gesture state so a carry can finish after the fingers lift.
mod adapter;
mod input;
mod lifecycle;
mod presentation;
mod transport;

use super::tween::WindowTarget;
use crate::glide::core::{desktop_gesture::DesktopGesture, desktops::Snapshot};
use core_graphics::geometry::CGPoint;
use std::{
  sync::{
    atomic::{AtomicBool, AtomicU64},
    Arc, LazyLock, Mutex,
  },
  time::Instant,
};

pub(super) use input::handle_event;
pub(super) use lifecycle::poll;

/// Loads a card for every Space or monitor, whichever there are more of.
/// Counting Spaces asks the window server, so it stays off the main thread.
pub(super) fn preload(app: &tauri::AppHandle) {
  let app = app.clone();
  let result = std::thread::Builder::new()
    .name("glide-space-preload".into())
    .spawn(move || {
      unsafe extern "C" {
        fn sw_glide_space_preview_count() -> usize;
      }
      let spaces = unsafe { sw_glide_space_preview_count() };
      let monitors = app
        .available_monitors()
        .map_or(0, |monitors| monitors.len());
      crate::glide::preview_cards::preload(&app, spaces.max(monitors));
    });
  if let Err(error) = result {
    eprintln!("Could not start Glide Space preload: {error}");
  }
}
pub(super) fn cancel(app: &tauri::AppHandle) {
  lifecycle::request_end(app, true);
}
pub(super) fn is_active() -> bool {
  SESSION.lock().is_ok_and(|state| state.is_some())
}
pub(super) fn is_synthetic(event: &core_graphics::event::CGEvent) -> bool {
  event.get_integer_value_field(core_graphics::event::EventField::EVENT_SOURCE_USER_DATA)
    == EVENT_TAG
}

#[derive(Clone, Copy, PartialEq)]
enum Input {
  Mouse,
  Trackpad,
  Wheel,
}
struct Session {
  id: u64,
  anchor: CGPoint,
  target: WindowTarget,
  input: Input,
  clock: Instant,
  last_input: Instant,
  detector: DesktopGesture,
  snapshot: Option<Snapshot>,
  selected: Option<String>,
  icon_path: Option<std::path::PathBuf>,
  preview_phase: &'static str,
  buffered: (f64, f64),
  moving: bool,
  ending: bool,
  revealed: bool,
  cancelled: Arc<AtomicBool>,
  armed: bool,
}
static SESSION: LazyLock<Mutex<Option<Session>>> = LazyLock::new(|| Mutex::new(None));
static CLOSING: AtomicBool = AtomicBool::new(false);
static NEXT_ID: AtomicU64 = AtomicU64::new(1_000_000_000);
const EVENT_TAG: i64 = 0x5357474c494445;

pub(in crate::glide) fn set_icon(
  app: &tauri::AppHandle,
  id: u64,
  path: Option<std::path::PathBuf>,
) {
  super::session::monitors::set_icon(id, path.clone());
  presentation::set_icon(id, path.clone());
  crate::glide::preview_cards::set_icon(app, id, path);
}
