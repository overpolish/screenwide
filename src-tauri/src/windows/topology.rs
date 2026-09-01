// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tauri::{AppHandle, Manager};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

use super::{geometry::contain_window_in_work_area, WindowLabel};

#[cfg(target_os = "macos")]
#[path = "topology/platform_macos.rs"]
mod platform;
#[cfg(target_os = "windows")]
#[path = "topology/platform_windows.rs"]
mod platform;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
#[path = "topology/platform_other.rs"]
mod platform;

static REVISION: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Policy {
  /// A normal app window remains fully usable on the nearest surviving work area.
  Contained,
  /// A persistent floating control keeps its location when valid and is saved after correction.
  PersistentControl,
  /// A transient is derived from another window and should not preserve stale topology geometry.
  OwnedTransient,
  /// Native desktop compositor surfaces own their complete display-union rebuild.
  DesktopSurface,
}

const fn policy(label: WindowLabel) -> Policy {
  match label {
    WindowLabel::ExportRecording
    | WindowLabel::ExportScreenshot
    | WindowLabel::QrDetails
    | WindowLabel::Settings
    | WindowLabel::Update => Policy::Contained,
    #[cfg(target_os = "macos")]
    WindowLabel::Permissions => Policy::Contained,
    WindowLabel::RecordingBar | WindowLabel::RecordingDock => Policy::PersistentControl,
    WindowLabel::Glide
    | WindowLabel::RecordingOptions
    | WindowLabel::RecordingSourceSelector
    | WindowLabel::StandaloneListbox => Policy::OwnedTransient,
    WindowLabel::RegionSelector | WindowLabel::Ruler | WindowLabel::TextRecognition => {
      Policy::DesktopSurface
    }
  }
}

fn close_transients(app: &AppHandle) {
  let _ = super::source_selector::collapse(app.clone(), Some(false));
  let _ = super::options::hide_recording_options(app.clone());
  let _ = super::options::hide_standalone_listbox(app.clone(), Some(false));
}

fn reconcile(app: &AppHandle) {
  close_transients(app);
  let mut persistent_position_may_have_changed = false;
  for &label in WindowLabel::ALL {
    match policy(label) {
      Policy::Contained | Policy::PersistentControl => {
        let Some(window) = app.get_webview_window(label.as_str()) else {
          continue;
        };
        if contain_window_in_work_area(app, &window).is_ok()
          && policy(label) == Policy::PersistentControl
        {
          persistent_position_may_have_changed = true;
        }
      }
      Policy::OwnedTransient | Policy::DesktopSurface => {}
    }
  }
  if persistent_position_may_have_changed {
    let _ = app.save_window_state(StateFlags::POSITION);
  }
}

fn schedule(app: AppHandle) {
  let revision = REVISION.fetch_add(1, Ordering::AcqRel).wrapping_add(1);
  tauri::async_runtime::spawn(async move {
    tokio::time::sleep(Duration::from_millis(120)).await;
    if REVISION.load(Ordering::Acquire) != revision {
      return;
    }
    reconcile(&app);
  });
}

pub(super) fn initialize(app: &AppHandle) {
  platform::initialize(app, schedule);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn every_managed_window_has_an_explicit_topology_policy() {
    for &label in WindowLabel::ALL {
      let _ = policy(label);
    }
  }

  #[test]
  fn desktop_surfaces_are_not_clamped_like_normal_windows() {
    assert_eq!(policy(WindowLabel::RegionSelector), Policy::DesktopSurface);
    assert_eq!(policy(WindowLabel::Ruler), Policy::DesktopSurface);
    assert_eq!(policy(WindowLabel::TextRecognition), Policy::DesktopSurface);
  }
}
