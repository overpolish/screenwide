// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
  ffi::c_void,
  time::{Duration, Instant},
};

use super::{
  ffi::{parse_color, parse_icon},
  Appearance, ConfirmAction, ConfirmActionSpec, ConfirmLayer, ConfirmUpdate,
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NativeConfirmSpec {
  pub idle_icon: u8,
  pub armed_icon: u8,
  pub idle_color: u8,
  pub armed_color: u8,
  pub timeout_ms: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct NativeConfirmUpdate {
  pub confirmed: u8,
  pub changed: u8,
  pub animating: u8,
  pub armed: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct NativeConfirmLayer {
  pub icon: u8,
  pub padding: [u8; 3],
  pub foreground: [f32; 4],
  pub opacity: f32,
  pub scale: f32,
}

impl From<ConfirmUpdate> for NativeConfirmUpdate {
  fn from(update: ConfirmUpdate) -> Self {
    Self {
      confirmed: u8::from(update.confirmed),
      changed: u8::from(update.changed),
      animating: u8::from(update.animating),
      armed: u8::from(update.armed),
    }
  }
}

impl From<ConfirmLayer> for NativeConfirmLayer {
  fn from(layer: ConfirmLayer) -> Self {
    Self {
      icon: layer.icon as u8,
      foreground: layer.foreground,
      opacity: layer.opacity,
      scale: layer.scale,
      ..Default::default()
    }
  }
}

unsafe fn action_mut<'a>(handle: *mut c_void) -> Option<&'a mut ConfirmAction> {
  handle.cast::<ConfirmAction>().as_mut()
}

#[no_mangle]
pub extern "C" fn screenwide_osc_confirm_create(spec: NativeConfirmSpec) -> *mut c_void {
  let Some(idle_icon) = parse_icon(spec.idle_icon) else {
    return std::ptr::null_mut();
  };
  let Some(armed_icon) = parse_icon(spec.armed_icon) else {
    return std::ptr::null_mut();
  };
  let Some(idle_color) = parse_color(spec.idle_color) else {
    return std::ptr::null_mut();
  };
  let Some(armed_color) = parse_color(spec.armed_color) else {
    return std::ptr::null_mut();
  };
  Box::into_raw(Box::new(ConfirmAction::new(ConfirmActionSpec {
    idle_icon,
    armed_icon,
    idle_color,
    armed_color,
    timeout: Duration::from_millis(u64::from(spec.timeout_ms)),
  })))
  .cast()
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_osc_confirm_destroy(handle: *mut c_void) {
  if !handle.is_null() {
    drop(Box::from_raw(handle.cast::<ConfirmAction>()));
  }
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_osc_confirm_press(handle: *mut c_void) -> NativeConfirmUpdate {
  action_mut(handle)
    .map(|action| action.press(Instant::now()).into())
    .unwrap_or_default()
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_osc_confirm_expire(handle: *mut c_void) -> NativeConfirmUpdate {
  action_mut(handle)
    .map(|action| action.expire(Instant::now()).into())
    .unwrap_or_default()
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_osc_confirm_layers(
  handle: *mut c_void,
  dark: u8,
  output: *mut NativeConfirmLayer,
  capacity: usize,
) -> usize {
  let Some(action) = action_mut(handle) else {
    return 0;
  };
  let layers = action.layers(
    Instant::now(),
    if dark != 0 {
      Appearance::Dark
    } else {
      Appearance::Light
    },
  );
  let count = layers.len().min(capacity);
  if count > 0 && !output.is_null() {
    for (destination, layer) in std::slice::from_raw_parts_mut(output, count)
      .iter_mut()
      .zip(layers)
    {
      *destination = layer.into();
    }
  }
  count
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_osc_confirm_is_animating(handle: *mut c_void) -> u8 {
  action_mut(handle).map_or(0, |action| u8::from(action.is_animating(Instant::now())))
}
