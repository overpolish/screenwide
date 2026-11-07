// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Building an on-demand window from its `tauri.conf.json` entry, and the
//! setup each predefined window had at launch.

use tauri::{AppHandle, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};

use super::super::WindowLabel;
use super::{
  destroyed, failed, prepare, ready, state, tool_panel_of, READY_TIMEOUT, TITLE_BAR_IN_PAGE,
};

pub(super) fn build_in_background(
  app: AppHandle,
  label: WindowLabel,
  query: Option<&'static str>,
  build: u64,
) {
  tauri::async_runtime::spawn_blocking(move || {
    let window = match create(&app, label, query) {
      Ok(window) => window,
      Err(error) => return failed(label, error.to_string()),
    };
    // Set up on the main thread, which owns the window, as the predefined
    // windows were at launch.
    let setup_app = app.clone();
    let setup_window = window.clone();
    let queued = app.run_on_main_thread(move || {
      if let Err(error) = set_up(&setup_app, label, &setup_window) {
        eprintln!("Could not set up the {} window: {error}", label.as_str());
      }
    });
    if let Err(error) = queued {
      let _ = window.destroy();
      return failed(label, error.to_string());
    }
    if let Some(panel) = tool_panel_of(label) {
      prepare(&app, panel);
    }
    std::thread::sleep(READY_TIMEOUT);
    ready(&app, label, Some(build));
  });
}

fn create(
  app: &AppHandle,
  label: WindowLabel,
  query: Option<&str>,
) -> tauri::Result<WebviewWindow> {
  let mut config = label.config(app).ok_or(tauri::Error::WindowNotFound)?;
  config.visible = false;
  if let (Some(query), WebviewUrl::App(path)) = (query, &config.url) {
    config.url = WebviewUrl::App(format!("{}?{query}", path.display()).into());
  }
  // Built after launch, the page is handed the accent before its first paint.
  WebviewWindowBuilder::from_config(app, &config)?
    .initialization_script(screenwide_accent::initialization_script())
    .build()
}

fn set_up(app: &AppHandle, label: WindowLabel, window: &WebviewWindow) -> tauri::Result<()> {
  if TITLE_BAR_IN_PAGE.contains(&label) {
    #[cfg(target_os = "windows")]
    {
      // Dropping the frame keeps the outer size, which would hand the page the
      // caption's height as extra content. Put the configured size back.
      let inner = window.inner_size()?;
      window.set_decorations(false)?;
      window.set_size(inner)?;
    }
    let frame = state().frames.get(&label).copied();
    if let Some(frame) = frame {
      window.set_size(frame.size)?;
      window.set_position(frame.position)?;
    }
  }
  let mut surface = None;
  match label {
    WindowLabel::EditorRecording | WindowLabel::EditorScreenshot => {
      surface = crate::editor::preview_platform::surface_key(window);
      super::super::initialize_editor(window)?;
    }
    WindowLabel::Projects | WindowLabel::Settings => {
      super::super::initialize_normal_window(window)?;
    }
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    WindowLabel::Glide => {
      super::super::platform::initialize_glide_window(window)?;
    }
    WindowLabel::RegionSelector => {
      #[cfg(not(target_os = "macos"))]
      {
        super::super::platform::initialize_region_selector(window)?;
        window.set_ignore_cursor_events(true)?;
      }
      // Converted to a panel when first shown, and shown with its webview.
      #[cfg(target_os = "macos")]
      super::super::hide(window)?;
      super::release::when_unused(app, label, super::super::region::region_selector_needed);
    }
    _ => {
      #[cfg(not(target_os = "macos"))]
      super::super::platform::initialize_standalone_listbox(window)?;
      // Converted to a panel when first shown, and shown with its webview.
      #[cfg(target_os = "macos")]
      super::super::hide(window)?;
      if label == WindowLabel::StandaloneListbox {
        super::release::when_unused(app, label, super::super::options::shared_listbox_needed);
      }
    }
  }
  super::super::hide_instead_of_close(app, label);
  let destroyed_app = app.clone();
  window.on_window_event(move |event| {
    if matches!(event, WindowEvent::Destroyed) {
      if let Some(surface) = surface {
        crate::editor::preview_platform::forget(surface);
        // What the editor drew and decoded is freed only once the device is
        // next told to, which with the editor gone nothing else would do.
        // Waiting on the device is kept off the main thread.
        std::thread::spawn(screenwide_gpu::release_unused);
      }
      destroyed(&destroyed_app, label);
      // A page destroyed with its window cannot close its native pictures.
      #[cfg(any(target_os = "macos", target_os = "windows"))]
      if matches!(label, WindowLabel::Settings) {
        crate::settings_preview::close_window_previews(label.as_str());
      }
    }
  });
  Ok(())
}
