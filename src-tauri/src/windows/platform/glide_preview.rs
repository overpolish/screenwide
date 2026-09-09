// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use tauri::WebviewWindow;

#[cfg(target_os = "windows")]
use tauri::Manager;

#[cfg(target_os = "macos")]
use std::{cell::Cell, ptr::NonNull};

#[cfg(target_os = "macos")]
use block2::RcBlock;
#[cfg(target_os = "macos")]
use objc2_app_kit::{NSAnimatablePropertyContainer, NSAnimationContext};
#[cfg(target_os = "macos")]
use tauri::Manager;

/// How long a committed glide takes to fade away. Long enough to read as a
/// dismissal, short enough that the cursor waiting for it never feels late.
#[cfg(target_os = "macos")]
const FADE_OUT_SECONDS: f64 = 0.16;

pub fn initialize_glide_preview(window: &WebviewWindow) -> tauri::Result<()> {
  #[cfg(target_os = "windows")]
  super::initialize_overlay(window)?;
  #[cfg(target_os = "macos")]
  super::ensure_recording_panel(window)?;
  window.set_ignore_cursor_events(true)?;
  crate::windows::hide(window)
}

#[cfg(target_os = "macos")]
pub fn show_glide(window: &WebviewWindow, opacity: f64, blocks_hover: bool) -> tauri::Result<()> {
  window.set_ignore_cursor_events(!blocks_hover)?;
  let panel = super::ensure_recording_panel(window)?;
  let app = window.app_handle().clone();
  let window = window.clone();
  app.run_on_main_thread(move || {
    let _ = crate::windows::webview_visibility::show_webview(&window);
    panel.set_alpha_value(opacity);
    panel.show();
  })
}

/// Fades the preview out and tears it down the way `super::hide` does, then
/// runs `completion` on the main thread. Returns as soon as the animation is
/// scheduled - nothing here waits for it. Ignoring cursor events is permanent
/// for this window, so the teardown only has the alpha and the two hides to
/// undo, leaving the panel at alpha 0 for the next show to raise.
#[cfg(target_os = "macos")]
pub fn fade_out(window: &WebviewWindow, completion: Box<dyn FnOnce() + Send>) -> tauri::Result<()> {
  let panel = super::ensure_recording_panel(window)?;
  let window = window.clone();
  let app = window.app_handle().clone();
  app.run_on_main_thread(move || {
    let animated = panel.clone();
    let changes = RcBlock::new(move |context: NonNull<NSAnimationContext>| {
      // SAFETY: AppKit hands the grouping's context to this block and keeps it
      // alive for the call, which runs on the main thread.
      let context = unsafe { context.as_ref() };
      context.setDuration(FADE_OUT_SECONDS);
      animated.as_panel().animator().setAlphaValue(0.0);
    });

    // AppKit blocks are `Fn`, so the one-shot completion rides in a cell the
    // block takes it out of. The handler runs once, on the main thread.
    let completion = Cell::new(Some(completion));
    let finished = RcBlock::new(move || {
      panel.set_alpha_value(0.0);
      let _ = window.set_ignore_cursor_events(true);
      let _ = crate::windows::hide(&window);
      panel.hide();
      if let Some(completion) = completion.take() {
        completion();
      }
    });

    NSAnimationContext::runAnimationGroup_completionHandler(&changes, Some(&finished));
  })
}

#[cfg(target_os = "windows")]
pub fn show_glide(window: &WebviewWindow, _opacity: f64, blocks_hover: bool) -> tauri::Result<()> {
  super::composition::set_opacity(window, 1.0)?;
  window.set_ignore_cursor_events(!blocks_hover)?;
  window.show()
}

#[cfg(target_os = "windows")]
pub fn fade_out(window: &WebviewWindow, completion: Box<dyn FnOnce() + Send>) -> tauri::Result<()> {
  use std::sync::{Arc, Mutex};
  use std::time::Duration;

  let window = window.clone();
  let app = window.app_handle().clone();
  let completion = Arc::new(Mutex::new(Some(completion)));
  let fallback = completion.clone();
  std::thread::Builder::new()
    .name("glide-preview-fade".to_owned())
    .spawn(move || {
      const STEPS: u32 = 8;
      for step in (0..STEPS).rev() {
        std::thread::sleep(Duration::from_millis(20));
        let opacity = f64::from(step) / f64::from(STEPS);
        let next = window.clone();
        if app
          .run_on_main_thread(move || {
            let _ = super::composition::set_opacity(&next, opacity);
          })
          .is_err()
        {
          break;
        }
      }
      if app
        .run_on_main_thread(move || {
          let _ = window.set_ignore_cursor_events(true);
          let _ = window.hide();
          let _ = super::composition::set_opacity(&window, 1.0);
          if let Some(completion) = completion.lock().ok().and_then(|mut value| value.take()) {
            completion();
          }
        })
        .is_err()
      {
        // Keep the lifecycle one-shot even if the UI thread is already
        // shutting down; the caller still needs to release its activity lease.
        if let Some(completion) = fallback.lock().ok().and_then(|mut value| value.take()) {
          completion();
        }
      }
    })
    .map_err(std::io::Error::other)?;
  Ok(())
}
