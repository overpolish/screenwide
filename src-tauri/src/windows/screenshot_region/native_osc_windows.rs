// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows D3D11 twin of `native_osc_macos`: the region scene, the desktop
//! peers and snapshots, the OCR overlay and the Ruler — surface, state,
//! pointer and keyboard input, cursors, text and chrome.

mod desktop;
mod input;
mod ocr;
mod ruler;
mod state;
mod surface;
mod text;

// Region supplies scene state; primitive construction and shader ownership
// live in the shared OSC layer used by Export as well.
pub(crate) use crate::osc::gpu::windows as renderer;

pub(crate) use input::OVERLAY_KEY_EVENT;

// Re-exported with the macOS facade's shape. The Region adapter is the only
// stage-1 consumer; the OCR and ruler overlays pick up the rest in stages 3-4.
#[allow(unused_imports)]
pub use crate::osc::{
  desktop::DesktopBinding,
  protocol::{OscResult as NativeOscResult, Purpose, ResultStatus},
};

pub const NATIVE_OSC_EVENT: &str = crate::osc::semantic::REGION_EVENT;
#[allow(dead_code)]
pub const NATIVE_OSC_LAYOUT_EVENT: &str = crate::osc::semantic::DESKTOP_LAYOUT_EVENT;

#[allow(unused_imports)]
pub(crate) use state::{
  apply_region_scene, claim_pointer_surface, clear_region, configure_desktop,
  configure_desktop_window, ensure_attached, ensure_ruler_attached,
  ensure_text_recognition_attached, focus_ruler_input, input_hwnd, present_region,
  reconcile_region_scene_request, refresh_ruler_pointer, region_scene, region_scene_request_base,
  reset_text_recognition_input, restore_normal_region_scene, set_allow_drawing, set_aspect,
  set_capture_affinity, set_committed, set_desktop_presented, set_input_enabled,
  set_magnifier_source, set_monitor, set_ocr, set_ocr_cancel_visible, set_ruler_transient_chrome,
  set_show_frame, set_show_handles, set_snapshot, set_snapshot_composited, set_snapshot_presented,
};
