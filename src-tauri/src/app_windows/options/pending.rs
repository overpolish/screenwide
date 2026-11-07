// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shows asked for while a panel's window is still to be built. The shared
//! listbox and the editors' tool panels are built when wanted, so the first
//! show after one was let go waits for it, and is dropped if the panel was
//! put away meanwhile.

use std::collections::BTreeMap;
use std::sync::{LazyLock, Mutex};

use tauri::AppHandle;

use super::super::WindowLabel;

/// Counts, per panel window, the shows asked for and the times it was put
/// away, so a show waiting for its window is dropped if the panel was put
/// away after it was asked for.
static PANEL_REQUESTS: LazyLock<Mutex<BTreeMap<String, u64>>> = LazyLock::new(Default::default);

/// Moves `panel`'s request count on, or every panel's for `None`, returning
/// the new count for a named panel.
pub(super) fn next_panel_request(panel: Option<&str>) -> u64 {
  let mut requests = PANEL_REQUESTS
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  match panel {
    Some(panel) => {
      let count = requests.entry(panel.to_owned()).or_default();
      *count += 1;
      *count
    }
    None => {
      requests.values_mut().for_each(|count| *count += 1);
      0
    }
  }
}

fn is_latest_panel_request(panel: &str, request: u64) -> bool {
  PANEL_REQUESTS
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .get(panel)
    == Some(&request)
}

/// Runs `show` for `panel` on the main thread once its window is built and
/// ready, unless the panel was put away first.
pub(super) fn show_once_built(
  app: &AppHandle,
  panel: &str,
  show: impl FnOnce(AppHandle, String) -> tauri::Result<()> + Send + 'static,
) {
  let Some(label) = WindowLabel::ALL
    .iter()
    .copied()
    .find(|label| label.as_str() == panel)
    .filter(|label| super::super::on_demand::is_on_demand(*label))
  else {
    return;
  };
  let request = next_panel_request(Some(panel));
  let shown = app.clone();
  super::super::on_demand::open(app, label, move |opened| {
    if opened.is_err() {
      return;
    }
    let main = shown.clone();
    let _ = shown.run_on_main_thread(move || {
      if !is_latest_panel_request(label.as_str(), request) {
        return;
      }
      if let Err(error) = show(main, label.as_str().to_owned()) {
        eprintln!("Could not show the {} panel: {error}", label.as_str());
      }
    });
  });
}
