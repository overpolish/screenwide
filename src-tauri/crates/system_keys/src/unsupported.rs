// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Only Windows' keys are taken so far; elsewhere nothing starts.

use super::OnKey;

pub(super) const SUPPORTED: bool = false;

pub(super) struct Hook;

impl Hook {
  pub(super) fn start(_on_key: OnKey) -> Result<Self, String> {
    Err("The system's screenshot keys cannot be taken on this platform".to_owned())
  }
}
