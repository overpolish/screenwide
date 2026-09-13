// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[path = "state/input.rs"]
mod input;
#[path = "state/presentation.rs"]
mod presentation;
#[path = "state/ruler_packets.rs"]
mod ruler_packets;
#[path = "state/scene.rs"]
mod scene;
pub(super) use input::{native_osc_input, native_osc_layout_changed};
pub use presentation::{
  claim_pointer_surface, configure_desktop, refresh_ruler_pointer, set_allow_drawing, set_aspect,
  set_desktop_presented, set_input_enabled, set_monitor, set_ruler_transient_chrome,
  set_show_frame, set_show_handles, set_snapshot, set_snapshot_composited, set_snapshot_presented,
};
pub use scene::{
  apply_region_scene, clear_region, present_region, reconcile_region_scene_request, region_scene,
  region_scene_request_base, restore_normal_region_scene, set_committed,
};

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
