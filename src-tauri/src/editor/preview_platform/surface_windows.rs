// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Windows preview surface: wgpu frames presented by DirectComposition
//! beneath WebView2's child window. Media Foundation decodes on a Direct3D 11
//! device on the shared wgpu device's adapter, and each frame is copied into
//! a texture both devices open, so live recording frames never enter system
//! memory or cross Tauri IPC, while transparent webview regions leave DOM
//! controls above the video.
//!
//! What a press, a drag or a key does is the shared editing engine's
//! ([`WorkspaceEditor`]); this shell gives it the windows, the drawables and
//! the keyboard through [`host::WindowsHost`].

mod audio_ribbon;
mod backdrop;
mod creation;
mod editor;
mod gpu;
mod host;
mod image_drop;
mod registry;
mod resources;
mod selection;
mod thread_dispatch;
mod workspace_view;

use std::{
  collections::HashMap,
  sync::{Mutex, OnceLock},
};

use backdrop::Backdrop;
use host::windows;
use registry::{preview_surfaces, surface_for_editor, surface_index};
use resources::Gpu;
use screenwide_compositor::PreviewSurfaceRect;
use screenwide_gpu::surface::{Frame, Surface};
use screenwide_gpu::D3d11Layer;
use screenwide_workspace::editing::pixels;
use screenwide_workspace::editing::*;
use tauri::WebviewWindow;
use thread_dispatch::create_editor_on_owning_thread;
use windows::Win32::{
  Foundation::HWND,
  Graphics::{
    Direct3D11::ID3D11Device,
    DirectComposition::{
      DCompositionCreateDevice, IDCompositionDevice, IDCompositionScaleTransform,
      IDCompositionTarget, IDCompositionVisual,
    },
    Dxgi::IDXGIDevice,
  },
  System::Threading::GetCurrentThreadId,
  UI::WindowsAndMessaging::GetWindowThreadProcessId,
};
use workspace_view::WorkspaceView;

use super::ImageDropCallback;

pub(crate) struct RecordingPreviewSurface {
  editor: WorkspaceEditor,
}

/// The surface answers every editing call through the shared engine; only
/// what Windows draws or owns is the surface's own.
impl std::ops::Deref for RecordingPreviewSurface {
  type Target = WorkspaceEditor;

  fn deref(&self) -> &WorkspaceEditor {
    &self.editor
  }
}

/// The engine behind the editor window `editor_hwnd`, once its surface is
/// whole.
fn engine_for(editor_hwnd: HWND) -> Option<WorkspaceEditor> {
  surface_for_editor(editor_hwnd).map(|inner| WorkspaceEditor { inner })
}

pub(super) fn handle_editor_input(editor_hwnd: HWND, input: PointerInput) {
  if let Some(engine) = engine_for(editor_hwnd) {
    engine.handle_pointer(input);
  }
}

pub(super) fn handle_typing_input(editor_hwnd: HWND, input: TypingInput) {
  if let Some(engine) = engine_for(editor_hwnd) {
    engine.handle_typing(input);
  }
}

pub(super) fn refresh_editor_cursor(editor_hwnd: HWND) {
  if let Some(engine) = engine_for(editor_hwnd) {
    engine.refresh_cursor();
  }
}
