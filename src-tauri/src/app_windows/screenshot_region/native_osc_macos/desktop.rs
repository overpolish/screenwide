// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::c_void;

use crate::osc::{
  desktop::{DesktopBinding, DesktopDisplay},
  geometry::{Point, Rect, Size},
};

use super::ffi;

const MAX_DISPLAYS: usize = 16;

/// Translates AppKit's display union into the shared desktop binding. All
/// projection and owner selection happens in the shared OSC runtime.
pub fn configure_window(view: *mut c_void, anchor_id: u32) -> Result<DesktopBinding, String> {
  let mut native = [ffi::NativeDesktopDisplay::default(); MAX_DISPLAYS];
  let mut width = 0.0;
  let mut height = 0.0;
  let mut resolved_anchor_id = anchor_id;
  let mut layout_changed = 0;
  let count = unsafe {
    ffi::screenwide_region_osc_configure_desktop(
      view,
      anchor_id,
      native.as_mut_ptr(),
      native.len(),
      &mut width,
      &mut height,
      &mut resolved_anchor_id,
      &mut layout_changed,
    )
  };
  let displays = native[..count.min(native.len())]
    .iter()
    .map(|display| DesktopDisplay {
      id: display.id,
      origin: Point {
        x: display.x,
        y: display.y,
      },
      size: Size {
        width: display.width,
        height: display.height,
      },
      scale: display.scale,
    })
    .collect::<Vec<_>>();
  if !displays
    .iter()
    .any(|display| display.id == resolved_anchor_id)
  {
    return Err(format!(
      "AppKit could not resolve a Region monitor after losing: {anchor_id}"
    ));
  }
  let size = Size { width, height };
  if displays.is_empty() || !size.valid() || width <= 0.0 || height <= 0.0 {
    return Err("AppKit returned no valid desktop displays".to_owned());
  }
  Ok(DesktopBinding {
    displays,
    anchor_id: resolved_anchor_id,
    size,
    layout_changed: layout_changed != 0,
  })
}

pub(super) fn global_committed(binding: &DesktopBinding, local: Option<Rect>) -> Option<Rect> {
  local.and_then(|region| binding.project_local(region))
}
