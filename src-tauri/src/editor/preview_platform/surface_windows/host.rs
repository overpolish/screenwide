// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the shared editing engine asks of Windows: the DirectComposition
//! drawables, the input window's cursor, the keyboard and the clipboard.

use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_CONTROL, VK_SHIFT};

use screenwide_compositor::PreviewSurfaceRect;
use screenwide_osc::gpu::magnifier::NativeMagnifier;
use screenwide_osc::gpu::vertices::{RenderConstants, Vertex};
use screenwide_workspace::backdrop::BackdropStyle;
use screenwide_workspace::editing::{CursorKind, Host, Modifiers, SurfaceInner};
use screenwide_workspace::scene::{NativeWorkspacePlacement, WorkspaceScene};

use super::{editor, Gpu, ImageDropCallback};

pub(super) struct WindowsHost {
  pub(super) editor: editor::EditorWindow,
  pub(super) gpu: Gpu,
  /// The callback a picture dropped on the workspace is handed to. Dropping
  /// a file is a Windows shell affair the engine never sees.
  pub(super) image_drop: std::sync::Mutex<Option<ImageDropCallback>>,
}

// The window handles and composition objects are only touched through calls
// Windows allows from any thread, or posted to the window's own.
unsafe impl Send for WindowsHost {}
unsafe impl Sync for WindowsHost {}

/// The Windows side of an engine this shell made.
pub(super) fn windows(inner: &SurfaceInner) -> &WindowsHost {
  let host: &dyn std::any::Any = inner.host.as_ref();
  host
    .downcast_ref()
    .expect("every Windows editor surface is hosted by WindowsHost")
}

fn down(key: u16) -> bool {
  unsafe { GetKeyState(i32::from(key)) < 0 }
}

impl Host for WindowsHost {
  fn scene(&self) -> &WorkspaceScene {
    &self.gpu.workspace.scene
  }

  fn draw_workspace(
    &self,
    frame: (i32, i32, u32, u32),
    placements: &[NativeWorkspacePlacement],
    magnifier: Option<&NativeMagnifier>,
    backdrop: BackdropStyle,
  ) -> Result<bool, String> {
    self
      .gpu
      .workspace
      .draw(frame, placements, magnifier, backdrop)
  }

  fn hide_workspace(&self) -> bool {
    self.gpu.workspace.hide()
  }

  fn place_viewport(
    &self,
    (x, y, width, height): (i32, i32, i32, i32),
    rect: PreviewSurfaceRect,
    scale: f64,
    backdrop: [f64; 4],
    active: bool,
  ) {
    self.editor.set_frame(x, y, width, height, active);
    self.gpu.backdrop.set_geometry(rect, scale);
    if let Ok(mut ribbon) = self.gpu.audio_ribbon.lock() {
      let _ = ribbon.set_viewport(rect, scale, backdrop);
    }
  }

  fn paint_backdrop(&self, color: [f64; 4]) -> bool {
    self.gpu.backdrop.paint(color).is_ok()
  }

  fn set_active(&self, active: bool) {
    self.editor.set_active(active);
  }

  fn release(&self) {
    if let Ok(mut selection) = self.gpu.selection.lock() {
      selection.renderer.release_drawables();
    }
    self.gpu.backdrop.hide();
  }

  fn present_chrome(&self, size: (u32, u32), vertices: &[Vertex], constants: &RenderConstants) {
    if let Ok(mut overlay) = self.gpu.selection.lock() {
      let _ = overlay.renderer.present(size, vertices, constants);
    }
  }

  fn commit(&self, wait: bool) {
    if unsafe { self.gpu.composition.Commit() }.is_ok() && wait {
      let _ = unsafe { self.gpu.composition.WaitForCommitCompletion() };
    }
  }

  fn set_cursor(&self, kind: CursorKind) {
    editor::EditorWindow::set_cursor(kind);
  }

  fn modifiers(&self) -> Modifiers {
    Modifiers {
      shift: down(VK_SHIFT.0),
      shortcut: down(VK_CONTROL.0),
    }
  }

  fn light_mode(&self) -> bool {
    crate::app_windows::overlay_surface::light_mode()
  }

  fn is_suspended(&self) -> bool {
    self.editor.is_suspended()
  }

  fn begin_typing(&self) {
    self.editor.begin_typing();
  }

  fn end_typing(&self) {
    self.editor.end_typing();
  }

  fn finish_typing(&self) {
    self.editor.finish_typing();
  }

  fn restart_blink(&self) {
    self.editor.restart_blink();
  }

  fn place_typing(&self, x: i32, y: i32) {
    self.editor.place_typing(x, y);
  }

  fn focus_webview(&self) {
    self.editor.focus_webview();
  }

  fn clipboard_text(&self) -> Option<String> {
    editor::clipboard::paste(self.editor.typing_window())
  }

  fn set_clipboard_text(&self, text: &str) {
    editor::clipboard::copy(self.editor.typing_window(), text);
  }
}
