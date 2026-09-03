// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Normalizes Precision Touchpad and legacy high-resolution wheel motion into
//! the shared Glide detector's physical finger coordinates.

use std::sync::atomic::{AtomicBool, Ordering};

use windows::Win32::UI::WindowsAndMessaging::{
  SystemParametersInfoW, SYSTEM_PARAMETERS_INFO_ACTION, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
};

use super::{begin_session, native_settings, session, InputKind, APP};

const SPI_GETTOUCHPADPARAMETERS: SYSTEM_PARAMETERS_INFO_ACTION =
  SYSTEM_PARAMETERS_INFO_ACTION(0x00ae);
const TOUCHPAD_PARAMETERS_VERSION_1: u32 = 1;
const TOUCHPAD_SCROLL_DIRECTION_REVERSED: u32 = 1 << 9;
static SCROLL_REVERSED: AtomicBool = AtomicBool::new(false);

#[derive(Default)]
#[repr(C)]
struct TouchpadParametersV1 {
  version_number: u32,
  max_supported_contacts: u32,
  legacy_touchpad_features: u32,
  status_bits: u32,
  settings_bits: u32,
  sensitivity_level: u32,
  cursor_speed: u32,
  feedback_intensity: u32,
  click_force_sensitivity: u32,
  right_click_zone_width: u32,
  right_click_zone_height: u32,
}

pub(super) fn handle_legacy_wheel(horizontal: bool, delta: i16) {
  let (delta_x, delta_y) = legacy_wheel_delta(horizontal, delta);
  handle_delta(delta_x, delta_y);
}

fn legacy_wheel_delta(horizontal: bool, delta: i16) -> (f64, f64) {
  if horizontal {
    // Keep legacy and Raw Input horizontal packets in the same coordinate
    // space before applying the user's physical-scroll-direction preference.
    (-f64::from(delta), 0.0)
  } else {
    (0.0, f64::from(delta))
  }
}

pub(super) fn handle_delta(delta_x: f64, delta_y: f64) {
  if super::native_trackpad::suppresses_scroll_fallback() {
    return;
  }
  if session::active_input().is_none() {
    SCROLL_REVERSED.store(scroll_reversed(), Ordering::Relaxed);
    if !begin_session(InputKind::TrackpadScroll) {
      return;
    }
  }
  if session::active_input() != Some(InputKind::TrackpadScroll) {
    return;
  }
  if let Some(app) = APP.get() {
    let settings = native_settings::snapshot();
    let (delta_x, delta_y) =
      physical_delta(delta_x, delta_y, SCROLL_REVERSED.load(Ordering::Relaxed));
    session::update(
      app,
      delta_x,
      delta_y,
      native_settings::is_down(settings.thirds_modifier),
    );
  }
}

fn physical_delta(delta_x: f64, delta_y: f64, scroll_reversed: bool) -> (f64, f64) {
  if scroll_reversed {
    (-delta_x, -delta_y)
  } else {
    (delta_x, delta_y)
  }
}

fn scroll_reversed() -> bool {
  let mut parameters = TouchpadParametersV1 {
    version_number: TOUCHPAD_PARAMETERS_VERSION_1,
    ..Default::default()
  };
  let result = unsafe {
    SystemParametersInfoW(
      SPI_GETTOUCHPADPARAMETERS,
      std::mem::size_of::<TouchpadParametersV1>() as u32,
      Some(std::ptr::from_mut(&mut parameters).cast()),
      SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS::default(),
    )
  };
  result.is_ok() && parameters.settings_bits & TOUCHPAD_SCROLL_DIRECTION_REVERSED != 0
}

#[cfg(test)]
mod tests {
  use super::{legacy_wheel_delta, physical_delta};

  #[test]
  fn trackpad_motion_ignores_the_content_scroll_direction() {
    assert_eq!(
      physical_delta(-20.0, -30.0, false),
      physical_delta(20.0, 30.0, true)
    );
  }

  #[test]
  fn legacy_precision_wheel_matches_raw_input_axes() {
    assert_eq!(legacy_wheel_delta(false, -42), (0.0, -42.0));
    assert_eq!(legacy_wheel_delta(true, 17), (-17.0, 0.0));
  }
}
