// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! C ABI used by the macOS Metal adapter. The opaque group owns no platform
//! objects, so the same Rust state machine is used directly by Windows D3D.

use std::ffi::c_void;

use crate::osc::geometry::Rect;

use super::{
  control_metrics, Appearance, ControlColor, ControlGroup, ControlIcon, ControlKind,
  ControlMetrics, ControlSize, ControlSpec, ControlStyle, ControlUpdate, ControlVisual,
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NativeControlSpec {
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
  pub kind: u8,
  pub color: u8,
  pub size: u8,
  pub disabled: u8,
  pub icon: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct NativeControlUpdate {
  pub consumed: u8,
  pub changed: u8,
  pub activated: u8,
  pub animating: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct NativeControlMetrics {
  pub height: f64,
  pub radius: f64,
  pub padding_x: f64,
  pub gap: f64,
  pub icon_size: f64,
  pub font_size: f64,
  pub line_height: f64,
}

impl From<ControlMetrics> for NativeControlMetrics {
  fn from(metrics: ControlMetrics) -> Self {
    Self {
      height: metrics.height,
      radius: metrics.radius,
      padding_x: metrics.padding_x,
      gap: metrics.gap,
      icon_size: metrics.icon_size,
      font_size: metrics.font_size,
      line_height: metrics.line_height,
    }
  }
}

fn style(spec: NativeControlSpec) -> Option<ControlStyle> {
  let kind = match spec.kind {
    0 => ControlKind::Button,
    1 => ControlKind::IconButton,
    _ => return None,
  };
  let color = parse_color(spec.color)?;
  let size = match spec.size {
    0 => ControlSize::Compact,
    1 => ControlSize::Default,
    _ => return None,
  };
  let mut style = match kind {
    ControlKind::Button => ControlStyle::button(color, size),
    ControlKind::IconButton => ControlStyle::icon_button(color, size),
  };
  style.disabled = spec.disabled != 0;
  Some(style)
}

pub(super) fn parse_color(value: u8) -> Option<ControlColor> {
  match value {
    0 => Some(ControlColor::Neutral),
    1 => Some(ControlColor::Primary),
    2 => Some(ControlColor::Error),
    _ => None,
  }
}

pub(super) fn parse_icon(value: u8) -> Option<ControlIcon> {
  match value {
    0 => Some(ControlIcon::None),
    1 => Some(ControlIcon::X),
    2 => Some(ControlIcon::Copy),
    3 => Some(ControlIcon::Pilcrow),
    4 => Some(ControlIcon::RotateCcw),
    5 => Some(ControlIcon::Trash2),
    _ => None,
  }
}

fn update(group: &ControlGroup, value: ControlUpdate) -> NativeControlUpdate {
  NativeControlUpdate {
    consumed: u8::from(value.consumed),
    changed: u8::from(value.changed),
    activated: value.activated.min(u8::MAX as usize) as u8,
    animating: u8::from(group.is_animating()),
  }
}

unsafe fn group_mut<'a>(handle: *mut c_void) -> Option<&'a mut ControlGroup> {
  handle.cast::<ControlGroup>().as_mut()
}

#[no_mangle]
pub extern "C" fn screenwide_osc_control_group_create() -> *mut c_void {
  Box::into_raw(Box::new(ControlGroup::default())).cast()
}

#[no_mangle]
pub extern "C" fn screenwide_osc_control_metrics(kind: u8, size: u8) -> NativeControlMetrics {
  let kind = match kind {
    0 => ControlKind::Button,
    1 => ControlKind::IconButton,
    _ => return NativeControlMetrics::default(),
  };
  let size = match size {
    0 => ControlSize::Compact,
    1 => ControlSize::Default,
    _ => return NativeControlMetrics::default(),
  };
  control_metrics(kind, size).into()
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_osc_control_group_destroy(handle: *mut c_void) {
  if !handle.is_null() {
    drop(Box::from_raw(handle.cast::<ControlGroup>()));
  }
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_osc_control_group_layout(
  handle: *mut c_void,
  specs: *const NativeControlSpec,
  count: usize,
) {
  let Some(group) = group_mut(handle) else {
    return;
  };
  let native = if count == 0 {
    &[]
  } else if specs.is_null() {
    return;
  } else {
    std::slice::from_raw_parts(specs, count)
  };
  let specs = native
    .iter()
    .copied()
    .filter_map(|spec| {
      Some(ControlSpec {
        rect: Rect::from_xywh(spec.x, spec.y, spec.width, spec.height),
        style: style(spec)?,
        icon: parse_icon(spec.icon)?,
      })
    })
    .collect::<Vec<_>>();
  group.layout(&specs);
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_osc_control_group_hover(
  handle: *mut c_void,
  x: f64,
  y: f64,
) -> NativeControlUpdate {
  let Some(group) = group_mut(handle) else {
    return NativeControlUpdate::default();
  };
  let value = group.move_to((x, y));
  update(group, value)
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_osc_control_group_hit(
  handle: *mut c_void,
  x: f64,
  y: f64,
) -> u8 {
  group_mut(handle).map_or(0, |group| {
    group.hit_index((x, y)).min(u8::MAX as usize) as u8
  })
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_osc_control_group_down(
  handle: *mut c_void,
  x: f64,
  y: f64,
) -> NativeControlUpdate {
  let Some(group) = group_mut(handle) else {
    return NativeControlUpdate::default();
  };
  let value = group.down((x, y));
  update(group, value)
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_osc_control_group_up(
  handle: *mut c_void,
  x: f64,
  y: f64,
) -> NativeControlUpdate {
  let Some(group) = group_mut(handle) else {
    return NativeControlUpdate::default();
  };
  let value = group.up((x, y));
  update(group, value)
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_osc_control_group_clear_hover(
  handle: *mut c_void,
) -> NativeControlUpdate {
  let Some(group) = group_mut(handle) else {
    return NativeControlUpdate::default();
  };
  let value = group.clear_hover();
  update(group, value)
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_osc_control_group_visuals(
  handle: *mut c_void,
  dark: u8,
  output: *mut ControlVisual,
  capacity: usize,
) -> usize {
  let Some(group) = group_mut(handle) else {
    return 0;
  };
  let visuals = group.visuals(if dark != 0 {
    Appearance::Dark
  } else {
    Appearance::Light
  });
  let count = visuals.len().min(capacity);
  if count > 0 && !output.is_null() {
    std::ptr::copy_nonoverlapping(visuals.as_ptr(), output, count);
  }
  count
}

#[no_mangle]
pub unsafe extern "C" fn screenwide_osc_control_group_is_animating(handle: *mut c_void) -> u8 {
  group_mut(handle).map_or(0, |group| u8::from(group.is_animating()))
}
