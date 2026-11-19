// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! The cue's window, made as a cue is first shown, and the height it is
//! given by hand, kept.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Emitter, Manager, PhysicalSize, WebviewUrl, WindowEvent};

use super::{place, shown, BESIDE_EVENT, LABEL, TELEPROMPTER};

/// Tells the teleprompter the cue's window was resized by hand, to the
/// height in points it should keep.
const HEIGHT_EVENT: &str = "teleprompter://cue-height";
/// How far a size may differ from the one set and still be taken as the
/// size set, in pixels, for the rounding a display's scale brings.
const SIZE_SLACK: u32 = 2;

/// Set while the window is being made, so a cue shown meanwhile does not
/// make a second.
static BUILDING: AtomicBool = AtomicBool::new(false);

/// Makes the window on a thread of its own, then places the cue in it and
/// tells the teleprompter whether it stands beside. A cue is shown from a
/// synchronous command, on the main thread, where making a webview
/// deadlocks on Windows. A cue shown while the window is made is placed
/// with it, as `place` reads the latest.
pub(super) fn build_then_place(app: &AppHandle) {
  if BUILDING.swap(true, Ordering::AcqRel) {
    return;
  }
  let app = app.clone();
  std::thread::spawn(move || {
    let built = build(&app);
    BUILDING.store(false, Ordering::Release);
    match built.and_then(|()| place(&app)) {
      Ok(beside) => {
        let _ = app.emit_to(TELEPROMPTER, BESIDE_EVENT, beside);
      }
      Err(error) => eprintln!("Could not show the cue: {error}"),
    }
  });
}

fn build(app: &AppHandle) -> Result<(), String> {
  let window =
    crate::app_windows::webview_window(app, LABEL, WebviewUrl::App("/teleprompter-cue".into()))
      .title(screenwide_i18n::t!("window-titles-teleprompter-cue"))
      .accept_first_mouse(true)
      .always_on_top(true)
      .decorations(false)
      // Shown as the reader reaches a word, so it must not take the keys
      // from the script being written or the app being recorded.
      .focusable(false)
      .focused(false)
      .resizable(true)
      .shadow(false)
      .skip_taskbar(true)
      .transparent(true)
      .visible(false)
      .visible_on_all_workspaces(true)
      .build()
      .map_err(|error| error.to_string())?;
  crate::app_windows::set_window_capture_affinity(&window, false)
    .map_err(|error| error.to_string())?;
  #[cfg(target_os = "macos")]
  round_corners(&window)?;
  let handle = app.clone();
  window.on_window_event(move |event| {
    if let WindowEvent::Resized(size) = event {
      resized_by_hand(&handle, *size);
    }
  });
  Ok(())
}

/// The picture's rounded corner (`rounded-island` in `CueCard`), in points.
#[cfg(target_os = "macos")]
const CORNER_RADIUS: f64 = 24.0;

/// Rounds the window itself as the page rounds the picture. Between two
/// cues the window changes shape a step at a time, and WebKit can show a
/// step drawn at the size before, which the window's square edge would cut;
/// rounded here, the window clips it to the same corner at its own size.
#[cfg(target_os = "macos")]
fn round_corners(window: &tauri::WebviewWindow) -> Result<(), String> {
  let target = window.clone();
  window
    .run_on_main_thread(move || {
      let Ok(raw) = target.ns_window() else {
        return;
      };
      let native: &objc2_app_kit::NSWindow = unsafe { &*raw.cast() };
      let Some(content) = native.contentView() else {
        return;
      };
      content.setWantsLayer(true);
      if let Some(layer) = content.layer() {
        layer.setCornerRadius(CORNER_RADIUS);
        layer.setMasksToBounds(true);
      }
    })
    .map_err(|error| error.to_string())
}

/// A size the window was not given was set by hand: its height is kept as
/// the tallest the cue grows to, and the teleprompter keeps it for later.
/// The sizes it passes through between two cues are its own.
fn resized_by_hand(app: &AppHandle, size: PhysicalSize<u32>) {
  if super::reshape::is_changing() {
    return;
  }
  let scale = app
    .get_webview_window(LABEL)
    .and_then(|window| window.scale_factor().ok())
    .unwrap_or(1.0);
  let height = {
    let mut state = shown();
    let Some(state) = state.as_mut() else {
      return;
    };
    let given = state.placed.is_some_and(|placed| {
      placed.width.abs_diff(size.width) <= SIZE_SLACK
        && placed.height.abs_diff(size.height) <= SIZE_SLACK
    });
    if given || size.height == 0 {
      return;
    }
    state.placed = Some(size);
    let height = f64::from(size.height) / scale;
    state.height = Some(height);
    height
  };
  let _ = app.emit_to(TELEPROMPTER, HEIGHT_EVENT, height);
}
