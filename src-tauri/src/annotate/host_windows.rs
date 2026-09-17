// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The Windows half of the host lifecycle.
//!
//! The overlay children, their swap chains and their messages all belong to
//! the thread that created the host windows, so every step here runs there.
//! Blocking on a dispatch from that thread would deadlock, which is what the
//! [`owns_hosts`] guard is for: it is the twin of the macOS side's
//! `MainThreadMarker::new()`.

use tauri::{AppHandle, WebviewWindow};

use crate::annotate::native_overlay;

/// Gives every host its overlay surface, puts them on screen and draws the
/// annotations once, so a session that opens with annotations kept from the
/// last one shows them immediately.
pub(super) fn present(app: &AppHandle, hosts: Vec<WebviewWindow>) -> Result<(), String> {
  if owns_hosts(&hosts) {
    present_on_owning_thread(app, &hosts)?;
  } else {
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let handle = app.clone();
    app
      .run_on_main_thread(move || {
        let _ = sender.send(present_on_owning_thread(&handle, &hosts));
      })
      .map_err(|error| error.to_string())?;
    receiver.recv().map_err(|error| error.to_string())??;
  }
  // After the hosts, so the toolbar stands above them among the topmost
  // windows rather than under what is drawn.
  crate::annotate::toolbar::present(app);
  Ok(())
}

fn present_on_owning_thread(app: &AppHandle, hosts: &[WebviewWindow]) -> Result<(), String> {
  for (index, host) in hosts.iter().enumerate() {
    native_overlay::attach(host, index as u32)?;
    host.show().map_err(|error| error.to_string())?;
    // Asserted, not assumed: `always_on_top` is a tao flag it only acts on
    // when the flag changes, and only `SetWindowPos` moves a window into the
    // band. A host that is merely styled topmost is covered by the next
    // window the user opens.
    if let Err(error) = crate::windows::raise_annotate_host(host) {
      eprintln!("An annotate host could not be raised: {error}");
    }
  }
  // After the hosts are on screen: the monitor targets the anchor's child, and
  // focusing a window that is not yet visible does nothing.
  native_overlay::install_input(app);
  native_overlay::redraw();
  Ok(())
}

/// Takes every host down. The surfaces are detached before the windows close:
/// a child window outlives nothing, but the swap chain that draws into it must
/// go while its window is still there.
pub(super) fn close(app: &AppHandle, hosts: Vec<WebviewWindow>) -> bool {
  if owns_hosts(&hosts) {
    return close_on_owning_thread(&hosts);
  }
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  if app
    .run_on_main_thread(move || {
      let _ = sender.send(close_on_owning_thread(&hosts));
    })
    .is_err()
  {
    return false;
  }
  receiver.recv().unwrap_or(false)
}

fn close_on_owning_thread(hosts: &[WebviewWindow]) -> bool {
  native_overlay::teardown_input();
  native_overlay::forget_displays();
  crate::annotate::input::cancel();
  let mut closed = false;
  for host in hosts {
    native_overlay::detach(host);
    closed |= host.close().is_ok();
  }
  closed
}

/// Hands the hosts over to showing annotations: input goes back to whatever is
/// underneath while the annotations stay drawn on top. The hosts and their
/// surfaces stay up, so [`crate::annotate::clear`] is the way out of them.
pub(super) fn show_only(app: &AppHandle, hosts: Vec<WebviewWindow>) -> bool {
  if owns_hosts(&hosts) {
    return show_only_on_owning_thread(&hosts);
  }
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  if app
    .run_on_main_thread(move || {
      let _ = sender.send(show_only_on_owning_thread(&hosts));
    })
    .is_err()
  {
    return false;
  }
  receiver.recv().unwrap_or(false)
}

fn show_only_on_owning_thread(hosts: &[WebviewWindow]) -> bool {
  native_overlay::teardown_input();
  crate::annotate::input::cancel();
  for host in hosts {
    // A press belongs to whatever is underneath now, including another
    // process: the child stops hit-testing and the host stops taking the
    // pointer at all. Only the pixels stay ours.
    native_overlay::set_click_through(host, true);
    if let Err(error) = crate::windows::set_host_pointer_passthrough(host, true) {
      eprintln!("An annotate host could not pass presses through: {error}");
    }
    // The band is asserted rather than assumed: a window created later lands
    // at the top of the ordinary band, and annotations left on screen have to
    // stay above it. Only `SetWindowPos` moves a window between bands - the
    // extended style's own topmost bit does not.
    if let Err(error) = crate::windows::raise_annotate_host(host) {
      eprintln!("An annotate host could not stay above other windows: {error}");
    }
  }
  native_overlay::redraw();
  true
}

/// True when this call is already on the thread that owns the hosts.
fn owns_hosts(hosts: &[WebviewWindow]) -> bool {
  use windows::Win32::{
    Foundation::HWND, System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::GetWindowThreadProcessId,
  };
  hosts.first().is_some_and(|host| {
    host.hwnd().is_ok_and(|hwnd| unsafe {
      GetWindowThreadProcessId(HWND(hwnd.0), None) == GetCurrentThreadId()
    })
  })
}
