// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the person chose to keep out of their recordings and screenshots,
//! beside Screenwide's own windows: apps, by the identifier each platform
//! names them by, and the desktop's icons and widgets. The app sets it as
//! its settings load and change; each capture reads it as it starts.

use std::sync::RwLock;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ChosenExclusions {
  /// macOS bundle identifiers of the apps left out.
  pub app_ids: Vec<String>,
  /// Whether the desktop's icons and widgets are left out.
  pub desktop: bool,
}

static CHOSEN: RwLock<ChosenExclusions> = RwLock::new(ChosenExclusions {
  app_ids: Vec::new(),
  desktop: false,
});

pub fn set(chosen: ChosenExclusions) {
  *CHOSEN
    .write()
    .unwrap_or_else(std::sync::PoisonError::into_inner) = chosen;
}

pub fn current() -> ChosenExclusions {
  CHOSEN
    .read()
    .unwrap_or_else(std::sync::PoisonError::into_inner)
    .clone()
}
