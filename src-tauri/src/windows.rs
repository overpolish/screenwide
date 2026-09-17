// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[path = "windows/glide_preview_control.rs"]
mod glide_preview_control;
pub use glide_preview_control::defer_hide_glide_preview;
pub(crate) use glide_preview_control::fade_glide_preview;
#[cfg(target_os = "macos")]
pub use glide_preview_control::hide_glide_preview;
pub use glide_preview_control::initialize_glide_preview;
#[cfg(target_os = "macos")]
pub(crate) use glide_preview_control::position_glide_preview;
pub use glide_preview_control::show_glide_preview;

#[path = "windows/recording_ui.rs"]
mod recording_ui;
pub use recording_ui::hide_recording_ui;
pub use recording_ui::is_recording_ui_visible;
pub use recording_ui::recording_ui_visible;
pub use recording_ui::show_recording_ui;
pub(crate) use recording_ui::sync_recording_ui_escape;
pub use recording_ui::toggle_recording_ui;
pub use recording_ui::{__cmd__hide_recording_ui, __tauri_command_name_hide_recording_ui};
pub use recording_ui::{__cmd__recording_ui_visible, __tauri_command_name_recording_ui_visible};
pub use recording_ui::{__cmd__toggle_recording_ui, __tauri_command_name_toggle_recording_ui};

#[path = "windows/labels.rs"]
mod labels;
#[path = "windows/recording_bar_movement.rs"]
mod recording_bar_movement;
pub use labels::WindowLabel;

pub use recording_bar_movement::finish_recording_bar_drag;
pub use recording_bar_movement::manage_recording_bar_movement;
pub use recording_bar_movement::{
  __cmd__finish_recording_bar_drag, __tauri_command_name_finish_recording_bar_drag,
};

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewWindow, WindowEvent};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

mod boot;
mod capture_affinity;
#[cfg(target_os = "windows")]
pub(crate) use capture_affinity::apply_capture_policy;
pub(crate) use capture_affinity::exclude_from_capture;
#[cfg(target_os = "windows")]
pub(crate) use capture_affinity::is_capturing;
#[cfg(target_os = "windows")]
pub(crate) use capture_affinity::set_window_capture_affinity;
#[cfg(target_os = "windows")]
pub use capture_affinity::sync_capture_affinity;
#[cfg(target_os = "windows")]
pub(crate) use capture_affinity::{mark_capturing, Capture};
pub(crate) mod color_panel;
mod dismissal;
pub(crate) mod dock;
#[cfg(target_os = "macos")]
mod dock_visibility;
mod escape;
mod geometry;
mod lifecycle;
pub(crate) mod monitor_capture;
pub(crate) mod options;
#[cfg(target_os = "windows")]
pub(crate) mod overlay_surface;
#[cfg(target_os = "macos")]
mod panel_presentation_macos;
pub(crate) mod panel_space;
pub(crate) mod platform;
pub(crate) mod region;
pub(crate) mod region_gesture;
pub(crate) mod screenshot_region;
pub(crate) mod source_selector;
mod source_selector_layout;
mod topology;
mod transient_popover;
mod webview_visibility;

pub(crate) use webview_visibility::hide_window as hide;

pub use boot::initialize_predefined_windows;
pub use dismissal::hide_without_focus_transfer;
#[cfg(not(target_os = "macos"))]
pub use dock::initialize_recording_dock;
pub use dock::{hide_recording_dock, manage_recording_dock_movement, show_recording_dock};
pub(crate) use geometry::centered_logical_position;
use geometry::monitor_with_most_overlap;
#[cfg(target_os = "macos")]
pub use lifecycle::get_or_create;
pub use lifecycle::{
  hide_instead_of_close, initialize_editor, initialize_normal_window,
  initialize_recording_bar_position, recover_window_position, show,
};
#[cfg(not(target_os = "macos"))]
pub use lifecycle::{
  initialize_recording_bar, initialize_recording_source_selector, initialize_region_selector,
  initialize_standalone_listbox,
};
#[cfg(target_os = "windows")]
pub(crate) use platform::round_corners;
pub use region::{
  hide_region_selector, is_region_selector_visible, set_region_selector_passthrough,
};

/// Applies the same non-animated, always-on-top policy as the predefined
/// overlays to capture tools whose transparent host windows are created only
/// when the tool starts (Ruler, OCR, and their auxiliary windows).
#[cfg(target_os = "windows")]
pub(crate) fn initialize_capture_overlay(window: &WebviewWindow) -> tauri::Result<()> {
  platform::initialize_capture_overlay(window)
}

/// Puts one overlay host back in the always-on-top band. Needed after any
/// call that rebuilds a window's extended style from tao's own flags, which
/// drops the band without tao noticing.
#[cfg(target_os = "windows")]
pub(crate) fn raise_annotate_host(window: &WebviewWindow) -> tauri::Result<()> {
  platform::raise_without_activation(window)
}

/// Lets presses through one overlay host to whatever is underneath, keeping
/// every style it holds - including its z-order band.
#[cfg(target_os = "windows")]
pub(crate) fn set_host_pointer_passthrough(
  window: &WebviewWindow,
  passthrough: bool,
) -> tauri::Result<()> {
  platform::set_pointer_passthrough(window, passthrough)
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

pub fn initialize_topology_management(app: &AppHandle) {
  topology::initialize(app);
}

// This is where a list of capture-excluded window labels used to live. Capture
// now excludes every window this process owns, matched on the owning process
// rather than by name, so a window added later is excluded the day it is added
// and there is no list left to forget to update. See `capture_kit::our_windows`.

static RECORDING_CONTROLS_VISIBLE: AtomicBool = AtomicBool::new(false);
static REGION_SELECTOR_INTERACTIVE: AtomicBool = AtomicBool::new(false);
#[cfg(target_os = "windows")]
static BAR_DRAG_ACTIVE: AtomicBool = AtomicBool::new(false);

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

#[derive(Clone, Copy, Default)]
struct PopoversOpenOnPress {
  source_selector: bool,
  standalone_listbox: bool,
}

impl PopoversOpenOnPress {
  fn capture() -> Self {
    Self {
      source_selector: source_selector::is_expanded(),
      standalone_listbox: options::is_standalone_listbox_open(),
    }
  }

  fn dismiss_outside(self, app: &AppHandle, x: f64, y: f64) {
    source_selector::dismiss_if_outside(app, self.source_selector, x, y);
    options::dismiss_standalone_listbox_if_outside(app, self.standalone_listbox, x, y);
  }
}

#[cfg(target_os = "windows")]
pub fn manage_transient_popover_dismissal(app: &AppHandle) {
  use std::sync::mpsc;

  use rdev::{listen, Button, EventType};

  let (dismiss_tx, dismiss_rx) = mpsc::channel::<(PopoversOpenOnPress, f64, f64)>();
  let dismiss_app = app.clone();
  std::thread::spawn(move || {
    while let Ok((open_on_press, x, y)) = dismiss_rx.recv() {
      open_on_press.dismiss_outside(&dismiss_app, x, y);
    }
  });
  std::thread::spawn(move || {
    let mut position = (0.0, 0.0);
    let mut open_on_press = PopoversOpenOnPress::default();
    let result = listen(move |event| match event.event_type {
      EventType::MouseMove { x, y } => {
        position = (x, y);
      }
      EventType::ButtonPress(Button::Left) => {
        open_on_press = PopoversOpenOnPress::capture();
      }
      EventType::ButtonRelease(Button::Left) => {
        let (x, y) = position;
        // rdev invokes this callback before CallNextHookEx. Defer dismissal
        // because its window geometry queries synchronously wait for the UI
        // thread, which cannot process them while the hook callback is live.
        if open_on_press.source_selector || open_on_press.standalone_listbox {
          let _ = dismiss_tx.send((open_on_press, x, y));
        }
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
