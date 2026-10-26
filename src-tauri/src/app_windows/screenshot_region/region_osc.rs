// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The region OSC as every platform runs it: the region scene, the desktop
//! peers and snapshots, the OCR overlay and the Ruler - surfaces, state,
//! pointer and keyboard input, cursors, text and chrome, drawn through the
//! shared OSC shader. A platform provides its windows, drawables, timers and
//! type through [`host::RegionHost`] and hands its decoded events to
//! [`input`]; Windows does so in `native_osc_windows` and macOS in
//! `native_osc_macos`.

mod desktop;
pub(crate) mod host;
pub(crate) mod input;
mod ocr;
#[cfg(target_os = "macos")]
pub(crate) mod preview;
mod ruler;
mod segment;
mod state;
mod surface;
pub(crate) mod text;

// Region supplies scene state; primitive construction and shader ownership
// live in the shared OSC layer used by Export as well.
pub(crate) use screenwide_osc::gpu::vertices as renderer;

// Re-exported with the macOS facade's shape.
#[allow(unused_imports)]
pub use screenwide_osc::{
  desktop::DesktopBinding,
  protocol::{OscResult as NativeOscResult, Purpose, ResultStatus},
};

pub const NATIVE_OSC_EVENT: &str = screenwide_osc::semantic::REGION_EVENT;
#[allow(dead_code)]
pub const NATIVE_OSC_LAYOUT_EVENT: &str = screenwide_osc::semantic::DESKTOP_LAYOUT_EVENT;

#[allow(unused_imports)]
pub(crate) use host::KeyboardOwner;
#[allow(unused_imports)]
pub(crate) use state::{
  apply_region_scene, capture_keyboard, claim_pointer_surface, clear_region, configure_desktop,
  configure_desktop_window, ensure_attached, ensure_ruler_attached,
  ensure_text_recognition_attached, focus_ruler_input, input_hwnd, on_owning_thread,
  present_region, reconcile_region_scene_request, refresh_ruler_pointer, region_scene,
  region_scene_request_base, release_keyboard, reset_text_recognition_input,
  restore_normal_region_scene, ruler_phase, set_allow_drawing, set_aspect, set_backdrops,
  set_capture_affinity, set_committed, set_desktop_presented, set_input_enabled,
  set_magnifier_source, set_monitor, set_ocr, set_ruler_transient_chrome, set_show_frame,
  set_show_handles, set_snapshot, set_snapshot_composited, set_snapshot_presented, take_presses,
};
