// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
#[derive(Clone, Copy)]
pub(in crate::ruler::snapshot) struct Settle {
  pub(crate) id: u64,
  pub(crate) from: Rect,
  pub(crate) to: Rect,
  pub(crate) started: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ruler::snapshot) struct Measurement {
  pub(crate) id: u64,
  pub(crate) bounds: Rect,
  pub(crate) label: ArtifactLabel,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ruler::snapshot) struct ProbeArtifact {
  pub(crate) id: u64,
  pub(crate) axis: ProbeAxis,
  pub(crate) start: f64,
  pub(crate) end: f64,
  pub(crate) position: f64,
  pub(crate) label: ArtifactLabel,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ruler::snapshot) struct GuideArtifact {
  pub(crate) id: u64,
  pub(crate) display_id: u32,
  pub(crate) axis: GuideAxis,
  pub(crate) position: f64,
  pub(crate) anchor: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ruler::snapshot) struct GuideGapArtifact {
  pub(crate) id: u64,
  pub(crate) first_id: u64,
  pub(crate) second_id: u64,
  pub(crate) label: ArtifactLabel,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::ruler::snapshot) struct RadiusArtifact {
  pub(crate) id: u64,
  pub(crate) display_id: u32,
  pub(crate) bounds: Rect,
  pub(crate) corner: Corner,
  pub(crate) radius: f64,
  pub(crate) low_confidence: bool,
  pub(crate) label: ArtifactLabel,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(in crate::ruler::snapshot) struct ArtifactLabel {
  pub(crate) anchor: Option<Point>,
  pub(crate) hidden: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::ruler::snapshot) struct Document {
  pub(crate) measurements: Vec<Measurement>,
  pub(crate) probes: Vec<ProbeArtifact>,
  pub(crate) guides: Vec<GuideArtifact>,
  pub(crate) guide_gaps: Vec<GuideGapArtifact>,
  pub(crate) radii: Vec<RadiusArtifact>,
  pub(crate) next_id: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ruler::snapshot) enum HoverTarget {
  Measurement(u64),
  Probe(u64),
  Guide(u64),
  GuideGap(u64),
  Radius(u64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LabelKind {
  Measurement,
  Probe,
  GuideGap,
  Radius,
}

#[derive(Clone, Copy)]
pub(in crate::ruler::snapshot) struct LabelDrag {
  pub(crate) target: HoverTarget,
  pub(crate) start_screen: Point,
  pub(crate) grab_offset: Point,
  pub(crate) changed: bool,
}

#[derive(Clone, Copy)]
pub(in crate::ruler::snapshot) struct RangeGesture {
  pub(crate) axis: ProbeAxis,
  pub(crate) start_pointer: RulerPointer,
  pub(crate) start_probe: RulerProbeVisual,
  pub(crate) draft: RulerProbeVisual,
}

#[derive(Clone, Copy)]
pub(in crate::ruler::snapshot) struct GuideGesture {
  pub(crate) visual: RulerGuideVisual,
  pub(crate) snapped: bool,
}

#[derive(Clone, Copy)]
pub(in crate::ruler::snapshot) struct GuideDrag {
  pub(crate) id: u64,
  pub(crate) start_screen: Point,
  pub(crate) original: GuideArtifact,
  pub(crate) changed: bool,
  pub(crate) snapped: bool,
}

#[derive(Clone, Copy)]
pub(in crate::ruler::snapshot) struct HoverExit {
  pub(crate) target: HoverTarget,
  pub(crate) started: Instant,
}

#[derive(Clone, Copy)]
pub(in crate::ruler::snapshot) struct RadiusGesture {
  pub(crate) visual: Option<RulerRadiusVisual>,
}

#[derive(Clone)]
pub(in crate::ruler::snapshot) struct CenterAidCache {
  pub(crate) document: Document,
  pub(crate) lines: Vec<RulerCenterlineVisual>,
  pub(crate) objects: Vec<RulerInnerObjectVisual>,
}
