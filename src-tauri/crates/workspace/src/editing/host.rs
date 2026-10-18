// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the editing engine asks of the platform it runs on: the window that
//! shows the workspace and its chrome, the pointer, the keyboard and the
//! clipboard. Everything else - hit-testing, gestures, annotations, typing,
//! snapping, the view's pan and zoom - is the engine's, the same on every
//! platform.

use std::any::Any;

use screenwide_osc::gpu::magnifier::NativeMagnifier;
use screenwide_osc::gpu::vertices::{RenderConstants, Vertex};

use super::{CursorKind, Modifiers};
use crate::backdrop::BackdropStyle;
use crate::scene::{NativeWorkspacePlacement, WorkspaceScene};

pub trait Host: Any + Send + Sync {
  /// The scene every layer of the workspace is drawn through.
  fn scene(&self) -> &WorkspaceScene;

  /// Draws the scene into the drawable laid over `frame` - x, y, width and
  /// height in physical pixels from the window's content corner. Answers
  /// whether the drawable moved, which the caller commits.
  fn draw_workspace(
    &self,
    frame: (i32, i32, u32, u32),
    placements: &[NativeWorkspacePlacement],
    magnifier: Option<&NativeMagnifier>,
    backdrop: BackdropStyle,
  ) -> Result<bool, String>;

  /// Takes the workspace drawable off screen. Answers whether it was on
  /// screen, which the caller commits.
  fn hide_workspace(&self) -> bool;

  /// Lays the window's editing surface over the viewport: `frame` in
  /// physical pixels - x, y, width, height - from the window's content
  /// corner, and `rect` in logical points at `scale`. The backstop takes
  /// `backdrop`'s place and colour; `active` shows the input surface.
  fn place_viewport(
    &self,
    frame: (i32, i32, i32, i32),
    rect: screenwide_compositor::PreviewSurfaceRect,
    scale: f64,
    backdrop: [f64; 4],
    active: bool,
  );

  /// Paints the backstop `color`. Answers whether it took.
  fn paint_backdrop(&self, color: [f64; 4]) -> bool;

  /// Shows or hides the input surface, which takes the pointer over the
  /// workspace and carries the chrome.
  fn set_active(&self, active: bool);

  /// Lets go of the drawables' memory and the backstop while the editor is
  /// hidden.
  fn release(&self);

  /// Draws the selection chrome over a viewport `size` physical pixels: one
  /// draw of the shared OSC shader, every vertex sharing `constants`.
  fn present_chrome(&self, size: (u32, u32), vertices: &[Vertex], constants: &RenderConstants);

  /// Publishes a change to what is on screen that a draw does not publish by
  /// itself: a drawable moved or hidden. `wait` holds the caller until it
  /// lands, so a fast drag cannot run ahead of the screen.
  fn commit(&self, wait: bool);

  fn set_cursor(&self, kind: CursorKind);

  /// The modifiers held now.
  fn modifiers(&self) -> Modifiers;

  /// Whether the system draws its chrome light, which the magnifier's
  /// outline and the backstop follow.
  fn light_mode(&self) -> bool;

  /// Whether the frontend has the workarea covered with its own chrome, in
  /// which case the workspace answers no press.
  fn is_suspended(&self) -> bool;

  /// Hands the keyboard to the box being typed into, and starts the caret's
  /// blink.
  fn begin_typing(&self);

  /// Stops the blink and hands the keyboard back to the webview.
  fn end_typing(&self);

  /// Asks for the typing to finish once the platform's input has settled,
  /// as [`super::TypingInput::Finish`].
  fn finish_typing(&self);

  /// Starts the caret's blink over, as a change to the text does.
  fn restart_blink(&self);

  /// Moves what takes the keyboard to the caret, in physical pixels from the
  /// viewport's corner, which is where an input method opens its candidates.
  fn place_typing(&self, x: i32, y: i32);

  /// Gives the keyboard back to the webview, whose shortcuts own it whenever
  /// nothing is being typed.
  fn focus_webview(&self);

  fn clipboard_text(&self) -> Option<String>;

  fn set_clipboard_text(&self, text: &str);
}
