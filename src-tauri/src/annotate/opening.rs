// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Opening the overlay: building the hosts, or handing hosts that are only
//! showing annotations back to drawing.

use tauri::{AppHandle, Manager, WebviewWindow};

use super::{abandon, host, is_active, settings, stop_drawing, AnnotateState};
use crate::capture_overlays;

/// Opens the overlay. Called off the thread that services the event loop: the
/// windows, their layer surfaces and the cursor lease are all put in place
/// through it.
///
/// Hosts that are only showing annotations kept from the last session are
/// handed back to drawing rather than rebuilt: taking them down and opening
/// new ones would blank the annotations until the new surfaces first draw.
pub fn start(app: &AppHandle) -> Result<(), String> {
  open(app, false)
}

fn open(app: &AppHandle, rebuild: bool) -> Result<(), String> {
  if !settings::enabled() {
    return Ok(());
  }

  let hosts = host::windows(app);
  let resume = !rebuild && app.state::<AnnotateState>().is_showing() && !hosts.is_empty();
  // Before the layout is read: closing hosts forgets the displays the native
  // side draws on, and the plan is what tells it the new ones.
  if !resume {
    abandon(app);
  }
  let plans = host::plan(app)?;
  // Hosts left up on a layout that has since changed are worth nothing.
  if resume && hosts.len() != plans.len() {
    return open(app, true);
  }
  capture_overlays::dismiss_except(app, &[capture_overlays::CaptureOverlay::Annotate]);
  // The island shows one thing at a time: idle recording controls step aside
  // for the annotate tools, which take the island once the hosts are up.
  if crate::app_windows::is_recording_ui_visible() && crate::recording::is_idle(app) {
    if let Err(error) = crate::app_windows::hide_recording_ui(app.clone()) {
      eprintln!("Could not put the recording controls away for Annotate: {error}");
    }
  }
  let generation = app.state::<AnnotateState>().begin();
  crate::glide::suspend_for_capture(app);
  // Taken before the windows exist: the lease remembers the application the
  // user was working in, which is what it puts back on release, and it is what
  // makes the pointer a crosshair over the overlay.
  #[cfg(target_os = "macos")]
  if let Err(error) = crate::osc::cursor_macos::acquire_annotate(app, super::text_cursor()) {
    abandon(app);
    return Err(error);
  }
  let result = if resume {
    resume_hosts(app, hosts)
  } else {
    start_hosts(app, generation, &plans)
  };
  if result.is_err() || !is_active(app) {
    abandon(app);
  }
  result
}

fn start_hosts(app: &AppHandle, generation: u64, plans: &[host::HostPlan]) -> Result<(), String> {
  let mut hosts = Vec::with_capacity(plans.len());
  for (index, plan) in plans.iter().enumerate() {
    hosts.push(host::build(app, index, plan)?);
  }
  // A newer session may have started while the windows were being built. It
  // owns the overlay now, and these windows are not part of it.
  if !app.state::<AnnotateState>().install(generation) {
    for window in hosts {
      let _ = window.close();
    }
    return Ok(());
  }
  host::present(app, hosts)?;
  hosts_presented(app);
  Ok(())
}

/// Drawing again on the hosts that were showing the annotations. No window is
/// built, so there is no gap for a newer session to have started in.
fn resume_hosts(app: &AppHandle, hosts: Vec<WebviewWindow>) -> Result<(), String> {
  host::resume(app, hosts)?;
  hosts_presented(app);
  Ok(())
}

/// What follows the hosts taking input, however they got there.
fn hosts_presented(app: &AppHandle) {
  // The pointer is a crosshair drawing annotations now, not something a viewer
  // should follow, so a running recording's cursor layer leaves it out for as
  // long as the overlay is up. A recording that starts later picks the same
  // state up from its first frame.
  screenwide_capture::cursor::set_cursor_visibility(false, None);
  capture_overlays::emit_lifecycle(app, true);
  crate::app_windows::sync_recording_ui_escape(app, crate::ruler::is_active(app));
}

/// A display change invalidates every host's geometry, so the overlay is
/// rebuilt on the layout that replaced it. The annotations are kept: they are held
/// in desktop points, and one whose display went away simply stops being
/// drawn.
pub fn restart_after_topology_change(app: &AppHandle) {
  if !is_active(app) && !app.state::<AnnotateState>().is_showing() {
    return;
  }
  let app = app.clone();
  tauri::async_runtime::spawn(async move {
    // Checked again here rather than against a captured generation: the user
    // may have left in the meantime, and nothing should reopen behind them.
    let showing = app.state::<AnnotateState>().is_showing();
    if !is_active(&app) && !showing {
      return;
    }
    if let Err(error) = open(&app, true) {
      eprintln!("Could not rebuild the annotate overlay after a display change: {error}");
      return;
    }
    // Annotations that were only being shown go back to only being shown.
    if showing {
      stop_drawing(&app);
    }
  });
}
