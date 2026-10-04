// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows preview surface: wgpu frames presented by DirectComposition
//! beneath WebView2's child window. Media Foundation decodes on a Direct3D 11
//! device on the shared wgpu device's adapter, and each frame is copied into
//! a texture both devices open, so live recording frames never enter system
//! memory or cross Tauri IPC, while transparent webview regions leave DOM
//! controls above the video.

mod annotation;
mod batch;
mod callbacks;
mod creation;
#[cfg(debug_assertions)]
mod editor_controls;
mod export;
mod geometry;
mod gpu;
mod input;
mod layout;
mod magnifier;
mod pane;
mod present_cached;
mod present_texture;
mod readback;
mod resources;
mod selection_draw;
mod selection_hit;
mod state;
mod still;
#[cfg(test)]
mod tests;
mod thread_dispatch;
mod workspace;

/// Inter SemiBold set by DirectWrite, which measures a text box as well as
/// drawing its type.
pub(crate) mod type_device;
use annotation::handle_typing_input;
use callbacks::{emit_gesture, emit_selection, emit_transform, refresh_cursor_for};
use geometry::{
  auto_fit_selection_bounds, display_selection, frame_resize_start, maximum_editor_zoom,
  pane_canvas_rect, set_pane_geometry,
};
use input::handle_editor_input;
use magnifier::{redraw_composed_panes, redraw_magnifier, update_magnifier};
use resources::{Backdrop, Gpu, Pane};
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
use windows::{
  core::Interface,
  Win32::{
    Foundation::HWND,
    Graphics::{
      Direct3D11::{
        ID3D11Device, ID3D11Resource, ID3D11Texture2D, D3D11_BIND_RENDER_TARGET,
        D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT,
      },
      DirectComposition::{
        DCompositionCreateDevice, IDCompositionDevice, IDCompositionRectangleClip,
        IDCompositionScaleTransform, IDCompositionTarget, IDCompositionVisual,
        DCOMPOSITION_BITMAP_INTERPOLATION_MODE_LINEAR,
      },
      Dxgi::{
        Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
        IDXGIDevice,
      },
    },
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::GetWindowThreadProcessId,
  },
};

mod audio_ribbon;
mod compositor_source;
mod cursor_artwork;
mod editor;
mod font;
mod keyboard_hit;
pub(crate) mod keyboard_raster;
use keyboard_hit::{keyboard_transform_start, redraw_keyboard_transform};
mod recenter;
mod selection;
mod snapping;
mod sticker_drop;
mod view_fit;
mod window;
mod workspace_layout;

use super::compositor::{self, ComposedFrame, CropMagnifier};
use super::{
  workspace_editor::{
    apply_crop_draw, apply_crop_move, apply_crop_resize, apply_framed_crop_resize,
    crop_magnifier_anchor, hit_test_display, rebase_display_fit_mode, DisplayRect, DisplayTarget,
    NormalizedRect,
  },
  workspace_transform::WorkspaceTransform,
  AnnotationGestureCallback, AnnotationHoverCallback, AnnotationTextCallback, AnnotationTextPhase,
  ContextMenuCallback, PointerDownCallback, PreviewSelection, PreviewSurfaceRect,
  SelectionCallback, SelectionGestureCallback, SelectionGestureOperation, SelectionGesturePhase,
  StickerDropCallback, TransformCallback,
};
use crate::editor::media_preview::{BakeGeometry, BakedVideoExportOptions, VideoExportOptions};
use crate::gpu::surface::{Frame, Surface};
use crate::gpu::D3d11Layer;
use crate::screenshots::{CapturedImage, ScreenshotOutputSettings};
use view_fit::fit_basis_transform;
use workspace_layout::{
  apply_workspace_transform, aspect_fit_rect, rebase_workspace_fit, reflow_workspace_panes,
  union_rect,
};

pub(crate) struct StillOverlay;

const FRAME_LAYER_ID: u32 = u32::MAX;
const CENTERED_RESIZE_EDGE: u32 = 1 << 16;
/// Edge bits shared with the macOS surface and both preview managers: an
/// Alt-drag Move grows the canvas around the layer, and releasing Alt
/// mid-drag accepts that canvas as the origin for the rest of the gesture.
const AUTO_FIT_MOVE_EDGE: u32 = 1 << 17;
const AUTO_FIT_COMMIT_EDGE: u32 = 1 << 18;

/// Logical placement of a recording layer in the retained workspace. Windows
/// keeps the placement in the DirectComposition pane geometry rather than
/// baking it into an intermediate bitmap, but the type mirrors the macOS
/// surface so the recording pipeline can submit one platform-independent
/// workspace description.
#[repr(C)]
#[derive(Clone, Copy, Default)]
// Retained-workspace contract kept in parity with macOS; not wired on Windows yet.
#[allow(dead_code)]
pub(crate) struct NativeWorkspacePlacement {
  pub(crate) x: i32,
  pub(crate) y: i32,
  pub(crate) width: u32,
  pub(crate) height: u32,
}

/// One decoded source in the retained recording workspace. Native Windows
/// playback currently supplies RGBA frames; the pixel-buffer fields are kept
/// in the contract for parity with the zero-copy macOS path and are rejected
/// clearly until the Media Foundation texture path is wired here.
// Retained-workspace contract kept in parity with macOS; not wired on Windows yet.
#[allow(dead_code)]
pub(crate) struct RecordingWorkspaceLayer<'a> {
  pub pane_index: u32,
  pub source_token: u64,
  pub source: Option<&'a CapturedImage>,
  pub source_pixels: Option<(*mut std::ffi::c_void, (u32, u32))>,
  pub settings: ScreenshotOutputSettings,
  pub placement: NativeWorkspacePlacement,
  pub seconds: f64,
  pub cursor: Option<&'a CapturedImage>,
  pub camera: Option<&'a CapturedImage>,
  pub camera_pixels: Option<(*mut std::ffi::c_void, (u32, u32))>,
  pub overlay: Option<&'a StillOverlay>,
  pub clip_cursor_at_video_edge: bool,
  pub foreground_only: bool,
}

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

/// An offscreen instance of the live preview compositor. Its source and target
/// textures are allocated once and reused for every exported frame.
pub(crate) struct WindowsExportCompositor {
  camera: Option<compositor::SourceTexture>,
  inner: std::sync::Arc<SurfaceInner>,
  output_size: (u32, u32),
  source: compositor::SourceTexture,
  /// What each frame is drawn into, which Direct3D 11 copies out of.
  target: crate::gpu::SharedTexture,
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

/// An open present batch. Dropping it presents every parked frame
/// back-to-back, applies every pending pane geometry, and commits once, so
/// all panes reach the compositor in (almost always) the same pass - the
/// DirectComposition counterpart of the macOS single-`CATransaction` batch.
pub(crate) struct PresentBatch<'a> {
  surface: &'a RecordingPreviewSurface,
}
