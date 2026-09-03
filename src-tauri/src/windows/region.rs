// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "macos")]
use std::time::Duration;

use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize};

use super::{
  escape, platform, region_gesture, source_selector, WindowLabel, RECORDING_CONTROLS_VISIBLE,
  REGION_SELECTOR_INTERACTIVE,
};

pub(super) static SCREENSHOT_REGION_SESSION: AtomicBool = AtomicBool::new(false);
static SCREENSHOT_REGION_RESTORING: AtomicBool = AtomicBool::new(false);
static RECORDING_CONTROLS_BORROWED: AtomicBool = AtomicBool::new(false);

pub(crate) fn is_screenshot_region_session() -> bool {
  SCREENSHOT_REGION_SESSION.load(Ordering::Acquire)
}

pub(crate) fn screenshot_region_scene_owner(
  app: &AppHandle,
) -> crate::osc::scene::RegionSceneOwner {
  region_scene_owner(
    SCREENSHOT_REGION_SESSION.load(Ordering::Acquire),
    SCREENSHOT_REGION_RESTORING.load(Ordering::Acquire),
    RECORDING_CONTROLS_VISIBLE.load(Ordering::Acquire),
    crate::recording::is_idle(app),
  )
}

const fn region_scene_owner(
  screenshot_session: bool,
  restoring: bool,
  controls_visible: bool,
  recording_is_idle: bool,
) -> crate::osc::scene::RegionSceneOwner {
  if screenshot_session {
    crate::osc::scene::RegionSceneOwner::Screenshot
  } else if restoring {
    crate::osc::scene::RegionSceneOwner::RestoringNormal
  } else if !controls_visible && recording_is_idle {
    crate::osc::scene::RegionSceneOwner::DormantNormal
  } else {
    crate::osc::scene::RegionSceneOwner::Normal
  }
}

pub(crate) fn finish_screenshot_region_restore() {
  SCREENSHOT_REGION_RESTORING.store(false, Ordering::Release);
}

pub fn is_region_selector_visible(app: &AppHandle) -> bool {
  app
    .get_webview_window(WindowLabel::RegionSelector.as_str())
    .is_some_and(|region| region.is_visible().unwrap_or(false))
}

#[tauri::command]
pub fn show_region_selector(
  app: AppHandle,
  position: PhysicalPosition<i32>,
  size: PhysicalSize<u32>,
  desktop: bool,
) -> tauri::Result<()> {
  if !region_selector_may_show(
    crate::recording::is_idle(&app),
    RECORDING_CONTROLS_VISIBLE.load(Ordering::Relaxed),
    SCREENSHOT_REGION_SESSION.load(Ordering::Relaxed),
  ) {
    return Ok(());
  }

  let region = app
    .get_webview_window(WindowLabel::RegionSelector.as_str())
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  // Persisting a resize rehydrates every window's shared source store. The
  // overlay is already covering this monitor, so do not run native show/order
  // choreography again merely because its React geometry changed.
  if region.is_visible()?
    && (desktop || (region.outer_position()? == position && region.outer_size()? == size))
  {
    #[cfg(target_os = "macos")]
    if SCREENSHOT_REGION_SESSION.load(Ordering::Acquire) {
      platform::show_interactive_overlay(&region, 1.0)?;
    }
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    if desktop {
      super::screenshot_region::set_recording_overlay_desktop_presented(&region, true)?;
    }
    apply_region_selector_interactivity(&app)?;
    return raise_recording_controls(&app);
  }
  if !desktop {
    region.set_size(size)?;
    region.set_position(position)?;
  }
  let initial_opacity = f64::from(region_selector_restores_opacity(
    SCREENSHOT_REGION_SESSION.load(Ordering::Relaxed),
  ) as u8);
  #[cfg(target_os = "macos")]
  if SCREENSHOT_REGION_SESSION.load(Ordering::Acquire) {
    platform::show_interactive_overlay(&region, initial_opacity)?;
  } else {
    platform::show(&region, initial_opacity)?;
  }
  #[cfg(not(target_os = "macos"))]
  platform::show(&region, initial_opacity)?;
  platform::restore_recording_level(&region)?;
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  if desktop {
    super::screenshot_region::set_recording_overlay_desktop_presented(&region, true)?;
  }

  apply_region_selector_interactivity(&app)?;
  // Applying editor interactivity focuses its full-monitor WebView. Put the
  // recording controls above it afterwards without taking that focus, so the
  // editor remains usable everywhere the controls do not cover it.
  raise_recording_controls(&app)?;
  #[cfg(target_os = "macos")]
  tauri::async_runtime::spawn_blocking(move || {
    // AppKit completes showing a previously hidden panel asynchronously and
    // can order it above panels raised in the same run-loop turn.
    std::thread::sleep(Duration::from_millis(75));
    let ordering_app = app.clone();
    let _ = app.run_on_main_thread(move || {
      let Some(region) = ordering_app.get_webview_window(WindowLabel::RegionSelector.as_str())
      else {
        return;
      };
      let _ = platform::restore_recording_level(&region);
      let _ = raise_recording_controls(&ordering_app);
    });
  });

  Ok(())
}

const fn region_selector_restores_opacity(_screenshot_session: bool) -> bool {
  true
}

#[tauri::command]
pub fn hide_region_selector(app: AppHandle) -> tauri::Result<()> {
  if let Some(region) = app.get_webview_window(WindowLabel::RegionSelector.as_str()) {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    super::screenshot_region::set_recording_overlay_desktop_presented(&region, false)?;
    platform::hide(&region)?;
  }
  set_recording_controls_borrowed(app, false)
}

fn raise_recording_controls(app: &AppHandle) -> tauri::Result<()> {
  if !recording_controls_may_raise(
    RECORDING_CONTROLS_VISIBLE.load(Ordering::Relaxed),
    region_gesture::is_active(),
    RECORDING_CONTROLS_BORROWED.load(Ordering::Relaxed),
  ) {
    return Ok(());
  }

  if let Some(bar) = app.get_webview_window(WindowLabel::RecordingBar.as_str()) {
    platform::raise_without_activation(&bar)?;
  }
  if source_selector::is_expanded() {
    if let Some(selector) = app.get_webview_window(WindowLabel::RecordingSourceSelector.as_str()) {
      platform::raise_without_activation(&selector)?;
    }
  }
  Ok(())
}

/// Cross-window persistence may ask the overlay to show while a region gesture
/// owns the pointer. The recording controls stay down until that gesture ends.
const fn recording_controls_may_raise(
  controls_visible: bool,
  gesture_active: bool,
  controls_borrowed: bool,
) -> bool {
  controls_visible && !gesture_active && !controls_borrowed
}

/// The region overlay may take clicks only while its frontend has made the
/// editor available, and only outside a recording. During a recording the same
/// window remains visible as a boundary but must let desktop clicks through.
const fn region_selector_is_interactive(is_interactive: bool, is_recording_idle: bool) -> bool {
  is_interactive && is_recording_idle
}

/// The region boundary belongs to the recording UI while idle. Delayed
/// frontend synchronization must not restore it after those controls hide,
/// while an active recording may keep its existing boundary visible.
/// A screenshot session is the exception: the screenshot shortcut draws the
/// overlay on its own and leaves the recording controls in place.
const fn region_selector_may_show(
  is_recording_idle: bool,
  controls_visible: bool,
  screenshot_session: bool,
) -> bool {
  !is_recording_idle || controls_visible || screenshot_session
}

pub(super) const fn recording_ui_may_hide(screenshot_session: bool) -> bool {
  !screenshot_session
}

const fn screenshot_region_may_restore(requested: bool, controls_visible: bool) -> bool {
  requested && controls_visible
}

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
    super::screenshot_region::acquire_quick_screenshot_cursor(&app)
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
        super::screenshot_region::prepare_recording_overlay_for_region_restore(&region)
      } else {
        super::screenshot_region::prepare_recording_overlay_for_screenshot(&region)
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
    if let Err(error) = super::screenshot_region::release_quick_screenshot_cursor(&app) {
      return Err(std::io::Error::other(error).into());
    }
  }
  if active {
    SCREENSHOT_REGION_RESTORING.store(false, Ordering::Release);
    SCREENSHOT_REGION_SESSION.store(true, Ordering::Release);
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
        let peers = super::screenshot_region::prepare_recording_overlay_for_screenshot(&region);
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
        let _ = super::screenshot_region::release_quick_screenshot_cursor(&app);
      }
      escape::sync(&app, controls_visible, false, crate::ruler::is_active(&app));
      return Err(error);
    }
  }
  Ok(restoring_region)
}

/// Re-asserts that invariant against the window.
///
/// This has to be called by everything that shows the overlay, because
/// `platform::show` turns cursor events back on every time it runs. Leaving it
/// to the caller to remember is what made the desktop stop accepting clicks
/// after a re-show.
fn apply_region_selector_interactivity(app: &AppHandle) -> tauri::Result<()> {
  let Some(region) = app.get_webview_window(WindowLabel::RegionSelector.as_str()) else {
    return Ok(());
  };
  let is_interactive = region_selector_is_interactive(
    REGION_SELECTOR_INTERACTIVE.load(Ordering::Relaxed),
    crate::recording::is_idle(app),
  );

  region.set_ignore_cursor_events(!is_interactive)?;
  if !is_interactive {
    // Going passthrough while still holding key status would leave the user
    // typing into an invisible overlay instead of the app they are recording,
    // so the overlay gives keyboard focus back as it stops taking clicks.
    #[cfg(target_os = "macos")]
    platform::release_key_focus(&region)?;
    raise_recording_controls(app)?;
  }

  Ok(())
}

#[tauri::command]
pub fn set_region_selector_passthrough(app: AppHandle, passthrough: bool) -> tauri::Result<()> {
  REGION_SELECTOR_INTERACTIVE.store(!passthrough, Ordering::Relaxed);
  apply_region_selector_interactivity(&app)
}

#[tauri::command]
#[allow(clippy::needless_return)]
pub async fn set_region_selector_opacity(
  window: tauri::WebviewWindow,
  opacity: f64,
) -> Result<(), String> {
  #[cfg(target_os = "windows")]
  {
    let capturable = region_selector_capturable_after_opacity(
      opacity,
      crate::settings::current(window.app_handle()).record_screenwide_windows,
    );
    super::sync_capture_affinity(window.app_handle(), capturable)
      .map_err(|error| error.to_string())?;
    super::screenshot_region::set_recording_overlay_capture_affinity(&window, capturable)
      .map_err(|error| error.to_string())?;
    if opacity <= 0.0 {
      // Display affinity is committed asynchronously by DWM. Let that frame
      // land before the shutter without changing anything visible onscreen.
      tokio::time::sleep(std::time::Duration::from_millis(75)).await;
    }
    return Ok(());
  }

  #[cfg(target_os = "macos")]
  if opacity <= 0.0 {
    super::screenshot_region::set_recording_overlay_desktop_presented(&window, false)
      .map_err(|error| error.to_string())?;
  }
  #[cfg(not(target_os = "windows"))]
  return platform::set_opacity(&window, opacity).map_err(|error| error.to_string());
}

const fn region_selector_capturable_after_opacity(
  opacity: f64,
  record_screenwide_windows: bool,
) -> bool {
  opacity > 0.0 && record_screenwide_windows
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
    super::hide_recording_options(app.clone())?;
    super::options::hide_standalone_listbox(app.clone(), Some(false))?;
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

const fn recording_controls_may_restore(controls_visible: bool, recording_idle: bool) -> bool {
  controls_visible && recording_idle
}

#[cfg(test)]
mod tests;
