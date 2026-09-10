// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum RulerAction {
  ToggleCrosshair,
  CopyColour,
  DeleteMeasurement,
  CopyMeasurement,
  Undo,
  Redo,
  StampHorizontal,
  StampVertical,
  GuideVertical,
  GuideHorizontal,
  CycleTolerance,
  MeasureRadius,
  ToggleCenterlines,
}

pub fn defaults() -> BTreeMap<RulerAction, Option<String>> {
  use RulerAction::*;
  [
    (ToggleCrosshair, "KeyX"),
    (CopyColour, "Tab"),
    (DeleteMeasurement, "Backspace"),
    (CopyMeasurement, "CommandOrControl+KeyC"),
    (Undo, "CommandOrControl+KeyZ"),
    (Redo, "CommandOrControl+Shift+KeyZ"),
    (StampHorizontal, "Digit1"),
    (StampVertical, "Digit2"),
    (GuideVertical, "KeyV"),
    (GuideHorizontal, "KeyH"),
    (CycleTolerance, "KeyT"),
    (MeasureRadius, "KeyR"),
    (ToggleCenterlines, "KeyM"),
  ]
  .into_iter()
  .map(|(action, key)| (action, Some(key.to_owned())))
  .collect()
}

impl RulerAction {
  pub fn phase(self) -> u32 {
    use RulerAction::*;
    match self {
      ToggleCrosshair => 13,
      CopyColour => 14,
      DeleteMeasurement => 16,
      CopyMeasurement => 17,
      Undo => 18,
      Redo => 19,
      StampHorizontal => 20,
      StampVertical => 21,
      GuideVertical => 26,
      GuideHorizontal => 27,
      CycleTolerance => 29,
      MeasureRadius => 31,
      ToggleCenterlines => 33,
    }
  }
  pub fn release(self) -> Option<u32> {
    match self {
      Self::StampHorizontal | Self::StampVertical => Some(22),
      Self::GuideVertical | Self::GuideHorizontal => Some(28),
      Self::MeasureRadius => Some(32),
      _ => None,
    }
  }
}
