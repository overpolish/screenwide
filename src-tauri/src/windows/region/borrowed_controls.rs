// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[tauri::command]
pub fn set_screenshot_region_session(
  app: AppHandle,
  active: bool,
  restore_region: Option<bool>,
) -> tauri::Result<bool> {
  let controls_visible = RECORDING_CONTROLS_VISIBLE.load(Ordering::Relaxed);
  let session_was_active = SCREENSHOT_REGION_SESSION.load(Ordering::Acquire);
  let mut restoring_region = false;
  if active && !session_was_active {
    // First, so the overlay's own cursor lease is gone before the
    // screenshot's is taken, and Escape is no longer the overlay's when
    // it is re-armed below.
    crate::annotate::set_screenshot_mode(&app, true);
    super::super::screenshot_region::acquire_quick_screenshot_cursor(&app)
      .map_err(std::io::Error::other)?;
  }
  if !active && session_was_active {
    let restore_region =
      screenshot_region_may_restore(restore_region.unwrap_or(false), controls_visible);
    restoring_region = restore_region;
    SCREENSHOT_REGION_SESSION.store(false, Ordering::Release);
    SCREENSHOT_REGION_RESTORING.store(restore_region, Ordering::Release);
    if let Some(region) = app.get_webview_window(WindowLabel::RegionSelector.as_str()) {
      #[cfg(any(target_os = "macos", target_os = "windows"))]
      let transition = if restore_region {
        super::super::screenshot_region::prepare_recording_overlay_for_region_restore(&region)
      } else {
        super::super::screenshot_region::prepare_recording_overlay_for_screenshot(&region)
      };
      #[cfg(not(any(target_os = "macos", target_os = "windows")))]
      let transition: tauri::Result<()> = Ok(());
      if let Err(error) = transition {
        SCREENSHOT_REGION_SESSION.store(true, Ordering::Release);
        SCREENSHOT_REGION_RESTORING.store(false, Ordering::Release);
        return Err(error);
      }
      #[cfg(target_os = "macos")]
      platform::restore_nonactivating_overlay(&region)?;
    }
    if let Err(error) = super::super::screenshot_region::release_quick_screenshot_cursor(&app) {
      return Err(std::io::Error::other(error).into());
    }
  }
  if active {
    SCREENSHOT_REGION_RESTORING.store(false, Ordering::Release);
    SCREENSHOT_REGION_SESSION.store(true, Ordering::Release);
  }
  if !active && session_was_active {
    crate::annotate::set_screenshot_mode(&app, false);
  }
  escape::sync(
    &app,
    controls_visible,
    active,
    crate::ruler::is_active(&app),
  );

  if active {
    let result = match app.get_webview_window(WindowLabel::RegionSelector.as_str()) {
      Some(region) => {
        #[cfg(any(target_os = "macos", target_os = "windows"))]
        let peers =
          super::super::screenshot_region::prepare_recording_overlay_for_screenshot(&region);
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let peers = Ok(());
        peers
          .and_then(|()| platform::hide(&region))
          .and_then(|()| platform::set_opacity(&region, 0.0))
      }
      None => Ok(()),
    };
    if let Err(error) = result {
      SCREENSHOT_REGION_SESSION.store(false, Ordering::Release);
      SCREENSHOT_REGION_RESTORING.store(false, Ordering::Release);
      if !session_was_active {
        let _ = super::super::screenshot_region::release_quick_screenshot_cursor(&app);
      }
      escape::sync(&app, controls_visible, false, crate::ruler::is_active(&app));
      return Err(error);
    }
  }
  Ok(restoring_region)
}

/// Temporarily removes the recording-control window graph while Quick
/// Screenshot borrows the shared region overlay.
///
/// This deliberately leaves `RECORDING_CONTROLS_VISIBLE` unchanged: borrowing
/// is presentation state, not a request to close the recording UI. Returning
/// the controls therefore restores the bar only when it is still logically
/// visible and the app is idle. The region-selector window is not touched
/// because it is the driver for the screenshot session.
#[tauri::command]
pub fn set_recording_controls_borrowed(app: AppHandle, borrowed: bool) -> tauri::Result<()> {
  RECORDING_CONTROLS_BORROWED.store(borrowed, Ordering::Release);

  if borrowed {
    // The capture flow puts the bar's own menus away with the bar. A tool
    // panel belongs to an editor, not to the bar, and stays with its editor:
    // a capture that lands in that editor must find the panel still there.
    super::super::options::close_standalone_listbox(
      app.clone(),
      false,
      WindowLabel::StandaloneListbox.as_str(),
    )?;
    source_selector::hide(&app)?;
    if let Some(bar) = app.get_webview_window(WindowLabel::RecordingBar.as_str()) {
      platform::hide(&bar)?;
    }
    return Ok(());
  }

  if recording_controls_may_restore(
    RECORDING_CONTROLS_VISIBLE.load(Ordering::Relaxed),
    crate::recording::is_idle(&app),
  ) {
    if let Some(bar) = app.get_webview_window(WindowLabel::RecordingBar.as_str()) {
      platform::show(&bar, 1.0)?;
      platform::restore_recording_level(&bar)?;
    }
  }
  Ok(())
}

pub(super) const fn recording_controls_may_restore(
  controls_visible: bool,
  recording_idle: bool,
) -> bool {
  controls_visible && recording_idle
}

pub(super) const fn screenshot_region_may_restore(requested: bool, controls_visible: bool) -> bool {
  requested && controls_visible
}
