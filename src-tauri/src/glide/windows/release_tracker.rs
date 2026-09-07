// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Orders raw release recovery against low-level hook transitions.

use std::{
  collections::HashMap,
  sync::{LazyLock, Mutex},
};

#[derive(Default)]
pub(super) struct Tracker {
  last: HashMap<u32, (bool, u32)>,
}
fn newer_or_equal(a: u32, b: u32) -> bool {
  a.wrapping_sub(b) < 0x8000_0000
}
impl Tracker {
  pub(super) fn hook(&mut self, key: u32, pressed: bool, time: u32) {
    self.last.insert(key, (pressed, time));
  }
  pub(super) fn raw_press(&mut self, key: u32, time: u32) -> bool {
    if self
      .last
      .get(&key)
      .is_some_and(|(pressed, last)| !newer_or_equal(time, *last) || (!*pressed && time == *last))
    {
      return false;
    }
    self.last.insert(key, (true, time));
    true
  }
  pub(super) fn raw_release(&mut self, key: u32, time: u32) -> bool {
    let Some((pressed, last)) = self.last.get(&key).copied() else {
      return false;
    };
    if !pressed || !newer_or_equal(time, last) {
      return false;
    }
    self.last.insert(key, (false, time));
    true
  }
}
static LAST: LazyLock<Mutex<Tracker>> = LazyLock::new(|| Mutex::new(Tracker::default()));
pub(super) fn hook(key: u32, pressed: bool, time: u32) {
  if let Ok(mut s) = LAST.lock() {
    s.hook(key, pressed, time);
  }
}
pub(super) fn raw_press(key: u32, time: u32) -> bool {
  let accepted = LAST.lock().is_ok_and(|mut s| s.raw_press(key, time));
  let settings = super::native_settings::snapshot();
  if super::native_settings::matches(settings.mouse_modifier, key)
    || super::native_settings::matches(settings.monitors_modifier, key)
    || super::native_settings::matches(settings.spaces_modifier, key)
  {
    crate::glide::core::trace::input(
      "windows-release",
      format!("raw-down key={key} time={time} accepted={accepted}"),
    );
  }
  accepted
}
pub(super) fn raw_release(key: u32, time: u32) -> bool {
  let accepted = LAST.lock().is_ok_and(|mut s| s.raw_release(key, time));
  let settings = super::native_settings::snapshot();
  if super::native_settings::matches(settings.mouse_modifier, key)
    || super::native_settings::matches(settings.monitors_modifier, key)
    || super::native_settings::matches(settings.spaces_modifier, key)
  {
    crate::glide::core::trace::input(
      "windows-release",
      format!("raw-up key={key} time={time} accepted={accepted}"),
    );
  }
  accepted
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn stale_release_and_wrap_safe_ordering() {
    let mut t = Tracker::default();
    t.hook(42, true, 100);
    assert!(!t.raw_release(42, 99));
    assert!(t.raw_release(42, 101));
    t.hook(42, true, u32::MAX - 2);
    assert!(t.raw_release(42, 1));
  }
  #[test]
  fn missing_up_is_recovered_before_new_press() {
    let mut t = Tracker::default();
    t.hook(9, true, 10);
    assert!(t.raw_release(9, 11));
    assert!(t.raw_press(9, 12));
    assert!(!t.raw_release(9, 11));
  }

  #[test]
  fn equal_timestamp_down_cannot_undo_hook_release() {
    let mut t = Tracker::default();
    t.hook(77, true, 100);
    t.hook(77, false, 100);
    assert!(!t.raw_press(77, 100));
    assert!(t.raw_press(77, 101));
  }

  #[test]
  fn recovered_release_allows_the_next_press() {
    use crate::glide::settings::GlideControl;
    use keyboard_types::Code;

    let mut t = Tracker::default();
    let key =
      super::super::control::NativeControl::from_control(GlideControl::Key(Code::KeyX)).unwrap();
    t.hook(88, true, 200);
    super::super::native_settings::observe(88, true);
    assert!(super::super::native_settings::is_down(key));
    assert!(t.raw_release(88, 201));
    super::super::native_settings::observe(88, false);
    assert!(!super::super::native_settings::is_down(key));
    assert!(t.raw_press(88, 202));
    super::super::native_settings::observe(88, true);
    assert!(super::super::native_settings::is_down(key));
    assert!(!t.raw_release(88, 201));
    assert!(t.raw_release(88, 203));
    super::super::native_settings::observe(88, false);
    assert!(!super::super::native_settings::is_down(key));
  }
}
