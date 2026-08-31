// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{ffi::c_void, panic::catch_unwind};

use crate::osc::geometry::{Monitor, Rect, Size};
use tauri::{Emitter, Manager};

use super::{ffi, Context, DesktopBinding, NativeOscResult, Point, Purpose};

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NativeRulerMeasurement {
  pub id: u64,
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
  pub flags: u8,
  pub padding: [u8; 7],
  pub label_anchor_x: f64,
  pub label_anchor_y: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NativeRulerViewport {
  pub display_id: u32,
  pub padding: u32,
  pub zoom: f64,
  pub origin_x: f64,
  pub origin_y: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NativeRulerProbe {
  pub id: u64,
  pub display_id: u32,
  pub axis: u8,
  pub flags: u8,
  pub padding: [u8; 2],
  pub start: f64,
  pub end: f64,
  pub position: f64,
  pub label_anchor_x: f64,
  pub label_anchor_y: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NativeRulerGuide {
  pub id: u64,
  pub display_id: u32,
  pub axis: u8,
  pub flags: u8,
  pub padding: [u8; 2],
  pub position: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NativeRulerGuideGap {
  pub id: u64,
  pub owner_id: u64,
  pub display_id: u32,
  pub axis: u8,
  pub flags: u8,
  pub padding: [u8; 2],
  pub start: f64,
  pub end: f64,
  pub position: f64,
  pub label_anchor_x: f64,
  pub label_anchor_y: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NativeRulerRadius {
  pub id: u64,
  pub display_id: u32,
  pub corner: u8,
  pub flags: u8,
  pub padding: [u8; 2],
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
  pub radius: f64,
  pub label_anchor_x: f64,
  pub label_anchor_y: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NativeRulerCenterline {
  pub id: u64,
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
  pub flags: u8,
  pub padding: [u8; 7],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NativeRulerInnerObject {
  pub owner_id: u64,
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
  pub flags: u8,
  pub padding: [u8; 7],
}

const _: () = assert!(std::mem::size_of::<NativeRulerMeasurement>() == 64);
const _: () = assert!(std::mem::offset_of!(NativeRulerMeasurement, flags) == 40);
const _: () = assert!(std::mem::offset_of!(NativeRulerMeasurement, label_anchor_x) == 48);
const _: () = assert!(std::mem::size_of::<NativeRulerViewport>() == 32);
const _: () = assert!(std::mem::offset_of!(NativeRulerViewport, zoom) == 8);
const _: () = assert!(std::mem::size_of::<NativeRulerProbe>() == 56);
const _: () = assert!(std::mem::offset_of!(NativeRulerProbe, start) == 16);
const _: () = assert!(std::mem::offset_of!(NativeRulerProbe, label_anchor_x) == 40);
const _: () = assert!(std::mem::size_of::<NativeRulerGuide>() == 24);
const _: () = assert!(std::mem::offset_of!(NativeRulerGuide, position) == 16);
const _: () = assert!(std::mem::size_of::<NativeRulerGuideGap>() == 64);
const _: () = assert!(std::mem::offset_of!(NativeRulerGuideGap, start) == 24);
const _: () = assert!(std::mem::offset_of!(NativeRulerGuideGap, label_anchor_x) == 48);
const _: () = assert!(std::mem::size_of::<NativeRulerRadius>() == 72);
const _: () = assert!(std::mem::offset_of!(NativeRulerRadius, x) == 16);
const _: () = assert!(std::mem::offset_of!(NativeRulerRadius, label_anchor_x) == 56);
const _: () = assert!(std::mem::size_of::<NativeRulerCenterline>() == 48);
const _: () = assert!(std::mem::offset_of!(NativeRulerCenterline, flags) == 40);
const _: () = assert!(std::mem::size_of::<NativeRulerInnerObject>() == 48);
const _: () = assert!(std::mem::offset_of!(NativeRulerInnerObject, flags) == 40);

pub fn invalid_result() -> NativeOscResult {
  NativeOscResult {
    status: super::ResultStatus::Invalid as u8,
    ..Default::default()
  }
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_input(
  context: *mut c_void,
  phase: u32,
  x: f64,
  y: f64,
  modifiers: u8,
  out: *mut NativeOscResult,
) {
  if out.is_null() {
    return;
  }
  let mut result = catch_unwind(|| {
    if context.is_null() {
      invalid_result()
    } else {
      (&*context.cast::<Context>()).input(phase, Point { x, y }, modifiers)
    }
  })
  .unwrap_or_else(|_| invalid_result());
  super::ocr::dismiss_on_idle_cancel(context, phase, &mut result);
  *out = result;
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_measurements(
  context: *mut c_void,
  output: *mut NativeRulerMeasurement,
  capacity: usize,
) -> usize {
  catch_unwind(|| {
    if context.is_null() {
      return 0;
    }
    let context = &*context.cast::<Context>();
    if context.purpose != Purpose::Ruler {
      return 0;
    }
    let state = context
      .window
      .app_handle()
      .state::<crate::ruler::RulerState>();
    let measurements = state.measurements();
    if output.is_null() || capacity == 0 {
      return measurements.len();
    }
    for (index, measurement) in measurements.iter().take(capacity).enumerate() {
      output.add(index).write(NativeRulerMeasurement {
        id: measurement.id,
        x: measurement.bounds.origin.x,
        y: measurement.bounds.origin.y,
        width: measurement.bounds.size.width,
        height: measurement.bounds.size.height,
        flags: u8::from(measurement.draft)
          | u8::from(measurement.animating) << 1
          | u8::from(measurement.hovered) << 2
          | u8::from(measurement.label_hidden) << 3,
        padding: [0; 7],
        label_anchor_x: measurement.label_anchor.map_or(f64::NAN, |point| point.x),
        label_anchor_y: measurement.label_anchor.map_or(f64::NAN, |point| point.y),
      });
    }
    measurements.len()
  })
  .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_viewports(
  context: *mut c_void,
  output: *mut NativeRulerViewport,
  capacity: usize,
) -> usize {
  catch_unwind(|| {
    if context.is_null() {
      return 0;
    }
    let context = &*context.cast::<Context>();
    if context.purpose != Purpose::Ruler {
      return 0;
    }
    let state = context
      .window
      .app_handle()
      .state::<crate::ruler::RulerState>();
    let viewports = state.viewports();
    if output.is_null() || capacity == 0 {
      return viewports.len();
    }
    for (index, visual) in viewports.iter().take(capacity).enumerate() {
      output.add(index).write(NativeRulerViewport {
        display_id: visual.display_id,
        padding: 0,
        zoom: visual.viewport.zoom,
        origin_x: visual.viewport.origin.x,
        origin_y: visual.viewport.origin.y,
      });
    }
    viewports.len()
  })
  .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_probes(
  context: *mut c_void,
  output: *mut NativeRulerProbe,
  capacity: usize,
) -> usize {
  catch_unwind(|| {
    if context.is_null() {
      return 0;
    }
    let context = &*context.cast::<Context>();
    if context.purpose != Purpose::Ruler {
      return 0;
    }
    let state = context
      .window
      .app_handle()
      .state::<crate::ruler::RulerState>();
    let probes = state.probes();
    if output.is_null() || capacity == 0 {
      return probes.len();
    }
    for (index, probe) in probes.iter().take(capacity).enumerate() {
      output.add(index).write(NativeRulerProbe {
        id: probe.id,
        display_id: probe.display_id,
        axis: match probe.axis {
          crate::ruler::probe::ProbeAxis::Horizontal => 1,
          crate::ruler::probe::ProbeAxis::Vertical => 2,
        },
        flags: u8::from(probe.draft)
          | u8::from(probe.hovered) << 1
          | u8::from(probe.id == 0 && !probe.draft) << 2
          | u8::from(probe.label_hidden) << 3,
        padding: [0; 2],
        start: probe.start,
        end: probe.end,
        position: probe.position,
        label_anchor_x: probe.label_anchor.map_or(f64::NAN, |point| point.x),
        label_anchor_y: probe.label_anchor.map_or(f64::NAN, |point| point.y),
      });
    }
    probes.len()
  })
  .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_guides(
  context: *mut c_void,
  output: *mut NativeRulerGuide,
  capacity: usize,
) -> usize {
  catch_unwind(|| {
    if context.is_null() {
      return 0;
    }
    let context = &*context.cast::<Context>();
    if context.purpose != Purpose::Ruler {
      return 0;
    }
    let state = context
      .window
      .app_handle()
      .state::<crate::ruler::RulerState>();
    let guides = state.guides();
    if output.is_null() || capacity == 0 {
      return guides.len();
    }
    for (index, guide) in guides.iter().take(capacity).enumerate() {
      output.add(index).write(NativeRulerGuide {
        id: guide.id,
        display_id: guide.display_id,
        axis: match guide.axis {
          crate::ruler::snapshot::GuideAxis::Vertical => 1,
          crate::ruler::snapshot::GuideAxis::Horizontal => 2,
        },
        flags: u8::from(guide.draft) | u8::from(guide.hovered) << 1,
        padding: [0; 2],
        position: guide.position,
      });
    }
    guides.len()
  })
  .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_guide_gaps(
  context: *mut c_void,
  output: *mut NativeRulerGuideGap,
  capacity: usize,
) -> usize {
  catch_unwind(|| {
    if context.is_null() {
      return 0;
    }
    let context = &*context.cast::<Context>();
    if context.purpose != Purpose::Ruler {
      return 0;
    }
    let state = context
      .window
      .app_handle()
      .state::<crate::ruler::RulerState>();
    let gaps = state.guide_gaps();
    if output.is_null() || capacity == 0 {
      return gaps.len();
    }
    for (index, gap) in gaps.iter().take(capacity).enumerate() {
      output.add(index).write(NativeRulerGuideGap {
        id: gap.id,
        owner_id: gap.owner_id,
        display_id: gap.display_id,
        axis: match gap.axis {
          crate::ruler::probe::ProbeAxis::Horizontal => 1,
          crate::ruler::probe::ProbeAxis::Vertical => 2,
        },
        flags: u8::from(gap.hovered) | u8::from(gap.label_hidden) << 1,
        padding: [0; 2],
        start: gap.start,
        end: gap.end,
        position: gap.position,
        label_anchor_x: gap.label_anchor.map_or(f64::NAN, |point| point.x),
        label_anchor_y: gap.label_anchor.map_or(f64::NAN, |point| point.y),
      });
    }
    gaps.len()
  })
  .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_radii(
  context: *mut c_void,
  output: *mut NativeRulerRadius,
  capacity: usize,
) -> usize {
  catch_unwind(|| {
    if context.is_null() {
      return 0;
    }
    let context = &*context.cast::<Context>();
    if context.purpose != Purpose::Ruler {
      return 0;
    }
    let state = context
      .window
      .app_handle()
      .state::<crate::ruler::RulerState>();
    let radii = state.radii();
    if output.is_null() || capacity == 0 {
      return radii.len();
    }
    for (index, radius) in radii.iter().take(capacity).enumerate() {
      output.add(index).write(NativeRulerRadius {
        id: radius.id,
        display_id: radius.display_id,
        corner: radius.corner as u8,
        flags: u8::from(radius.low_confidence)
          | u8::from(radius.draft) << 1
          | u8::from(radius.hovered) << 2
          | u8::from(radius.label_hidden) << 3,
        padding: [0; 2],
        x: radius.bounds.origin.x,
        y: radius.bounds.origin.y,
        width: radius.bounds.size.width,
        height: radius.bounds.size.height,
        radius: radius.radius,
        label_anchor_x: radius.label_anchor.map_or(f64::NAN, |point| point.x),
        label_anchor_y: radius.label_anchor.map_or(f64::NAN, |point| point.y),
      });
    }
    radii.len()
  })
  .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_centerlines(
  context: *mut c_void,
  output: *mut NativeRulerCenterline,
  capacity: usize,
) -> usize {
  catch_unwind(|| {
    if context.is_null() {
      return 0;
    }
    let context = &*context.cast::<Context>();
    if context.purpose != Purpose::Ruler {
      return 0;
    }
    let state = context
      .window
      .app_handle()
      .state::<crate::ruler::RulerState>();
    let (centerlines, _) = state.center_aids();
    if output.is_null() || capacity == 0 {
      return centerlines.len();
    }
    for (index, line) in centerlines.iter().take(capacity).enumerate() {
      output.add(index).write(NativeRulerCenterline {
        id: line.id,
        x: line.bounds.origin.x,
        y: line.bounds.origin.y,
        width: line.bounds.size.width,
        height: line.bounds.size.height,
        flags: u8::from(line.x_accent) | u8::from(line.y_accent) << 1,
        padding: [0; 7],
      });
    }
    centerlines.len()
  })
  .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_inner_objects(
  context: *mut c_void,
  output: *mut NativeRulerInnerObject,
  capacity: usize,
) -> usize {
  catch_unwind(|| {
    if context.is_null() {
      return 0;
    }
    let context = &*context.cast::<Context>();
    if context.purpose != Purpose::Ruler {
      return 0;
    }
    let state = context
      .window
      .app_handle()
      .state::<crate::ruler::RulerState>();
    let (_, objects) = state.center_aids();
    if output.is_null() || capacity == 0 {
      return objects.len();
    }
    for (index, object) in objects.iter().take(capacity).enumerate() {
      output.add(index).write(NativeRulerInnerObject {
        owner_id: object.owner_id,
        x: object.bounds.origin.x,
        y: object.bounds.origin.y,
        width: object.bounds.size.width,
        height: object.bounds.size.height,
        flags: u8::from(object.aligned_x) | u8::from(object.aligned_y) << 1,
        padding: [0; 7],
      });
    }
    objects.len()
  })
  .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_viewport_input(
  context: *mut c_void,
  display_id: u32,
  operation: u32,
  anchor_x: f64,
  anchor_y: f64,
  delta_x: f64,
  delta_y: f64,
  out: *mut NativeOscResult,
) -> i32 {
  if out.is_null() {
    return 0;
  }
  let result = catch_unwind(|| {
    if context.is_null() {
      invalid_result()
    } else {
      (&*context.cast::<Context>()).ruler_viewport_input(
        display_id,
        operation,
        Point {
          x: anchor_x,
          y: anchor_y,
        },
        Point {
          x: delta_x,
          y: delta_y,
        },
      )
    }
  })
  .unwrap_or_else(|_| invalid_result());
  let handled = result.status != super::ResultStatus::Invalid as u8;
  *out = result;
  i32::from(handled)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_label_input(
  context: *mut c_void,
  operation: u32,
  kind: u8,
  id: u64,
  pointer_x: f64,
  pointer_y: f64,
  label_center_x: f64,
  label_center_y: f64,
  out: *mut NativeOscResult,
) {
  if out.is_null() {
    return;
  }
  *out = catch_unwind(|| {
    if context.is_null() {
      invalid_result()
    } else {
      (&*context.cast::<Context>()).ruler_label_input(
        operation,
        kind,
        id,
        Point {
          x: pointer_x,
          y: pointer_y,
        },
        Point {
          x: label_center_x,
          y: label_center_y,
        },
      )
    }
  })
  .unwrap_or_else(|_| invalid_result());
}

pub unsafe extern "C" fn native_osc_layout_changed(context: *mut c_void) {
  let _ = catch_unwind(|| {
    if context.is_null() {
      return;
    }
    let context = &*context.cast::<Context>();
    if context.purpose == super::Purpose::TextRecognition {
      crate::text_recognition::restart_after_topology_change(context.window.app_handle());
      return;
    }
    if context.purpose == super::Purpose::Ruler {
      crate::ruler::restart_after_topology_change(context.window.app_handle());
      return;
    }
    let _ = context.window.emit_to(
      tauri::EventTarget::webview_window(context.window.label()),
      super::NATIVE_OSC_LAYOUT_EVENT,
      (),
    );
  });
}

fn attach(
  view: *mut c_void,
  window: super::WebviewWindow,
  width: f64,
  height: f64,
  purpose: super::Purpose,
) -> bool {
  let context = Box::into_raw(Context::new(window, width, height, purpose)).cast();
  !ffi::attach(view, context).is_null()
}

pub fn ensure_attached(
  view: *mut c_void,
  window: super::WebviewWindow,
  width: f64,
  height: f64,
) -> bool {
  with_context(view, |_| ()).is_some()
    || attach(view, window, width, height, super::Purpose::Region)
}

pub fn ensure_text_recognition_attached(
  view: *mut c_void,
  window: super::WebviewWindow,
  width: f64,
  height: f64,
) -> bool {
  with_context(view, |_| ()).is_some()
    || attach(view, window, width, height, super::Purpose::TextRecognition)
}

pub fn ensure_ruler_attached(
  view: *mut c_void,
  window: super::WebviewWindow,
  width: f64,
  height: f64,
) -> bool {
  with_context(view, |_| ()).is_some() || attach(view, window, width, height, super::Purpose::Ruler)
}

pub(super) fn with_context<T>(view: *mut c_void, work: impl FnOnce(&Context) -> T) -> Option<T> {
  let ptr = unsafe { ffi::screenwide_region_osc_context(view) };
  (!ptr.is_null()).then(|| work(unsafe { &*ptr.cast::<Context>() }))
}

pub fn set_committed(view: *mut c_void, rect: Option<Rect>) -> bool {
  with_context(view, |context| {
    context
      .controller
      .lock()
      .map(|mut controller| controller.set_committed(rect))
      .unwrap_or(false)
  })
  .unwrap_or(false)
}

/// Clears the borrowed OSC before its window can be presented for a quick
/// screenshot. The recording region remains in frontend storage and will be
/// synchronized back when the normal Region editor resumes.
pub fn clear_region(view: *mut c_void) -> bool {
  if with_context(view, |context| {
    if let Ok(mut controller) = context.controller.lock() {
      let _ = controller.set_committed(None);
    }
  })
  .is_none()
  {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set(view, 0.0, 0.0, 0.0, 0.0, 0) != 0 }
}

pub fn present_region(view: *mut c_void, rect: Option<Rect>) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  let rect = rect.unwrap_or_default();
  unsafe {
    ffi::screenwide_region_osc_set(
      view,
      rect.origin.x,
      rect.origin.y,
      rect.size.width,
      rect.size.height,
      1,
    ) != 0
  }
}

pub fn configure_desktop(view: *mut c_void, binding: DesktopBinding, local: Option<Rect>) -> bool {
  with_context(view, |context| {
    if binding.anchor().is_none() {
      return false;
    }
    let committed = super::desktop::global_committed(&binding, local);
    let controller = super::RegionController::new(binding.virtual_monitor(), committed, None);
    let Ok(mut current_controller) = context.controller.lock() else {
      return false;
    };
    let Ok(mut desktop) = context.desktop.lock() else {
      return false;
    };
    *current_controller = controller;
    *desktop = Some(binding);
    true
  })
  .unwrap_or(false)
}

pub fn set_monitor(view: *mut c_void, width: f64, height: f64) -> bool {
  with_context(view, |context| {
    context
      .controller
      .lock()
      .map(|mut controller| {
        controller.set_monitor(Monitor {
          size: Size { width, height },
        })
      })
      .unwrap_or(false)
  })
  .unwrap_or(false)
}

pub fn set_allow_drawing(view: *mut c_void, allow_drawing: bool) -> bool {
  with_context(view, |context| {
    context
      .allow_drawing
      .store(allow_drawing, std::sync::atomic::Ordering::Relaxed);
  })
  .is_some()
}

pub fn set_magnifier_source(view: *mut c_void, rgba: &[u8], width: u32, height: u32) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe {
    ffi::screenwide_region_osc_set_magnifier_source(view, rgba.as_ptr(), rgba.len(), width, height)
      != 0
  }
}

pub fn set_aspect(view: *mut c_void, aspect: Option<f64>) -> bool {
  with_context(view, |context| {
    context
      .controller
      .lock()
      .map(|mut controller| {
        controller.set_aspect(aspect);
        true
      })
      .unwrap_or(false)
  })
  .unwrap_or(false)
}

pub fn set_input_enabled(view: *mut c_void, enabled: bool) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set_input_enabled(view, i32::from(enabled)) };
  true
}

pub fn set_show_handles(view: *mut c_void, show_handles: bool) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set_show_handles(view, i32::from(show_handles)) };
  true
}

pub fn set_show_frame(view: *mut c_void, show_frame: bool) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set_show_frame(view, i32::from(show_frame)) };
  true
}

pub fn set_exclusion_rect(view: *mut c_void, rect: Option<Rect>) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  let rect = rect.unwrap_or_default();
  unsafe {
    ffi::screenwide_region_osc_set_exclusion_rect(
      view,
      rect.origin.x,
      rect.origin.y,
      rect.size.width,
      rect.size.height,
    );
  }
  true
}

pub fn set_desktop_presented(view: *mut c_void, presented: bool) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set_desktop_presented(view, i32::from(presented)) };
  true
}

pub fn claim_pointer_surface(view: *mut c_void) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_claim_pointer_surface(view) };
  true
}

pub fn refresh_ruler_pointer(view: *mut c_void) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_ruler_refresh_pointer(view) };
  true
}

pub fn set_ruler_transient_chrome(view: *mut c_void, visible: bool) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_ruler_set_transient_chrome(view, i32::from(visible)) };
  true
}

pub fn set_snapshot(
  view: *mut c_void,
  display_id: u32,
  rgba: &[u8],
  width: u32,
  height: u32,
) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe {
    ffi::screenwide_region_osc_set_snapshot(
      view,
      display_id,
      rgba.as_ptr(),
      rgba.len(),
      width,
      height,
    ) != 0
  }
}

pub fn set_snapshot_presented(view: *mut c_void, presented: bool) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set_snapshot_presented(view, i32::from(presented)) };
  true
}

pub fn set_snapshot_composited(view: *mut c_void, composited: bool) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set_snapshot_composited(view, i32::from(composited)) };
  true
}
