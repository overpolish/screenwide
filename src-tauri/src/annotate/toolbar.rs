// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The overlay's floating toolbar: one window above every host.
//!
//! It opens top-centre on the anchor display, or wherever it was last dropped
//! on that display, and is sized by the plate it holds: the page measures
//! itself and asks for the window it needs, so the toolbar is only ever seen
//! at the size of its own contents.
//!
//! It is a non-activating panel, so a press on its controls leaves the
//! keyboard with the anchor host and the keys that draw keep working.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::utils::config::WindowEffectsConfig;
use tauri::window::{Effect, EffectState};
use tauri::{AppHandle, LogicalSize, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use super::host::HostPlan;
use super::settings::{self, ToolbarPosition};
use crate::capture_overlays;
use crate::windows::{platform, WindowLabel};

#[path = "toolbar_anchor.rs"]
mod anchor;
use anchor::{Anchor, INITIAL_SIZE};

/// Whether the session has got as far as putting the toolbar on screen, and
/// whether the plate has reported its size. The window is shown by whichever
/// of the two comes last, so it never appears at the size it was built at.
static PRESENTED: AtomicBool = AtomicBool::new(false);
static FITTED: AtomicBool = AtomicBool::new(false);

fn toolbar(app: &AppHandle) -> Option<WebviewWindow> {
  app.get_webview_window(WindowLabel::AnnotateToolbar.as_str())
}

/// Puts the toolbar's window in place for a session, hidden. Presentation is a
/// separate step, the way the hosts' is: every window is in place before any
/// of them is seen.
///
/// The window outlives the session. It is a converted `NSPanel`, and a panel
/// is hidden rather than closed here as everywhere else in the app: the
/// handle the panel manager keeps would outlive a closed window. Reopening
/// therefore only has to place the plate the page already measured.
pub(super) fn build(app: &AppHandle, anchor_plan: &HostPlan) -> Result<WebviewWindow, String> {
  let plan = Anchor {
    display_id: anchor_plan.display_id,
    origin: anchor_plan.work_position,
    size: anchor_plan.work_size,
  };
  PRESENTED.store(false, Ordering::Release);

  if let Some(window) = toolbar(app) {
    let scale = window.scale_factor().map_err(|error| error.to_string())?;
    // The inner size, because that is what the plate reported and what
    // `set_frame` sets. Reading the outer size back would feed a window frame
    // into the content size, growing the toolbar by that frame every time a
    // session reopened it.
    let size = window
      .inner_size()
      .map_err(|error| error.to_string())?
      .to_logical::<f64>(scale);
    platform::set_frame(&window, anchor::placement(&plan, size), size)
      .map_err(|error| error.to_string())?;
    *anchor::current() = Some(plan);
    own_by_anchor(app, &window)?;
    return Ok(window);
  }

  let position = anchor::placement(&plan, INITIAL_SIZE);
  *anchor::current() = Some(plan);
  FITTED.store(false, Ordering::Release);

  let window = WebviewWindowBuilder::new(
    app,
    WindowLabel::AnnotateToolbar.as_str(),
    WebviewUrl::App("/annotate-toolbar".into()),
  )
  .title("Screenwide Annotate Toolbar")
  .accept_first_mouse(true)
  .always_on_top(true)
  .decorations(false)
  .focused(false)
  .inner_size(INITIAL_SIZE.width, INITIAL_SIZE.height)
  .position(position.x, position.y)
  .resizable(false)
  .shadow(true)
  .skip_taskbar(true)
  .transparent(true)
  .visible(false)
  .visible_on_all_workspaces(true)
  .effects(WindowEffectsConfig {
    color: None,
    effects: vec![if cfg!(target_os = "windows") {
      Effect::Mica
    } else {
      Effect::UnderWindowBackground
    }],
    radius: Some(10.0),
    state: Some(EffectState::Active),
  })
  .build()
  .map_err(|error| error.to_string())?;

  // One level above the hosts: a stroke can never be drawn over the controls,
  // and the controls are never hidden by what has been drawn.
  capture_overlays::set_level(&window, capture_overlays::FOREGROUND_LEVEL + 1)?;
  // Refusing activation is window-server work on both platforms - a panel
  // conversion on macOS, an extended style on Windows - and the overlay's
  // windows are built off the thread that services the event loop. Waiting
  // keeps the toolbar from being shown before it refuses key status.
  let panelled = window.clone();
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app
    .run_on_main_thread(move || {
      let _ = sender
        .send(platform::initialize_annotate_toolbar(&panelled).map_err(|error| error.to_string()));
    })
    .map_err(|error| error.to_string())?;
  receiver.recv().map_err(|error| error.to_string())??;
  crate::windows::exclude_from_capture(&window).map_err(|error| error.to_string())?;
  own_by_anchor(app, &window)?;
  Ok(window)
}

/// Puts the toolbar above the hosts. On macOS its window level does that; on
/// Windows there are no levels, so it is made an owned window of the anchor
/// host, which always stands above its owner. The ownership is released in
/// [`close`], because the hosts are destroyed with the session and a window
/// takes the windows it owns with it.
fn own_by_anchor(app: &AppHandle, window: &WebviewWindow) -> Result<(), String> {
  #[cfg(target_os = "windows")]
  {
    let hosts = super::host::windows(app);
    let Some(host) = hosts.first() else {
      return Err("No annotate host is available for the toolbar".to_owned());
    };
    platform::set_owner(window, Some(host)).map_err(|error| error.to_string())?;
  }
  #[cfg(not(target_os = "windows"))]
  let _ = (app, window);
  Ok(())
}

/// The session has reached the point of showing the toolbar. It goes on screen
/// here only if the plate has already reported its size; otherwise the fit
/// that follows shows it.
pub(super) fn present(app: &AppHandle) {
  PRESENTED.store(true, Ordering::Release);
  show_if_ready(app);
}

/// Takes the toolbar off screen for as long as the annotations are only being
/// shown. It comes back with the next session rather than from here.
pub(super) fn hide(app: &AppHandle) {
  PRESENTED.store(false, Ordering::Release);
  // The cursor lease is macOS's: on Windows the crosshair is set per surface
  // in the overlay's own window procedure, so the toolbar's arrow needs no
  // exemption.
  #[cfg(target_os = "macos")]
  let _ = app.run_on_main_thread(super::cursor::forget_toolbar);
  if let Some(window) = toolbar(app) {
    if let Err(error) = platform::hide(&window) {
      eprintln!("Could not hide the annotate toolbar: {error}");
    }
  }
}

/// Takes the toolbar off screen with the session that owned it. The window
/// itself is kept: see [`build`].
///
/// The ownership goes with the session too. A window takes the windows it owns
/// with it when it is destroyed, and the hosts are destroyed here: a toolbar
/// still owned by one of them would be destroyed along with it, leaving the
/// next session to rebuild the plate it had already measured.
pub(super) fn close(app: &AppHandle) {
  *anchor::current() = None;
  hide(app);
  #[cfg(target_os = "windows")]
  if let Some(window) = toolbar(app) {
    if let Err(error) = platform::set_owner(&window, None) {
      eprintln!("Could not release the annotate toolbar's owner: {error}");
    }
  }
}

fn show_if_ready(app: &AppHandle) {
  if !PRESENTED.load(Ordering::Acquire) || !FITTED.load(Ordering::Acquire) {
    return;
  }
  let Some(window) = toolbar(app) else {
    return;
  };
  if window.is_visible().unwrap_or(false) {
    return;
  }
  #[cfg(target_os = "macos")]
  {
    // The pointer is an arrow over the toolbar: the crosshair belongs to the
    // canvas, and the guard that keeps it there exempts this window.
    let exempt = window.clone();
    let _ = app.run_on_main_thread(move || {
      super::cursor::exempt_toolbar(&exempt);
    });
  }
  if let Err(error) = platform::show(&window, 1.0) {
    eprintln!("Could not show the annotate toolbar: {error}");
  }
}

/// The plate reporting the size it needs, which is when the window is sized,
/// placed and - the first time - seen.
#[tauri::command]
pub fn resize_annotate_toolbar(app: AppHandle, width: f64, height: f64) -> Result<(), String> {
  if !width.is_finite() || width <= 0.0 || !height.is_finite() || height <= 0.0 {
    return Err("The annotate toolbar size must be positive".to_owned());
  }
  let Some(window) = toolbar(&app) else {
    return Ok(());
  };
  {
    let guard = anchor::current();
    let Some(anchor) = guard.as_ref() else {
      return Ok(());
    };
    let size = LogicalSize::new(
      width.ceil().min(anchor.size.width),
      height.ceil().min(anchor.size.height),
    );
    // One AppKit operation: a plate that grew a colour card must not be seen
    // at the old size in its new place.
    platform::set_frame(&window, anchor::placement(anchor, size), size)
      .map_err(|error| error.to_string())?;
  }
  FITTED.store(true, Ordering::Release);
  show_if_ready(&app);
  Ok(())
}

/// The toolbar reporting that it was dropped somewhere new. The place is kept
/// against the display it landed on, so the toolbar comes back where it was
/// left on that screen and top-centre on any other.
#[tauri::command]
pub fn persist_annotate_toolbar_position(app: AppHandle) -> Result<(), String> {
  let Some(window) = toolbar(&app) else {
    return Ok(());
  };
  let scale = window.scale_factor().map_err(|error| error.to_string())?;
  let position = window
    .outer_position()
    .map_err(|error| error.to_string())?
    .to_logical::<f64>(scale);
  let Some((display_id, origin)) = anchor::display_under(&app, position)? else {
    return Ok(());
  };
  settings::store_toolbar_position(
    &app,
    ToolbarPosition {
      display_id,
      x: position.x - origin.x,
      y: position.y - origin.y,
    },
  )
}
