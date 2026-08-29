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
  platform::show(&region, initial_opacity)?;
  platform::restore_recording_level(&region)?;
  #[cfg(target_os = "macos")]
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
    #[cfg(target_os = "macos")]
    super::screenshot_region::set_recording_overlay_desktop_presented(&region, false)?;
    platform::hide(&region)?;
  }
  set_recording_controls_opacity(app, 1.0)
}

fn raise_recording_controls(app: &AppHandle) -> tauri::Result<()> {
  if !recording_controls_may_raise(
    RECORDING_CONTROLS_VISIBLE.load(Ordering::Relaxed),
    region_gesture::is_active(),
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
const fn recording_controls_may_raise(controls_visible: bool, gesture_active: bool) -> bool {
  controls_visible && !gesture_active
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
pub fn set_screenshot_region_session(app: AppHandle, active: bool) -> tauri::Result<()> {
  let controls_visible = RECORDING_CONTROLS_VISIBLE.load(Ordering::Relaxed);
  SCREENSHOT_REGION_SESSION.store(active, Ordering::Release);
  escape::sync(
    &app,
    controls_visible,
    active,
    crate::ruler::is_active(&app),
  );

  if active {
    let result = match app.get_webview_window(WindowLabel::RegionSelector.as_str()) {
      Some(region) => {
        #[cfg(target_os = "macos")]
        let peers = super::screenshot_region::prepare_recording_overlay_for_screenshot(&region);
        #[cfg(not(target_os = "macos"))]
        let peers = Ok(());
        peers
          .and_then(|()| platform::hide(&region))
          .and_then(|()| platform::set_opacity(&region, 0.0))
      }
      None => Ok(()),
    };
    if let Err(error) = result {
      SCREENSHOT_REGION_SESSION.store(false, Ordering::Release);
      escape::sync(&app, controls_visible, false, crate::ruler::is_active(&app));
      return Err(error);
    }
  }
  Ok(())
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
pub fn set_region_selector_opacity(
  window: tauri::WebviewWindow,
  opacity: f64,
) -> Result<(), String> {
  #[cfg(target_os = "macos")]
  if opacity <= 0.0 {
    super::screenshot_region::set_recording_overlay_desktop_presented(&window, false)
      .map_err(|error| error.to_string())?;
  }
  platform::set_opacity(&window, opacity).map_err(|error| error.to_string())
}

/// Fades the recording controls while the region overlay is borrowed for a
/// screenshot session.
///
/// Fading them *in* is refused outside an idle app. The controls belong to the
/// idle state; while a recording is starting or running they are deliberately
/// gone. Hiding the region overlay asks for opacity 1.0 as cleanup without
/// knowing that, and `prepare_windows` itself hides the bar before hiding the
/// region overlay. The guard prevents that cleanup from reviving the bar.
#[tauri::command]
pub fn set_recording_controls_opacity(app: AppHandle, opacity: f64) -> tauri::Result<()> {
  if !RECORDING_CONTROLS_VISIBLE.load(Ordering::Relaxed) {
    return Ok(());
  }
  if opacity > 0.0 && !crate::recording::is_idle(&app) {
    return Ok(());
  }

  if let Some(bar) = app.get_webview_window(WindowLabel::RecordingBar.as_str()) {
    platform::set_opacity(&bar, opacity)?;
  }
  if source_selector::is_expanded() {
    if let Some(selector) = app.get_webview_window(WindowLabel::RecordingSourceSelector.as_str()) {
      platform::set_opacity(&selector, opacity)?;
    }
  }
  if opacity > 0.0 {
    raise_recording_controls(&app)?;
  } else {
    let _ = source_selector::collapse(app.clone(), Some(false));
  }
  Ok(())
}

#[cfg(test)]
mod tests;
