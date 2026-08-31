// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{ffi::c_void, panic::catch_unwind};

use crate::osc::{
  controller::RegionController,
  geometry::{Monitor, Point, Rect, Size},
  scene::{RegionScene, RegionSceneOwner},
  style::overlay_palette,
};
use crate::ruler::render::{
  CenterlinePacket, GuideGapPacket, GuidePacket, InnerObjectPacket, MeasurementPacket, ProbePacket,
  RadiusPacket, ViewportPacket,
};
use tauri::{Emitter, Manager, WebviewWindow};

use super::{ffi, Context, DesktopBinding, NativeOscResult, Purpose};

pub fn invalid_result() -> NativeOscResult {
  crate::osc::runtime::invalid_result()
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
  let result = catch_unwind(|| {
    if context.is_null() {
      invalid_result()
    } else {
      (&*context.cast::<Context>()).input(phase, Point { x, y }, modifiers)
    }
  })
  .unwrap_or_else(|_| invalid_result());
  *out = result;
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_measurements(
  context: *mut c_void,
  output: *mut MeasurementPacket,
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
      output.add(index).write(measurement.into());
    }
    measurements.len()
  })
  .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_viewports(
  context: *mut c_void,
  output: *mut ViewportPacket,
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
      output.add(index).write(visual.into());
    }
    viewports.len()
  })
  .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_probes(
  context: *mut c_void,
  output: *mut ProbePacket,
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
      output.add(index).write(probe.into());
    }
    probes.len()
  })
  .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_guides(
  context: *mut c_void,
  output: *mut GuidePacket,
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
      output.add(index).write(guide.into());
    }
    guides.len()
  })
  .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_guide_gaps(
  context: *mut c_void,
  output: *mut GuideGapPacket,
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
      output.add(index).write(gap.into());
    }
    gaps.len()
  })
  .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_radii(
  context: *mut c_void,
  output: *mut RadiusPacket,
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
      output.add(index).write(radius.into());
    }
    radii.len()
  })
  .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_centerlines(
  context: *mut c_void,
  output: *mut CenterlinePacket,
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
      output.add(index).write(line.into());
    }
    centerlines.len()
  })
  .unwrap_or(0)
}

#[no_mangle]
pub unsafe extern "C" fn native_osc_ruler_inner_objects(
  context: *mut c_void,
  output: *mut InnerObjectPacket,
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
      output.add(index).write(object.into());
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
  window: WebviewWindow,
  width: f64,
  height: f64,
  purpose: super::Purpose,
) -> bool {
  let context = Box::into_raw(Context::new(window, width, height, purpose)).cast();
  !ffi::attach(view, context).is_null()
}

pub fn ensure_attached(view: *mut c_void, window: WebviewWindow, width: f64, height: f64) -> bool {
  with_context(view, |_| ()).is_some()
    || attach(view, window, width, height, super::Purpose::Region)
}

pub fn ensure_text_recognition_attached(
  view: *mut c_void,
  window: WebviewWindow,
  width: f64,
  height: f64,
) -> bool {
  with_context(view, |_| ()).is_some()
    || attach(view, window, width, height, super::Purpose::TextRecognition)
}

pub fn ensure_ruler_attached(
  view: *mut c_void,
  window: WebviewWindow,
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
    if let Ok(mut scene) = context.scene.lock() {
      scene.region = Rect::default();
      scene.visible = false;
    }
  })
  .is_none()
  {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set(view, 0.0, 0.0, 0.0, 0.0, 0) != 0 }
}

pub fn present_region(view: *mut c_void, rect: Option<Rect>) -> bool {
  let rect = rect.unwrap_or_default();
  if with_context(view, |context| {
    if let Ok(mut scene) = context.scene.lock() {
      scene.region = rect;
      scene.visible = true;
    }
  })
  .is_none()
  {
    return false;
  }
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

pub fn region_scene(view: *mut c_void) -> Option<RegionScene> {
  with_context(view, |context| {
    context.scene.lock().ok().map(|scene| scene.presented())
  })
  .flatten()
}

pub fn region_scene_request_base(
  view: *mut c_void,
  owner: RegionSceneOwner,
) -> Option<RegionScene> {
  with_context(view, |context| {
    context
      .scene
      .lock()
      .ok()
      .map(|scene| scene.request_base(owner))
  })
  .flatten()
}

pub fn reconcile_region_scene_request(
  view: *mut c_void,
  requested: RegionScene,
  owner: RegionSceneOwner,
) -> Option<RegionScene> {
  with_context(view, |context| {
    context
      .scene
      .lock()
      .ok()
      .and_then(|mut state| state.reconcile_request(requested, owner))
  })
  .flatten()
}

pub fn restore_normal_region_scene(view: *mut c_void) -> bool {
  let Some(scene) = with_context(view, |context| {
    context
      .scene
      .lock()
      .ok()
      .map(|state| state.normal_presentation())
  })
  .flatten() else {
    return false;
  };
  apply_region_scene(view, scene)
}

/// Applies the portable Region scene to the macOS compositor. The adapter
/// diffs lifecycle-owned fields so a workflow refresh cannot needlessly
/// re-present desktop or snapshot surfaces.
pub fn apply_region_scene(view: *mut c_void, next: RegionScene) -> bool {
  if next.overlay != overlay_palette() {
    return false;
  }
  let Some(previous) = with_context(view, |context| {
    let mut scene = context.scene.lock().ok()?;
    let previous = scene.presented();
    scene.set_presented(next);
    context.allow_drawing.store(
      next.interaction.allow_drawing,
      std::sync::atomic::Ordering::Relaxed,
    );
    let mut controller = context.controller.lock().ok()?;
    controller.set_aspect(next.interaction.aspect);
    Some(previous)
  })
  .flatten() else {
    return false;
  };

  unsafe {
    if previous.chrome.frame_visible != next.chrome.frame_visible {
      ffi::screenwide_region_osc_set_show_frame(view, i32::from(next.chrome.frame_visible));
    }
    if previous.chrome.handles_visible != next.chrome.handles_visible {
      ffi::screenwide_region_osc_set_show_handles(view, i32::from(next.chrome.handles_visible));
    }
    if previous.interaction.input_enabled != next.interaction.input_enabled {
      ffi::screenwide_region_osc_set_input_enabled(view, i32::from(next.interaction.input_enabled));
    }
    if previous.interaction.exclusion_rect != next.interaction.exclusion_rect {
      let rect = next.interaction.exclusion_rect.unwrap_or_default();
      ffi::screenwide_region_osc_set_exclusion_rect(
        view,
        rect.origin.x,
        rect.origin.y,
        rect.size.width,
        rect.size.height,
      );
    }
    if previous.snapshot.presented != next.snapshot.presented {
      ffi::screenwide_region_osc_set_snapshot_presented(view, i32::from(next.snapshot.presented));
    }
    if previous.snapshot.composited != next.snapshot.composited {
      ffi::screenwide_region_osc_set_snapshot_composited(view, i32::from(next.snapshot.composited));
    }
    // Geometry is submitted before desktop peers are presented so a newly
    // shown surface can never expose the previous tool's cutout for a frame.
    let presented = ffi::screenwide_region_osc_set(
      view,
      next.region.origin.x,
      next.region.origin.y,
      next.region.size.width,
      next.region.size.height,
      i32::from(next.visible),
    ) != 0;
    if !presented {
      return false;
    }
    if previous.desktop_presented != next.desktop_presented {
      ffi::screenwide_region_osc_set_desktop_presented(view, i32::from(next.desktop_presented));
    }
    true
  }
}

pub fn configure_desktop(view: *mut c_void, binding: DesktopBinding, local: Option<Rect>) -> bool {
  with_context(view, |context| {
    if binding.anchor().is_none() {
      return false;
    }
    let committed = super::desktop::global_committed(&binding, local);
    let controller = RegionController::new(binding.virtual_monitor(), committed, None);
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
    if let Ok(mut scene) = context.scene.lock() {
      scene.interaction.allow_drawing = allow_drawing;
    }
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
    if let Ok(mut scene) = context.scene.lock() {
      scene.interaction.aspect = aspect;
    }
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
  if with_context(view, |context| {
    if let Ok(mut scene) = context.scene.lock() {
      scene.interaction.input_enabled = enabled;
    }
  })
  .is_none()
  {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set_input_enabled(view, i32::from(enabled)) };
  true
}

pub fn set_show_handles(view: *mut c_void, show_handles: bool) -> bool {
  if with_context(view, |context| {
    if let Ok(mut scene) = context.scene.lock() {
      scene.chrome.handles_visible = show_handles;
    }
  })
  .is_none()
  {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set_show_handles(view, i32::from(show_handles)) };
  true
}

pub fn set_show_frame(view: *mut c_void, show_frame: bool) -> bool {
  if with_context(view, |context| {
    if let Ok(mut scene) = context.scene.lock() {
      scene.chrome.frame_visible = show_frame;
    }
  })
  .is_none()
  {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set_show_frame(view, i32::from(show_frame)) };
  true
}

pub fn set_desktop_presented(view: *mut c_void, presented: bool) -> bool {
  if with_context(view, |context| {
    if let Ok(mut scene) = context.scene.lock() {
      scene.desktop_presented = presented;
    }
  })
  .is_none()
  {
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
  if with_context(view, |context| {
    if let Ok(mut scene) = context.scene.lock() {
      scene.snapshot.presented = presented;
    }
  })
  .is_none()
  {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set_snapshot_presented(view, i32::from(presented)) };
  true
}

pub fn set_snapshot_composited(view: *mut c_void, composited: bool) -> bool {
  if with_context(view, |context| {
    if let Ok(mut scene) = context.scene.lock() {
      scene.snapshot.composited = composited;
    }
  })
  .is_none()
  {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set_snapshot_composited(view, i32::from(composited)) };
  true
}
