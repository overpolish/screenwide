// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{OnMain, Refusal};

pub(super) fn paste(_text: &str, _on_main: OnMain<'_>) -> Result<(), Refusal> {
  Err(Refusal::Failed(
    "Pasting is only built for macOS and Windows".to_owned(),
  ))
}
