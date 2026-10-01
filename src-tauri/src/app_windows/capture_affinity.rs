// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which of our own windows a capture is allowed to see.

#[cfg(target_os = "windows")]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_os = "windows")]
use tauri::Manager;
use tauri::WebviewWindow;
#[cfg(target_os = "windows")]
use tauri::{AppHandle, Result};

#[cfg(target_os = "windows")]
use super::platform;
#[cfg(target_os = "windows")]
use super::WindowLabel;

/// Our own captures of the desktop, each tracked on its own: a still can be
/// taken while a recording runs, and its end must not put the recording's
/// windows back on screen.
#[cfg(target_os = "windows")]
#[derive(Clone, Copy)]
pub(crate) enum Capture {
  /// From the moment a recording prepares its windows until it has stopped.
  Recording,
  /// The pass that hides our windows for a still.
  Still,
}

#[cfg(target_os = "windows")]
static RECORDING: AtomicBool = AtomicBool::new(false);
#[cfg(target_os = "windows")]
static STILL: AtomicBool = AtomicBool::new(false);

/// Whether Screenwide is capturing the desktop right now, by any means.
#[cfg(target_os = "windows")]
pub(crate) fn is_capturing() -> bool {
  RECORDING.load(Ordering::Acquire) || STILL.load(Ordering::Acquire)
}

/// Marks one kind of our own captures starting or ending. The caller applies
/// the policy afterwards, because a capture's own pass is not the resting one.
#[cfg(target_os = "windows")]
pub(crate) fn mark_capturing(capture: Capture, capturing: bool) {
  let flag = match capture {
    Capture::Recording => &RECORDING,
    Capture::Still => &STILL,
  };
  flag.store(capturing, Ordering::Release);
}

/// Puts every window back on the resting policy: the user's preference, plus
/// the rules that do not follow it.
#[cfg(target_os = "windows")]
pub(crate) fn apply_capture_policy(app: &AppHandle) -> Result<()> {
  sync_capture_affinity(app, crate::settings::current(app).record_screenwide_windows)
}

/// Windows exclusion hides a window from every capturer, ours and a video
/// call's alike, so anything that must stay out of our own captures and
/// nowhere else can only be excluded while we are capturing.
///
/// Live annotation's hosts are exactly that, and do not follow the preference:
/// a recording receives their arrows as editable clips and a still receives
/// them as a layer, so baked pixels would draw every arrow twice. Excluding
/// them only while we capture is what keeps them visible in a screen share.
#[cfg(target_os = "windows")]
const fn window_capturable(
  record_screenwide_windows: bool,
  preserve_ruler: bool,
  annotate_host: bool,
  capturing: bool,
) -> bool {
  if annotate_host {
    return !capturing;
  }
  record_screenwide_windows || preserve_ruler
}

#[cfg(target_os = "windows")]
pub fn sync_capture_affinity(app: &AppHandle, record_screenwide_windows: bool) -> Result<()> {
  let capturing = is_capturing();
  for window in app.webview_windows().values() {
    if platform::is_visible(window)? {
      // Quick Screenshot temporarily preserves Ruler in the captured pixels.
      // Region's pre-shutter exclusion pass must not overwrite the anchor
      // host's affinity: its additional-display peers are native windows and
      // would otherwise remain capturable while only the anchor vanished.
      let preserve_ruler =
        window.label() == WindowLabel::Ruler.as_str() && crate::ruler::is_screenshot_mode();
      platform::set_capture_affinity(
        window,
        window_capturable(
          record_screenwide_windows,
          preserve_ruler,
          crate::annotate::is_host_label(window.label()),
          capturing,
        ),
      )?;
    }
  }
  Ok(())
}

/// Keeps one window out of every capture, whatever the persistent "record
/// Screenwide's windows" preference says. A recording's macOS content filter
/// is fixed when it starts, so a window opened later excludes itself.
pub(crate) fn exclude_from_capture(window: &WebviewWindow) -> tauri::Result<()> {
  #[cfg(target_os = "windows")]
  return platform::set_capture_affinity(window, false);

  #[cfg(target_os = "macos")]
  return super::platform::exclude_from_capture(window);

  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    let _ = window;
    Ok(())
  }
}

/// Overrides capture affinity for one overlay host. Native desktop peers need
/// their own matching update because they are separate top-level windows.
#[cfg(target_os = "windows")]
pub(crate) fn set_window_capture_affinity(
  window: &WebviewWindow,
  capturable: bool,
) -> tauri::Result<()> {
  platform::set_capture_affinity(window, capturable)
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
  use super::{is_capturing, mark_capturing, window_capturable, Capture};

  #[test]
  fn quick_screenshot_preserves_the_ruler_anchor_during_global_exclusion() {
    assert!(window_capturable(false, true, false, false));
    assert!(!window_capturable(false, false, false, false));
    assert!(window_capturable(true, false, false, false));
  }

  #[test]
  fn an_annotate_host_is_hidden_from_our_captures_and_from_nothing_else() {
    // Visible to a screen share while the overlay is only drawing, whatever
    // the preference says, because the recording gets the arrows as clips.
    assert!(window_capturable(false, false, true, false));
    assert!(!window_capturable(true, false, true, true));
    assert!(!window_capturable(false, false, true, true));
  }

  #[test]
  fn a_still_taken_during_a_recording_does_not_end_the_recording_capture() {
    mark_capturing(Capture::Recording, true);
    mark_capturing(Capture::Still, true);
    mark_capturing(Capture::Still, false);
    assert!(is_capturing());
    mark_capturing(Capture::Recording, false);
    assert!(!is_capturing());
  }
}
