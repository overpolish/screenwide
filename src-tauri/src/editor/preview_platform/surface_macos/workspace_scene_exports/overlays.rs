// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::c_void;

use super::scene;

/// Writes `bounds`, x, y, width and height, through the out pointers; 1 when
/// there are any, 0 otherwise.
unsafe fn write_bounds(
  bounds: Option<[f64; 4]>,
  x: *mut f64,
  y: *mut f64,
  width: *mut f64,
  height: *mut f64,
) -> i32 {
  let Some([left, top, wide, tall]) = bounds else {
    return 0;
  };
  for (out, value) in [(x, left), (y, top), (width, wide), (height, tall)] {
    if let Some(out) = unsafe { out.as_mut() } {
      *out = value;
    }
  }
  1
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_workspace_scene_keyboard_bounds(
  scene: *const c_void,
  pane_index: u32,
  x: *mut f64,
  y: *mut f64,
  width: *mut f64,
  height: *mut f64,
) -> i32 {
  let bounds = unsafe { self::scene(scene) }.and_then(|scene| scene.keyboard_bounds(pane_index));
  unsafe { write_bounds(bounds, x, y, width, height) }
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_workspace_scene_caption_bounds(
  scene: *const c_void,
  pane_index: u32,
  x: *mut f64,
  y: *mut f64,
  width: *mut f64,
  height: *mut f64,
) -> i32 {
  let bounds = unsafe { self::scene(scene) }.and_then(|scene| scene.caption_bounds(pane_index));
  unsafe { write_bounds(bounds, x, y, width, height) }
}

/// Where the caption being typed into on `pane_index` sets its words: its
/// block as x, y, width and height shares of its canvas, its type size as a
/// share of the block's height, and how its lines line up, a share of the
/// block's width from the left. Zero where no caption is typed into there.
#[no_mangle]
pub unsafe extern "C" fn screenwide_workspace_scene_caption_editing_block(
  scene: *const c_void,
  pane_index: u32,
  block: *mut f64,
  size: *mut f64,
  align: *mut f64,
) -> i32 {
  let found =
    unsafe { self::scene(scene) }.and_then(|scene| scene.caption_editing_block(pane_index));
  let (Some((rect, type_size, share)), false) =
    (found, block.is_null() || size.is_null() || align.is_null())
  else {
    return 0;
  };
  unsafe {
    std::slice::from_raw_parts_mut(block, 4).copy_from_slice(&rect);
    *size = type_size;
    *align = share;
  }
  1
}

/// Which word of the caption showing on `pane_index` is drawn under the
/// point `x`, `y`, shares of its canvas, counted across its lines; -1 over
/// none.
#[no_mangle]
pub unsafe extern "C" fn screenwide_workspace_scene_caption_word_at(
  scene: *const c_void,
  pane_index: u32,
  x: f64,
  y: f64,
) -> i32 {
  unsafe { self::scene(scene) }
    .and_then(|scene| scene.caption_word_at(pane_index, [x, y]))
    .and_then(|word| i32::try_from(word).ok())
    .unwrap_or(-1)
}
