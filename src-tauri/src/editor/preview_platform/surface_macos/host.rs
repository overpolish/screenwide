// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the shared editing engine asks of macOS: the Metal layers, the input
//! view's cursor and keyboard, and the pasteboard. The twin of
//! `surface_windows/host.rs`.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{
  NSAppearanceNameAqua, NSAppearanceNameDarkAqua, NSApplication, NSEvent, NSEventModifierFlags,
  NSResponder, NSView,
};
use objc2_foundation::{NSArray, NSPoint};
use objc2_quartz_core::CATransaction;

use screenwide_compositor::PreviewSurfaceRect;
use screenwide_osc::gpu::magnifier::NativeMagnifier;
use screenwide_osc::gpu::vertices::{RenderConstants, Vertex};
use screenwide_workspace::backdrop::BackdropStyle;
use screenwide_workspace::editing::{ChromeRenderer, CursorKind, Host, Modifiers, SurfaceInner};
use screenwide_workspace::scene::{NativeWorkspacePlacement, WorkspaceScene};

use super::audio_ribbon::AudioRibbon;
use super::layers::{place, Layers};
use super::main_thread::OnMain;
use super::view::EditorView;
use super::workspace_view::WorkspaceView;
use super::{cursor, pasteboard, ImageDropCallback};

/// The views, which only the main thread may touch.
pub(super) struct Views {
  /// The window's content view, which holds the webview.
  pub(super) host: Retained<NSView>,
  /// Over the webview's workarea, taking the pointer.
  pub(super) editor: Retained<EditorView>,
}

pub(super) struct Gpu {
  pub(super) layers: Layers,
  pub(super) workspace: WorkspaceView,
  pub(super) selection: Mutex<ChromeRenderer>,
  pub(super) audio_ribbon: Mutex<AudioRibbon>,
}

pub(super) struct MacHost {
  pub(super) views: OnMain<Views>,
  pub(super) gpu: Gpu,
  pub(super) webview: tauri::WebviewWindow,
  /// Pixels to the point, as the last viewport was laid out at.
  scale: AtomicU64,
  /// Set while the frontend covers the workarea with its own chrome: the
  /// input view stays hidden whatever the editor's active state does.
  suspended: AtomicBool,
  active: AtomicBool,
  /// Whether the system draws its chrome light, read on the main thread.
  light: Arc<AtomicBool>,
  /// The callback a picture dropped on the workspace is handed to.
  pub(super) image_drop: Mutex<Option<ImageDropCallback>>,
}

/// The macOS side of an engine this shell made.
pub(super) fn mac(inner: &SurfaceInner) -> &MacHost {
  let host: &dyn std::any::Any = inner.host.as_ref();
  host
    .downcast_ref()
    .expect("every macOS editor surface is hosted by MacHost")
}

/// Whether the application draws in the light appearance.
pub(super) fn light_appearance(mtm: MainThreadMarker) -> bool {
  let appearance = NSApplication::sharedApplication(mtm).effectiveAppearance();
  // Safety: the names are AppKit's own constants.
  let names = unsafe { NSArray::from_slice(&[NSAppearanceNameAqua, NSAppearanceNameDarkAqua]) };
  appearance
    .bestMatchFromAppearancesWithNames(&names)
    .is_some_and(|name| &*name == unsafe { NSAppearanceNameAqua })
}

impl MacHost {
  pub(super) fn new(
    views: OnMain<Views>,
    gpu: Gpu,
    webview: tauri::WebviewWindow,
    scale: f64,
    light: bool,
  ) -> Self {
    Self {
      views,
      gpu,
      webview,
      scale: AtomicU64::new(scale.to_bits()),
      suspended: AtomicBool::new(false),
      active: AtomicBool::new(false),
      light: Arc::new(AtomicBool::new(light)),
      image_drop: Mutex::new(None),
    }
  }

  fn scale(&self) -> f64 {
    f64::from_bits(self.scale.load(Ordering::Relaxed)).max(0.1)
  }

  /// Shows the input view when the editor is active and not suspended.
  fn sync_input_view(&self) {
    let shown = self.active.load(Ordering::Relaxed) && !self.suspended.load(Ordering::Relaxed);
    self
      .views
      .later(move |views, _| views.editor.setHidden(!shown));
  }

  pub(super) fn set_suspended(&self, suspended: bool, active: bool) {
    self.suspended.store(suspended, Ordering::Relaxed);
    self.active.store(active, Ordering::Relaxed);
    self.sync_input_view();
  }
}

impl Host for MacHost {
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
      .draw(frame, self.scale(), placements, magnifier, backdrop)
  }

  fn hide_workspace(&self) -> bool {
    self.gpu.workspace.hide()
  }

  fn place_viewport(
    &self,
    _frame: (i32, i32, i32, i32),
    rect: PreviewSurfaceRect,
    scale: f64,
    backdrop: [f64; 4],
    active: bool,
  ) {
    if self.scale.swap(scale.to_bits(), Ordering::Relaxed) != scale.to_bits() {
      self.gpu.layers.set_scale(scale);
    }
    self.active.store(active, Ordering::Relaxed);
    let shown = rect.width >= 1.0 && rect.height >= 1.0;
    place(
      &self.gpu.layers.backdrop,
      shown.then_some((rect.x, rect.y, rect.width, rect.height)),
    );
    place(
      &self.gpu.layers.chrome,
      Some((0.0, 0.0, rect.width, rect.height)),
    );
    if let Ok(mut ribbon) = self.gpu.audio_ribbon.lock() {
      let _ = ribbon.set_viewport(rect, scale, backdrop);
    }
    let visible = active && !self.suspended.load(Ordering::Relaxed);
    let light = Arc::clone(&self.light);
    self.views.later(move |views, mtm| {
      let bounds = views.host.bounds();
      let y = if views.host.isFlipped() {
        rect.y
      } else {
        bounds.size.height - rect.y - rect.height
      };
      views
        .editor
        .setFrame(super::layers::rect(rect.x, y, rect.width, rect.height));
      views.editor.setHidden(!visible);
      light.store(light_appearance(mtm), Ordering::Relaxed);
    });
  }

  fn paint_backdrop(&self, color: [f64; 4]) -> bool {
    self.gpu.layers.paint_backdrop(color);
    true
  }

  fn set_active(&self, active: bool) {
    self.active.store(active, Ordering::Relaxed);
    self.sync_input_view();
  }

  fn release(&self) {
    if let Ok(mut selection) = self.gpu.selection.lock() {
      selection.release_drawables();
    }
    place(&self.gpu.layers.backdrop, None);
  }

  fn present_chrome(&self, size: (u32, u32), vertices: &[Vertex], constants: &RenderConstants) {
    if let Ok(mut selection) = self.gpu.selection.lock() {
      let _ = selection.present(size, vertices, constants);
    }
  }

  fn commit(&self, _wait: bool) {
    // Every move is its own committed transaction; flushing pushes them to
    // the render server now rather than at the run loop's next turn.
    CATransaction::flush();
  }

  fn set_cursor(&self, kind: CursorKind) {
    match MainThreadMarker::new() {
      Some(mtm) => cursor::set(kind, mtm),
      None => self.views.later(move |_, mtm| cursor::set(kind, mtm)),
    }
  }

  fn modifiers(&self) -> Modifiers {
    let flags = NSEvent::modifierFlags_class();
    Modifiers {
      shift: flags.contains(NSEventModifierFlags::Shift),
      shortcut: flags.contains(NSEventModifierFlags::Command),
    }
  }

  fn light_mode(&self) -> bool {
    self.light.load(Ordering::Relaxed)
  }

  fn is_suspended(&self) -> bool {
    self.suspended.load(Ordering::Relaxed)
  }

  fn begin_typing(&self) {
    self.views.later(|views, _| views.editor.begin_typing());
  }

  fn end_typing(&self) {
    self.views.later(|views, _| views.editor.end_typing());
  }

  fn finish_typing(&self) {
    self.views.later(|views, _| views.editor.finish_typing());
  }

  fn restart_blink(&self) {
    self.views.later(|views, _| views.editor.restart_blink());
  }

  fn place_typing(&self, x: i32, y: i32) {
    let scale = self.scale();
    let point = NSPoint::new(f64::from(x) / scale, f64::from(y) / scale);
    self
      .views
      .later(move |views, _| views.editor.place_caret(point));
  }

  fn focus_webview(&self) {
    let _ = self.webview.set_focus();
    // The typing made the input view the window's first responder. Focusing
    // the window leaves it there, so the webview's shortcuts would never see
    // a key again until something in the page was clicked.
    self.views.later(|views, _| {
      let (Some(window), Some(page)) = (
        views.host.window(),
        crate::app_windows::platform::webview_view(&views.host),
      ) else {
        return;
      };
      let page: &NSResponder = &page;
      window.makeFirstResponder(Some(page));
    });
  }

  fn clipboard_text(&self) -> Option<String> {
    pasteboard::paste()
  }

  fn set_clipboard_text(&self, text: &str) {
    pasteboard::copy(text);
  }
}
