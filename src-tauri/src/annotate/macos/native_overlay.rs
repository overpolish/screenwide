// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Rust side of the overlay's Metal surfaces.
//!
//! Native code owns pixels and events; this owns the annotations and the displays
//! they are drawn on. The native side pulls rather than being pushed: every
//! frame it asks for the display's annotations, so there is no second copy of the
//! document to keep in step with [`super::live_clips`].

use std::cell::RefCell;
use std::ffi::c_void;
use std::sync::{LazyLock, OnceLock, RwLock};

use objc2_app_kit::NSWindow;
use tauri::{AppHandle, WebviewWindow};

use crate::editor::annotations::native::{
  native_annotations, NativeAnnotations, NativeAnnotationsView,
};

/// One display the overlay covers: the capture display it is, where it
/// starts and how far it reaches in desktop points, and how many pixels one
/// point is.
#[derive(Clone, Copy)]
pub(in super::super) struct Display {
  pub id: u32,
  pub origin: (f64, f64),
  pub size: (f64, f64),
  pub scale: f64,
}

static DISPLAYS: LazyLock<RwLock<Vec<Display>>> = LazyLock::new(|| RwLock::new(Vec::new()));
/// The key callback is a bare C function pointer, so the handle a tool key
/// needs to write a setting cannot ride along with it.
static APP: OnceLock<AppHandle> = OnceLock::new();

unsafe extern "C" {
  fn screenwide_annotate_attach(view: *mut c_void, display: u32) -> u32;
  fn screenwide_annotate_detach(view: *mut c_void);
  fn screenwide_annotate_redraw();
  fn screenwide_annotate_install_scene(
    scene: extern "C" fn(u32, *mut NativeAnnotationsView),
    underlay: extern "C" fn(u32, *mut NativeUnderlay),
  );
  fn screenwide_annotate_install_input(
    pointer: extern "C" fn(u32, f64, f64),
    key: extern "C" fn(u16, u32) -> u32,
  );
  fn screenwide_annotate_teardown_input();
  fn screenwide_annotate_set_text_cursor(text: u32);
}

thread_local! {
  /// The list the last [`scene`] call handed out. The native side draws it
  /// before it asks again, so it lives exactly as long as it is read.
  static SCENE: RefCell<NativeAnnotations> = RefCell::default();
  /// The underlay the last [`underlay`] call handed out, under the same rule.
  static UNDERLAY: RefCell<Option<std::sync::Arc<super::super::highlight::Underlay>>> =
    RefCell::default();
}

/// What a display's highlights are recoloured from, as the native side reads
/// it: the pixels, their size, and a revision that changes whenever they do;
/// and while a spotlight blurs, the same softened for its blur. No pixels
/// where no highlight or blurring spotlight has been drawn on the display.
/// The twin of `ScreenwideAnnotateUnderlay`.
#[repr(C)]
pub(in super::super) struct NativeUnderlay {
  rgba: *const u8,
  width: u32,
  height: u32,
  revision: u64,
  soft_rgba: *const u8,
  soft_width: u32,
  soft_height: u32,
}

const _: () = assert!(std::mem::size_of::<NativeUnderlay>() == 40);

/// Fills one display's underlay.
///
/// # Safety
/// `out` must point at one writable [`NativeUnderlay`], whose pixels stay
/// valid until the next call on this thread.
extern "C" fn underlay(display: u32, out: *mut NativeUnderlay) {
  if out.is_null() {
    return;
  }
  UNDERLAY.with_borrow_mut(|held| {
    *held = super::super::highlight::underlay(display as usize);
    let mut view = NativeUnderlay {
      rgba: std::ptr::null(),
      width: 0,
      height: 0,
      revision: 0,
      soft_rgba: std::ptr::null(),
      soft_width: 0,
      soft_height: 0,
    };
    if let Some(underlay) = held.as_ref() {
      (view.rgba, view.width, view.height) = (
        underlay.image.rgba.as_ptr(),
        underlay.image.width,
        underlay.image.height,
      );
      view.revision = underlay.revision;
      if super::super::input::spotlight_blurs() {
        let soft = underlay.softened();
        (view.soft_rgba, view.soft_width, view.soft_height) =
          (soft.rgba.as_ptr(), soft.width, soft.height);
      }
    }
    unsafe { out.write(view) };
  });
}

/// Fills one display's annotation list: everything on screen plus the stroke in
/// hand, in that display's layer pixels.
///
/// # Safety
/// `out` must point at one writable [`NativeAnnotationsView`], which stays
/// valid until the next call on this thread.
extern "C" fn scene(display: u32, out: *mut NativeAnnotationsView) {
  if out.is_null() {
    return;
  }
  let Some(display) = DISPLAYS
    .read()
    .unwrap_or_else(|error| error.into_inner())
    .get(display as usize)
    .copied()
  else {
    return;
  };
  let drawn: Vec<_> = super::super::live_clips::annotations()
    .iter()
    .chain(super::super::input::in_progress().iter())
    .map(|annotation| {
      super::super::geometry::display_annotation(annotation, display.origin, display.scale)
    })
    .collect();
  SCENE.with_borrow_mut(|scene| {
    // Sizes are points, drawn at the display's scale.
    *scene = native_annotations(
      &drawn,
      crate::editor::annotations::redact::native::RedactSource::None,
      display.scale,
    );
    unsafe { out.write(scene.view()) };
  });
}

extern "C" fn pointer(phase: u32, x: f64, y: f64) {
  // A press on the picture is drawing, so the keyboard belongs to the canvas
  // again. It has to be said here: the monitor swallows this press, so AppKit
  // never sees it and a toolbar field holding key status would keep it.
  if phase == super::super::input::PHASE_DOWN {
    if let Some(app) = APP.get() {
      super::super::toolbar::return_keyboard(app);
    }
  }
  super::super::input::pointer(APP.get(), phase, x, y);
}

extern "C" fn key(key_code: u16, modifiers: u32) -> u32 {
  let Some(app) = APP.get() else {
    return 0;
  };
  u32::from(super::super::input::key(app, key_code, modifiers))
}

/// Names the displays the overlay is about to cover. Their order is the order
/// the host windows are created in, which is what `display` indexes.
pub(in super::super) fn set_displays(displays: Vec<Display>) {
  *DISPLAYS.write().unwrap_or_else(|error| error.into_inner()) = displays;
}

/// Gives one host window its Metal layer. Main thread only.
pub(in super::super) fn attach(window: &WebviewWindow, display: u32) -> Result<(), String> {
  let view = window.ns_view().map_err(|error| error.to_string())?;
  if unsafe { screenwide_annotate_attach(view.cast(), display) } == 0 {
    return Err("The annotate overlay could not open a Metal surface".to_owned());
  }
  Ok(())
}

/// Takes one host window's Metal layer away. Main thread only.
pub(in super::super) fn detach(window: &WebviewWindow) {
  if let Ok(view) = window.ns_view() {
    unsafe { screenwide_annotate_detach(view.cast()) };
  }
}

/// Names where the annotations come from. Outlives input: hosts left showing annotations
/// still draw from it. Main thread only.
pub(in super::super) fn install_scene() {
  unsafe { screenwide_annotate_install_scene(scene, underlay) };
}

/// Starts swallowing pointer and key events. Main thread only.
pub(in super::super) fn install_input(app: &AppHandle) {
  let _ = APP.set(app.clone());
  unsafe { screenwide_annotate_install_input(pointer, key) };
}

/// Stops swallowing input. The displays stay: annotations left on screen are still
/// drawn on them. Main thread only.
pub(in super::super) fn teardown_input() {
  unsafe { screenwide_annotate_teardown_input() };
}

/// Forgets the displays, once nothing is drawn on them any more.
pub(in super::super) fn forget_displays() {
  set_displays(Vec::new());
  super::super::highlight::forget();
}

/// The display under a point in global desktop points, and its place in the
/// overlay's order.
pub(in super::super) fn display_at(x: f64, y: f64) -> Option<(usize, Display)> {
  DISPLAYS
    .read()
    .unwrap_or_else(|error| error.into_inner())
    .iter()
    .copied()
    .enumerate()
    .find(|(_, display)| {
      (display.origin.0..display.origin.0 + display.size.0).contains(&x)
        && (display.origin.1..display.origin.1 + display.size.1).contains(&y)
    })
}

/// Redraws every display from any thread.
pub(in super::super) fn request_redraw(app: &AppHandle) {
  let _ = app.run_on_main_thread(redraw);
}

/// Whether the pointer over the canvas is the I-beam. Main thread only.
pub(in super::super) fn set_text_cursor(text: bool) {
  unsafe { screenwide_annotate_set_text_cursor(u32::from(text)) };
}

/// Redraws every attached display. Main thread only.
pub(in super::super) fn redraw() {
  unsafe { screenwide_annotate_redraw() };
}

/// Puts a peer host on screen without asking for keyboard focus: only the
/// anchor window is made key, and `makeKeyAndOrderFront:` on the others would
/// take it straight back off it. Main thread only.
pub(in super::super) fn order_front(window: &WebviewWindow) {
  if let Ok(raw) = window.ns_window() {
    let native: &NSWindow = unsafe { &*raw.cast() };
    native.orderFrontRegardless();
  }
}

/// Lets everything under a host through while its annotations stay drawn: the
/// display-only state annotations kept after exiting are shown in. Main thread only.
pub(in super::super) fn set_click_through(window: &WebviewWindow, through: bool) {
  if let Ok(raw) = window.ns_window() {
    let native: &NSWindow = unsafe { &*raw.cast() };
    native.setIgnoresMouseEvents(through);
  }
}

#[cfg(test)]
mod tests {
  unsafe extern "C" {
    fn screenwide_annotate_shader_check(message: *mut std::ffi::c_char, capacity: u32) -> u32;
  }

  #[test]
  fn the_overlays_metal_library_builds() {
    let mut message = [0 as std::ffi::c_char; 1024];
    let outcome =
      unsafe { screenwide_annotate_shader_check(message.as_mut_ptr(), message.len() as u32) };
    if outcome == 2 {
      // No Metal device: nothing to compile against, and nothing to conclude.
      return;
    }
    let reason = unsafe { std::ffi::CStr::from_ptr(message.as_ptr()) }
      .to_string_lossy()
      .into_owned();
    assert_eq!(outcome, 1, "{reason}");
  }
}
