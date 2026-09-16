// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The overlay's host windows: one transparent surface per display.
//!
//! The first display's window is the anchor and the one made key; the rest are
//! ordered on screen without asking for focus, the way the region overlay's
//! peer panels are. Each carries a Metal layer that draws that display's
//! annotations.

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use crate::{capture_overlays, windows::WindowLabel};

/// One host's geometry, read from the monitor layout before any window is
/// built: xcap's display handles are not `Send`, so nothing here may await.
pub(super) struct HostPlan {
  position: tauri::LogicalPosition<f64>,
  size: tauri::LogicalSize<f64>,
}

/// The anchor keeps the plain label so the window is recognisable; the peers
/// are numbered after it.
fn label(index: usize) -> String {
  let anchor = WindowLabel::Annotate.as_str();
  if index == 0 {
    anchor.to_owned()
  } else {
    format!("{anchor}-{index}")
  }
}

/// Every window the overlay may have opened, whatever the display count was
/// when it opened them.
pub(super) fn windows(app: &AppHandle) -> Vec<WebviewWindow> {
  let anchor = WindowLabel::Annotate.as_str();
  let peer = format!("{anchor}-");
  let mut hosts: Vec<_> = app
    .webview_windows()
    .into_iter()
    .filter(|(label, _)| label == anchor || label.starts_with(&peer))
    .collect();
  // Anchor first: it is the window that owns focus and the cursor lease.
  hosts.sort_by(|(first, _), (second, _)| first.len().cmp(&second.len()).then(first.cmp(second)));
  hosts.into_iter().map(|(_, window)| window).collect()
}

/// Moves every host to one window level. A screenshot in progress drops the
/// annotations under the region overlay so the selection is drawn over them.
pub(super) fn set_level(app: &AppHandle, level: isize) -> Result<(), String> {
  for host in windows(app) {
    capture_overlays::set_level(&host, level)?;
  }
  Ok(())
}

/// Reads the layout and tells the native side what it will be drawing on.
pub(super) fn plan(app: &AppHandle) -> Result<Vec<HostPlan>, String> {
  let monitors = capture_overlays::monitor_layout(app)?;
  if monitors.is_empty() {
    return Err("No monitor is available for Annotate".to_owned());
  }
  #[cfg(target_os = "macos")]
  let mut displays = Vec::with_capacity(monitors.len());
  let mut plans = Vec::with_capacity(monitors.len());
  for (_, scale, monitor) in &monitors {
    let position = monitor.position().to_logical::<f64>(*scale);
    let size = monitor.size().to_logical::<f64>(*scale);
    #[cfg(target_os = "macos")]
    displays.push(super::native_overlay::Display {
      origin: (position.x, position.y),
      scale: *scale,
    });
    plans.push(HostPlan { position, size });
  }
  #[cfg(target_os = "macos")]
  super::native_overlay::set_displays(displays);
  Ok(plans)
}

/// Opens one display's host, hidden. Presentation is a separate step so every
/// window is in place before any of them takes the foreground.
pub(super) fn build(
  app: &AppHandle,
  index: usize,
  host: &HostPlan,
) -> Result<WebviewWindow, String> {
  let window = WebviewWindowBuilder::new(app, label(index), WebviewUrl::App("/annotate".into()))
    .accept_first_mouse(true)
    .always_on_top(true)
    .decorations(false)
    .focused(false)
    .inner_size(host.size.width, host.size.height)
    .position(host.position.x, host.position.y)
    .resizable(false)
    .shadow(false)
    .skip_taskbar(true)
    .transparent(true)
    .visible(false)
    .visible_on_all_workspaces(true)
    .build()
    .map_err(|error| error.to_string())?;
  capture_overlays::set_level(&window, capture_overlays::FOREGROUND_LEVEL)?;
  // macOS capture already excludes this process's own windows. Windows needs
  // to be told, and is told here so the overlay is out of the recording from
  // the moment it exists rather than from the moment it draws.
  crate::windows::exclude_from_capture(&window).map_err(|error| error.to_string())?;
  Ok(window)
}

/// Puts every host on screen, with its Metal layer and the crosshair, and
/// starts routing input. Called off the thread that services the event loop.
pub(super) fn present(app: &AppHandle, hosts: Vec<WebviewWindow>) -> Result<(), String> {
  #[cfg(not(target_os = "macos"))]
  {
    let _ = app;
    for host in &hosts {
      host.show().map_err(|error| error.to_string())?;
    }
    Ok(())
  }
  #[cfg(target_os = "macos")]
  {
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    app
      .run_on_main_thread(move || {
        let _ = sender.send(present_on_main_thread(&hosts));
      })
      .map_err(|error| error.to_string())?;
    receiver.recv().map_err(|error| error.to_string())?
  }
}

#[cfg(target_os = "macos")]
fn present_on_main_thread(hosts: &[WebviewWindow]) -> Result<(), String> {
  let Some((anchor, peers)) = hosts.split_first() else {
    return Err("No monitor is available for Annotate".to_owned());
  };
  // Before the surfaces exist: attaching draws immediately, and the first
  // frame should already carry whatever is on screen.
  super::native_overlay::install_scene();
  for (index, host) in hosts.iter().enumerate() {
    super::native_overlay::attach(host, index as u32)?;
    super::cursor::claim(host);
  }
  // The anchor takes the foreground through the cursor lease, which is what
  // gives the overlay the key window and puts the user's application back
  // when it is released. The peers must not take it off the anchor again.
  crate::osc::cursor::macos::present_window(anchor).map_err(|error| error.to_string())?;
  for peer in peers {
    super::native_overlay::order_front(peer);
  }
  super::native_overlay::install_input();
  super::native_overlay::redraw();
  Ok(())
}

/// Takes every host down, reporting whether there was one. Runs on the main
/// thread, resigning key and main first so AppKit cannot promote the Settings
/// or Editor window while the cursor lease is still putting the user's own
/// application back.
pub(super) fn close(app: &AppHandle) -> bool {
  let hosts = windows(app);
  if hosts.is_empty() {
    return false;
  }
  #[cfg(not(target_os = "macos"))]
  {
    hosts.iter().any(|host| host.close().is_ok())
  }
  #[cfg(target_os = "macos")]
  {
    if objc2::MainThreadMarker::new().is_some() {
      return close_on_main_thread(&hosts);
    }
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    if app
      .run_on_main_thread(move || {
        let _ = sender.send(close_on_main_thread(&hosts));
      })
      .is_err()
    {
      return false;
    }
    receiver.recv().unwrap_or(false)
  }
}

#[cfg(target_os = "macos")]
fn close_on_main_thread(hosts: &[WebviewWindow]) -> bool {
  super::native_overlay::teardown_input();
  super::native_overlay::forget_displays();
  super::input::cancel();
  let mut closed = false;
  for host in hosts {
    super::native_overlay::detach(host);
    super::cursor::release(host);
    crate::osc::cursor::macos::prepare_window_close(host);
    closed |= host.close().is_ok();
  }
  closed
}

/// Hands the hosts over to showing annotations: input goes back to whatever is
/// underneath while the annotations stay drawn on top. Reports whether there were
/// hosts to hand over.
pub(super) fn show_only(app: &AppHandle) -> bool {
  let hosts = windows(app);
  if hosts.is_empty() {
    return false;
  }
  #[cfg(not(target_os = "macos"))]
  {
    // Nothing is drawn without the native renderer, so there is nothing to
    // leave on screen either.
    let _ = app;
    hosts.iter().any(|host| host.close().is_ok())
  }
  #[cfg(target_os = "macos")]
  {
    if objc2::MainThreadMarker::new().is_some() {
      return show_only_on_main_thread(&hosts);
    }
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    if app
      .run_on_main_thread(move || {
        let _ = sender.send(show_only_on_main_thread(&hosts));
      })
      .is_err()
    {
      return false;
    }
    receiver.recv().unwrap_or(false)
  }
}

#[cfg(target_os = "macos")]
fn show_only_on_main_thread(hosts: &[WebviewWindow]) -> bool {
  super::native_overlay::teardown_input();
  super::input::cancel();
  for host in hosts {
    // The crosshair and the key window belong to the user's application again;
    // only the pixels stay ours.
    super::cursor::release(host);
    crate::osc::cursor::macos::prepare_window_close(host);
    super::native_overlay::set_click_through(host, true);
  }
  super::native_overlay::redraw();
  true
}
