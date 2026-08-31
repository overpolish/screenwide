// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Stable Ruler draw packets shared by every native compositor adapter.

use super::{
  probe::ProbeAxis,
  snapshot::{
    GuideAxis, RulerCenterlineVisual, RulerGuideGapVisual, RulerGuideVisual,
    RulerInnerObjectVisual, RulerMeasurementVisual, RulerProbeVisual, RulerRadiusVisual,
    RulerViewportVisual,
  },
};

const fn hover_alpha_byte(alpha: f32) -> u8 {
  (alpha.clamp(0.0, 1.0) * 255.0).round() as u8
}

const fn label_anchor(anchor: Option<crate::osc::geometry::Point>) -> (f64, f64) {
  match anchor {
    Some(point) => (point.x, point.y),
    None => (f64::NAN, f64::NAN),
  }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct MeasurementPacket {
  pub id: u64,
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
  pub flags: u8,
  pub padding: [u8; 7],
  pub label_anchor_x: f64,
  pub label_anchor_y: f64,
}

impl From<&RulerMeasurementVisual> for MeasurementPacket {
  fn from(value: &RulerMeasurementVisual) -> Self {
    let (label_anchor_x, label_anchor_y) = label_anchor(value.label_anchor);
    let mut padding = [0; 7];
    padding[0] = hover_alpha_byte(value.hover_alpha);
    Self {
      id: value.id,
      x: value.bounds.origin.x,
      y: value.bounds.origin.y,
      width: value.bounds.size.width,
      height: value.bounds.size.height,
      flags: u8::from(value.draft)
        | u8::from(value.animating) << 1
        | u8::from(value.hovered) << 2
        | u8::from(value.label_hidden) << 3,
      padding,
      label_anchor_x,
      label_anchor_y,
    }
  }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ViewportPacket {
  pub display_id: u32,
  pub padding: u32,
  pub zoom: f64,
  pub origin_x: f64,
  pub origin_y: f64,
}

impl From<&RulerViewportVisual> for ViewportPacket {
  fn from(value: &RulerViewportVisual) -> Self {
    Self {
      display_id: value.display_id,
      zoom: value.viewport.zoom,
      origin_x: value.viewport.origin.x,
      origin_y: value.viewport.origin.y,
      ..Default::default()
    }
  }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct ProbePacket {
  pub id: u64,
  pub display_id: u32,
  pub axis: u8,
  pub flags: u8,
  pub padding: [u8; 2],
  pub start: f64,
  pub end: f64,
  pub position: f64,
  pub label_anchor_x: f64,
  pub label_anchor_y: f64,
}

impl From<&RulerProbeVisual> for ProbePacket {
  fn from(value: &RulerProbeVisual) -> Self {
    let (label_anchor_x, label_anchor_y) = label_anchor(value.label_anchor);
    Self {
      id: value.id,
      display_id: value.display_id,
      axis: match value.axis {
        ProbeAxis::Horizontal => 1,
        ProbeAxis::Vertical => 2,
      },
      flags: u8::from(value.draft)
        | u8::from(value.hovered) << 1
        | u8::from(value.id == 0 && !value.draft) << 2
        | u8::from(value.label_hidden) << 3,
      padding: [hover_alpha_byte(value.hover_alpha), 0],
      start: value.start,
      end: value.end,
      position: value.position,
      label_anchor_x,
      label_anchor_y,
    }
  }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct GuidePacket {
  pub id: u64,
  pub display_id: u32,
  pub axis: u8,
  pub flags: u8,
  pub padding: [u8; 2],
  pub position: f64,
}

impl From<&RulerGuideVisual> for GuidePacket {
  fn from(value: &RulerGuideVisual) -> Self {
    Self {
      id: value.id,
      display_id: value.display_id,
      axis: match value.axis {
        GuideAxis::Vertical => 1,
        GuideAxis::Horizontal => 2,
      },
      flags: u8::from(value.draft) | u8::from(value.hovered) << 1,
      padding: [hover_alpha_byte(value.hover_alpha), 0],
      position: value.position,
    }
  }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct GuideGapPacket {
  pub id: u64,
  pub owner_id: u64,
  pub display_id: u32,
  pub axis: u8,
  pub flags: u8,
  pub padding: [u8; 2],
  pub start: f64,
  pub end: f64,
  pub position: f64,
  pub label_anchor_x: f64,
  pub label_anchor_y: f64,
}

impl From<&RulerGuideGapVisual> for GuideGapPacket {
  fn from(value: &RulerGuideGapVisual) -> Self {
    let (label_anchor_x, label_anchor_y) = label_anchor(value.label_anchor);
    Self {
      id: value.id,
      owner_id: value.owner_id,
      display_id: value.display_id,
      axis: match value.axis {
        ProbeAxis::Horizontal => 1,
        ProbeAxis::Vertical => 2,
      },
      flags: u8::from(value.hovered) | u8::from(value.label_hidden) << 1,
      padding: [hover_alpha_byte(value.hover_alpha), 0],
      start: value.start,
      end: value.end,
      position: value.position,
      label_anchor_x,
      label_anchor_y,
    }
  }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct RadiusPacket {
  pub id: u64,
  pub display_id: u32,
  pub corner: u8,
  pub flags: u8,
  pub padding: [u8; 2],
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
  pub radius: f64,
  pub label_anchor_x: f64,
  pub label_anchor_y: f64,
}

impl From<&RulerRadiusVisual> for RadiusPacket {
  fn from(value: &RulerRadiusVisual) -> Self {
    let (label_anchor_x, label_anchor_y) = label_anchor(value.label_anchor);
    Self {
      id: value.id,
      display_id: value.display_id,
      corner: value.corner as u8,
      flags: u8::from(value.low_confidence)
        | u8::from(value.draft) << 1
        | u8::from(value.hovered) << 2
        | u8::from(value.label_hidden) << 3,
      padding: [hover_alpha_byte(value.hover_alpha), 0],
      x: value.bounds.origin.x,
      y: value.bounds.origin.y,
      width: value.bounds.size.width,
      height: value.bounds.size.height,
      radius: value.radius,
      label_anchor_x,
      label_anchor_y,
    }
  }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct CenterlinePacket {
  pub id: u64,
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
  pub flags: u8,
  pub padding: [u8; 7],
}

impl From<&RulerCenterlineVisual> for CenterlinePacket {
  fn from(value: &RulerCenterlineVisual) -> Self {
    Self {
      id: value.id,
      x: value.bounds.origin.x,
      y: value.bounds.origin.y,
      width: value.bounds.size.width,
      height: value.bounds.size.height,
      flags: u8::from(value.x_accent) | u8::from(value.y_accent) << 1,
      padding: [0; 7],
    }
  }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct InnerObjectPacket {
  pub owner_id: u64,
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
  pub flags: u8,
  pub padding: [u8; 7],
}

impl From<&RulerInnerObjectVisual> for InnerObjectPacket {
  fn from(value: &RulerInnerObjectVisual) -> Self {
    Self {
      owner_id: value.owner_id,
      x: value.bounds.origin.x,
      y: value.bounds.origin.y,
      width: value.bounds.size.width,
      height: value.bounds.size.height,
      flags: u8::from(value.aligned_x) | u8::from(value.aligned_y) << 1,
      padding: [0; 7],
    }
  }
}

const _: () = assert!(std::mem::size_of::<MeasurementPacket>() == 64);
const _: () = assert!(std::mem::offset_of!(MeasurementPacket, flags) == 40);
const _: () = assert!(std::mem::offset_of!(MeasurementPacket, label_anchor_x) == 48);
const _: () = assert!(std::mem::size_of::<ViewportPacket>() == 32);
const _: () = assert!(std::mem::offset_of!(ViewportPacket, zoom) == 8);
const _: () = assert!(std::mem::size_of::<ProbePacket>() == 56);
const _: () = assert!(std::mem::offset_of!(ProbePacket, start) == 16);
const _: () = assert!(std::mem::offset_of!(ProbePacket, label_anchor_x) == 40);
const _: () = assert!(std::mem::size_of::<GuidePacket>() == 24);
const _: () = assert!(std::mem::offset_of!(GuidePacket, position) == 16);
const _: () = assert!(std::mem::size_of::<GuideGapPacket>() == 64);
const _: () = assert!(std::mem::offset_of!(GuideGapPacket, start) == 24);
const _: () = assert!(std::mem::offset_of!(GuideGapPacket, label_anchor_x) == 48);
const _: () = assert!(std::mem::size_of::<RadiusPacket>() == 72);
const _: () = assert!(std::mem::offset_of!(RadiusPacket, x) == 16);
const _: () = assert!(std::mem::offset_of!(RadiusPacket, label_anchor_x) == 56);
const _: () = assert!(std::mem::size_of::<CenterlinePacket>() == 48);
const _: () = assert!(std::mem::offset_of!(CenterlinePacket, flags) == 40);
const _: () = assert!(std::mem::size_of::<InnerObjectPacket>() == 48);
const _: () = assert!(std::mem::offset_of!(InnerObjectPacket, flags) == 40);
