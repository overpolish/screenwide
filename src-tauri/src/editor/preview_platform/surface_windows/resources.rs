// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) struct Gpu {
  pub(super) audio_ribbon: std::sync::Mutex<audio_ribbon::AudioRibbon>,
  pub(super) backdrop: Backdrop,
  /// Direct3D 11 on the shared device: Media Foundation decodes on it.
  pub(super) d3d11: &'static D3d11Layer,
  /// Composes stills read back for the clipboard, apart from the workspace.
  pub(super) compositor: compositor::Compositor,
  pub(super) composition: IDCompositionDevice,
  pub(super) selection: Mutex<selection::SelectionOverlay>,
  pub(super) shared: &'static screenwide_gpu::Gpu,
  pub(super) _editor_target: IDCompositionTarget,
  pub(super) _target: IDCompositionTarget,
  pub(super) workspace: WorkspaceView,
}

/// One layer's box in the workspace and what was last presented in it, which
/// selection, hit-testing and typing read. The workspace scene draws the
/// layer itself.
pub(super) struct Pane {
  /// The box being typed into, if it belongs to this pane's layer: its place
  /// in that layer's list and the caret and selection drawn into its type.
  /// Kept across presents, so every redraw the typing causes carries it.
  pub(super) annotation_typing: Option<(usize, screenwide_text::engine::typing::TypingMarks)>,
  /// The caret and selection of the caption being typed into on this pane,
  /// kept across presents as the box's are.
  pub(super) caption_typing: Option<screenwide_text::engine::typing::TypingMarks>,
  /// The captions drawn over this pane's canvas, and the moment of the
  /// edited video they are drawn at.
  pub(super) captions: Option<(
    std::sync::Arc<screenwide_text::captions::CaptionTrack>,
    screenwide_text::captions::CaptionMoment,
  )>,
  /// Stable viewport-local geometry before the shared workspace transform.
  pub(super) base_rect: PreviewSurfaceRect,
  /// The size of the canvas last presented.
  pub(super) content_size: (u32, u32),
  pub(super) display_size: (i32, i32),
  pub(super) settings: Option<ScreenshotOutputSettings>,
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
  pub(super) seen: bool,
  /// The screenshot item the pane holds, by its token.
  pub(super) source_token: Option<u64>,
}

impl Default for Pane {
  fn default() -> Self {
    Self {
      annotation_typing: None,
      caption_typing: None,
      captions: None,
      base_rect: PreviewSurfaceRect {
        height: 0.0,
        width: 0.0,
        x: 0.0,
        y: 0.0,
      },
      content_size: (2, 2),
      display_size: (2, 2),
      settings: None,
      position: (0, 0),
      selection_stale: false,
      scale: 1.0,
      seen: true,
      source_token: None,
    }
  }
}
