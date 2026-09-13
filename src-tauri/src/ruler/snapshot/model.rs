// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
mod artifacts;
pub(crate) use artifacts::*;

pub(in crate::ruler::snapshot) const COPIED_FEEDBACK_DURATION: Duration =
  Duration::from_millis(900);
pub(in crate::ruler::snapshot) const TOLERANCE_FEEDBACK_DURATION: Duration =
  Duration::from_millis(900);
pub(in crate::ruler::snapshot) const DRAG_THRESHOLD: f64 = 4.0;
pub(in crate::ruler::snapshot) const SETTLE_DURATION: Duration = Duration::from_millis(180);
pub(in crate::ruler::snapshot) const SETTLE_OVERSHOOT: f64 = 1.15;
pub(in crate::ruler::snapshot) const ARTIFACT_HIT_SLOP: f64 = 6.0;
pub(in crate::ruler::snapshot) const HISTORY_LIMIT: usize = 100;
pub(in crate::ruler::snapshot) const GUIDE_SNAP_RADIUS: f64 = 10.0;
pub(in crate::ruler::snapshot) const GUIDE_RELEASE_RADIUS: f64 = 16.0;
pub(in crate::ruler::snapshot) const HOVER_EXIT_DURATION: Duration = Duration::from_millis(160);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Tolerance {
  ClearEdges,
  #[default]
  Balanced,
  SubtleEdges,
}

impl Tolerance {
  pub(crate) const fn threshold(self) -> u8 {
    match self {
      Self::ClearEdges => 40,
      Self::Balanced => 24,
      Self::SubtleEdges => 5,
    }
  }

  pub(crate) const fn next(self) -> Self {
    match self {
      Self::ClearEdges => Self::Balanced,
      Self::Balanced => Self::SubtleEdges,
      Self::SubtleEdges => Self::ClearEdges,
    }
  }

  pub(crate) const fn index(self) -> usize {
    match self {
      Self::ClearEdges => 0,
      Self::Balanced => 1,
      Self::SubtleEdges => 2,
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RulerMeasurementVisual {
  pub id: u64,
  pub bounds: Rect,
  pub draft: bool,
  pub animating: bool,
  pub hovered: bool,
  pub hover_alpha: f32,
  pub label_anchor: Option<Point>,
  pub label_hidden: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RulerVisual {
  /// Frozen desktop/source coordinate used by analysis and artifacts.
  pub point: Point,
  /// Desktop surface coordinate used by native cursor furniture.
  pub screen_point: Point,
  pub display_id: u32,
  pub zoom: f64,
  pub rgba: [u8; 4],
  pub crosshair: bool,
  pub copied: bool,
}

impl RulerVisual {
  pub fn packed_rgba(self) -> u32 {
    u32::from_be_bytes(self.rgba)
  }

  pub fn hex(self) -> String {
    format!(
      "#{:02X}{:02X}{:02X}",
      self.rgba[0], self.rgba[1], self.rgba[2]
    )
  }
}

pub(in crate::ruler::snapshot) struct DisplaySnapshot {
  pub(crate) display: DesktopDisplay,
  pub(crate) image: CapturedImage,
  pub(crate) gradients: GradientMaps,
  pub(crate) probes: ProbeIndex,
  pub(crate) boxes_by_tolerance: [Vec<ComponentBox>; 3],
  pub(crate) viewport: Viewport,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RulerPointer {
  pub world: Point,
  pub screen: Point,
  pub display_id: u32,
  pub zoom: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RulerViewportVisual {
  pub display_id: u32,
  pub viewport: Viewport,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RulerProbeVisual {
  pub id: u64,
  pub display_id: u32,
  pub axis: ProbeAxis,
  pub start: f64,
  pub end: f64,
  pub position: f64,
  pub draft: bool,
  pub hovered: bool,
  pub hover_alpha: f32,
  pub label_anchor: Option<Point>,
  pub label_hidden: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GuideAxis {
  Vertical,
  Horizontal,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RulerGuideVisual {
  pub id: u64,
  pub display_id: u32,
  pub axis: GuideAxis,
  pub position: f64,
  pub draft: bool,
  pub hovered: bool,
  pub hover_alpha: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RulerGuideGapVisual {
  pub id: u64,
  pub owner_id: u64,
  pub display_id: u32,
  pub axis: ProbeAxis,
  pub start: f64,
  pub end: f64,
  pub position: f64,
  pub hovered: bool,
  pub hover_alpha: f32,
  pub label_anchor: Option<Point>,
  pub label_hidden: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RulerRadiusVisual {
  pub id: u64,
  pub display_id: u32,
  pub bounds: Rect,
  pub corner: Corner,
  pub radius: f64,
  pub low_confidence: bool,
  pub draft: bool,
  pub hovered: bool,
  pub hover_alpha: f32,
  pub label_anchor: Option<Point>,
  pub label_hidden: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RulerCenterlineVisual {
  pub id: u64,
  pub bounds: Rect,
  pub x_accent: bool,
  pub y_accent: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RulerInnerObjectVisual {
  pub owner_id: u64,
  pub bounds: Rect,
  pub aligned_x: bool,
  pub aligned_y: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RangeAxis {
  Horizontal,
  Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ViewportAction {
  Zoom { anchor: Point, factor: f64 },
  Pan { anchor: Point, delta: Point },
  Reset { anchor: Point },
}
