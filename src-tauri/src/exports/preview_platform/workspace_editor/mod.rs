// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Platform-neutral model for the native export workspace.
//!
//! This module deliberately contains no renderer or webview code.  Both GPU
//! backends can consume the same frames, layers, hit-test results, and gesture
//! transaction boundaries.  Coordinates in a scene are in workspace points;
//! layer rectangles are normalized to their owning frame.

mod crop;
mod display;
mod ffi;
mod frame_resize;
mod geometry;
mod hit_test;
mod inset_resize;
mod limits;
#[cfg(any(target_os = "windows", test))]
mod magnifier;
mod scene;
#[cfg(any(target_os = "windows", test))]
mod selection_resize;

#[cfg(test)]
mod tests;

#[cfg(any(target_os = "windows", test))]
pub use crop::{apply_crop_move, apply_crop_resize};
// Preserve the original flat module API while implementations live at their
// feature extension points.
#[allow(unused_imports)]
pub use display::{
  rebase_display_fit, rebase_display_fit_mode, DisplayFitRebase, DisplayHandle, DisplayHit,
  DisplayRect, DisplayTarget,
};
#[allow(unused_imports)]
pub use frame_resize::FrameResizeResult;
pub use geometry::{
  apply_layer_gesture, fit_canvas_to_layers, rebase_layer_geometry, GestureOperation,
  LayerGeometry, NormalizedRect, WorldRect,
};
#[allow(unused_imports)]
pub use hit_test::hit_test_display;
pub use inset_resize::resize_uniform_inset_from_scale;
#[cfg(any(target_os = "windows", test))]
#[allow(unused_imports)]
pub use inset_resize::{resize_uniform_inset, InsetResize};
#[allow(unused_imports)]
pub use limits::{
  FRAME_EDGE_BOTTOM, FRAME_EDGE_CENTERED, FRAME_EDGE_LEFT, FRAME_EDGE_RIGHT, FRAME_EDGE_TOP,
};
#[cfg(test)]
use limits::{FRAME_MAX_AREA, FRAME_MIN_SIZE};
#[cfg(target_os = "windows")]
pub use magnifier::crop_magnifier_anchor;
#[allow(unused_imports)]
pub use scene::{FrameId, LayerId, WorkspaceFrame, WorkspaceKind, WorkspaceLayer, WorkspaceScene};
#[cfg(target_os = "windows")]
pub use selection_resize::{selection_resize, SelectionResize};
