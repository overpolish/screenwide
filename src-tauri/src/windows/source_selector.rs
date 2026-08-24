// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Recording source-selector layout, animation and visibility lifecycle.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewWindow};

use super::{
  platform, region,
  source_selector_layout::{selector_frames, SelectorFrame, ANIMATION_STEPS},
  WindowLabel,
};

static ANIMATION: AtomicU64 = AtomicU64::new(0);
static EXPANDED: AtomicBool = AtomicBool::new(false);
static VISIBLE: AtomicBool = AtomicBool::new(true);
static WINDOW_SELECTOR_ACTIVE: AtomicBool = AtomicBool::new(false);
static REGION_CONTROLS_VISIBLE: AtomicBool = AtomicBool::new(false);

fn frames(
  app: &AppHandle,
) -> tauri::Result<(
  super::source_selector_layout::SelectorPlacement,
  SelectorFrame,
  SelectorFrame,
)> {
  selector_frames(
    app,
    REGION_CONTROLS_VISIBLE.load(Ordering::Relaxed),
    WINDOW_SELECTOR_ACTIVE.load(Ordering::Relaxed),
  )
}

fn animate<F>(window: WebviewWindow, from: SelectorFrame, to: SelectorFrame, on_complete: F)
where
  F: FnOnce() + Send + 'static,
{
  let animation = ANIMATION.fetch_add(1, Ordering::Relaxed) + 1;
  tauri::async_runtime::spawn_blocking(move || {
    for step in 1..=ANIMATION_STEPS {
      if ANIMATION.load(Ordering::Relaxed) != animation {
        return;
      }

      let progress = step as f64 / ANIMATION_STEPS as f64;
      let eased = 1.0 - (1.0 - progress).powi(3);
      let interpolate = |start: f64, end: f64| start + (end - start) * eased;
      let position = LogicalPosition::new(
        interpolate(from.position.x, to.position.x),
        interpolate(from.position.y, to.position.y),
      );
      let size = LogicalSize::new(
        interpolate(from.size.width, to.size.width),
        interpolate(from.size.height, to.size.height),
      );

      let _ = window.set_position(position);
      let _ = window.set_size(size);
      std::thread::sleep(Duration::from_millis(10));
    }

    if ANIMATION.load(Ordering::Relaxed) == animation {
      on_complete();
    }
  });
}

pub(super) fn reposition(app: &AppHandle) -> tauri::Result<()> {
  let selector = app
    .get_webview_window(WindowLabel::RecordingSourceSelector.as_str())
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  if !selector.is_visible()? {
    return Ok(());
  }

  let (placement, collapsed, expanded) = frames(app)?;
  let target = if EXPANDED.load(Ordering::Relaxed) {
    expanded
  } else {
    collapsed
  };
  ANIMATION.fetch_add(1, Ordering::Relaxed);
  selector.set_size(target.size)?;
  selector.set_position(target.position)?;
  app.emit_to(
    WindowLabel::RecordingSourceSelector.as_str(),
    "recording-source-selector://placement",
    placement,
  )?;

  Ok(())
}

pub(super) fn is_expanded() -> bool {
  EXPANDED.load(Ordering::Relaxed)
}

pub(super) fn is_visible() -> bool {
  VISIBLE.load(Ordering::Relaxed)
}

#[tauri::command]
pub fn toggle_recording_source_selector(
  app: AppHandle,
  window_selector: bool,
) -> tauri::Result<()> {
  // A recording hides this chrome deliberately; nothing may bring it back
  // until the recording is over.
  if !crate::recording::is_idle(&app) {
    return Ok(());
  }

  let window = app
    .get_webview_window(WindowLabel::RecordingSourceSelector.as_str())
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  if is_expanded() {
    return collapse_recording_source_selector(app);
  }
  WINDOW_SELECTOR_ACTIVE.store(window_selector, Ordering::Relaxed);
  let (placement, collapsed, expanded) = frames(&app)?;

  if !window.is_visible()? {
    window.set_size(collapsed.size)?;
    window.set_position(collapsed.position)?;
    platform::show(&window, 1.0)?;
  }
  EXPANDED.store(true, Ordering::Relaxed);
  app.emit_to(
    WindowLabel::RecordingSourceSelector.as_str(),
    "recording-source-selector://expanded",
    placement,
  )?;
  animate(window, collapsed, expanded, || {});

  Ok(())
}

#[tauri::command]
pub fn collapse_recording_source_selector(app: AppHandle) -> tauri::Result<()> {
  let window = app
    .get_webview_window(WindowLabel::RecordingSourceSelector.as_str())
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  if !window.is_visible()? || !EXPANDED.swap(false, Ordering::Relaxed) {
    return Ok(());
  }

  let (_, collapsed, _) = frames(&app)?;
  let scale = window.scale_factor()?;
  let current = SelectorFrame {
    position: window.outer_position()?.to_logical(scale),
    size: window.outer_size()?.to_logical(scale),
  };
  let event_app = app.clone();
  animate(window, current, collapsed, move || {
    let _ = event_app.emit_to(
      WindowLabel::RecordingSourceSelector.as_str(),
      "recording-source-selector://collapsed",
      (),
    );
  });

  Ok(())
}

pub(super) fn show(app: &AppHandle) -> tauri::Result<()> {
  let selector = app
    .get_webview_window(WindowLabel::RecordingSourceSelector.as_str())
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  let (placement, collapsed, _) = frames(app)?;
  #[cfg(target_os = "macos")]
  let positioning = ANIMATION.fetch_add(1, Ordering::Relaxed) + 1;
  #[cfg(not(target_os = "macos"))]
  ANIMATION.fetch_add(1, Ordering::Relaxed);
  EXPANDED.store(false, Ordering::Relaxed);
  selector.set_size(collapsed.size)?;
  selector.set_position(collapsed.position)?;
  platform::show(&selector, 1.0)?;
  platform::restore_recording_level(&selector)?;
  app.emit_to(
    WindowLabel::RecordingSourceSelector.as_str(),
    "recording-source-selector://collapsed",
    placement,
  )?;

  #[cfg(target_os = "macos")]
  let app = app.clone();
  #[cfg(target_os = "macos")]
  tauri::async_runtime::spawn_blocking(move || {
    std::thread::sleep(Duration::from_millis(75));
    if ANIMATION.load(Ordering::Relaxed) != positioning {
      return;
    }
    let Some(selector) = app.get_webview_window(WindowLabel::RecordingSourceSelector.as_str())
    else {
      return;
    };
    if let Ok((_, collapsed, _)) = frames(&app) {
      let _ = selector.set_size(collapsed.size);
      let _ = selector.set_position(collapsed.position);
    }
  });

  Ok(())
}

pub(super) fn hide(app: &AppHandle) -> tauri::Result<()> {
  ANIMATION.fetch_add(1, Ordering::Relaxed);
  EXPANDED.store(false, Ordering::Relaxed);
  if let Some(selector) = app.get_webview_window(WindowLabel::RecordingSourceSelector.as_str()) {
    platform::hide(&selector)?;
  }
  Ok(())
}

#[tauri::command]
pub fn set_recording_source_selector_visible(app: AppHandle, visible: bool) -> tauri::Result<()> {
  VISIBLE.store(visible, Ordering::Relaxed);
  if visible {
    if region::source_selector_may_show() {
      show(&app)
    } else {
      Ok(())
    }
  } else {
    hide(&app)
  }
}

#[tauri::command]
pub fn set_recording_source_selector_region_controls(
  app: AppHandle,
  visible: bool,
) -> tauri::Result<()> {
  REGION_CONTROLS_VISIBLE.store(visible, Ordering::Relaxed);
  reposition(&app)
}
