// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! macOS preview surface: wgpu frames presented into `CAMetalLayer`s below
//! the `WKWebView`, with the selection chrome on a layer of the input view
//! laid over it, so transparent webview regions leave DOM controls above the
//! picture.
//!
//! What a press, a drag or a key does is the shared editing engine's
//! ([`WorkspaceEditor`]); this shell gives it the views, the drawables and
//! the keyboard through [`host::MacHost`]. It is the twin of
//! `surface_windows`.

mod audio_ribbon;
mod creation;
mod cursor;
mod host;
mod image_drop;
mod layers;
mod main_thread;
mod pasteboard;
mod registry;
mod view;
mod workspace_view;

use std::sync::Mutex;

use host::mac;
use screenwide_gpu::surface::{Frame, Surface};
use screenwide_workspace::editing::*;
use tauri::WebviewWindow;

use super::ImageDropCallback;

pub(crate) struct RecordingPreviewSurface {
  editor: WorkspaceEditor,
}

/// The surface answers every editing call through the shared engine; only
/// what macOS draws or owns is the surface's own.
impl std::ops::Deref for RecordingPreviewSurface {
  type Target = WorkspaceEditor;

  fn deref(&self) -> &WorkspaceEditor {
    &self.editor
  }
}

impl RecordingPreviewSurface {
  /// While the frontend covers the workarea with its own chrome the input
  /// view goes away, and with it the selection chrome it carries; the
  /// workspace transform is untouched, so nothing has to be restored after.
  pub(crate) fn set_editor_suspended(&self, suspended: bool) {
    let Ok(state) = self.inner.state.lock() else {
      return;
    };
    mac(&self.inner).set_suspended(suspended, state.editor_active);
  }
}
