// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! A reference on Windows: the picture's and the chrome's visuals in a
//! topmost DirectComposition target on its window, over the webview, which
//! the engine draws through the shared OSC renderer, and a child window with
//! no content of its own over the picture and its handles' reach, taking the
//! pointer there. The pad keeps clear of the island, whose clicks must reach
//! the webview: a child window cannot pass them on to it, since WebView2
//! takes input in another process. Desktop lengths are in points at the
//! scale of the display the reference stands on (`desktop`).

mod compose;
pub(super) mod desktop;
mod host;
mod window_proc;

use std::cell::Cell;
use std::rc::Rc;

use screenwide_osc::geometry::{Point, Size};
use tauri::{Manager, Position, WebviewWindow};
use windows::Win32::{
  Foundation::{HWND, POINT, RECT},
  UI::WindowsAndMessaging::DestroyWindow,
};

use super::engine::Cursor;
use crate::app_windows::overlay_surface;
use compose::Composition;

/// What the reference's host and its input pad's procedure both keep.
struct Shared {
  label: String,
  /// Pixels to the point the reference measures and draws at: its display's,
  /// held through a press so a gesture keeps one measure.
  scale: Cell<f64>,
  cursor: Cell<Cursor>,
  /// A press under way, which holds the pointer until it ends.
  pressed: Cell<bool>,
  /// Whether the pad is watching for the pointer to leave it.
  tracking: Cell<bool>,
  /// The pad's corner in the view the engine measures in, in pixels.
  pad: Cell<POINT>,
  /// The window came onto a display of another scale during a press, which
  /// is taken up as the press ends.
  rescale_pending: Cell<bool>,
}

/// Where the engine last placed the window, its view and the input pad,
/// shown together once the view is drawn (`commit_placement`).
#[derive(Clone, Copy, Default)]
struct Placed {
  /// The window's corner on the desktop, in points, before it is rounded
  /// to a pixel.
  origin: Point,
  /// The window on the desktop, in pixels.
  window: RECT,
  /// The view's corner in the window, in pixels.
  view: POINT,
  /// The input pad in the window, in pixels.
  pad: RECT,
}

pub(crate) struct WindowsHost {
  /// The reference's own window.
  top: HWND,
  /// The input pad over the picture.
  pad: HWND,
  placed: Cell<Placed>,
  composition: Composition,
  shared: Rc<Shared>,
}

impl WindowsHost {
  /// Takes up the scale of the display the window now stands on.
  pub(super) fn refresh_scale(&self) {
    self.shared.scale.set(desktop::window_scale(self.top));
  }
}

/// Gives the reference's `window` its view, showing `texture` at `size` with
/// its corner at `origin`, or centred on the primary display. Runs on the
/// thread that owns the window.
pub(super) fn attach(
  window: &WebviewWindow,
  texture: &wgpu::Texture,
  size: Size,
  origin: Option<Position>,
) -> Result<(), String> {
  let gpu = screenwide_gpu::shared()?;
  let top = HWND(window.hwnd().map_err(|error| error.to_string())?.0);
  window_proc::register();
  let pad = overlay_surface::create_child(top, window_proc::CLASS)?;
  // The window is still where Windows first put it; the reference measures
  // at the scale of the display it opens on.
  let scale = match origin {
    Some(Position::Physical(at)) => desktop::scale_at(POINT { x: at.x, y: at.y }),
    Some(Position::Logical(_)) => desktop::window_scale(top),
    None => desktop::scale_at(POINT::default()),
  };
  let shared = Rc::new(Shared {
    label: window.label().to_owned(),
    scale: Cell::new(scale),
    cursor: Cell::new(Cursor::Arrow),
    pressed: Cell::new(false),
    tracking: Cell::new(false),
    pad: Cell::new(POINT::default()),
    rescale_pending: Cell::new(false),
  });
  window_proc::remember(pad, Rc::clone(&shared));
  let (composition, surfaces) = match Composition::new(&gpu, top) {
    Ok(made) => made,
    Err(error) => {
      window_proc::forget(pad);
      let _ = unsafe { DestroyWindow(pad) };
      return Err(error);
    }
  };
  let host = WindowsHost {
    top,
    pad,
    placed: Cell::new(Placed::default()),
    composition,
    shared,
  };
  super::engine::attach(
    window.app_handle(),
    window.label(),
    host,
    surfaces,
    texture,
    size,
    origin,
  )
}
