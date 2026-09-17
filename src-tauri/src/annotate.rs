// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Live annotation: annotations drawn straight onto the desktop.
//!
//! The overlay is never captured, so what a recording keeps is timing rather
//! than pixels: every stroke becomes an ordinary editable annotation clip that
//! the editor can move, retime and delete. Outside a recording the overlay is
//! purely visual.
//!
//! The host windows are transparent surfaces the annotations are drawn on, one per
//! display. They own pointer and keyboard input for as long as the overlay is
//! up: there is no pass-through mode, and the shortcut or Escape is the way
//! out.

pub(crate) mod commands;
#[cfg(target_os = "macos")]
#[path = "annotate/cursor_macos.rs"]
mod cursor;
mod geometry;
mod host;
#[cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]
mod input;
pub(crate) mod live_clips;
#[cfg(target_os = "macos")]
#[path = "annotate/native_overlay_macos.rs"]
mod native_overlay;
#[cfg(target_os = "windows")]
#[path = "annotate/native_overlay_windows.rs"]
mod native_overlay;
mod opening;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod screenshot;
mod screenshot_mode;
mod session;
pub(crate) mod settings;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod toolbar;

#[cfg(target_os = "windows")]
pub(crate) use host::is_host_label;
pub(crate) use live_clips::has_annotations;
pub use opening::{restart_after_topology_change, start};
pub use session::AnnotateState;

use tauri::{AppHandle, Manager};

use crate::capture_overlays;

pub fn is_active(app: &AppHandle) -> bool {
  app.state::<AnnotateState>().active_generation().is_some()
}

/// Gives the foreground, the pointer and Escape back. Shared by every way the
/// overlay stops drawing, whether its annotations stay on screen or not.
fn release_input_ownership(app: &AppHandle) {
  // Releasing the lease is what hands the foreground back to the application
  // the user was in when the overlay opened.
  #[cfg(target_os = "macos")]
  if let Err(error) = crate::osc::cursor::macos::release_annotate(app) {
    eprintln!("Could not release the Annotate cursor ownership: {error}");
  }
  // The pointer belongs to the user again. Restored on every path out,
  // including a failed start: a recording must never be left with a cursor
  // layer that is hidden for good.
  crate::recording::cursor::set_cursor_visibility(true, None);
  // Escape is released here even though every way out of the overlay runs
  // inside a native shortcut callback: `escape::sync` is what knows to wait
  // for a later turn before touching the shortcut registry.
  crate::windows::sync_recording_ui_escape(app, crate::ruler::is_active(app));
}

/// Takes the overlay's surfaces down and ends the session, leaving the annotations
/// alone. A rebuild and a failed start both go through here: a display change
/// must not cost the user what they have drawn.
fn abandon(app: &AppHandle) {
  let closed = host::close(app);
  let ended = app.state::<AnnotateState>().cancel();
  release_input_ownership(app);
  if ended || closed {
    capture_overlays::emit_lifecycle(app, false);
  }
}

/// Stops drawing and leaves the annotations on screen, drawn by hosts that take no
/// input. The way out of these is [`clear`].
fn stop_drawing(app: &AppHandle) {
  let handed_over = host::show_only(app);
  let ended = app.state::<AnnotateState>().show();
  release_input_ownership(app);
  if ended || handed_over {
    capture_overlays::emit_lifecycle(app, false);
  }
  crate::tray::refresh(app);
}

/// Takes everything down: the annotations, the surfaces showing them, and the
/// session. Losing the annotations is what closes their clips in a running
/// recording.
fn take_down(app: &AppHandle) {
  screenshot_mode::forget_resume();
  live_clips::clear();
  abandon(app);
  crate::tray::refresh(app);
}

/// Leaves the overlay. Annotations asked to stay are left on screen and keep their
/// clips open in a running recording, because they really are still visible.
pub fn dismiss(app: &AppHandle) {
  if settings::keep_annotations_between_sessions() && live_clips::has_annotations() {
    stop_drawing(app);
  } else {
    take_down(app);
  }
}

/// Takes every annotation off the screen, and any surface showing them with it.
/// Drawing carries on if that is what the overlay was doing.
pub fn clear(app: &AppHandle) {
  if is_active(app) {
    live_clips::clear();
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    native_overlay::redraw();
  } else {
    take_down(app);
  }
}

/// Switching the feature off leaves nothing behind, whatever the keep setting
/// says: a annotation cannot outlive the tool that draws it.
pub(crate) fn disable(app: &AppHandle) {
  take_down(app);
}

/// Quick Screenshot's handoff: while its region overlay is up the annotations
/// stay visible under it and take no input, and drawing resumes afterwards
/// if that is what the overlay was doing.
pub(crate) fn set_screenshot_mode(app: &AppHandle, active: bool) {
  screenshot_mode::set(app, active);
}

/// The annotations on screen that a still just taken covers, in its pixels.
/// The still itself never holds them: the overlay is kept out of every
/// capture, so they arrive as an editable layer instead.
pub(crate) fn screenshot_annotations(
  target: crate::screenshots::ScreenshotTarget,
  image: &crate::screenshots::CapturedImage,
) -> Vec<crate::editor::annotations::Annotation> {
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  {
    screenshot::annotations_for(target, image)
  }
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    let _ = (target, image);
    Vec::new()
  }
}

/// The annotations a still covered, drawn into its pixels. The clipboard has
/// no layers, so a shot that is copied rather than opened carries them there
/// instead of alongside.
#[cfg(target_os = "windows")]
pub(crate) fn bake_annotations(
  image: &crate::screenshots::CapturedImage,
  annotations: &[crate::editor::annotations::Annotation],
) -> Result<crate::screenshots::CapturedImage, String> {
  native_overlay::bake(image, annotations)
}

/// The shortcut's and the tray item's way in: the same press that opens the
/// overlay closes it again.
pub fn toggle_detached(app: &AppHandle) {
  if is_active(app) {
    dismiss(app);
    return;
  }
  let app = app.clone();
  // Window creation runs off the thread that services the event loop, the way
  // every other capture overlay opens.
  tauri::async_runtime::spawn(async move {
    if let Err(error) = start(&app) {
      eprintln!("Could not start the annotate overlay: {error}");
    }
  });
}
