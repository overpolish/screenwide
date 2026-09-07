// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::{Duration, Instant};

use super::input_kind::InputKind;
use windows::Win32::Foundation::HWND;

#[derive(Clone, Copy)]
pub(super) struct Click {
  pub at: Instant,
  pub point: (i32, i32),
  pub hwnd: isize,
}

#[derive(Default)]
pub(super) struct Matcher {
  previous: Option<Click>,
}

impl Matcher {
  pub(super) const fn new() -> Self {
    Self { previous: None }
  }
  pub(super) fn clear(&mut self) {
    self.previous = None;
  }

  pub(super) fn observe(&mut self, current: Click, interval: Duration, radius: (i32, i32)) -> bool {
    let paired = matches(self.previous, current, interval, radius);
    self.previous = if paired { None } else { Some(current) };
    paired
  }
}

pub(super) fn trace(message: &str) {
  if std::env::var_os("SCREENWIDE_GLIDE_TRACE").is_some() {
    eprintln!("[glide-center] {message}");
  }
}

pub(super) fn allowed(app: &tauri::AppHandle, input: InputKind) -> bool {
  let settings = super::native_settings::snapshot();
  let active = super::session::active_input();
  let session_ok = match input {
    InputKind::Mouse => active.is_none() || active == Some(InputKind::Mouse),
    InputKind::TrackpadContacts | InputKind::TrackpadScroll => {
      active.is_none()
        || (active.is_some_and(InputKind::is_trackpad) && !super::session::revealed())
    }
  };
  let allowed = settings.enabled
    && settings.double_tap_center
    && !crate::shortcuts::is_capturing()
    && !super::spaces::is_closing()
    && (active == Some(InputKind::Mouse) || !crate::glide::core::activity::BusyLease::is_busy())
    && super::spaces::active_input().is_none()
    && !super::session::monitor_mode()
    && !super::native_settings::is_down(settings.monitors_modifier)
    && !super::native_settings::is_down(settings.spaces_modifier)
    && session_ok
    && (input != InputKind::Mouse || super::native_settings::is_down(settings.mouse_modifier))
    && !crate::capture_overlays::blocks_glide(app);
  trace(&format!("eligibility input={input:?} allowed={allowed}"));
  allowed
}

pub(super) fn matches(
  previous: Option<Click>,
  current: Click,
  interval: Duration,
  radius: (i32, i32),
) -> bool {
  let Some(previous) = previous else {
    return false;
  };
  previous.hwnd == current.hwnd
    && current.at.duration_since(previous.at) <= interval
    && (current.point.0 - previous.point.0).unsigned_abs() <= radius.0 as u32
    && (current.point.1 - previous.point.1).unsigned_abs() <= radius.1 as u32
}

pub(super) fn hwnd(value: isize) -> HWND {
  HWND(value as *mut std::ffi::c_void)
}

pub(super) fn center_target(
  target: super::target::WindowTarget,
  frame: crate::glide::core::GlideFrame,
  work: crate::glide::core::GlideFrame,
) {
  let x = work.x + ((work.width - frame.width) / 2.0).max(0.0);
  let y = work.y + ((work.height - frame.height) / 2.0).max(0.0);
  super::tween::animate_to(
    target,
    crate::glide::core::GlideFrame { x, y, ..frame },
    None,
  );
}

#[cfg(test)]
mod tests {
  use super::*;
  fn click(at: u64, x: i32, y: i32, hwnd: isize) -> Click {
    Click {
      at: Instant::now() + Duration::from_millis(at),
      point: (x, y),
      hwnd,
    }
  }
  #[test]
  fn matches_interval_radius_and_target() {
    assert!(matches(
      Some(click(0, 10, 10, 1)),
      click(100, 12, 11, 1),
      Duration::from_millis(500),
      (4, 4)
    ));
    assert!(!matches(
      Some(click(0, 10, 10, 1)),
      click(100, 15, 10, 1),
      Duration::from_millis(500),
      (4, 4)
    ));
    assert!(!matches(
      Some(click(0, 10, 10, 1)),
      click(100, 10, 10, 2),
      Duration::from_millis(500),
      (4, 4)
    ));
    assert!(!matches(
      Some(click(0, 10, 10, 1)),
      click(600, 10, 10, 1),
      Duration::from_millis(500),
      (4, 4)
    ));
  }
  #[test]
  fn triple_click_clears_pairing() {
    let mut matcher = Matcher::default();
    let interval = Duration::from_millis(500);
    assert!(!matcher.observe(click(0, 10, 10, 1), interval, (4, 4)));
    assert!(matcher.observe(click(100, 10, 10, 1), interval, (4, 4)));
    assert!(!matcher.observe(click(150, 10, 10, 1), interval, (4, 4)));
  }
}
