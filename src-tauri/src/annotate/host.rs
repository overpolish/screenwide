// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The overlay's host windows: one transparent surface per display.
//!
//! The first display's window is the anchor and the one made key; the rest are
//! ordered on screen without asking for focus, the way the region overlay's
//! peer panels are. Each carries a Metal layer that draws that display's
//! annotations.

#[cfg(target_os = "windows")]
#[path = "host_windows.rs"]
mod platform;

use tauri::{AppHandle, Manager, WebviewWindow};

use crate::{capture_overlays, windows::WindowLabel};

#[path = "host_planning.rs"]
mod planning;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(super) use planning::HostPlan;
pub(super) use planning::{build, plan};

/// Whether a label names one of the overlay's hosts. The peers are numbered
/// after the anchor, and the number is what tells a host from the toolbar,
/// whose label shares the feature's prefix.
pub(crate) fn is_host_label(label: &str) -> bool {
  let anchor = WindowLabel::Annotate.as_str();
  label == anchor
    || label
      .strip_prefix(&format!("{anchor}-"))
      .is_some_and(|index| !index.is_empty() && index.chars().all(|digit| digit.is_ascii_digit()))
}

/// Every host window the overlay may have opened, whatever the display count
/// was when it opened them.
pub(super) fn windows(app: &AppHandle) -> Vec<WebviewWindow> {
  let mut hosts: Vec<_> = app
    .webview_windows()
    .into_iter()
    .filter(|(label, _)| is_host_label(label))
    .collect();
  // Anchor first: it is the window that owns focus and the cursor lease.
  hosts.sort_by(|(first, _), (second, _)| first.len().cmp(&second.len()).then(first.cmp(second)));
  hosts.into_iter().map(|(_, window)| window).collect()
}

/// The hosts' capture affinity: excluded exactly while Screenwide itself is
/// capturing, and capturable otherwise, so a screen share in another
/// application still shows the annotations. Windows exclusion hides a window
/// from every capturer, so this is the only way to be absent from a recording
/// without being absent from everything.
#[cfg(target_os = "windows")]
fn apply_capture_affinity(window: &WebviewWindow) -> Result<(), String> {
  crate::windows::set_window_capture_affinity(window, !crate::windows::is_capturing())
    .map_err(|error| error.to_string())
}

/// Moves every host to one window level. A screenshot in progress drops the
/// annotations under the region overlay so the selection is drawn over them.
pub(super) fn set_level(app: &AppHandle, level: isize) -> Result<(), String> {
  for host in windows(app) {
    capture_overlays::set_level(&host, level)?;
    // On Windows `set_level` reapplies the overlay policy, which reads the
    // capture preference; the hosts follow their own rule instead.
    #[cfg(target_os = "windows")]
    apply_capture_affinity(&host)?;
  }
  Ok(())
}

/// Puts every host on screen, with its Metal layer and the crosshair, and
/// starts routing input. Called off the thread that services the event loop.
pub(super) fn present(app: &AppHandle, hosts: Vec<WebviewWindow>) -> Result<(), String> {
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    let _ = app;
    for host in &hosts {
      host.show().map_err(|error| error.to_string())?;
    }
    Ok(())
  }
  #[cfg(target_os = "windows")]
  {
    platform::present(app, hosts)
  }
  #[cfg(target_os = "macos")]
  {
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let handle = app.clone();
    app
      .run_on_main_thread(move || {
        let _ = sender.send(present_on_main_thread(&handle, &hosts));
      })
      .map_err(|error| error.to_string())?;
    receiver.recv().map_err(|error| error.to_string())??;
    // After the anchor has the foreground, so ordering the toolbar front
    // cannot take it back off the window that owns the keyboard.
    super::toolbar::present(app);
    Ok(())
  }
}

#[cfg(target_os = "macos")]
fn present_on_main_thread(app: &AppHandle, hosts: &[WebviewWindow]) -> Result<(), String> {
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
  super::native_overlay::install_input(app);
  super::native_overlay::redraw();
  Ok(())
}

/// Takes every host down, reporting whether there was one. Runs on the main
/// thread, resigning key and main first so AppKit cannot promote the Settings
/// or Editor window while the cursor lease is still putting the user's own
/// application back.
pub(super) fn close(app: &AppHandle) -> bool {
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  super::toolbar::close(app);
  let hosts = windows(app);
  if hosts.is_empty() {
    return false;
  }
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    hosts.iter().any(|host| host.close().is_ok())
  }
  #[cfg(target_os = "windows")]
  {
    platform::close(app, hosts)
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
  // Annotations that are only being shown take no input, so there is nothing
  // for a toolbar to edit.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  super::toolbar::hide(app);
  let hosts = windows(app);
  if hosts.is_empty() {
    return false;
  }
  #[cfg(not(any(target_os = "macos", target_os = "windows")))]
  {
    // Nothing is drawn without a native renderer, so there is nothing to
    // leave on screen either.
    let _ = app;
    hosts.iter().any(|host| host.close().is_ok())
  }
  #[cfg(target_os = "windows")]
  {
    platform::show_only(app, hosts)
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
