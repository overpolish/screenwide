// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! macOS surface and FFI adapter for the shared OSC runtime.

mod desktop;
mod ffi;
mod magnifier;
mod ocr;
mod state;

#[cfg(test)]
use crate::osc::protocol::InputPhase;
#[cfg(test)]
use crate::osc::protocol::RESULT_GESTURE_DRAWING;
use crate::osc::runtime::OscRuntime as Context;
#[cfg(test)]
use crate::osc::semantic::SemanticStatus;
#[cfg(test)]
use crate::osc::semantic::{event_payload as payload_for, semantic_handle as native_handle};
#[cfg(test)]
use crate::osc::session::{apply_phase_cursor, event_kind, result_for};

pub use crate::osc::{
  desktop::DesktopBinding,
  protocol::{OscResult as NativeOscResult, Purpose, ResultStatus},
};
pub use desktop::configure_window as configure_desktop_window;
pub use magnifier::set_source as set_magnifier_source;

pub const NATIVE_OSC_EVENT: &str = crate::osc::semantic::REGION_EVENT;
pub const NATIVE_OSC_LAYOUT_EVENT: &str = crate::osc::semantic::DESKTOP_LAYOUT_EVENT;

pub use ocr::{reset_input as reset_text_recognition_input, set_ocr};
#[cfg(test)]
use state::invalid_result;
pub use state::{
  apply_region_scene, claim_pointer_surface, clear_region, configure_desktop, ensure_attached,
  ensure_ruler_attached, ensure_text_recognition_attached, present_region,
  reconcile_region_scene_request, refresh_ruler_pointer, region_scene, region_scene_request_base,
  restore_normal_region_scene, set_allow_drawing, set_aspect, set_committed, set_desktop_presented,
  set_input_enabled, set_monitor, set_ruler_transient_chrome, set_show_frame, set_show_handles,
  set_snapshot, set_snapshot_composited, set_snapshot_presented,
};

#[cfg(test)]
mod tests;
