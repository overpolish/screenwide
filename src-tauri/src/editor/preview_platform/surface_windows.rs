// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows preview surface: GPU frames presented by DirectComposition beneath
//! WebView2's child window. Media Foundation and this surface share one D3D11 device, so live
//! recording frames never enter system memory or cross Tauri IPC, while transparent webview
//! regions leave DOM controls above the video.

#[path = "surface_windows/annotation.rs"]
mod annotation;
#[path = "surface_windows/batch.rs"]
mod batch;
#[path = "surface_windows/callbacks.rs"]
mod callbacks;
#[path = "surface_windows/creation.rs"]
mod creation;
#[cfg(debug_assertions)]
#[path = "surface_windows/editor_controls.rs"]
mod editor_controls;
#[path = "surface_windows/export.rs"]
mod export;
#[path = "surface_windows/geometry.rs"]
mod geometry;
#[path = "surface_windows/gpu.rs"]
mod gpu;
#[path = "surface_windows/input.rs"]
mod input;
#[path = "surface_windows/layout.rs"]
mod layout;
#[path = "surface_windows/magnifier.rs"]
mod magnifier;
#[path = "surface_windows/pane.rs"]
mod pane;
#[path = "surface_windows/present_cached.rs"]
mod present_cached;
#[path = "surface_windows/present_texture.rs"]
mod present_texture;
#[path = "surface_windows/readback.rs"]
mod readback;
#[path = "surface_windows/resources.rs"]
mod resources;
#[path = "surface_windows/selection_draw.rs"]
mod selection_draw;
#[path = "surface_windows/selection_hit.rs"]
mod selection_hit;
#[path = "surface_windows/state.rs"]
mod state;
#[path = "surface_windows/still.rs"]
mod still;
#[cfg(test)]
#[path = "surface_windows/tests.rs"]
mod tests;
#[path = "surface_windows/thread_dispatch.rs"]
mod thread_dispatch;
#[path = "surface_windows/workspace.rs"]
mod workspace;

/// The arrow renderer's shareable parts. Live annotation draws the desktop's
/// arrows with this pipeline: the same prepared geometry, the same structured
/// buffers and the same shader, in one display's layer pixels rather than in
/// the canvas's.
pub(crate) mod arrows {
  pub(crate) use super::annotation::placed_arrows;
  pub(crate) use super::compositor::{
    PreparedArrows, PreviewArrow, PreviewSample, StructuredBuffer,
  };
  /// A counter's number is type, so it is rasterised rather than drawn by the
  /// shader. The overlay lays its numbers out in an atlas of its own, the same
  /// way the editor's compositor does.
  pub(crate) use super::counter_artwork::{numbered_arrows, CounterAtlas};
}
/// Inter SemiBold set by DirectWrite, which measures a text box as well as
/// drawing its type.
#[path = "surface_windows/type_device.rs"]
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
  clear_selection_snap_guides, cursor_for_state, radius_point, selection_pane_rect,
  selection_snap_targets, shared_handle_edges, shared_selection_hit,
};
use state::{
  ActiveGesture, EditorCallbacks, EditorGesture, FrameResizeStart, MoveAutoFit, SurfaceInner,
  SurfaceState,
};
use thread_dispatch::create_editor_on_owning_thread;

#[path = "surface_windows/registry.rs"]
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
    Foundation::{HMODULE, HWND},
    Graphics::{
      Direct3D::{D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_11_1},
      Direct3D10::ID3D10Multithread,
      Direct3D11::{
        D3D11CreateDevice, ID3D11Buffer, ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView,
        ID3D11Resource, ID3D11Texture2D, D3D11_BIND_RENDER_TARGET, D3D11_CPU_ACCESS_READ,
        D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_CREATE_DEVICE_VIDEO_SUPPORT,
        D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_READ, D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC,
        D3D11_USAGE_DEFAULT, D3D11_USAGE_STAGING,
      },
      DirectComposition::{
        DCompositionCreateDevice, IDCompositionDevice, IDCompositionRectangleClip,
        IDCompositionScaleTransform, IDCompositionTarget, IDCompositionVisual,
        DCOMPOSITION_BITMAP_INTERPOLATION_MODE_LINEAR,
      },
      Dxgi::{
        Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
        IDXGIAdapter, IDXGIDevice, IDXGIFactory2, IDXGISwapChain3, DXGI_PRESENT,
        DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_CHAIN_FLAG,
        DXGI_SWAP_EFFECT_FLIP_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT,
      },
    },
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::GetWindowThreadProcessId,
  },
};

#[path = "surface_windows/audio_ribbon.rs"]
mod audio_ribbon;
#[path = "surface_windows/background_image.rs"]
mod background_image;
#[path = "surface_windows/compositor.rs"]
mod compositor;
#[path = "surface_windows/counter_artwork.rs"]
mod counter_artwork;
#[path = "surface_windows/editor.rs"]
mod editor;
#[path = "surface_windows/font.rs"]
mod font;
#[path = "surface_windows/keyboard_artwork.rs"]
mod keyboard_artwork;
#[path = "surface_windows/keyboard_hit.rs"]
mod keyboard_hit;
use keyboard_hit::{keyboard_transform_start, redraw_keyboard_transform};
#[path = "surface_windows/recenter.rs"]
mod recenter;
#[path = "surface_windows/selection.rs"]
mod selection;
#[path = "surface_windows/snapping.rs"]
mod snapping;
#[path = "surface_windows/view_fit.rs"]
mod view_fit;
#[path = "surface_windows/window.rs"]
mod window;
#[path = "surface_windows/workspace_layout.rs"]
mod workspace_layout;

use super::{
  workspace_editor::{
    apply_crop_move, apply_crop_resize, crop_magnifier_anchor, hit_test_display,
    rebase_display_fit_mode, DisplayRect, DisplayTarget, NormalizedRect,
  },
  workspace_transform::WorkspaceTransform,
  AnnotationGestureCallback, AnnotationHoverCallback, AnnotationTextCallback, AnnotationTextPhase,
  ContextMenuCallback, PointerDownCallback, PreviewSelection, PreviewSurfaceRect,
  SelectionCallback, SelectionGestureCallback, SelectionGestureOperation, SelectionGesturePhase,
  TransformCallback,
};
use crate::editor::media_preview::{BakeGeometry, BakedVideoExportOptions, VideoExportOptions};
use crate::screenshots::{CapturedImage, ScreenshotOutputSettings};
use view_fit::fit_basis_transform;
use workspace_layout::{
  apply_workspace_transform, aspect_fit_rect, rebase_workspace_fit, reflow_workspace_panes,
  union_rect,
};

pub(crate) struct StillOverlay;

const FRAME_LAYER_ID: u32 = u32::MAX;
const CENTERED_RESIZE_EDGE: u32 = 1 << 16;
/// Edge bits shared with the Metal backend and both preview managers: an
/// Alt-drag Move grows the canvas around the layer, and releasing Alt
/// mid-drag accepts that canvas as the origin for the rest of the gesture.
const AUTO_FIT_MOVE_EDGE: u32 = 1 << 17;
const AUTO_FIT_COMMIT_EDGE: u32 = 1 << 18;

/// Logical placement of a recording layer in the retained workspace. Windows
/// keeps the placement in the DirectComposition pane geometry rather than
/// baking it into an intermediate bitmap, but the type mirrors the Metal
/// backend so the recording pipeline can submit one platform-independent
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

#[derive(Clone, Copy)]
pub(crate) struct ComposedFrame {
  pub cursor: Option<crate::editor::cursor_effects::GpuCursor>,
  pub keyboard: Option<crate::editor::keyboard_effects::KeyboardOverlay>,
  pub foreground_only: bool,
  pub seconds: f64,
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
