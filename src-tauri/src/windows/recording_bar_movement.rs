// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn contain_recording_bar(app: &AppHandle) -> tauri::Result<()> {
  let bar = app
    .get_webview_window(WindowLabel::RecordingBar.as_str())
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  let bar_position = bar.outer_position()?;
  let bar_size = bar.outer_size()?;
  let target = monitor_with_most_overlap(app, &bar)?.ok_or_else(|| tauri::Error::WindowNotFound)?;
  let monitor_position = target.position();
  let monitor_size = target.size();
  let max_x = monitor_position.x + monitor_size.width.saturating_sub(bar_size.width) as i32;
  let max_y = monitor_position.y + monitor_size.height.saturating_sub(bar_size.height) as i32;
  let contained = PhysicalPosition::new(
    bar_position.x.clamp(monitor_position.x, max_x),
    bar_position.y.clamp(monitor_position.y, max_y),
  );

  if contained != bar_position {
    bar.set_position(contained)?;
  }

  Ok(())
}

pub fn manage_recording_bar_movement(app: &AppHandle) {
  let Some(window) = app.get_webview_window(WindowLabel::RecordingBar.as_str()) else {
    return;
  };
  // AppKit repositions the source selector itself, so the handler captures
  // nothing there.
  #[cfg(not(target_os = "macos"))]
  let app = app.clone();

  window.on_window_event(move |event| {
    let WindowEvent::Moved(_) = event else {
      return;
    };

    // AppKit moves the source selector as a native child of the bar, avoiding
    // the visible delay caused by chasing Moved events with a second window.
    #[cfg(not(target_os = "macos"))]
    let _ = source_selector::reposition(&app);

    #[cfg(target_os = "windows")]
    watch_for_recording_bar_mouse_up(app.clone());
  });
}

#[cfg(target_os = "windows")]
pub(super) fn watch_for_recording_bar_mouse_up(app: AppHandle) {
  use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};

  if unsafe { GetAsyncKeyState(VK_LBUTTON.0.into()) } >= 0
    || BAR_DRAG_ACTIVE.swap(true, Ordering::Relaxed)
  {
    return;
  }

  tauri::async_runtime::spawn_blocking(move || {
    loop {
      let is_pressed = unsafe { GetAsyncKeyState(VK_LBUTTON.0.into()) } < 0;
      if !is_pressed {
        break;
      }
      std::thread::sleep(Duration::from_millis(8));
    }

    let _ = finish_recording_bar_drag(app);
    BAR_DRAG_ACTIVE.store(false, Ordering::Relaxed);
  });
}

#[tauri::command]
pub fn finish_recording_bar_drag(app: AppHandle) -> Result<(), String> {
  contain_recording_bar(&app).map_err(|error| error.to_string())?;
  source_selector::reposition(&app).map_err(|error| error.to_string())?;
  app
    .save_window_state(StateFlags::POSITION)
    .map_err(|error| error.to_string())?;
  Ok(())
}
