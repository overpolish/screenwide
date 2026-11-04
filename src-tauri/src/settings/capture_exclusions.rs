// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What General's Capture group keeps out of recordings and screenshots,
//! handed to the capture code as the settings load and change.

use screenwide_capture::chosen_exclusions::{self, ChosenExclusions};
use serde::{Deserialize, Serialize};

use super::preferences::GeneralSettings;

/// An app kept out of captures: its bundle identifier, and its name as it
/// was when chosen, so Settings can name it while it is not running.
#[derive(Clone, Debug, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HiddenApp {
  pub id: String,
  pub label: String,
}

/// Hands what the person keeps out of captures to the capture code, which
/// reads it as each capture starts.
pub(super) fn apply(settings: &GeneralSettings) {
  chosen_exclusions::set(ChosenExclusions {
    app_ids: settings
      .hidden_apps
      .iter()
      .map(|app| app.id.clone())
      .collect(),
    desktop: settings.hide_desktop_icons,
  });
}

/// The running apps Settings offers to hide: on Windows only those
/// Screenwide can hide, as an app running as administrator, a protected one
/// or one built for another kind of processor refuses the hider.
#[tauri::command]
pub async fn list_hideable_applications(
  app: tauri::AppHandle,
) -> Result<Vec<crate::recording_sources::ApplicationDetails>, String> {
  let applications = crate::recording_sources::list_applications(app).await?;
  #[cfg(target_os = "windows")]
  let applications = applications
    .into_iter()
    .filter(|application| crate::hidden_apps_windows::can_hide_any(application.process_ids()))
    .collect();
  Ok(applications)
}
