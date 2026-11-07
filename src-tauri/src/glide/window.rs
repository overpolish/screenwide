// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! Glide's preview window, kept only while Glide may be used: switched on and
//! licensed, in its trial or before the trial has begun. A gesture has to find
//! it already built, so it is built as soon as that becomes true and destroyed
//! as soon as it stops, rather than when a gesture asks. With the trial over
//! and no key, a gesture opens Settings on the Licences pane without the
//! preview's lock.

use std::time::Duration;

use tauri::{AppHandle, Listener};

use crate::app_windows::{on_demand, WindowLabel};

/// How often the trial's end is looked for, which nothing else announces.
const TRIAL_CHECK: Duration = Duration::from_secs(10 * 60);

fn wanted(app: &AppHandle) -> bool {
  super::settings::current(app).enabled
    && crate::licensing::allows(app, crate::licensing::Tool::Glide)
}

/// Builds or destroys the window to match whether Glide may be used now.
pub(crate) fn reconcile(app: &AppHandle) {
  if wanted(app) {
    on_demand::prepare(app, WindowLabel::Glide);
  } else {
    on_demand::close(app, WindowLabel::Glide);
  }
}

/// Matches the window to Glide now, as its settings change, and again
/// whenever the trial may have run out. Licence changes reconcile as they are
/// made (`crate::licensing`).
pub(super) fn start(app: &AppHandle) {
  reconcile(app);
  let changed = app.clone();
  app.listen(super::settings::SETTINGS_CHANGED_EVENT, move |_| {
    reconcile(&changed);
  });
  let app = app.clone();
  let watched = std::thread::Builder::new()
    .name("glide-window".to_owned())
    .spawn(move || loop {
      std::thread::sleep(TRIAL_CHECK);
      reconcile(&app);
    });
  if let Err(error) = watched {
    eprintln!("Could not watch Glide's licence: {error}");
  }
}
