// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What each tray menu item does.

use tauri::AppHandle;

use super::*;

pub(super) fn handle(app: &AppHandle, id: &str) {
  // The replay items leave every overlay up: a save looks back at what was
  // on screen.
  if replay::handle(app, id) {
    return;
  }
  let preserved: &[crate::capture_overlays::CaptureOverlay] = match id {
    ANNOTATE_CLEAR_MENU_ID | ANNOTATE_MENU_ID | DELAYED_SCREENSHOT_MENU_ID => {
      &[crate::capture_overlays::CaptureOverlay::Annotate]
    }
    RECOGNIZE_TEXT_MENU_ID => &[crate::capture_overlays::CaptureOverlay::TextRecognition],
    RULER_OVERLAY_MENU_ID => &[crate::capture_overlays::CaptureOverlay::Ruler],
    _ => &[],
  };
  crate::capture_overlays::dismiss_except(app, preserved);
  match id {
    ANNOTATE_MENU_ID => {
      crate::annotate::toggle_detached(app);
    }
    ANNOTATE_CLEAR_MENU_ID => {
      crate::annotate::clear(app);
    }
    DELAYED_SCREENSHOT_MENU_ID => {
      if crate::screenshots::delayed::remaining().is_some() {
        crate::screenshots::delayed::cancel(app);
      } else if let Err(error) = crate::screenshots::delayed::start(
        app,
        crate::screenshots::delayed::DelayedTarget::DisplayUnderPointer,
      ) {
        eprintln!("Could not start a delayed screenshot from the tray: {error}");
      }
    }
    DISCARD_MENU_ID | CANCEL_RECORDING_MENU_ID => report("discard", crate::recording::cancel(app)),
    OPEN_CLIPBOARD_SCREENSHOT_MENU_ID => {
      crate::screenshots::open_clipboard_in_export(app);
    }
    OPEN_MENU_ID => show_main_window(app),
    OPEN_PROJECT_MENU_ID => {
      if let Err(error) = crate::project_browser::show(app) {
        eprintln!("Could not open the projects window from the tray: {error}");
      }
    }
    PAUSE_MENU_ID | RESUME_MENU_ID => report("pause", crate::recording::toggle_pause(app)),
    QUIT_MENU_ID => app.exit(0),
    RECOGNIZE_TEXT_MENU_ID => {
      crate::text_recognition::start_detached(app);
    }
    RULER_OVERLAY_MENU_ID => {
      crate::ruler::start_detached(app);
    }
    SETTINGS_MENU_ID => {
      if let Err(error) = crate::settings::show(app) {
        eprintln!("Could not open settings from the tray: {error}");
      }
    }
    STOP_MENU_ID => report("stop", crate::recording::stop(app)),
    _ => {}
  }
}

fn report(action: &str, result: Result<(), String>) {
  if let Err(error) = result {
    eprintln!("Could not {action} the recording from the tray: {error}");
  }
}
