// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[path = "region/borrowed_controls.rs"]
mod borrowed_controls;
pub use borrowed_controls::set_recording_controls_borrowed;
pub use borrowed_controls::set_screenshot_region_session;
pub use borrowed_controls::{
  __cmd__set_recording_controls_borrowed, __tauri_command_name_set_recording_controls_borrowed,
};
pub use borrowed_controls::{
  __cmd__set_screenshot_region_session, __tauri_command_name_set_screenshot_region_session,
};
#[path = "region/interactivity.rs"]
mod interactivity;
use interactivity::apply_region_selector_interactivity;
pub use interactivity::set_region_selector_passthrough;
pub use interactivity::{
  __cmd__set_region_selector_passthrough, __tauri_command_name_set_region_selector_passthrough,
};

#[cfg(test)]
use borrowed_controls::recording_controls_may_restore;
#[cfg(test)]
use borrowed_controls::screenshot_region_may_restore;

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

#[tauri::command]
#[allow(clippy::needless_return)]
pub async fn set_region_selector_opacity(
  window: tauri::WebviewWindow,
  opacity: f64,
) -> Result<(), String> {
  #[cfg(target_os = "windows")]
  {
    let affinity = region_selector_capture_affinity(
      opacity,
      crate::settings::current(window.app_handle()).record_screenwide_windows,
    );
    // The shutter is one of our own captures: the windows whose exclusion
    // depends on that, rather than on the preference, follow it here.
    super::mark_capturing(super::Capture::Still, opacity <= 0.0);
    // The shutter only excludes the borrowed Region overlay. Every other
    // Screenwide window continues to follow the user's capture preference.
    super::sync_capture_affinity(window.app_handle(), affinity.other_windows)
      .map_err(|error| error.to_string())?;
    super::set_window_capture_affinity(&window, affinity.region_selector)
      .map_err(|error| error.to_string())?;
    super::screenshot_region::set_recording_overlay_capture_affinity(
      &window,
      affinity.region_selector,
    )
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

#[cfg(any(test, target_os = "windows"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RegionSelectorCaptureAffinity {
  other_windows: bool,
  region_selector: bool,
}

#[cfg(any(test, target_os = "windows"))]
const fn region_selector_capture_affinity(
  opacity: f64,
  record_screenwide_windows: bool,
) -> RegionSelectorCaptureAffinity {
  RegionSelectorCaptureAffinity {
    other_windows: record_screenwide_windows,
    region_selector: opacity > 0.0 && record_screenwide_windows,
  }
}

#[cfg(test)]
mod tests;
