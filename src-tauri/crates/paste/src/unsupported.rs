// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{OnMain, Refusal};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Target;

impl Target {
  pub(super) const fn current() -> Option<Self> {
    None
  }
}

pub(super) const fn bring_back(_target: Target, _on_main: OnMain<'_>) -> bool {
  false
}

pub(super) const fn is_in_front(_target: Target, _on_main: OnMain<'_>) -> bool {
  false
}

pub(super) fn paste(_text: &str, _on_main: OnMain<'_>) -> Result<(), Refusal> {
  Err(Refusal::Failed(
    "Pasting is only built for macOS and Windows".to_owned(),
  ))
}
