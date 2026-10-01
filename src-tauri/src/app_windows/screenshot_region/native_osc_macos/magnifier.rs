// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::c_void;

use super::ffi;

pub fn set_source(
  view: *mut c_void,
  display_id: u32,
  rgba: &[u8],
  width: u32,
  height: u32,
) -> bool {
  unsafe {
    ffi::screenwide_region_osc_set_magnifier_source(
      view,
      display_id,
      rgba.as_ptr(),
      rgba.len(),
      width,
      height,
    ) != 0
  }
}
