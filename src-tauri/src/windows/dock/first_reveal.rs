// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Holds the pill's very first show back until its page has drawn.
//!
//! WebKit takes about 300ms after a show to draw a page it kept hidden. From
//! the second show on, the window only appears once that draw lands, but the
//! first show of a page that has never drawn goes on screen at once, as an
//! empty pill. So until the page reports its first frame, the pill is shown
//! transparent and faded in on that report.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use tauri::{AppHandle, Manager};

use super::{platform, WindowLabel};

/// Whether the page has drawn at least once.
static PAINTED: AtomicBool = AtomicBool::new(false);
/// The show waiting on the page's first frame, or 0 when none is.
static PENDING: AtomicU64 = AtomicU64::new(0);
#[cfg(target_os = "macos")]
static NEXT_SHOW: AtomicU64 = AtomicU64::new(1);
/// Reveals the pill even if the page never reports, so a page that failed to
/// load cannot leave a recording without its controls.
#[cfg(target_os = "macos")]
const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);

/// The opacity to show the pill with: transparent until the page has drawn.
#[cfg(target_os = "macos")]
pub(super) fn show_opacity(app: &AppHandle) -> f64 {
  if PAINTED.load(Ordering::Acquire) {
    return 1.0;
  }

  let show = NEXT_SHOW.fetch_add(1, Ordering::Relaxed);
  PENDING.store(show, Ordering::Release);
  let app = app.clone();
  let _ = std::thread::Builder::new()
    .name("recording-dock-reveal".to_owned())
    .spawn(move || {
      std::thread::sleep(TIMEOUT);
      reveal(&app, show);
    });
  0.0
}

/// Drops a pending reveal, so a pill hidden before its page drew is not made
/// opaque behind the next show's back.
pub(super) fn cancel() {
  PENDING.store(0, Ordering::Release);
}

fn reveal(app: &AppHandle, show: u64) {
  if PENDING
    .compare_exchange(show, 0, Ordering::AcqRel, Ordering::Acquire)
    .is_err()
  {
    return;
  }
  if let Some(dock) = app.get_webview_window(WindowLabel::RecordingDock.as_str()) {
    if let Err(error) = platform::set_opacity(&dock, 1.0) {
      eprintln!("Could not reveal the recording pill: {error}");
    }
  }
}

/// Sent by the page once a frame has actually reached the screen.
#[tauri::command]
pub fn recording_dock_painted(app: AppHandle) {
  PAINTED.store(true, Ordering::Release);
  let show = PENDING.load(Ordering::Acquire);
  if show != 0 {
    reveal(&app, show);
  }
}
