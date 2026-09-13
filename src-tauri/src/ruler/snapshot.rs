// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

mod snapping;
use snapping::snap_bounds;
mod helpers;
use helpers::*;
mod model;
use model::*;
pub(crate) use model::{
  GuideAxis, LabelKind, RangeAxis, RulerCenterlineVisual, RulerGuideGapVisual, RulerGuideVisual,
  RulerInnerObjectVisual, RulerMeasurementVisual, RulerProbeVisual, RulerRadiusVisual,
  RulerViewportVisual, RulerVisual, Tolerance, ViewportAction,
};

use std::{
  sync::Mutex,
  time::{Duration, Instant},
};

use crate::{
  osc::{
    desktop::DesktopDisplay,
    geometry::{Point, Rect},
  },
  ruler::viewport::Viewport,
  ruler::{
    analysis::{compute_gradients, detect_boxes, ComponentBox, GradientMaps},
    centerlines,
    probe::{probes_at_threshold, ProbeAxis, ProbeIndex},
    radius::{corner_radius_at, Corner},
  },
  screenshots::CapturedImage,
};

#[derive(Default)]
struct Drag {
  pending: Option<RulerPointer>,
  start: Option<RulerPointer>,
  draft: Option<Rect>,
}

#[derive(Default)]
struct Session {
  active: bool,
  generation: u64,
  displays: Vec<DisplaySnapshot>,
  visual: Option<RulerVisual>,
  copied_until: Option<Instant>,
  tolerance: Tolerance,
  tolerance_until: Option<Instant>,
  option_active: bool,
  boxes: Vec<Rect>,
  drag: Drag,
  document: Document,
  undo: Vec<Document>,
  redo: Vec<Document>,
  hovered_target: Option<HoverTarget>,
  hover_exit: Option<HoverExit>,
  label_drag: Option<LabelDrag>,
  range: Option<RangeGesture>,
  guide: Option<GuideGesture>,
  guide_drag: Option<GuideDrag>,
  radius: Option<RadiusGesture>,
  centerlines_visible: bool,
  center_aid_cache: Option<CenterAidCache>,
  settle: Option<Settle>,
}

#[derive(Default)]
pub struct RulerState(Mutex<Session>);

mod artifact_accessors;
mod gesture_modes;
mod labels_viewport;
mod lifecycle;
mod mutation_history;
mod pointer_input;
mod selection_input;
mod session_control;
#[cfg(test)]
mod tests;
