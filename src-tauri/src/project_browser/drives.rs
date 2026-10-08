// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The browser's locations coming and going as drives are plugged in and
//! taken out, so the sidebar follows without waiting to be clicked.
//!
//! Neither platform's mount notifications cover a network share that drops
//! or a folder deleted by hand, so the folders themselves are looked at. A
//! handful of checks every few seconds costs nothing worth measuring.

use std::path::PathBuf;
use std::sync::Once;
use std::time::Duration;

use tauri::AppHandle;

/// Soon enough that a drive plugged in shows as one by the time the user
/// looks, seldom enough to cost nothing.
const INTERVAL: Duration = Duration::from_secs(2);

static WATCHING: Once = Once::new();

/// Starts telling the browser to look again whenever a location becomes
/// available or unavailable. Once per run of the app.
pub(super) fn watch(app: &AppHandle) {
  WATCHING.call_once(|| {
    let app = app.clone();
    let started = std::thread::Builder::new()
      .name("project-locations".to_owned())
      .spawn(move || {
        let mut seen = availability(&app);
        loop {
          std::thread::sleep(INTERVAL);
          let now = availability(&app);
          if now != seen {
            seen = now;
            crate::project::library::notify(&app);
          }
        }
      });
    if let Err(error) = started {
      eprintln!("Could not watch the project locations: {error}");
    }
  });
}

/// Each location and whether it is there now.
fn availability(app: &AppHandle) -> Vec<(PathBuf, bool)> {
  crate::settings::current(app)
    .project_directory
    .or_else(|| crate::project::projects_directory(app).ok())
    .into_iter()
    .chain(crate::project::library::load(app).locations)
    .map(|path| {
      let available = path.is_dir();
      (path, available)
    })
    .collect()
}
