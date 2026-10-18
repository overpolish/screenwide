// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The editor workspace's editing engine: what a press, a drag, a key or the
//! wheel does to the layers, annotations and view, and the chrome drawn over
//! them. A platform hands its input to [`WorkspaceEditor`] and provides a
//! [`Host`] for the window, the drawables and the keyboard; nothing here
//! knows which platform it runs on.

mod annotation;
mod batch;
mod callbacks;
mod chrome;
mod controls;
mod drop_landing;
mod geometry;
mod host;
mod input;
mod layout;
mod lifecycle;
mod magnifier;
mod object_picking;
mod overlay_hit;
mod pane;
pub mod pixels;
mod recenter;
mod recording_workspace;
mod screenshot_workspace;
mod selection_draw;
mod selection_hit;
mod snapping;
mod state;
#[cfg(test)]
mod tests;
mod turn;
mod view_fit;
mod vocabulary;
mod workspace_draw;
mod workspace_editor;
mod workspace_layout;

pub use batch::PresentBatch;
pub use chrome::ChromeRenderer;
pub use host::Host;
pub use pane::Pane;
pub use state::{EditorCallbacks, SurfaceInner, SurfaceState};
pub use vocabulary::*;
pub use workspace_draw::WorkspaceDraw;
pub use workspace_editor::WorkspaceEditor;

use std::{
  collections::HashMap,
  sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
  },
};

use annotation::handle_typing_input;
use callbacks::{emit_gesture, emit_selection, emit_transform, refresh_cursor_for, resize_cursor};
use geometry::{
  auto_fit_selection_bounds, display_selection, frame_resize_start, maximum_editor_zoom,
  pane_canvas_rect, set_pane_geometry,
};
use input::handle_editor_input;
use magnifier::{redraw_workspace, update_magnifier};
use screenwide_compositor::workspace_editor::{
  apply_crop_draw, apply_crop_move, apply_crop_resize, apply_framed_crop_resize,
  crop_magnifier_anchor, hit_test_display, rebase_display_fit_mode, DisplayRect, DisplayTarget,
  NormalizedRect,
};
use screenwide_compositor::workspace_transform::WorkspaceTransform;
use screenwide_compositor::PreviewSurfaceRect;
use screenwide_imaging::CapturedImage;
use screenwide_output::ScreenshotOutputSettings;
use selection_draw::{draw_selection, redraw_stale_selection};
use selection_hit::{
  clear_selection_snap_guides, crop_press_draws, cursor_for_state, radius_point,
  selection_pane_rect, selection_snap_targets, shared_handle_edges, shared_selection_hit,
};
use state::{ActiveGesture, CropDrawStart, EditorGesture, FrameResizeStart, MoveAutoFit};
use view_fit::fit_basis_transform;
use workspace_draw::{clear_workspace, draw_workspace, present_workspace};
use workspace_layout::{
  apply_workspace_transform, aspect_fit_rect, rebase_workspace_fit, reflow_workspace_panes,
  union_rect,
};

const FRAME_LAYER_ID: u32 = u32::MAX;
const CENTERED_RESIZE_EDGE: u32 = 1 << 16;
/// Edge bits shared with the macOS surface and both preview managers: an
/// Alt-drag Move grows the canvas around the layer, and releasing Alt
/// mid-drag accepts that canvas as the origin for the rest of the gesture.
const AUTO_FIT_MOVE_EDGE: u32 = 1 << 17;
const AUTO_FIT_COMMIT_EDGE: u32 = 1 << 18;

const MINIMUM_EDITOR_ZOOM_CEILING: f64 = 16.0;
const NATIVE_PIXEL_ZOOM_HEADROOM: f64 = 4.0;
