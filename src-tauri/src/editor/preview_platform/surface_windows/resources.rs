// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) struct Gpu {
  pub(super) audio_ribbon: std::sync::Mutex<audio_ribbon::AudioRibbon>,
  pub(super) backdrop: Backdrop,
  pub(super) compositor: compositor::Compositor,
  pub(super) composition: IDCompositionDevice,
  pub(super) context: ID3D11DeviceContext,
  pub(super) device: ID3D11Device,
  pub(super) factory: IDXGIFactory2,
  pub(super) root: IDCompositionVisual,
  pub(super) selection: Mutex<selection::SelectionOverlay>,
  pub(super) _editor_target: IDCompositionTarget,
  pub(super) _target: IDCompositionTarget,
}

pub(super) struct Backdrop {
  pub(super) scale_transform: IDCompositionScaleTransform,
  pub(super) swap_chain: IDXGISwapChain3,
  pub(super) visual: IDCompositionVisual,
}

pub(super) struct Pane {
  /// The halo this pane's composition draws, if the hovered arrow belongs to
  /// its layer: the arrow's place in that layer's list and the halo's width
  /// in canvas pixels. Preview chrome, resolved when the layer is presented.
  pub(super) annotation_halo: Option<(usize, f32)>,
  /// The box being typed into, if it belongs to this pane's layer: its place
  /// in that layer's list and the caret and selection drawn into its type.
  /// Kept across presents, so every redraw the typing causes carries it.
  pub(super) annotation_typing:
    Option<(usize, crate::editor::annotations::text::typing::TypingMarks)>,
  /// Stable viewport-local geometry before the shared workspace transform.
  pub(super) base_rect: PreviewSurfaceRect,
  /// Retained swap-chain allocation; `content_size` is the presented region.
  pub(super) buffer_size: (u32, u32),
  pub(super) clip: IDCompositionRectangleClip,
  pub(super) clip_edges: (i32, i32, i32, i32),
  /// The presented region of the buffer - the actual output resolution.
  pub(super) content_size: (u32, u32),
  pub(super) display_size: (i32, i32),
  /// Geometry waiting to publish atomically with its matching frame.
  pub(super) pending_geometry: bool,
  /// Last composition, retained for redraws that do not need another decode.
  pub(super) last_composition: Option<ComposedFrame>,
  /// The camera overlay the most recent present composed over the source, so a
  /// local redraw (magnifier, geometry) never drops the baked camera for a frame.
  pub(super) last_camera: Option<(BakeGeometry, bool, bool)>,
  pub(super) settings: Option<ScreenshotOutputSettings>,
  pub(super) magnifier: Option<recenter::CropMagnifier>,
  /// A frame drawn inside an open present batch, waiting for the batch flush
  /// to call `Present` so every pane's new pixels reach the compositor in the
  /// same pass.
  pub(super) pending_present: bool,
  pub(super) position: (i32, i32),
  /// The last present changed the composed canvas size. The selection overlay
  /// is fitted to that canvas (`pane_canvas_rect`), so it has to be redrawn
  /// once the present lands or it keeps the previous canvas's aspect until
  /// the next layout - a fresh capture of a different shape showed the old
  /// OSC until the user dragged it.
  pub(super) selection_stale: bool,
  /// The window's device-pixel scale as of the pane's last layout, so a blur
  /// expressed in CSS pixels can be converted without the surface state.
  pub(super) scale: f64,
  pub(super) scale_transform: IDCompositionScaleTransform,
  pub(super) seen: bool,
  pub(super) source: Option<compositor::SourceTexture>,
  pub(super) source_token: Option<u64>,
  pub(super) swap_chain: IDXGISwapChain3,
  pub(super) visual: IDCompositionVisual,
}
