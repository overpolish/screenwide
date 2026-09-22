// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The counter atlas's layout, as the Metal compositor reaches it. The
//! compositor keeps one atlas per thread with the pixels beside it; the text
//! engine measures and draws, and [`super::atlas`] decides where.

use std::ffi::c_void;

use super::atlas::{AtlasDraw, AtlasRect, CounterAtlas, CounterNumber};

/// One number, the twin of `ScreenwideCounterNumber`.
#[repr(C)]
pub struct NativeCounterNumber {
  text: *const u8,
  length: u32,
  radius: f32,
}

/// One number to rasterise now, the twin of `ScreenwideCounterDraw`.
#[repr(C)]
pub struct NativeCounterDraw {
  index: u32,
  radius: f32,
}

/// The twin of `ScreenwideCounterAtlasLayout`.
#[repr(C)]
pub struct NativeCounterAtlasLayout {
  width: u32,
  height: u32,
  fresh: u32,
  draw_count: u32,
}

/// Measures the cell the number at `index` needs at `radius`, returning zero
/// when it has none.
type Measure = extern "C" fn(
  context: *mut c_void,
  index: u32,
  radius: f32,
  width: *mut u32,
  height: *mut u32,
) -> u32;

#[no_mangle]
pub extern "C" fn screenwide_counter_atlas_create() -> *mut c_void {
  Box::into_raw(Box::<CounterAtlas>::default()).cast()
}

/// # Safety
/// `atlas` must come from [`screenwide_counter_atlas_create`] and not have
/// been destroyed.
#[no_mangle]
pub unsafe extern "C" fn screenwide_counter_atlas_destroy(atlas: *mut c_void) {
  if !atlas.is_null() {
    drop(Box::from_raw(atlas.cast::<CounterAtlas>()));
  }
}

/// Lays out one composition's `count` numbers: a rectangle for each into
/// `rects`, and the ones to rasterise now into `draws`, both `count` long.
///
/// # Safety
/// `atlas` must be live and used from one thread at a time; `numbers`,
/// `rects` and `draws` must each hold `count` elements; `out` must be
/// writable.
#[no_mangle]
pub unsafe extern "C" fn screenwide_counter_atlas_frame(
  atlas: *mut c_void,
  numbers: *const NativeCounterNumber,
  count: u32,
  measure: Measure,
  context: *mut c_void,
  rects: *mut AtlasRect,
  draws: *mut NativeCounterDraw,
  out: *mut NativeCounterAtlasLayout,
) -> u32 {
  if atlas.is_null()
    || out.is_null()
    || (count > 0 && (numbers.is_null() || rects.is_null() || draws.is_null()))
  {
    return 0;
  }
  let atlas = &mut *atlas.cast::<CounterAtlas>();
  let count = count as usize;
  let numbers: Vec<CounterNumber<'_>> = if count == 0 {
    Vec::new()
  } else {
    std::slice::from_raw_parts(numbers, count)
      .iter()
      .map(|number| CounterNumber {
        text: if number.text.is_null() {
          &[]
        } else {
          std::slice::from_raw_parts(number.text, number.length as usize)
        },
        radius: number.radius,
      })
      .collect()
  };
  let rects: &mut [AtlasRect] = if count == 0 {
    &mut []
  } else {
    std::slice::from_raw_parts_mut(rects, count)
  };
  let mut planned: Vec<AtlasDraw> = Vec::new();
  let layout = atlas.frame(
    &numbers,
    |index, radius| {
      let (mut width, mut height) = (0, 0);
      (measure(context, index as u32, radius, &mut width, &mut height) != 0)
        .then_some((width, height))
    },
    rects,
    &mut planned,
  );
  for (slot, draw) in planned.iter().enumerate() {
    draws.add(slot).write(NativeCounterDraw {
      index: draw.index as u32,
      radius: draw.radius,
    });
  }
  out.write(NativeCounterAtlasLayout {
    width: layout.size.0,
    height: layout.size.1,
    fresh: u32::from(layout.fresh),
    draw_count: planned.len() as u32,
  });
  1
}
