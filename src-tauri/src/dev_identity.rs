// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Debug builds run as Screenwide Dev, a separate app from an installed
//! release, so developing never disturbs the copy in use. The identifier
//! names the settings, data and cache folders and, on Windows, the
//! single-instance lock; the name labels the launch-at-login entry and the
//! default projects folder. On macOS the dev bundle's Info.plist
//! (`scripts/macos-dev-info.plist`) carries the same identifier and name,
//! which keeps privacy permissions and webview storage apart as well.

const NAME: &str = "Screenwide Dev";
const IDENTIFIER: &str = "co.overpolish.screenwide.dev";

pub fn apply<R: tauri::Runtime>(mut context: tauri::Context<R>) -> tauri::Context<R> {
  let config = context.config_mut();
  config.identifier = IDENTIFIER.into();
  config.product_name = Some(NAME.into());
  context.package_info_mut().name = NAME.into();
  context
}
