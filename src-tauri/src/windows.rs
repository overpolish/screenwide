// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewWindow, WindowEvent};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

pub(crate) mod dock;
mod escape;
mod geometry;
mod lifecycle;
pub(crate) mod monitor_capture;
pub(crate) mod options;
mod platform;
mod recording_options_layout;
pub(crate) mod region;
pub(crate) mod region_gesture;
pub(crate) mod screenshot_region;
pub(crate) mod source_selector;
mod source_selector_layout;
mod transient_popover;

#[cfg(not(target_os = "macos"))]
pub use dock::initialize_recording_dock;
pub use dock::{hide_recording_dock, manage_recording_dock_movement, show_recording_dock};
use geometry::monitor_with_most_overlap;
#[cfg(target_os = "macos")]
pub use lifecycle::get_or_create;
pub use lifecycle::{
  contain_export, contain_normal_window, hide_instead_of_close, initialize_export,
  initialize_normal_window, initialize_recording_bar_position, show, sync_dock_visibility,
};
#[cfg(not(target_os = "macos"))]
pub use lifecycle::{
  initialize_recording_bar, initialize_recording_options, initialize_recording_source_selector,
  initialize_region_selector, initialize_standalone_listbox,
};
pub use options::hide_recording_options;
pub use region::{
  hide_region_selector, is_region_selector_visible, set_region_selector_passthrough,
};

#[cfg(target_os = "windows")]
pub fn sync_capture_affinity(
  app: &AppHandle,
  record_screenwide_windows: bool,
) -> tauri::Result<()> {
  for window in app.webview_windows().values() {
    if platform::is_visible(window)? {
      platform::set_capture_affinity(window, record_screenwide_windows)?;
    }
  }
  Ok(())
}

/// Keeps one window out of every capture, whatever the persistent "record
/// Screenwide's windows" preference says. Windows excludes per window; macOS
/// excludes by owning process at the capture call, so there is nothing to do
/// to the window itself there.
pub(crate) fn exclude_from_capture(window: &WebviewWindow) -> tauri::Result<()> {
  #[cfg(target_os = "windows")]
  {
    platform::set_capture_affinity(window, false)
  }

  #[cfg(not(target_os = "windows"))]
  {
    let _ = window;
    Ok(())
  }
}

/// Removes a disposable overlay's pixels before Windows runs its native hide
/// or close transition. This is intentionally reserved for windows that will
/// be destroyed rather than shown again, because their layered alpha remains
/// zero afterwards.
#[cfg(target_os = "windows")]
pub(crate) fn conceal_disposable_overlay(window: &WebviewWindow) -> tauri::Result<()> {
  platform::set_opacity(window, 0.0)?;
  platform::hide(window)
}

#[derive(Clone, Copy)]
pub enum WindowLabel {
  /// The recording workspace's window. Each export workspace has one of its
  /// own so a recording can wait for a decision while a screenshot is edited.
  ExportRecording,
  ExportScreenshot,
  #[cfg(target_os = "macos")]
  Permissions,
  RecordingBar,
  RecordingDock,
  RecordingOptions,
  Settings,
  RegionSelector,
  RecordingSourceSelector,
  StandaloneListbox,
  Update,
}

impl WindowLabel {
  pub const fn as_str(self) -> &'static str {
    match self {
      Self::ExportRecording => "export-recording",
      Self::ExportScreenshot => "export-screenshot",
      #[cfg(target_os = "macos")]
      Self::Permissions => "permissions",
      Self::RecordingBar => "recording-bar",
      Self::RecordingDock => "recording-dock",
      Self::RecordingOptions => "recording-options",
      Self::Settings => "settings",
      Self::RegionSelector => "region-selector",
      Self::RecordingSourceSelector => "recording-source-selector",
      Self::StandaloneListbox => "standalone-listbox",
      Self::Update => "update",
    }
  }
}

// This is where a list of capture-excluded window labels used to live. Capture
// now excludes every window this process owns, matched on the owning process
// rather than by name, so a window added later is excluded the day it is added
// and there is no list left to forget to update. See `capture_kit::our_windows`.

static RECORDING_CONTROLS_VISIBLE: AtomicBool = AtomicBool::new(false);
static REGION_SELECTOR_INTERACTIVE: AtomicBool = AtomicBool::new(false);
#[cfg(target_os = "windows")]
static BAR_DRAG_ACTIVE: AtomicBool = AtomicBool::new(false);

fn contain_recording_bar(app: &AppHandle) -> tauri::Result<()> {
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

pub fn hide_recording_bar(app: &AppHandle) -> tauri::Result<()> {
  escape::sync(app, false, false, crate::ruler::is_active(app));
  // Clear first so later overlay ordering cannot raise the bar again.
  RECORDING_CONTROLS_VISIBLE.store(false, Ordering::Relaxed);
  if let Some(bar) = app.get_webview_window(WindowLabel::RecordingBar.as_str()) {
    platform::hide(&bar)?;
    app.emit_to(
      WindowLabel::RecordingBar.as_str(),
      "recording-ui://hidden",
      (),
    )?;
  }

  Ok(())
}

pub fn manage_recording_bar_movement(app: &AppHandle) {
  let Some(window) = app.get_webview_window(WindowLabel::RecordingBar.as_str()) else {
    return;
  };
  let app = app.clone();

  window.on_window_event(move |event| {
    let WindowEvent::Moved(_) = event else {
      return;
    };

    // AppKit moves the source selector as a native child of the bar, avoiding
    // the visible delay caused by chasing Moved events with a second window.
    #[cfg(not(target_os = "macos"))]
    let _ = source_selector::reposition(&app);
    let _ = hide_recording_options(app.clone());

    #[cfg(target_os = "windows")]
    watch_for_recording_bar_mouse_up(app.clone());
  });
}

#[derive(Clone, Copy, Default)]
struct PopoversOpenOnPress {
  recording_options: bool,
  source_selector: bool,
}

impl PopoversOpenOnPress {
  fn capture() -> Self {
    Self {
      recording_options: options::is_recording_options_open(),
      source_selector: source_selector::is_expanded(),
    }
  }

  fn dismiss_outside(self, app: &AppHandle, x: f64, y: f64) {
    source_selector::dismiss_if_outside(app, self.source_selector, x, y);
    options::dismiss_recording_options_if_outside(app, self.recording_options, x, y);
  }
}

#[cfg(target_os = "windows")]
pub fn manage_transient_popover_dismissal(app: &AppHandle) {
  use std::sync::{Arc, Mutex};

  use rdev::{listen, Button, EventType};

  let app = app.clone();
  let mouse_position = Arc::new(Mutex::new((0.0, 0.0)));
  std::thread::spawn(move || {
    let position = mouse_position.clone();
    let mut open_on_press = PopoversOpenOnPress::default();
    let result = listen(move |event| match event.event_type {
      EventType::MouseMove { x, y } => {
        if let Ok(mut position) = position.lock() {
          *position = (x, y);
        }
      }
      EventType::ButtonPress(Button::Left) => {
        open_on_press = PopoversOpenOnPress::capture();
      }
      EventType::ButtonRelease(Button::Left) => {
        let Ok((x, y)) = position.lock().map(|position| *position) else {
          return;
        };
        open_on_press.dismiss_outside(&app, x, y);
        open_on_press = PopoversOpenOnPress::default();
      }
      _ => {}
    });

    if let Err(error) = result {
      eprintln!("Could not monitor clicks for transient popover dismissal: {error:?}");
    }
  });
}

#[cfg(target_os = "macos")]
pub fn manage_transient_popover_dismissal(app: &AppHandle) {
  use cidre::cg::{Event, EventSrcState, MouseButton};

  let app = app.clone();
  std::thread::spawn(move || {
    let mut was_pressed = EventSrcState::CombinedSession.button_state(MouseButton::Left);
    let mut open_on_press = PopoversOpenOnPress::default();

    loop {
      let is_pressed = EventSrcState::CombinedSession.button_state(MouseButton::Left);
      if !was_pressed && is_pressed {
        open_on_press = PopoversOpenOnPress::capture();
      }
      if was_pressed && !is_pressed {
        let Some(event) = Event::with_src(None) else {
          break;
        };
        let position = event.location();
        open_on_press.dismiss_outside(&app, position.x, position.y);
        open_on_press = PopoversOpenOnPress::default();
      }

      was_pressed = is_pressed;
      std::thread::sleep(Duration::from_millis(8));
    }
  });
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn manage_transient_popover_dismissal(_app: &AppHandle) {}

#[cfg(target_os = "windows")]
fn watch_for_recording_bar_mouse_up(app: AppHandle) {
  use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON};

  if BAR_DRAG_ACTIVE.swap(true, Ordering::Relaxed) {
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

#[tauri::command]
pub fn hide_recording_ui(app: AppHandle) -> tauri::Result<()> {
  // A Quick Screenshot is driven by the Region Selector webview. Hiding that
  // window during `capture_still` suspends its promise continuation before it
  // can restore the ruler's click handling and clear the screenshot session.
  // The frontend calls this command again after that cleanup is complete.
  if !region::recording_ui_may_hide(region::SCREENSHOT_REGION_SESSION.load(Ordering::Acquire)) {
    return Ok(());
  }

  RECORDING_CONTROLS_VISIBLE.store(false, Ordering::Relaxed);
  hide_recording_options(app.clone())?;
  source_selector::hide(&app)?;
  hide_recording_bar(&app)?;
  if let Some(region) = app.get_webview_window(WindowLabel::RegionSelector.as_str()) {
    platform::hide(&region)?;
  }

  Ok(())
}

pub fn show_recording_ui(app: &AppHandle) -> tauri::Result<()> {
  crate::capture_overlays::dismiss_all(app);
  if !crate::recording::is_idle(app) {
    return Ok(());
  }

  RECORDING_CONTROLS_VISIBLE.store(true, Ordering::Relaxed);
  let bar = app
    .get_webview_window(WindowLabel::RecordingBar.as_str())
    .ok_or_else(|| tauri::Error::WindowNotFound)?;
  platform::show(&bar, 1.0)?;
  escape::sync(
    app,
    true,
    region::SCREENSHOT_REGION_SESSION.load(Ordering::Relaxed),
    crate::ruler::is_active(app),
  );
  // Asserted rather than assumed: a screenshot session may have faded the bar
  // out, and requests to fade it back in are refused while a recording is on.
  // Coming back to idle is where that is put right.
  platform::set_opacity(&bar, 1.0)?;
  platform::restore_recording_level(&bar)?;

  app.emit_to(
    WindowLabel::RecordingBar.as_str(),
    "recording-ui://shown",
    (),
  )
}

pub fn is_recording_ui_visible() -> bool {
  RECORDING_CONTROLS_VISIBLE.load(Ordering::Relaxed)
}

#[tauri::command]
pub fn toggle_recording_ui(app: AppHandle) -> tauri::Result<()> {
  crate::capture_overlays::dismiss_all(&app);
  if !crate::recording::is_idle(&app) || crate::exports::focus_pending_workspace(&app) {
    return Ok(());
  }

  // Tauri's visibility query describes the original webview window and is not
  // authoritative after macOS converts it into an NSPanel.
  if is_recording_ui_visible() {
    return hide_recording_ui(app);
  }

  #[cfg(target_os = "macos")]
  if !crate::permissions::has_required_recording_permissions(&app) {
    return crate::permissions::show_permissions_window(&app);
  }

  show_recording_ui(&app)
}

pub(crate) fn sync_recording_ui_escape(app: &AppHandle, ruler_active: bool) {
  escape::sync(
    app,
    RECORDING_CONTROLS_VISIBLE.load(Ordering::Relaxed),
    region::SCREENSHOT_REGION_SESSION.load(Ordering::Relaxed),
    ruler_active,
  );
}

#[tauri::command]
pub fn recording_ui_visible() -> bool {
  is_recording_ui_visible()
}
