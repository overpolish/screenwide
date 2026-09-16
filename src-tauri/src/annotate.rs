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

#[cfg(target_os = "macos")]
#[path = "annotate/cursor_macos.rs"]
mod cursor;
mod geometry;
mod host;
#[cfg(any(target_os = "macos", test))]
mod input;
pub(crate) mod live_clips;
#[cfg(target_os = "macos")]
#[path = "annotate/native_overlay_macos.rs"]
mod native_overlay;
mod session;
pub(crate) mod settings;

pub(crate) use live_clips::has_annotations;
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
    #[cfg(target_os = "macos")]
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

/// Opens the overlay. Called off the thread that services the event loop: the
/// windows, their Metal surfaces and the cursor lease are all put in place
/// through it.
pub fn start(app: &AppHandle) -> Result<(), String> {
  if !settings::enabled() {
    return Ok(());
  }

  abandon(app);
  capture_overlays::dismiss_except(app, Some(capture_overlays::CaptureOverlay::Annotate));
  let generation = app.state::<AnnotateState>().begin();
  crate::glide::suspend_for_capture(app);
  // Taken before the windows exist: the lease remembers the application the
  // user was working in, which is what it puts back on release, and it is what
  // makes the pointer a crosshair over the overlay.
  #[cfg(target_os = "macos")]
  if let Err(error) = crate::osc::cursor::macos::acquire_annotate(app) {
    abandon(app);
    return Err(error);
  }
  let result = start_hosts(app, generation);
  if result.is_err() || !is_active(app) {
    abandon(app);
  }
  result
}

fn start_hosts(app: &AppHandle, generation: u64) -> Result<(), String> {
  let plans = host::plan(app)?;
  let mut hosts = Vec::with_capacity(plans.len());
  for (index, plan) in plans.iter().enumerate() {
    hosts.push(host::build(app, index, plan)?);
  }
  // A newer session may have started while the windows were being built. It
  // owns the overlay now, and these windows are not part of it.
  if !app.state::<AnnotateState>().install(generation) {
    for window in hosts {
      let _ = window.close();
    }
    return Ok(());
  }
  host::present(app, hosts)?;
  // The pointer is a crosshair drawing annotations now, not something a viewer
  // should follow, so a running recording's cursor layer leaves it out for as
  // long as the overlay is up. A recording that starts later picks the same
  // state up from its first frame.
  crate::recording::cursor::set_cursor_visibility(false, None);
  capture_overlays::emit_lifecycle(app, true);
  crate::windows::sync_recording_ui_escape(app, crate::ruler::is_active(app));
  Ok(())
}

/// A display change invalidates every host's geometry, so the overlay is
/// rebuilt on the layout that replaced it. The annotations are kept: they are held
/// in desktop points, and one whose display went away simply stops being
/// drawn.
pub fn restart_after_topology_change(app: &AppHandle) {
  if !is_active(app) && !app.state::<AnnotateState>().is_showing() {
    return;
  }
  let app = app.clone();
  tauri::async_runtime::spawn(async move {
    // Checked again here rather than against a captured generation: the user
    // may have left in the meantime, and nothing should reopen behind them.
    let showing = app.state::<AnnotateState>().is_showing();
    if !is_active(&app) && !showing {
      return;
    }
    if let Err(error) = start(&app) {
      eprintln!("Could not rebuild the annotate overlay after a display change: {error}");
      return;
    }
    // Annotations that were only being shown go back to only being shown.
    if showing {
      stop_drawing(&app);
    }
  });
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
