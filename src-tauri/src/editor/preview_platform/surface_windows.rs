// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows preview surface: wgpu frames presented by DirectComposition
//! beneath WebView2's child window. Media Foundation decodes on a Direct3D 11
//! device on the shared wgpu device's adapter, and each frame is copied into
//! a texture both devices open, so live recording frames never enter system
//! memory or cross Tauri IPC, while transparent webview regions leave DOM
//! controls above the video.

mod annotation;
mod backdrop;
mod batch;
mod callbacks;
mod creation;
mod editor_controls;
mod export;
pub(crate) use export::recording_export_surface;
mod geometry;
mod gpu;
mod input;
mod layout;
mod magnifier;
mod object_picking;
mod readback;
mod recording_workspace;
pub(crate) use super::workspace_scene::baked_camera_overlay;
pub(crate) use recording_workspace::DecodedTexture;
mod resources;
mod screenshot_workspace;
mod selection_draw;
mod selection_hit;
mod state;
#[cfg(test)]
mod tests;
mod thread_dispatch;
mod turn;
mod workspace;
mod workspace_view;

use annotation::handle_typing_input;
use backdrop::Backdrop;
use callbacks::{emit_gesture, emit_selection, emit_transform, refresh_cursor_for, resize_cursor};
use geometry::{
  auto_fit_selection_bounds, display_selection, frame_resize_start, maximum_editor_zoom,
  pane_canvas_rect, set_pane_geometry,
};
use input::handle_editor_input;
use magnifier::{redraw_workspace, update_magnifier};
use resources::{Gpu, Pane};
/// Inter SemiBold set by DirectWrite, which measures a text box as well as
/// drawing its type: the annotations' face, which the macOS surface has a
/// Core Text twin of.
pub(crate) use screenwide_text::engine::type_device;
use selection_draw::{draw_selection, redraw_stale_selection};
use selection_hit::{
  clear_selection_snap_guides, crop_press_draws, cursor_for_state, radius_point,
  selection_pane_rect, selection_snap_targets, shared_handle_edges, shared_selection_hit,
};
use state::{
  ActiveGesture, CropDrawStart, EditorCallbacks, EditorGesture, FrameResizeStart, MoveAutoFit,
  SurfaceInner, SurfaceState,
};
use thread_dispatch::create_editor_on_owning_thread;
use workspace_view::{
  clear_workspace, draw_workspace, present_workspace, WorkspaceDraw, WorkspaceView,
};

mod registry;
use registry::{preview_surfaces, surface_for_editor, surface_index};

use std::{
  collections::HashMap,
  sync::{
    atomic::{AtomicU32, Ordering},
    Mutex, OnceLock,
  },
};

use tauri::WebviewWindow;
use windows::Win32::{
  Foundation::HWND,
  Graphics::{
    Direct3D11::{ID3D11Device, ID3D11Texture2D},
    DirectComposition::{
      DCompositionCreateDevice, IDCompositionDevice, IDCompositionScaleTransform,
      IDCompositionTarget, IDCompositionVisual,
    },
    Dxgi::IDXGIDevice,
  },
  System::Threading::GetCurrentThreadId,
  UI::WindowsAndMessaging::GetWindowThreadProcessId,
};

mod audio_ribbon;
mod cursor_artwork;
mod editor;
mod image_drop;
mod overlay_hit;
mod recenter;
mod selection;
mod snapping;
mod view_fit;
mod window;
mod workspace_layout;

use super::{
  AnnotationGestureCallback, AnnotationHoverCallback, AnnotationTextCallback, AnnotationTextPhase,
  ContextMenuCallback, ImageDropCallback, PointerDownCallback, PreviewSelection, SelectionCallback,
  SelectionGestureCallback, SelectionGestureOperation, SelectionGesturePhase, TransformCallback,
};
use screenwide_compositor::compositor::{self, ComposedFrame};
use screenwide_compositor::workspace_editor::{
  apply_crop_draw, apply_crop_move, apply_crop_resize, apply_framed_crop_resize,
  crop_magnifier_anchor, hit_test_display, rebase_display_fit_mode, DisplayRect, DisplayTarget,
  NormalizedRect,
};
use screenwide_compositor::workspace_transform::WorkspaceTransform;
use screenwide_compositor::PreviewSurfaceRect;
use screenwide_export::media_preview::BakeGeometry;
use screenwide_gpu::surface::{Frame, Surface};
use screenwide_gpu::D3d11Layer;
use screenwide_imaging::CapturedImage;
use screenwide_output::ScreenshotOutputSettings;
use view_fit::fit_basis_transform;
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

type ClipboardCamera<'a> = (
  &'a ID3D11Texture2D,
  u32,
  (u32, u32),
  BakeGeometry,
  bool,
  bool,
);

const MINIMUM_EDITOR_ZOOM_CEILING: f64 = 16.0;
const NATIVE_PIXEL_ZOOM_HEADROOM: f64 = 4.0;

pub(crate) struct RecordingPreviewSurface {
  inner: std::sync::Arc<SurfaceInner>,
}

unsafe impl Send for RecordingPreviewSurface {}
unsafe impl Sync for RecordingPreviewSurface {}
unsafe impl Send for SurfaceInner {}
unsafe impl Sync for SurfaceInner {}

pub(super) fn refresh_editor_cursor(editor_hwnd: HWND) {
  let Some(inner) = surface_for_editor(editor_hwnd) else {
    return;
  };
  refresh_cursor_for(&inner);
}

/// An open present batch. Dropping it draws the workspace once, with every
/// layout and present made meanwhile, and commits once - the
/// DirectComposition counterpart of the macOS single-`CATransaction` batch.
pub(crate) struct PresentBatch<'a> {
  surface: &'a RecordingPreviewSurface,
}
