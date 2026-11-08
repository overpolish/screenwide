// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Letting go of the on-demand windows nobody closes. The region overlay and
//! the shared menu window are put away by what used them, never closed, so
//! each is destroyed once it has stayed hidden, with nothing needing it, for
//! [`GRACE`]. The grace keeps a quick hide and show, such as a mode change or
//! a second menu, from paying for a new window each time.
//!
//! A window is looked at only every [`CHECK`], and a quick use can start and
//! end between two looks, so each hide is noted as it happens and the grace
//! counts from the latest one.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex, PoisonError};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager};

use super::super::WindowLabel;

/// How long a window stays after it was last on screen or needed.
const GRACE: Duration = Duration::from_secs(30);
/// How often a window being watched is looked at.
const CHECK: Duration = Duration::from_secs(5);

/// The watch each window has, so one left over from a window destroyed and
/// built again stops rather than watching the new one twice.
static WATCHES: LazyLock<Mutex<HashMap<WindowLabel, u64>>> = LazyLock::new(Default::default);
static NEXT_WATCH: AtomicU64 = AtomicU64::new(0);
/// When each watched window was last put away.
static HIDDEN_AT: LazyLock<Mutex<HashMap<WindowLabel, Instant>>> = LazyLock::new(Default::default);

fn is_current(label: WindowLabel, watch: u64) -> bool {
  WATCHES
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
    .get(&label)
    == Some(&watch)
}

/// Notes that the window labelled `label` has just been hidden, if it is
/// one being watched.
pub(super) fn note_hidden(label: &str) {
  let watched = WATCHES
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
    .keys()
    .copied()
    .find(|watched| watched.as_str() == label);
  if let Some(watched) = watched {
    HIDDEN_AT
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .insert(watched, Instant::now());
  }
}

fn hidden_at(label: WindowLabel) -> Option<Instant> {
  HIDDEN_AT
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
    .get(&label)
    .copied()
}

/// Destroys `label`'s window once it has been hidden, and `needed` has said
/// no, for the whole grace.
pub(super) fn when_unused(app: &AppHandle, label: WindowLabel, needed: fn(&AppHandle) -> bool) {
  let watch = NEXT_WATCH.fetch_add(1, Ordering::Relaxed);
  WATCHES
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
    .insert(label, watch);
  let app = app.clone();
  let spawned = std::thread::Builder::new()
    .name(format!("release-{}", label.as_str()))
    .spawn(move || {
      let mut unused_since: Option<Instant> = None;
      loop {
        std::thread::sleep(CHECK);
        if !is_current(label, watch) {
          return;
        }
        let Some(window) = app.get_webview_window(label.as_str()) else {
          return;
        };
        if window.is_visible().unwrap_or(true) || needed(&app) {
          unused_since = None;
          continue;
        }
        let since = *unused_since.get_or_insert_with(Instant::now);
        let since = hidden_at(label).map_or(since, |hidden| hidden.max(since));
        if since.elapsed() >= GRACE {
          super::close(&app, label);
          return;
        }
      }
    });
  if let Err(error) = spawned {
    eprintln!("Could not watch the {} window: {error}", label.as_str());
  }
}
