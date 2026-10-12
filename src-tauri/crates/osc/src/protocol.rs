// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Platform-neutral input and result contract for native OSC hosts.
//!
//! Platform adapters translate their native events into [`InputPhase`] values
//! and consume [`OscResult`] without owning tool behaviour. The C-compatible
//! layout is also used by the current macOS host.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Purpose {
  Region,
  Ruler,
  TextRecognition,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputPhase {
  Hover = 1,
  Down = 2,
  Drag = 3,
  Up = 4,
  Cancel = 5,
  OcrSelectAll = 6,
  OcrCopy = 7,
  OcrCancel = 8,
  OcrCopyAll = 9,
  OcrCopyParagraph = 10,
  OcrReset = 11,
  OcrClose = 12,
  RulerToggleCrosshair = 13,
  RulerCopyColour = 14,
  RulerAnimationFrame = 15,
  RulerDeleteMeasurement = 16,
  RulerCopyMeasurement = 17,
  RulerUndo = 18,
  RulerRedo = 19,
  RulerBeginHorizontalRange = 20,
  RulerBeginVerticalRange = 21,
  RulerFinishRange = 22,
  RulerCancelRange = 23,
  RulerHoverProbeLabel = 24,
  RulerHoverMeasurementLabel = 25,
  RulerBeginVerticalGuide = 26,
  RulerBeginHorizontalGuide = 27,
  RulerCancelGuide = 28,
  RulerCycleTolerance = 29,
  RulerSetOptionActive = 30,
  RulerBeginRadius = 31,
  RulerCancelRadius = 32,
  RulerToggleCenterlines = 33,
}

impl InputPhase {
  pub fn from_raw(value: u32) -> Option<Self> {
    Some(match value {
      1 => Self::Hover,
      2 => Self::Down,
      3 => Self::Drag,
      4 => Self::Up,
      5 => Self::Cancel,
      6 => Self::OcrSelectAll,
      7 => Self::OcrCopy,
      8 => Self::OcrCancel,
      9 => Self::OcrCopyAll,
      10 => Self::OcrCopyParagraph,
      11 => Self::OcrReset,
      12 => Self::OcrClose,
      13 => Self::RulerToggleCrosshair,
      14 => Self::RulerCopyColour,
      15 => Self::RulerAnimationFrame,
      16 => Self::RulerDeleteMeasurement,
      17 => Self::RulerCopyMeasurement,
      18 => Self::RulerUndo,
      19 => Self::RulerRedo,
      20 => Self::RulerBeginHorizontalRange,
      21 => Self::RulerBeginVerticalRange,
      22 => Self::RulerFinishRange,
      23 => Self::RulerCancelRange,
      24 => Self::RulerHoverProbeLabel,
      25 => Self::RulerHoverMeasurementLabel,
      26 => Self::RulerBeginVerticalGuide,
      27 => Self::RulerBeginHorizontalGuide,
      28 => Self::RulerCancelGuide,
      29 => Self::RulerCycleTolerance,
      30 => Self::RulerSetOptionActive,
      31 => Self::RulerBeginRadius,
      32 => Self::RulerCancelRadius,
      33 => Self::RulerToggleCenterlines,
      _ => return None,
    })
  }

  pub fn pointer(self) -> bool {
    matches!(self, Self::Hover | Self::Down | Self::Drag | Self::Up)
  }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InputModifiers {
  pub free_aspect: bool,
  pub additive: bool,
  pub double_click: bool,
  pub option: bool,
}

impl InputModifiers {
  pub fn from_bits(bits: u8) -> Self {
    Self {
      free_aspect: bits & 1 != 0,
      additive: bits & 2 != 0,
      double_click: bits & 4 != 0,
      option: bits & 8 != 0,
    }
  }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CursorIcon {
  #[default]
  Unchanged = 0,
  Crosshair = 1,
  OpenHand = 2,
  ClosedHand = 3,
  HorizontalResize = 4,
  VerticalResize = 5,
  DiagonalResize = 6,
  Arrow = 7,
  IBeam = 8,
  PointingHand = 9,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResultStatus {
  None = 0,
  Changed = 1,
  Finished = 2,
  Cancelled = 3,
  Invalid = 255,
}

pub const RESULT_GESTURE_DRAWING: u8 = 1;
pub const RESULT_GESTURE_MOVING: u8 = 2;
pub const RESULT_GESTURE_RESIZING: u8 = 3;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OscResult {
  pub status: u8,
  pub gesture: u8,
  pub handle: u8,
  pub cursor: u8,
  pub has_region: u8,
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
  pub ruler_color: u32,
  pub ruler_flags: u8,
  pub ruler_padding: [u8; 3],
}

const _: () = assert!(std::mem::size_of::<OscResult>() == 48);
const _: () = assert!(std::mem::offset_of!(OscResult, x) == 8);

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn every_wire_phase_round_trips_through_the_shared_decoder() {
    for raw in 1..=33 {
      assert_eq!(
        InputPhase::from_raw(raw).map(|phase| phase as u32),
        Some(raw)
      );
    }
    assert_eq!(InputPhase::from_raw(0), None);
    assert_eq!(InputPhase::from_raw(34), None);
  }

  #[test]
  fn modifier_bits_have_one_cross_platform_meaning() {
    assert_eq!(
      InputModifiers::from_bits(0b1111),
      InputModifiers {
        free_aspect: true,
        additive: true,
        double_click: true,
        option: true,
      }
    );
  }
}
