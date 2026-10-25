// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

mod glide_island_control;
pub use glide_island_control::defer_hide_glide_island;
pub(crate) use glide_island_control::fade_glide_island;
#[cfg(target_os = "macos")]
pub use glide_island_control::hide_glide_island;
pub use glide_island_control::initialize_glide_island;
#[cfg(target_os = "macos")]
pub(crate) use glide_island_control::position_glide_island;
pub use glide_island_control::show_glide_island;

mod recording_ui;
pub use recording_ui::hide_recording_ui;
pub use recording_ui::is_recording_ui_visible;
pub use recording_ui::open_recording_ui;
pub use recording_ui::recording_ui_visible;
pub use recording_ui::show_recording_ui;
pub(crate) use recording_ui::sync_recording_ui_escape;
pub use recording_ui::toggle_recording_ui;
pub use recording_ui::{__cmd__hide_recording_ui, __tauri_command_name_hide_recording_ui};
pub use recording_ui::{__cmd__open_recording_ui, __tauri_command_name_open_recording_ui};
pub use recording_ui::{__cmd__recording_ui_visible, __tauri_command_name_recording_ui_visible};
pub use recording_ui::{__cmd__toggle_recording_ui, __tauri_command_name_toggle_recording_ui};

pub(crate) mod island;
#[cfg(target_os = "macos")]
mod island_macos;
#[cfg(target_os = "windows")]
mod island_windows;
pub use island::{__cmd__finish_hiding_island, __tauri_command_name_finish_hiding_island};
pub use island::{__cmd__island_presence, __tauri_command_name_island_presence};
pub use island::{__cmd__set_island_hit_area, __tauri_command_name_set_island_hit_area};
pub use island::{finish_hiding_island, island_presence, set_island_hit_area};

mod labels;
pub use labels::WindowLabel;

use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_os = "windows")]
use tauri::WebviewWindow;
use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

mod boot;
mod capture_affinity;
#[cfg(target_os = "windows")]
pub(crate) use capture_affinity::apply_capture_policy;
pub(crate) use capture_affinity::exclude_from_capture;
#[cfg(target_os = "windows")]
pub(crate) use capture_affinity::is_capturing;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) use capture_affinity::set_window_capture_affinity;
#[cfg(target_os = "windows")]
pub use capture_affinity::sync_capture_affinity;
#[cfg(target_os = "windows")]
pub(crate) use capture_affinity::{mark_capturing, Capture};
pub(crate) mod color_panel;
mod dismissal;
#[cfg(target_os = "macos")]
mod dock_visibility;
#[cfg(target_os = "windows")]
pub(crate) mod drag_release;
mod escape;
pub(crate) mod geometry;
mod lifecycle;
pub(crate) mod monitor_capture;
pub(crate) mod options;
#[cfg(target_os = "windows")]
pub(crate) mod overlay_surface;
#[cfg(target_os = "macos")]
pub(crate) mod panel_presentation_macos;
pub(crate) mod panel_space;
pub(crate) mod platform;
mod popover_dismissal;
pub(crate) mod region;
pub(crate) mod region_gesture;
pub(crate) mod screenshot_region;
mod topology;
mod transient_popover;
mod webview_visibility;

pub(crate) use webview_visibility::hide_window as hide;

pub use boot::initialize_predefined_windows;
pub use dismissal::hide_without_focus_transfer;
pub(crate) use geometry::centered_logical_position;
#[cfg(target_os = "macos")]
pub use lifecycle::get_or_create;
pub use lifecycle::{
  hide_instead_of_close, initialize_editor, initialize_normal_window, recover_window_position, show,
};
#[cfg(not(target_os = "macos"))]
pub use lifecycle::{initialize_island, initialize_region_selector, initialize_standalone_listbox};
#[cfg(target_os = "windows")]
pub(crate) use platform::round_corners;
pub use popover_dismissal::manage_transient_popover_dismissal;
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

/// The builder for a window created after launch. The windows in
/// `tauri.conf.json` load hidden at startup and have the accent long before
/// they are shown; one built on demand is shown while its page still loads,
/// so it is handed the accent before its first paint. Its title is the app's
/// name until the caller names it, so no window reads as Tauri's default to
/// assistive technology.
pub(crate) fn webview_window<'a, R: Runtime, M: Manager<R>>(
  manager: &'a M,
  label: impl Into<String>,
  url: WebviewUrl,
) -> WebviewWindowBuilder<'a, R, M> {
  WebviewWindowBuilder::new(manager, label, url)
    .title(screenwide_i18n::t!("window-titles-app"))
    .initialization_script(screenwide_accent::initialization_script())
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
/// Puts the recording controls away: Escape no longer dismisses them, and the
/// island stops showing them. The island itself stays for what it shows
/// next, such as the recording they started; `hide_recording_ui` takes it
/// away too.
pub fn close_recording_controls(app: &AppHandle) -> tauri::Result<()> {
  escape::sync(app, false, false, crate::ruler::is_active(app));
  // Cleared first, so later overlay ordering cannot raise them again.
  RECORDING_CONTROLS_VISIBLE.store(false, Ordering::Relaxed);
  app.emit_to(WindowLabel::Island.as_str(), "recording-ui://hidden", ())
}
