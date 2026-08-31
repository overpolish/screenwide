// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

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

const COPIED_FEEDBACK_DURATION: Duration = Duration::from_millis(900);
const TOLERANCE_FEEDBACK_DURATION: Duration = Duration::from_millis(900);
const DRAG_THRESHOLD: f64 = 4.0;
const CONTAINMENT_SLACK: f64 = 12.0;
const MINIMUM_IOU: f64 = 0.25;
const EDGE_SEARCH: f64 = 20.0;
const SETTLE_DURATION: Duration = Duration::from_millis(180);
const SETTLE_OVERSHOOT: f64 = 1.15;
const ARTIFACT_HIT_SLOP: f64 = 6.0;
const HISTORY_LIMIT: usize = 100;
const GUIDE_SNAP_RADIUS: f64 = 10.0;
const GUIDE_RELEASE_RADIUS: f64 = 16.0;
const HOVER_EXIT_DURATION: Duration = Duration::from_millis(160);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Tolerance {
  ClearEdges,
  #[default]
  Balanced,
  SubtleEdges,
}

impl Tolerance {
  const fn threshold(self) -> u8 {
    match self {
      Self::ClearEdges => 40,
      Self::Balanced => 24,
      Self::SubtleEdges => 5,
    }
  }

  const fn next(self) -> Self {
    match self {
      Self::ClearEdges => Self::Balanced,
      Self::Balanced => Self::SubtleEdges,
      Self::SubtleEdges => Self::ClearEdges,
    }
  }

  const fn index(self) -> usize {
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

struct DisplaySnapshot {
  display: DesktopDisplay,
  image: CapturedImage,
  gradients: GradientMaps,
  probes: ProbeIndex,
  boxes_by_tolerance: [Vec<ComponentBox>; 3],
  viewport: Viewport,
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

#[derive(Clone, Copy)]
struct Settle {
  id: u64,
  from: Rect,
  to: Rect,
  started: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Measurement {
  id: u64,
  bounds: Rect,
  label: ArtifactLabel,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ProbeArtifact {
  id: u64,
  axis: ProbeAxis,
  start: f64,
  end: f64,
  position: f64,
  label: ArtifactLabel,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct GuideArtifact {
  id: u64,
  display_id: u32,
  axis: GuideAxis,
  position: f64,
  anchor: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct GuideGapArtifact {
  id: u64,
  first_id: u64,
  second_id: u64,
  label: ArtifactLabel,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct RadiusArtifact {
  id: u64,
  display_id: u32,
  bounds: Rect,
  corner: Corner,
  radius: f64,
  low_confidence: bool,
  label: ArtifactLabel,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct ArtifactLabel {
  anchor: Option<Point>,
  hidden: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Document {
  measurements: Vec<Measurement>,
  probes: Vec<ProbeArtifact>,
  guides: Vec<GuideArtifact>,
  guide_gaps: Vec<GuideGapArtifact>,
  radii: Vec<RadiusArtifact>,
  next_id: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HoverTarget {
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
struct LabelDrag {
  target: HoverTarget,
  start_screen: Point,
  grab_offset: Point,
  changed: bool,
}

#[derive(Clone, Copy)]
struct RangeGesture {
  axis: ProbeAxis,
  start_pointer: RulerPointer,
  start_probe: RulerProbeVisual,
  draft: RulerProbeVisual,
}

#[derive(Clone, Copy)]
struct GuideGesture {
  visual: RulerGuideVisual,
  snapped: bool,
}

#[derive(Clone, Copy)]
struct GuideDrag {
  id: u64,
  start_screen: Point,
  original: GuideArtifact,
  changed: bool,
  snapped: bool,
}

#[derive(Clone, Copy)]
struct HoverExit {
  target: HoverTarget,
  started: Instant,
}

#[derive(Clone, Copy)]
struct RadiusGesture {
  visual: Option<RulerRadiusVisual>,
}

#[derive(Clone)]
struct CenterAidCache {
  document: Document,
  lines: Vec<RulerCenterlineVisual>,
  objects: Vec<RulerInnerObjectVisual>,
}

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

impl RulerState {
  pub(super) fn begin(&self) -> u64 {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.generation = session.generation.wrapping_add(1);
    session.active = true;
    session.generation
  }

  pub(super) fn is_current(&self, generation: u64) -> bool {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.active && session.generation == generation
  }

  pub(super) fn install(
    &self,
    generation: u64,
    displays: &[DesktopDisplay],
    snapshots: &[(u32, CapturedImage)],
  ) -> bool {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session.active || session.generation != generation {
      return false;
    }
    session.displays = displays
      .iter()
      .filter_map(|display| {
        snapshots
          .iter()
          .find(|(id, _)| *id == display.id)
          .map(|(_, image)| {
            let gradients = compute_gradients(&image.rgba, image.width, image.height);
            let probes = ProbeIndex::new(&gradients, Tolerance::Balanced.threshold());
            let boxes_by_tolerance = [
              detect_boxes(&gradients, Tolerance::ClearEdges.threshold()),
              detect_boxes(&gradients, Tolerance::Balanced.threshold()),
              detect_boxes(&gradients, Tolerance::SubtleEdges.threshold()),
            ];
            DisplaySnapshot {
              display: *display,
              image: image.clone(),
              gradients,
              probes,
              boxes_by_tolerance,
              viewport: Viewport::default(),
            }
          })
      })
      .collect();
    session.tolerance = Tolerance::Balanced;
    session.boxes = session
      .displays
      .iter()
      .flat_map(|snapshot| detected_boxes(snapshot, Tolerance::Balanced))
      .collect();
    session.visual = None;
    session.copied_until = None;
    session.tolerance_until = None;
    session.option_active = false;
    session.drag = Drag::default();
    session.document = Document::default();
    session.undo.clear();
    session.redo.clear();
    session.hovered_target = None;
    session.hover_exit = None;
    session.label_drag = None;
    session.range = None;
    session.guide = None;
    session.guide_drag = None;
    session.radius = None;
    session.centerlines_visible = true;
    session.center_aid_cache = None;
    session.settle = None;
    session.displays.len() == displays.len()
  }

  pub(crate) fn map_pointer(&self, screen: Point) -> Option<RulerPointer> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    map_pointer(&session.displays, screen)
  }

  pub(crate) fn hover(&self, pointer: RulerPointer) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session.active {
      return None;
    }
    let rgba = sample(&session.displays, pointer.world)?;
    let now = Instant::now();
    let copied = session.copied_until.is_some_and(|deadline| deadline > now);
    let crosshair = session.visual.is_some_and(|visual| visual.crosshair);
    let hovered_target = if session.range.is_none()
      && session.guide.is_none()
      && session.guide_drag.is_none()
      && session.radius.is_none()
    {
      hit_test_artifact(&session, pointer)
    } else {
      None
    };
    update_hover_target(&mut session, hovered_target, Instant::now());
    update_range(&mut session, pointer);
    update_guide(&mut session, pointer);
    update_radius(&mut session, pointer);
    let visual = RulerVisual {
      point: pointer.world,
      screen_point: pointer.screen,
      display_id: pointer.display_id,
      zoom: pointer.zoom,
      rgba,
      crosshair,
      copied,
    };
    session.visual = Some(visual);
    Some(visual)
  }

  pub(crate) fn pointer_down(&self, pointer: RulerPointer) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session.active || sample(&session.displays, pointer.world).is_none() {
      return None;
    }
    if session.range.is_some() {
      update_range(&mut session, pointer);
      return refresh_visual(&mut session, pointer);
    }
    if session.guide.is_some() {
      update_guide(&mut session, pointer);
      let guide = session.guide?.visual;
      record_history(&mut session);
      session.document.next_id = session.document.next_id.wrapping_add(1).max(1);
      let id = session.document.next_id;
      session.document.guides.push(GuideArtifact {
        id,
        display_id: guide.display_id,
        axis: guide.axis,
        position: guide.position,
        anchor: guide_cross_axis_position(guide.axis, pointer.world),
      });
      reconcile_guide_gaps(&mut session.document);
      return refresh_visual(&mut session, pointer);
    }
    if session.radius.is_some() {
      update_radius(&mut session, pointer);
      if let Some(radius) = session.radius.and_then(|gesture| gesture.visual) {
        let duplicate = session.document.radii.iter().any(|item| {
          item.display_id == radius.display_id
            && item.bounds == radius.bounds
            && item.corner == radius.corner
            && (item.radius - radius.radius).abs() < f64::EPSILON
        });
        if !duplicate {
          record_history(&mut session);
          session.document.next_id = session.document.next_id.wrapping_add(1).max(1);
          let id = session.document.next_id;
          session.document.radii.push(RadiusArtifact {
            id,
            display_id: radius.display_id,
            bounds: radius.bounds,
            corner: radius.corner,
            radius: radius.radius,
            low_confidence: radius.low_confidence,
            label: ArtifactLabel::default(),
          });
        }
      }
      return refresh_visual(&mut session, pointer);
    }
    if let Some(HoverTarget::Guide(id)) = session.hovered_target {
      let original = session
        .document
        .guides
        .iter()
        .find(|guide| guide.id == id)
        .copied()?;
      session.drag = Drag::default();
      session.label_drag = None;
      session.settle = None;
      session.guide_drag = Some(GuideDrag {
        id,
        start_screen: pointer.screen,
        original,
        changed: false,
        snapped: false,
      });
      return refresh_visual(&mut session, pointer);
    }
    session.drag = Drag {
      pending: Some(pointer),
      ..Default::default()
    };
    session.hovered_target = None;
    session.label_drag = None;
    session.settle = None;
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn pointer_drag(&self, pointer: RulerPointer) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session.active {
      return None;
    }
    if session.guide.is_some() {
      update_guide(&mut session, pointer);
      return refresh_visual(&mut session, pointer);
    }
    if session.radius.is_some() {
      update_radius(&mut session, pointer);
      return refresh_visual(&mut session, pointer);
    }
    if session.guide_drag.is_some() {
      update_guide_drag_session(&mut session, pointer)?;
      return refresh_visual(&mut session, pointer);
    }
    if session.drag.start.is_none() {
      let pending = session.drag.pending?;
      if (pointer.screen.x - pending.screen.x).hypot(pointer.screen.y - pending.screen.y)
        >= DRAG_THRESHOLD
      {
        session.drag.start = Some(pending);
        session.hovered_target = None;
      }
    }
    if let Some(start) = session.drag.start {
      session.drag.draft = Some(ordered_rect(start.world, pointer.world));
    }
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn pointer_up(&self, pointer: RulerPointer) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session.active {
      return None;
    }
    if session.guide.is_some() {
      update_guide(&mut session, pointer);
      session.drag = Drag::default();
      return refresh_visual(&mut session, pointer);
    }
    if session.radius.is_some() {
      update_radius(&mut session, pointer);
      session.drag = Drag::default();
      return refresh_visual(&mut session, pointer);
    }
    if session.guide_drag.is_some() {
      update_guide_drag_session(&mut session, pointer)?;
      let id = session.guide_drag.take()?.id;
      let visual = refresh_visual(&mut session, pointer)?;
      session.hovered_target = Some(HoverTarget::Guide(id));
      return Some(visual);
    }
    if let Some(start) = session.drag.start {
      let raw = ordered_rect(start.world, pointer.world);
      if raw.size.width >= 2.0 || raw.size.height >= 2.0 {
        let snapped = snap_bounds(&session.boxes, raw);
        record_history(&mut session);
        session.document.next_id = session.document.next_id.wrapping_add(1).max(1);
        let id = session.document.next_id;
        session.document.measurements.push(Measurement {
          id,
          bounds: snapped,
          label: ArtifactLabel::default(),
        });
        session.hovered_target = Some(HoverTarget::Measurement(id));
        session.settle = settle_worthwhile(raw, snapped).then(|| Settle {
          id,
          from: raw,
          to: snapped,
          started: Instant::now(),
        });
      }
    }
    session.drag = Drag::default();
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn cancel_pointer(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.drag = Drag::default();
    session.guide_drag = None;
    session.radius = None;
    let pointer = pointer_from_visual(session.visual?);
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn animation_frame(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let pointer = pointer_from_visual(session.visual?);
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn toggle_crosshair(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut visual = session.visual?;
    visual.crosshair = !visual.crosshair;
    visual.copied = session
      .copied_until
      .is_some_and(|deadline| deadline > Instant::now());
    session.visual = Some(visual);
    Some(visual)
  }

  pub(crate) fn toggle_centerlines(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.centerlines_visible = !session.centerlines_visible;
    session.visual
  }

  pub(crate) fn cycle_tolerance(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let pointer = pointer_from_visual(session.visual?);
    session.tolerance = session.tolerance.next();
    let tolerance = session.tolerance;
    session.boxes = session
      .displays
      .iter()
      .flat_map(|snapshot| detected_boxes(snapshot, tolerance))
      .collect();
    session.center_aid_cache = None;
    session.tolerance_until = Some(Instant::now() + TOLERANCE_FEEDBACK_DURATION);
    if let Some(guide) = &mut session.guide {
      guide.snapped = false;
    }
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn tolerance_notice(&self) -> Option<Tolerance> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session
      .tolerance_until
      .is_some_and(|deadline| deadline > Instant::now())
      .then_some(session.tolerance)
  }

  pub(crate) fn interaction_active(&self) -> bool {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.drag.pending.is_some()
      || session.drag.start.is_some()
      || session.label_drag.is_some()
      || session.range.is_some()
      || session.guide.is_some()
      || session.guide_drag.is_some()
      || session.radius.is_some()
  }

  pub(crate) fn hover_fade_active(&self) -> bool {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    expire_hover_exit(&mut session, Instant::now());
    session.hover_exit.is_some()
  }

  pub(crate) fn set_option_active(&self, active: bool) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.option_active = active;
    let pointer = pointer_from_visual(session.visual?);
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn copy_colour(&self) -> Option<(RulerVisual, String)> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut visual = session.visual?;
    let text = visual.hex();
    session.copied_until = Some(Instant::now() + COPIED_FEEDBACK_DURATION);
    visual.copied = true;
    session.visual = Some(visual);
    Some((visual, text))
  }

  pub(crate) fn copy_latest_artifact(&self) -> Option<(RulerVisual, String)> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let target = latest_target(&session.document)?;
    let text = artifact_text(&session.document, target)?;
    let mut visual = session.visual?;
    session.copied_until = Some(Instant::now() + COPIED_FEEDBACK_DURATION);
    visual.copied = true;
    session.visual = Some(visual);
    Some((visual, text))
  }

  pub(crate) fn delete_targeted_artifact(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let target = session
      .hovered_target
      .or_else(|| latest_target(&session.document))?;
    record_history(&mut session);
    match target {
      HoverTarget::Measurement(id) => {
        let index = session
          .document
          .measurements
          .iter()
          .position(|item| item.id == id)?;
        session.document.measurements.remove(index);
      }
      HoverTarget::Probe(id) => {
        let index = session
          .document
          .probes
          .iter()
          .position(|item| item.id == id)?;
        session.document.probes.remove(index);
      }
      HoverTarget::Guide(id) => {
        let index = session
          .document
          .guides
          .iter()
          .position(|item| item.id == id)?;
        session.document.guides.remove(index);
        reconcile_guide_gaps(&mut session.document);
      }
      HoverTarget::GuideGap(id) => {
        let owner_id = session
          .document
          .guide_gaps
          .iter()
          .find(|item| item.id == id)?
          .second_id;
        let index = session
          .document
          .guides
          .iter()
          .position(|item| item.id == owner_id)?;
        session.document.guides.remove(index);
        reconcile_guide_gaps(&mut session.document);
      }
      HoverTarget::Radius(id) => {
        let index = session
          .document
          .radii
          .iter()
          .position(|item| item.id == id)?;
        session.document.radii.remove(index);
      }
    }
    session.hovered_target = None;
    session.hover_exit = None;
    session.label_drag = None;
    session.settle = None;
    let pointer = pointer_from_visual(session.visual?);
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn undo(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = session.undo.pop()?;
    let current = std::mem::replace(&mut session.document, previous);
    session.redo.push(current);
    clear_transient_artifact_state(&mut session);
    let pointer = pointer_from_visual(session.visual?);
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn redo(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let next = session.redo.pop()?;
    let current = std::mem::replace(&mut session.document, next);
    session.undo.push(current);
    trim_history(&mut session.undo);
    clear_transient_artifact_state(&mut session);
    let pointer = pointer_from_visual(session.visual?);
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn measurements(&self) -> Vec<RulerMeasurementVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    measurement_visuals(&mut session, Instant::now())
  }

  pub(crate) fn viewports(&self) -> Vec<RulerViewportVisual> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session
      .displays
      .iter()
      .map(|snapshot| RulerViewportVisual {
        display_id: snapshot.display.id,
        viewport: snapshot.viewport,
      })
      .collect()
  }

  pub(crate) fn probes(&self) -> Vec<RulerProbeVisual> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    probe_visuals(&session)
  }

  pub(crate) fn guides(&self) -> Vec<RulerGuideVisual> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let now = Instant::now();
    let mut guides = session
      .document
      .guides
      .iter()
      .map(|guide| RulerGuideVisual {
        id: guide.id,
        display_id: guide.display_id,
        axis: guide.axis,
        position: guide.position,
        draft: false,
        hovered: session.hovered_target == Some(HoverTarget::Guide(guide.id)),
        hover_alpha: hover_alpha(&session, HoverTarget::Guide(guide.id), now),
      })
      .collect::<Vec<_>>();
    if let Some(gesture) = session.guide {
      guides.push(gesture.visual);
    }
    guides
  }

  pub(crate) fn guide_gaps(&self) -> Vec<RulerGuideGapVisual> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    guide_gap_visuals(&session)
  }

  pub(crate) fn hovered_guide_axis(&self) -> Option<GuideAxis> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let HoverTarget::Guide(id) = session.hovered_target? else {
      return None;
    };
    session
      .document
      .guides
      .iter()
      .find(|guide| guide.id == id)
      .map(|guide| guide.axis)
  }

  pub(crate) fn radii(&self) -> Vec<RulerRadiusVisual> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let now = Instant::now();
    let mut radii = session
      .document
      .radii
      .iter()
      .map(|radius| RulerRadiusVisual {
        id: radius.id,
        display_id: radius.display_id,
        bounds: radius.bounds,
        corner: radius.corner,
        radius: radius.radius,
        low_confidence: radius.low_confidence,
        draft: false,
        hovered: session.hovered_target == Some(HoverTarget::Radius(radius.id)),
        hover_alpha: hover_alpha(&session, HoverTarget::Radius(radius.id), now),
        label_anchor: radius.label.anchor,
        label_hidden: radius.label.hidden,
      })
      .collect::<Vec<_>>();
    if let Some(visual) = session.radius.and_then(|gesture| gesture.visual) {
      let already_stamped = radii.iter().any(|radius| {
        radius.display_id == visual.display_id
          && radius.bounds == visual.bounds
          && radius.corner == visual.corner
          && (radius.radius - visual.radius).abs() < f64::EPSILON
      });
      if !already_stamped {
        radii.push(visual);
      }
    }
    radii
  }

  pub(crate) fn center_aids(&self) -> (Vec<RulerCenterlineVisual>, Vec<RulerInnerObjectVisual>) {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session.centerlines_visible {
      return (Vec::new(), Vec::new());
    }
    center_aid_visuals(&mut session, Instant::now())
  }

  pub(crate) fn begin_radius(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if session.radius.is_some() || session.range.is_some() || session.guide.is_some() {
      return None;
    }
    let visual = session.visual?;
    session.drag = Drag::default();
    session.hovered_target = None;
    session.label_drag = None;
    session.settle = None;
    session.radius = Some(RadiusGesture { visual: None });
    update_radius(&mut session, pointer_from_visual(visual));
    Some(visual)
  }

  pub(crate) fn cancel_radius(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.radius.take()?;
    session.visual
  }

  pub(crate) fn begin_guide(&self, axis: GuideAxis) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if session.guide.is_some() || session.range.is_some() || session.radius.is_some() {
      return None;
    }
    let visual = session.visual?;
    let pointer = pointer_from_visual(visual);
    session.drag = Drag::default();
    session.hovered_target = None;
    session.label_drag = None;
    session.settle = None;
    session.guide = Some(GuideGesture {
      visual: RulerGuideVisual {
        id: 0,
        display_id: pointer.display_id,
        axis,
        position: guide_pointer_position(axis, pointer.world),
        draft: true,
        hovered: false,
        hover_alpha: 0.0,
      },
      snapped: false,
    });
    update_guide(&mut session, pointer);
    Some(visual)
  }

  pub(crate) fn cancel_guide(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.guide.take()?;
    session.visual
  }

  pub(crate) fn begin_range(&self, axis: RangeAxis) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if session.range.is_some() || session.guide.is_some() || session.radius.is_some() {
      return None;
    }
    let visual = session.visual?;
    let pointer = pointer_from_visual(visual);
    let axis = match axis {
      RangeAxis::Horizontal => ProbeAxis::Horizontal,
      RangeAxis::Vertical => ProbeAxis::Vertical,
    };
    let start_probe = automatic_probes(&session, pointer)?
      .into_iter()
      .find(|probe| probe.axis == axis)?;
    session.hovered_target = None;
    session.label_drag = None;
    session.range = Some(RangeGesture {
      axis,
      start_pointer: pointer,
      start_probe,
      draft: RulerProbeVisual {
        draft: true,
        ..start_probe
      },
    });
    Some(visual)
  }

  pub(crate) fn finish_range(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let range = session.range.take()?;
    record_history(&mut session);
    session.document.next_id = session.document.next_id.wrapping_add(1).max(1);
    let id = session.document.next_id;
    session.document.probes.push(ProbeArtifact {
      id,
      axis: range.axis,
      start: range.draft.start,
      end: range.draft.end,
      position: range.draft.position,
      label: ArtifactLabel::default(),
    });
    session.hovered_target = Some(HoverTarget::Probe(id));
    session.visual
  }

  pub(crate) fn cancel_range(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.range.take()?;
    session.visual
  }

  pub(crate) fn hover_probe_label(&self, id: u64) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session.document.probes.iter().any(|probe| probe.id == id) {
      return None;
    }
    session.hovered_target = Some(HoverTarget::Probe(id));
    session.visual
  }

  pub(crate) fn hover_measurement_label(&self, id: u64) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session
      .document
      .measurements
      .iter()
      .any(|measurement| measurement.id == id)
    {
      return None;
    }
    session.hovered_target = Some(HoverTarget::Measurement(id));
    session.visual
  }

  pub(crate) fn hover_guide_gap_label(&self, id: u64) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session.document.guide_gaps.iter().any(|gap| gap.id == id) {
      return None;
    }
    session.hovered_target = Some(HoverTarget::GuideGap(id));
    session.visual
  }

  pub(crate) fn begin_label_drag(
    &self,
    kind: LabelKind,
    id: u64,
    pointer: RulerPointer,
    label_center: RulerPointer,
  ) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let target = label_target(kind, id);
    if label_state(&session.document, target)?.hidden {
      return None;
    }
    session.drag = Drag::default();
    session.range = None;
    session.guide = None;
    session.guide_drag = None;
    session.radius = None;
    session.settle = None;
    session.hovered_target = Some(target);
    session.label_drag = Some(LabelDrag {
      target,
      start_screen: pointer.screen,
      grab_offset: Point {
        x: label_center.world.x - pointer.world.x,
        y: label_center.world.y - pointer.world.y,
      },
      changed: false,
    });
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn update_label_drag(&self, pointer: RulerPointer) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    update_label_drag_session(&mut session, pointer)?;
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn finish_label_drag(&self, pointer: RulerPointer) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    update_label_drag_session(&mut session, pointer)?;
    let target = session.label_drag.take()?.target;
    let visual = refresh_visual(&mut session, pointer)?;
    session.hovered_target = Some(target);
    Some(visual)
  }

  pub(crate) fn hover_radius_label(&self, id: u64) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session.document.radii.iter().any(|radius| radius.id == id) {
      return None;
    }
    session.hovered_target = Some(HoverTarget::Radius(id));
    session.visual
  }

  pub(crate) fn cancel_label_drag(&self) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.label_drag.take()?;
    session.visual
  }

  pub(crate) fn hide_label(&self, kind: LabelKind, id: u64) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let target = label_target(kind, id);
    if label_state(&session.document, target)?.hidden {
      return None;
    }
    record_history(&mut session);
    label_state_mut(&mut session.document, target)?.hidden = true;
    session.hovered_target = None;
    session.label_drag = None;
    session.visual
  }

  pub(crate) fn toggle_label_at(&self, pointer: RulerPointer) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let target = hit_test_artifact(&session, pointer)?;
    if let HoverTarget::Guide(id) = target {
      let gap_ids = session
        .document
        .guide_gaps
        .iter()
        .filter(|gap| gap.second_id == id)
        .map(|gap| gap.id)
        .collect::<Vec<_>>();
      let hidden = gap_ids
        .first()
        .and_then(|gap_id| {
          session
            .document
            .guide_gaps
            .iter()
            .find(|gap| gap.id == *gap_id)
        })
        .map(|gap| gap.label.hidden)?;
      record_history(&mut session);
      for gap in &mut session.document.guide_gaps {
        if gap_ids.contains(&gap.id) {
          gap.label.hidden = !hidden;
        }
      }
      session.hovered_target = hidden.then_some(target);
      return refresh_visual(&mut session, pointer);
    }
    let hidden = label_state(&session.document, target)?.hidden;
    record_history(&mut session);
    label_state_mut(&mut session.document, target)?.hidden = !hidden;
    session.hovered_target = hidden.then_some(target);
    refresh_visual(&mut session, pointer)
  }

  pub(crate) fn update_viewport(
    &self,
    display_id: u32,
    action: ViewportAction,
  ) -> Option<RulerVisual> {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if !session.active {
      return None;
    }
    let index = session
      .displays
      .iter()
      .position(|snapshot| snapshot.display.id == display_id)?;
    let snapshot = &mut session.displays[index];
    let display = snapshot.display;
    let local_screen = match action {
      ViewportAction::Zoom { anchor, factor } => {
        snapshot
          .viewport
          .zoom_at(snapshot.display.size, anchor, factor);
        anchor
      }
      ViewportAction::Pan { anchor, delta } => {
        snapshot.viewport.pan_content(snapshot.display.size, delta);
        anchor
      }
      ViewportAction::Reset { anchor } => {
        snapshot.viewport.reset();
        anchor
      }
    };
    let local_screen = Point {
      x: local_screen
        .x
        .clamp(0.0, (display.size.width - f64::EPSILON).max(0.0)),
      y: local_screen
        .y
        .clamp(0.0, (display.size.height - f64::EPSILON).max(0.0)),
    };
    let screen = Point {
      x: display.origin.x + local_screen.x,
      y: display.origin.y + local_screen.y,
    };
    let pointer = map_pointer(&session.displays, screen)?;
    refresh_visual(&mut session, pointer)
  }

  pub(super) fn active_generation(&self) -> Option<u64> {
    let session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.active.then_some(session.generation)
  }

  pub(super) fn cancel(&self) -> bool {
    let mut session = self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    session.generation = session.generation.wrapping_add(1);
    session.displays.clear();
    session.visual = None;
    session.copied_until = None;
    session.tolerance_until = None;
    session.option_active = false;
    session.boxes.clear();
    session.drag = Drag::default();
    session.document = Document::default();
    session.undo.clear();
    session.redo.clear();
    session.hovered_target = None;
    session.label_drag = None;
    session.range = None;
    session.guide = None;
    session.guide_drag = None;
    session.radius = None;
    session.settle = None;
    std::mem::replace(&mut session.active, false)
  }
}

fn refresh_visual(session: &mut Session, pointer: RulerPointer) -> Option<RulerVisual> {
  let rgba = sample(&session.displays, pointer.world)
    .or_else(|| session.visual.map(|visual| visual.rgba))?;
  if session.drag.start.is_none()
    && session.range.is_none()
    && session.guide.is_none()
    && session.guide_drag.is_none()
    && session.radius.is_none()
    && session.label_drag.is_none()
  {
    let hovered_target = hit_test_artifact(session, pointer);
    update_hover_target(session, hovered_target, Instant::now());
  }
  update_range(session, pointer);
  update_guide(session, pointer);
  update_radius(session, pointer);
  let now = Instant::now();
  let visual = RulerVisual {
    point: pointer.world,
    screen_point: pointer.screen,
    display_id: pointer.display_id,
    zoom: pointer.zoom,
    rgba,
    crosshair: session.visual.is_some_and(|visual| visual.crosshair),
    copied: session.copied_until.is_some_and(|deadline| deadline > now),
  };
  session.visual = Some(visual);
  Some(visual)
}

fn update_hover_target(session: &mut Session, target: Option<HoverTarget>, now: Instant) {
  if session.hovered_target == target {
    expire_hover_exit(session, now);
    return;
  }
  session.hover_exit = match (session.hovered_target, target) {
    (Some(previous), None) => Some(HoverExit {
      target: previous,
      started: now,
    }),
    _ => None,
  };
  session.hovered_target = target;
}

fn expire_hover_exit(session: &mut Session, now: Instant) {
  if session
    .hover_exit
    .is_some_and(|exit| now.saturating_duration_since(exit.started) >= HOVER_EXIT_DURATION)
  {
    session.hover_exit = None;
  }
}

fn hover_alpha(session: &Session, target: HoverTarget, now: Instant) -> f32 {
  if session.hovered_target == Some(target) {
    return 1.0;
  }
  let Some(exit) = session.hover_exit.filter(|exit| exit.target == target) else {
    return 0.0;
  };
  let progress =
    now.saturating_duration_since(exit.started).as_secs_f32() / HOVER_EXIT_DURATION.as_secs_f32();
  (1.0 - progress.clamp(0.0, 1.0)).powi(3)
}

fn pointer_from_visual(visual: RulerVisual) -> RulerPointer {
  RulerPointer {
    world: visual.point,
    screen: visual.screen_point,
    display_id: visual.display_id,
    zoom: visual.zoom,
  }
}

fn map_pointer(displays: &[DisplaySnapshot], screen: Point) -> Option<RulerPointer> {
  let snapshot = displays.iter().find(|snapshot| {
    let display = snapshot.display;
    screen.x >= display.origin.x
      && screen.y >= display.origin.y
      && screen.x < display.origin.x + display.size.width
      && screen.y < display.origin.y + display.size.height
  })?;
  let local_screen = Point {
    x: screen.x - snapshot.display.origin.x,
    y: screen.y - snapshot.display.origin.y,
  };
  let local_world = snapshot.viewport.screen_to_source(local_screen);
  Some(RulerPointer {
    world: Point {
      x: snapshot.display.origin.x + local_world.x,
      y: snapshot.display.origin.y + local_world.y,
    },
    screen,
    display_id: snapshot.display.id,
    zoom: snapshot.viewport.zoom,
  })
}

fn measurement_visuals(session: &mut Session, now: Instant) -> Vec<RulerMeasurementVisual> {
  let mut visuals = session
    .document
    .measurements
    .iter()
    .map(|measurement| RulerMeasurementVisual {
      id: measurement.id,
      bounds: measurement.bounds,
      draft: false,
      animating: false,
      hovered: session.hovered_target == Some(HoverTarget::Measurement(measurement.id)),
      hover_alpha: hover_alpha(session, HoverTarget::Measurement(measurement.id), now),
      label_anchor: measurement.label.anchor,
      label_hidden: measurement.label.hidden,
    })
    .collect::<Vec<_>>();
  if let Some(draft) = session.drag.draft {
    visuals.push(RulerMeasurementVisual {
      id: 0,
      bounds: draft,
      draft: true,
      animating: false,
      hovered: false,
      hover_alpha: 0.0,
      label_anchor: None,
      label_hidden: false,
    });
    return visuals;
  }
  let Some(settle) = session.settle else {
    return visuals;
  };
  let progress = now.duration_since(settle.started).as_secs_f64() / SETTLE_DURATION.as_secs_f64();
  if progress >= 1.0 {
    session.settle = None;
    return visuals;
  }
  let rest = progress - 1.0;
  let eased = 1.0 + (SETTLE_OVERSHOOT + 1.0) * rest.powi(3) + SETTLE_OVERSHOOT * rest.powi(2);
  if let Some(visual) = visuals.iter_mut().find(|item| item.id == settle.id) {
    visual.bounds = mix_rect(settle.from, settle.to, eased);
    visual.animating = true;
  }
  visuals
}

fn measurement_device_scale(displays: &[DisplaySnapshot], bounds: Rect) -> f64 {
  let center = Point {
    x: bounds.origin.x + bounds.size.width * 0.5,
    y: bounds.origin.y + bounds.size.height * 0.5,
  };
  displays
    .iter()
    .find(|snapshot| {
      center.x >= snapshot.display.origin.x
        && center.y >= snapshot.display.origin.y
        && center.x < snapshot.display.origin.x + snapshot.display.size.width
        && center.y < snapshot.display.origin.y + snapshot.display.size.height
    })
    .map_or(1.0, |snapshot| {
      f64::from(snapshot.image.width.max(1)) / snapshot.display.size.width.max(1.0)
    })
}

fn center_aid_visuals(
  session: &mut Session,
  now: Instant,
) -> (Vec<RulerCenterlineVisual>, Vec<RulerInnerObjectVisual>) {
  if session.settle.is_none() {
    if let Some(cache) = &session.center_aid_cache {
      if cache.document == session.document {
        return (cache.lines.clone(), cache.objects.clone());
      }
    }
  }
  let measurements = session.document.measurements.clone();
  let drawn = measurement_visuals(session, now);
  let mut lines = Vec::with_capacity(measurements.len());
  let mut objects = Vec::new();
  for measurement in &measurements {
    let Some(visual) = drawn.iter().find(|item| item.id == measurement.id) else {
      continue;
    };
    let peers = measurements
      .iter()
      .filter(|peer| peer.id != measurement.id)
      .map(|peer| peer.bounds)
      .collect::<Vec<_>>();
    let analysis = centerlines::analyze(
      measurement.bounds,
      &session.boxes,
      &peers,
      measurement_device_scale(&session.displays, measurement.bounds),
    );
    lines.push(RulerCenterlineVisual {
      id: measurement.id,
      bounds: visual.bounds,
      x_accent: analysis.x_accent,
      y_accent: analysis.y_accent,
    });
    if !visual.animating {
      objects.extend(
        analysis
          .objects
          .into_iter()
          .map(|object| RulerInnerObjectVisual {
            owner_id: measurement.id,
            bounds: object.bounds,
            aligned_x: object.aligned_x,
            aligned_y: object.aligned_y,
          }),
      );
    }
  }
  if session.settle.is_none() {
    session.center_aid_cache = Some(CenterAidCache {
      document: session.document.clone(),
      lines: lines.clone(),
      objects: objects.clone(),
    });
  }
  (lines, objects)
}

fn record_history(session: &mut Session) {
  session.undo.push(session.document.clone());
  trim_history(&mut session.undo);
  session.redo.clear();
}

fn trim_history(history: &mut Vec<Document>) {
  let excess = history.len().saturating_sub(HISTORY_LIMIT);
  if excess > 0 {
    history.drain(..excess);
  }
}

fn clear_transient_artifact_state(session: &mut Session) {
  session.drag = Drag::default();
  session.hovered_target = None;
  session.label_drag = None;
  session.range = None;
  session.guide = None;
  session.guide_drag = None;
  session.settle = None;
}

fn hit_test_measurement(measurements: &[Measurement], point: Point, hit_slop: f64) -> Option<u64> {
  measurements.iter().rev().find_map(|measurement| {
    let bounds = measurement.bounds;
    let outer = Rect::from_xywh(
      bounds.origin.x - hit_slop,
      bounds.origin.y - hit_slop,
      bounds.size.width + hit_slop * 2.0,
      bounds.size.height + hit_slop * 2.0,
    );
    let inner_width = (bounds.size.width - hit_slop * 2.0).max(0.0);
    let inner_height = (bounds.size.height - hit_slop * 2.0).max(0.0);
    let inner = Rect::from_xywh(
      bounds.origin.x + hit_slop,
      bounds.origin.y + hit_slop,
      inner_width,
      inner_height,
    );
    (rect_contains(outer, point) && !rect_contains(inner, point)).then_some(measurement.id)
  })
}

fn hit_test_probe(probes: &[ProbeArtifact], point: Point, hit_slop: f64) -> Option<u64> {
  probes.iter().rev().find_map(|probe| {
    let start = probe.start.min(probe.end) - hit_slop;
    let end = probe.start.max(probe.end) + hit_slop;
    let hit = match probe.axis {
      ProbeAxis::Horizontal => {
        point.x >= start && point.x <= end && (point.y - probe.position).abs() <= hit_slop
      }
      ProbeAxis::Vertical => {
        point.y >= start && point.y <= end && (point.x - probe.position).abs() <= hit_slop
      }
    };
    hit.then_some(probe.id)
  })
}

fn hit_test_guide(session: &Session, point: Point, hit_slop: f64) -> Option<u64> {
  session.document.guides.iter().rev().find_map(|guide| {
    let display = session
      .displays
      .iter()
      .find(|snapshot| snapshot.display.id == guide.display_id)?
      .display;
    let within_display = point.x >= display.origin.x
      && point.y >= display.origin.y
      && point.x <= display.origin.x + display.size.width
      && point.y <= display.origin.y + display.size.height;
    let hit = match guide.axis {
      GuideAxis::Vertical => (point.x - guide.position).abs() <= hit_slop,
      GuideAxis::Horizontal => (point.y - guide.position).abs() <= hit_slop,
    };
    (within_display && hit).then_some(guide.id)
  })
}

fn hit_test_guide_gap(session: &Session, point: Point, hit_slop: f64) -> Option<u64> {
  guide_gap_visuals(session)
    .into_iter()
    .rev()
    .filter(|gap| !gap.label_hidden)
    .find_map(|gap| {
      let start = gap.start.min(gap.end) - hit_slop;
      let end = gap.start.max(gap.end) + hit_slop;
      let hit = match gap.axis {
        ProbeAxis::Horizontal => {
          point.x >= start && point.x <= end && (point.y - gap.position).abs() <= hit_slop
        }
        ProbeAxis::Vertical => {
          point.y >= start && point.y <= end && (point.x - gap.position).abs() <= hit_slop
        }
      };
      hit.then_some(gap.id)
    })
}

fn target_id(target: HoverTarget) -> u64 {
  match target {
    HoverTarget::Measurement(id)
    | HoverTarget::Probe(id)
    | HoverTarget::Guide(id)
    | HoverTarget::GuideGap(id)
    | HoverTarget::Radius(id) => id,
  }
}

fn radius_geometry(radius: &RadiusArtifact) -> (Point, Point) {
  let sign_x = if radius.corner.right() { 1.0 } else { -1.0 };
  let sign_y = if radius.corner.bottom() { 1.0 } else { -1.0 };
  let corner = Point {
    x: radius.bounds.origin.x
      + if radius.corner.right() {
        radius.bounds.size.width
      } else {
        0.0
      },
    y: radius.bounds.origin.y
      + if radius.corner.bottom() {
        radius.bounds.size.height
      } else {
        0.0
      },
  };
  let center = Point {
    x: corner.x - sign_x * radius.radius,
    y: corner.y - sign_y * radius.radius,
  };
  let diagonal = std::f64::consts::FRAC_1_SQRT_2;
  let arc_midpoint = Point {
    x: center.x + sign_x * radius.radius * diagonal,
    y: center.y + sign_y * radius.radius * diagonal,
  };
  (center, arc_midpoint)
}

fn distance_to_segment(point: Point, start: Point, end: Point) -> f64 {
  let dx = end.x - start.x;
  let dy = end.y - start.y;
  let length_squared = dx * dx + dy * dy;
  if length_squared <= f64::EPSILON {
    return (point.x - start.x).hypot(point.y - start.y);
  }
  let t = (((point.x - start.x) * dx + (point.y - start.y) * dy) / length_squared).clamp(0.0, 1.0);
  (point.x - (start.x + dx * t)).hypot(point.y - (start.y + dy * t))
}

fn hit_test_radius(radii: &[RadiusArtifact], point: Point, hit_slop: f64) -> Option<u64> {
  radii.iter().rev().find_map(|radius| {
    let (center, arc_midpoint) = radius_geometry(radius);
    let radial_hit =
      ((point.x - center.x).hypot(point.y - center.y) - radius.radius).abs() <= hit_slop;
    let correct_quadrant = if radius.corner.right() {
      point.x >= center.x - hit_slop
    } else {
      point.x <= center.x + hit_slop
    } && if radius.corner.bottom() {
      point.y >= center.y - hit_slop
    } else {
      point.y <= center.y + hit_slop
    };
    let line_hit = distance_to_segment(point, center, arc_midpoint) <= hit_slop;
    (line_hit || (radial_hit && correct_quadrant)).then_some(radius.id)
  })
}

fn hit_test_artifact(session: &Session, pointer: RulerPointer) -> Option<HoverTarget> {
  let slop = ARTIFACT_HIT_SLOP / pointer.zoom;
  [
    hit_test_measurement(&session.document.measurements, pointer.world, slop)
      .map(HoverTarget::Measurement),
    hit_test_probe(&session.document.probes, pointer.world, slop).map(HoverTarget::Probe),
    hit_test_guide(session, pointer.world, slop).map(HoverTarget::Guide),
    hit_test_guide_gap(session, pointer.world, slop).map(HoverTarget::GuideGap),
    hit_test_radius(&session.document.radii, pointer.world, slop).map(HoverTarget::Radius),
  ]
  .into_iter()
  .flatten()
  .max_by_key(|target| target_id(*target))
}

fn latest_target(document: &Document) -> Option<HoverTarget> {
  [
    document
      .measurements
      .last()
      .map(|item| HoverTarget::Measurement(item.id)),
    document
      .probes
      .last()
      .map(|item| HoverTarget::Probe(item.id)),
    document
      .guides
      .last()
      .map(|item| HoverTarget::Guide(item.id)),
    document
      .guide_gaps
      .last()
      .map(|item| HoverTarget::GuideGap(item.id)),
    document
      .radii
      .last()
      .map(|item| HoverTarget::Radius(item.id)),
  ]
  .into_iter()
  .flatten()
  .max_by_key(|target| target_id(*target))
}

fn artifact_text(document: &Document, target: HoverTarget) -> Option<String> {
  match target {
    HoverTarget::Measurement(id) => document
      .measurements
      .iter()
      .find(|measurement| measurement.id == id)
      .map(|measurement| measurement_text(measurement.bounds)),
    HoverTarget::Probe(id) => document
      .probes
      .iter()
      .find(|probe| probe.id == id)
      .copied()
      .map(probe_text),
    HoverTarget::Guide(id) => document
      .guides
      .iter()
      .find(|guide| guide.id == id)
      .map(|guide| format!("{} px", guide.position.round() as i64)),
    HoverTarget::GuideGap(id) => document
      .guide_gaps
      .iter()
      .find(|gap| gap.id == id)
      .and_then(|gap| {
        let first = document
          .guides
          .iter()
          .find(|guide| guide.id == gap.first_id)?;
        let second = document
          .guides
          .iter()
          .find(|guide| guide.id == gap.second_id)?;
        Some(format!(
          "{} px",
          (second.position - first.position).abs().round() as u64
        ))
      }),
    HoverTarget::Radius(id) => document
      .radii
      .iter()
      .find(|radius| radius.id == id)
      .map(|radius| {
        format!(
          "{}{} px",
          if radius.low_confidence { "≈ " } else { "" },
          radius.radius.round() as u64
        )
      }),
  }
}

fn label_target(kind: LabelKind, id: u64) -> HoverTarget {
  match kind {
    LabelKind::Measurement => HoverTarget::Measurement(id),
    LabelKind::Probe => HoverTarget::Probe(id),
    LabelKind::GuideGap => HoverTarget::GuideGap(id),
    LabelKind::Radius => HoverTarget::Radius(id),
  }
}

fn label_state(document: &Document, target: HoverTarget) -> Option<&ArtifactLabel> {
  match target {
    HoverTarget::Measurement(id) => document
      .measurements
      .iter()
      .find(|measurement| measurement.id == id)
      .map(|measurement| &measurement.label),
    HoverTarget::Probe(id) => document
      .probes
      .iter()
      .find(|probe| probe.id == id)
      .map(|probe| &probe.label),
    HoverTarget::GuideGap(id) => document
      .guide_gaps
      .iter()
      .find(|gap| gap.id == id)
      .map(|gap| &gap.label),
    HoverTarget::Radius(id) => document
      .radii
      .iter()
      .find(|radius| radius.id == id)
      .map(|radius| &radius.label),
    HoverTarget::Guide(_) => None,
  }
}

fn label_state_mut(document: &mut Document, target: HoverTarget) -> Option<&mut ArtifactLabel> {
  match target {
    HoverTarget::Measurement(id) => document
      .measurements
      .iter_mut()
      .find(|measurement| measurement.id == id)
      .map(|measurement| &mut measurement.label),
    HoverTarget::Probe(id) => document
      .probes
      .iter_mut()
      .find(|probe| probe.id == id)
      .map(|probe| &mut probe.label),
    HoverTarget::GuideGap(id) => document
      .guide_gaps
      .iter_mut()
      .find(|gap| gap.id == id)
      .map(|gap| &mut gap.label),
    HoverTarget::Radius(id) => document
      .radii
      .iter_mut()
      .find(|radius| radius.id == id)
      .map(|radius| &mut radius.label),
    HoverTarget::Guide(_) => None,
  }
}

fn update_label_drag_session(session: &mut Session, pointer: RulerPointer) -> Option<()> {
  let mut drag = session.label_drag?;
  if !drag.changed {
    let distance =
      (pointer.screen.x - drag.start_screen.x).hypot(pointer.screen.y - drag.start_screen.y);
    if distance < DRAG_THRESHOLD {
      session.hovered_target = Some(drag.target);
      return Some(());
    }
    record_history(session);
    drag.changed = true;
    session.label_drag = Some(drag);
  }
  let anchor = Point {
    x: pointer.world.x + drag.grab_offset.x,
    y: pointer.world.y + drag.grab_offset.y,
  };
  let label = label_state_mut(&mut session.document, drag.target)?;
  label.anchor = Some(anchor);
  label.hidden = false;
  session.label_drag = Some(drag);
  session.hovered_target = Some(drag.target);
  Some(())
}

fn rect_contains(rect: Rect, point: Point) -> bool {
  point.x >= rect.origin.x
    && point.y >= rect.origin.y
    && point.x <= rect.origin.x + rect.size.width
    && point.y <= rect.origin.y + rect.size.height
}

fn measurement_text(bounds: Rect) -> String {
  let width = bounds.size.width.round().max(0.0) as u64;
  let height = bounds.size.height.round().max(0.0) as u64;
  if bounds.size.height < 8.0 {
    format!("{width} px")
  } else if bounds.size.width < 8.0 {
    format!("{height} px")
  } else {
    format!("{width} × {height} px")
  }
}

fn probe_text(probe: ProbeArtifact) -> String {
  format!(
    "{} px",
    (probe.end - probe.start).abs().round().max(0.0) as u64
  )
}

fn mix_rect(from: Rect, to: Rect, amount: f64) -> Rect {
  Rect::from_xywh(
    from.origin.x + (to.origin.x - from.origin.x) * amount,
    from.origin.y + (to.origin.y - from.origin.y) * amount,
    from.size.width + (to.size.width - from.size.width) * amount,
    from.size.height + (to.size.height - from.size.height) * amount,
  )
}

fn ordered_rect(start: Point, end: Point) -> Rect {
  Rect::from_xywh(
    start.x.min(end.x),
    start.y.min(end.y),
    (end.x - start.x).abs(),
    (end.y - start.y).abs(),
  )
}

fn detected_boxes(snapshot: &DisplaySnapshot, tolerance: Tolerance) -> Vec<Rect> {
  snapshot.boxes_by_tolerance[tolerance.index()]
    .iter()
    .copied()
    .map(|item| {
      let display = snapshot.display;
      Rect::from_xywh(
        display.origin.x + f64::from(item.x) / f64::from(snapshot.image.width) * display.size.width,
        display.origin.y
          + f64::from(item.y) / f64::from(snapshot.image.height) * display.size.height,
        f64::from(item.width) / f64::from(snapshot.image.width) * display.size.width,
        f64::from(item.height) / f64::from(snapshot.image.height) * display.size.height,
      )
    })
    .collect()
}

fn probe_visuals(session: &Session) -> Vec<RulerProbeVisual> {
  let now = Instant::now();
  let mut visuals = session
    .document
    .probes
    .iter()
    .map(|probe| RulerProbeVisual {
      id: probe.id,
      display_id: 0,
      axis: probe.axis,
      start: probe.start,
      end: probe.end,
      position: probe.position,
      draft: false,
      hovered: session.hovered_target == Some(HoverTarget::Probe(probe.id)),
      hover_alpha: hover_alpha(session, HoverTarget::Probe(probe.id), now),
      label_anchor: probe.label.anchor,
      label_hidden: probe.label.hidden,
    })
    .collect::<Vec<_>>();
  if let Some(range) = session.range {
    visuals.push(range.draft);
    return visuals;
  }
  if session.drag.pending.is_some()
    || session.drag.start.is_some()
    || session.drag.draft.is_some()
    || session.hovered_target.is_some()
    || session.guide.is_some()
    || session.guide_drag.is_some()
    || session.radius.is_some()
  {
    return visuals;
  }
  let Some(visual) = session.visual else {
    return visuals;
  };
  if let Some(automatic) = automatic_probes(session, pointer_from_visual(visual)) {
    visuals.extend(automatic);
  }
  visuals
}

fn automatic_probes(session: &Session, pointer: RulerPointer) -> Option<[RulerProbeVisual; 2]> {
  let Some(snapshot) = session
    .displays
    .iter()
    .find(|snapshot| snapshot.display.id == pointer.display_id)
  else {
    return None;
  };
  let display = snapshot.display;
  let image_width = snapshot.image.width.max(1);
  let image_height = snapshot.image.height.max(1);
  let x = (((pointer.world.x - display.origin.x) / display.size.width) * f64::from(image_width))
    .floor()
    .clamp(0.0, f64::from(image_width.saturating_sub(1))) as u32;
  let y = (((pointer.world.y - display.origin.y) / display.size.height) * f64::from(image_height))
    .floor()
    .clamp(0.0, f64::from(image_height.saturating_sub(1))) as u32;
  let pixel_probes = if session.tolerance == Tolerance::Balanced {
    snapshot.probes.probes_at(x, y)
  } else {
    probes_at_threshold(&snapshot.gradients, x, y, session.tolerance.threshold())
  };
  let mut visuals: [RulerProbeVisual; 2] = pixel_probes
    .into_iter()
    .map(|probe| {
      let (axis_scale, axis_origin, position) = match probe.axis {
        ProbeAxis::Horizontal => (
          display.size.width / f64::from(image_width),
          display.origin.x,
          pointer.world.y,
        ),
        ProbeAxis::Vertical => (
          display.size.height / f64::from(image_height),
          display.origin.y,
          pointer.world.x,
        ),
      };
      RulerProbeVisual {
        id: 0,
        display_id: display.id,
        axis: probe.axis,
        start: axis_origin + f64::from(probe.start) * axis_scale,
        end: axis_origin + f64::from(probe.end) * axis_scale,
        position,
        draft: false,
        hovered: false,
        hover_alpha: 0.0,
        label_anchor: None,
        label_hidden: false,
      }
    })
    .collect::<Vec<_>>()
    .try_into()
    .ok()?;
  if session.option_active {
    clip_transient_probes_to_guides(&session.document.guides, pointer, &mut visuals);
  }
  Some(visuals)
}

fn clip_transient_probes_to_guides(
  guides: &[GuideArtifact],
  pointer: RulerPointer,
  probes: &mut [RulerProbeVisual; 2],
) {
  for probe in probes {
    let (guide_axis, pointer_axis) = match probe.axis {
      ProbeAxis::Horizontal => (GuideAxis::Vertical, pointer.world.x),
      ProbeAxis::Vertical => (GuideAxis::Horizontal, pointer.world.y),
    };
    let positions = guides
      .iter()
      .filter(|guide| guide.display_id == pointer.display_id && guide.axis == guide_axis)
      .map(|guide| guide.position)
      .filter(|position| *position >= probe.start && *position <= probe.end)
      .collect::<Vec<_>>();
    probe.start = positions
      .iter()
      .copied()
      .filter(|position| *position <= pointer_axis)
      .max_by(|left, right| left.total_cmp(right))
      .map_or(probe.start, |position| probe.start.max(position));
    probe.end = positions
      .iter()
      .copied()
      .filter(|position| *position >= pointer_axis)
      .min_by(|left, right| left.total_cmp(right))
      .map_or(probe.end, |position| probe.end.min(position));
  }
}

fn guide_pointer_position(axis: GuideAxis, point: Point) -> f64 {
  match axis {
    GuideAxis::Vertical => point.x,
    GuideAxis::Horizontal => point.y,
  }
}

fn guide_cross_axis_position(axis: GuideAxis, point: Point) -> f64 {
  match axis {
    GuideAxis::Vertical => point.y,
    GuideAxis::Horizontal => point.x,
  }
}

fn update_guide(session: &mut Session, pointer: RulerPointer) {
  let Some(mut gesture) = session.guide else {
    return;
  };
  let raw = guide_pointer_position(gesture.visual.axis, pointer.world);
  let retain_snap = gesture.snapped
    && gesture.visual.display_id == pointer.display_id
    && (gesture.visual.position - raw).abs() * pointer.zoom <= GUIDE_RELEASE_RADIUS;
  if !retain_snap {
    let snapped = session
      .displays
      .iter()
      .find(|snapshot| snapshot.display.id == pointer.display_id)
      .and_then(|snapshot| snap_guide(snapshot, gesture.visual.axis, pointer, session.tolerance));
    gesture.visual.display_id = pointer.display_id;
    gesture.visual.position = snapped.unwrap_or(raw);
    gesture.snapped = snapped.is_some();
  }
  session.guide = Some(gesture);
}

fn update_radius(session: &mut Session, pointer: RulerPointer) {
  if session.radius.is_none() {
    return;
  }
  let visual = session
    .displays
    .iter()
    .find(|snapshot| snapshot.display.id == pointer.display_id)
    .and_then(|snapshot| {
      let display = snapshot.display;
      let scale_x = display.size.width / f64::from(snapshot.image.width.max(1));
      let scale_y = display.size.height / f64::from(snapshot.image.height.max(1));
      let cursor = Point {
        x: (pointer.world.x - display.origin.x) / scale_x,
        y: (pointer.world.y - display.origin.y) / scale_y,
      };
      corner_radius_at(
        &snapshot.boxes_by_tolerance[session.tolerance.index()],
        cursor,
        &snapshot.gradients,
        session.tolerance.threshold(),
        scale_x,
        scale_y,
      )
      .map(|estimate| RulerRadiusVisual {
        id: 0,
        display_id: display.id,
        bounds: Rect::from_xywh(
          display.origin.x + f64::from(estimate.bounds.x) * scale_x,
          display.origin.y + f64::from(estimate.bounds.y) * scale_y,
          f64::from(estimate.bounds.width) * scale_x,
          f64::from(estimate.bounds.height) * scale_y,
        ),
        corner: estimate.corner,
        radius: f64::from(estimate.radius) * (scale_x + scale_y) * 0.5,
        low_confidence: estimate.low_confidence,
        draft: true,
        hovered: false,
        hover_alpha: 0.0,
        label_anchor: None,
        label_hidden: false,
      })
    });
  session.radius = Some(RadiusGesture { visual });
}

fn update_guide_drag_session(session: &mut Session, pointer: RulerPointer) -> Option<()> {
  let mut drag = session.guide_drag?;
  if !drag.changed {
    let distance =
      (pointer.screen.x - drag.start_screen.x).hypot(pointer.screen.y - drag.start_screen.y);
    if distance < DRAG_THRESHOLD {
      session.hovered_target = Some(HoverTarget::Guide(drag.id));
      return Some(());
    }
    record_history(session);
    drag.changed = true;
  }

  let raw = guide_pointer_position(drag.original.axis, pointer.world);
  let current = session
    .document
    .guides
    .iter()
    .find(|guide| guide.id == drag.id)
    .copied()?;
  let retain_snap = drag.snapped
    && current.display_id == pointer.display_id
    && (current.position - raw).abs() * pointer.zoom <= GUIDE_RELEASE_RADIUS;
  let (position, snapped) = if retain_snap {
    (current.position, true)
  } else {
    let snapped = session
      .displays
      .iter()
      .find(|snapshot| snapshot.display.id == pointer.display_id)
      .and_then(|snapshot| snap_guide(snapshot, drag.original.axis, pointer, session.tolerance));
    (snapped.unwrap_or(raw), snapped.is_some())
  };
  let guide = session
    .document
    .guides
    .iter_mut()
    .find(|guide| guide.id == drag.id)?;
  guide.display_id = pointer.display_id;
  guide.position = position;
  if current.display_id != pointer.display_id {
    if let Some(display) = session
      .displays
      .iter()
      .find(|snapshot| snapshot.display.id == pointer.display_id)
      .map(|snapshot| snapshot.display)
    {
      guide.anchor = match guide.axis {
        GuideAxis::Vertical => guide
          .anchor
          .clamp(display.origin.y, display.origin.y + display.size.height),
        GuideAxis::Horizontal => guide
          .anchor
          .clamp(display.origin.x, display.origin.x + display.size.width),
      };
    }
  }
  drag.snapped = snapped;
  session.guide_drag = Some(drag);
  session.hovered_target = Some(HoverTarget::Guide(drag.id));
  reconcile_guide_gaps(&mut session.document);
  Some(())
}

fn reconcile_guide_gaps(document: &mut Document) {
  let mut pairs = Vec::new();
  for guide in &document.guides {
    let mut peers = document
      .guides
      .iter()
      .filter(|peer| peer.display_id == guide.display_id && peer.axis == guide.axis)
      .copied()
      .collect::<Vec<_>>();
    peers.sort_by(|left, right| {
      left
        .position
        .total_cmp(&right.position)
        .then_with(|| left.id.cmp(&right.id))
    });
    for pair in peers.windows(2) {
      let first_id = pair[0].id.min(pair[1].id);
      let second_id = pair[0].id.max(pair[1].id);
      if !pairs.contains(&(first_id, second_id)) {
        pairs.push((first_id, second_id));
      }
    }
  }

  document
    .guide_gaps
    .retain(|gap| pairs.contains(&(gap.first_id, gap.second_id)));
  for (first_id, second_id) in pairs {
    if document
      .guide_gaps
      .iter()
      .any(|gap| gap.first_id == first_id && gap.second_id == second_id)
    {
      continue;
    }
    document.next_id = document.next_id.wrapping_add(1).max(1);
    document.guide_gaps.push(GuideGapArtifact {
      id: document.next_id,
      first_id,
      second_id,
      label: ArtifactLabel::default(),
    });
  }
}

fn guide_gap_visuals(session: &Session) -> Vec<RulerGuideGapVisual> {
  let now = Instant::now();
  session
    .document
    .guide_gaps
    .iter()
    .filter_map(|gap| {
      let first = session
        .document
        .guides
        .iter()
        .find(|guide| guide.id == gap.first_id)?;
      let second = session
        .document
        .guides
        .iter()
        .find(|guide| guide.id == gap.second_id)?;
      if first.display_id != second.display_id || first.axis != second.axis {
        return None;
      }
      let owner = if first.id > second.id { first } else { second };
      let axis = match first.axis {
        GuideAxis::Vertical => ProbeAxis::Horizontal,
        GuideAxis::Horizontal => ProbeAxis::Vertical,
      };
      let default_position = (first.anchor + second.anchor) * 0.5;
      let position = gap
        .label
        .anchor
        .map_or(default_position, |anchor| match axis {
          ProbeAxis::Horizontal => anchor.y,
          ProbeAxis::Vertical => anchor.x,
        });
      Some(RulerGuideGapVisual {
        id: gap.id,
        owner_id: owner.id,
        display_id: first.display_id,
        axis,
        start: first.position.min(second.position),
        end: first.position.max(second.position),
        position,
        hovered: session.hovered_target == Some(HoverTarget::GuideGap(gap.id)),
        hover_alpha: hover_alpha(session, HoverTarget::GuideGap(gap.id), now),
        label_anchor: gap.label.anchor,
        label_hidden: gap.label.hidden,
      })
    })
    .collect()
}

fn snap_guide(
  snapshot: &DisplaySnapshot,
  axis: GuideAxis,
  pointer: RulerPointer,
  tolerance: Tolerance,
) -> Option<f64> {
  let display = snapshot.display;
  let (axis_pixels, across_pixels, local_axis, local_across, gradients) = match axis {
    GuideAxis::Vertical => (
      snapshot.image.width,
      snapshot.image.height,
      pointer.world.x - display.origin.x,
      pointer.world.y - display.origin.y,
      &snapshot.gradients.gx,
    ),
    GuideAxis::Horizontal => (
      snapshot.image.height,
      snapshot.image.width,
      pointer.world.y - display.origin.y,
      pointer.world.x - display.origin.x,
      &snapshot.gradients.gy,
    ),
  };
  if axis_pixels < 2 || across_pixels == 0 {
    return None;
  }
  let axis_world = match axis {
    GuideAxis::Vertical => display.size.width,
    GuideAxis::Horizontal => display.size.height,
  };
  let across_world = match axis {
    GuideAxis::Vertical => display.size.height,
    GuideAxis::Horizontal => display.size.width,
  };
  if axis_world <= 0.0 || across_world <= 0.0 {
    return None;
  }
  let pixels_per_world = f64::from(axis_pixels) / axis_world;
  let raw_pixel =
    (local_axis * pixels_per_world).clamp(1.0, f64::from(axis_pixels.saturating_sub(1)));
  let across_pixel = ((local_across / across_world) * f64::from(across_pixels))
    .floor()
    .clamp(0.0, f64::from(across_pixels.saturating_sub(1))) as u32;
  let radius = ((GUIDE_SNAP_RADIUS / pointer.zoom.max(1.0)) * pixels_per_world)
    .ceil()
    .max(1.0) as u32;
  let center = raw_pixel.round() as u32;
  let start = center.saturating_sub(radius).max(1);
  let end = center
    .saturating_add(radius)
    .min(axis_pixels.saturating_sub(1));
  let mut best: Option<(u32, f64, u32)> = None;
  for position in start..=end {
    let score = (-2..=2)
      .filter_map(|offset| across_pixel.checked_add_signed(offset))
      .filter(|across| *across < across_pixels)
      .map(|across| {
        let index = match axis {
          GuideAxis::Vertical => across * snapshot.image.width + position,
          GuideAxis::Horizontal => position * snapshot.image.width + across,
        };
        u32::from(gradients[index as usize])
      })
      .sum::<u32>();
    if score < u32::from(tolerance.threshold()) * 2 {
      continue;
    }
    let distance = (f64::from(position) - raw_pixel).abs();
    if best.is_none_or(|(best_score, best_distance, _)| {
      score > best_score || (score == best_score && distance < best_distance)
    }) {
      best = Some((score, distance, position));
    }
  }
  let position = best?.2;
  let origin = match axis {
    GuideAxis::Vertical => display.origin.x,
    GuideAxis::Horizontal => display.origin.y,
  };
  Some(origin + f64::from(position) / pixels_per_world)
}

fn update_range(session: &mut Session, pointer: RulerPointer) {
  let Some(mut range) = session.range else {
    return;
  };
  let Some(end_probe) = automatic_probes(session, pointer)
    .and_then(|probes| probes.into_iter().find(|probe| probe.axis == range.axis))
  else {
    return;
  };
  let tracking_world = match range.axis {
    ProbeAxis::Horizontal => Point {
      x: range.start_pointer.world.x,
      y: pointer.world.y,
    },
    ProbeAxis::Vertical => Point {
      x: pointer.world.x,
      y: range.start_pointer.world.y,
    },
  };
  let tracking_pointer = session
    .displays
    .iter()
    .find(|snapshot| {
      let display = snapshot.display;
      tracking_world.x >= display.origin.x
        && tracking_world.y >= display.origin.y
        && tracking_world.x < display.origin.x + display.size.width
        && tracking_world.y < display.origin.y + display.size.height
    })
    .map(|snapshot| RulerPointer {
      world: tracking_world,
      screen: pointer.screen,
      display_id: snapshot.display.id,
      zoom: snapshot.viewport.zoom,
    });
  let start_probe = tracking_pointer
    .and_then(|tracking| automatic_probes(session, tracking))
    .and_then(|probes| probes.into_iter().find(|probe| probe.axis == range.axis))
    .unwrap_or(range.start_probe);
  let forward = match range.axis {
    ProbeAxis::Horizontal => pointer.world.x >= range.start_pointer.world.x,
    ProbeAxis::Vertical => pointer.world.y >= range.start_pointer.world.y,
  };
  range.draft = RulerProbeVisual {
    id: 0,
    display_id: 0,
    axis: range.axis,
    start: if forward {
      start_probe.start
    } else {
      end_probe.start
    },
    end: if forward {
      end_probe.end
    } else {
      start_probe.end
    },
    position: match range.axis {
      ProbeAxis::Horizontal => pointer.world.y,
      ProbeAxis::Vertical => pointer.world.x,
    },
    draft: true,
    hovered: false,
    hover_alpha: 0.0,
    label_anchor: None,
    label_hidden: false,
  };
  session.range = Some(range);
}

fn rect_area(rect: Rect) -> f64 {
  rect.size.width.max(0.0) * rect.size.height.max(0.0)
}

fn contains(outer: Rect, inner: Rect) -> bool {
  inner.origin.x >= outer.origin.x
    && inner.origin.y >= outer.origin.y
    && inner.right() <= outer.right()
    && inner.bottom() <= outer.bottom()
}

fn intersection_over(a: Rect, b: Rect) -> f64 {
  let width = (a.right().min(b.right()) - a.origin.x.max(b.origin.x)).max(0.0);
  let height = (a.bottom().min(b.bottom()) - a.origin.y.max(b.origin.y)).max(0.0);
  let overlap = width * height;
  let union = rect_area(a) + rect_area(b) - overlap;
  if union > 0.0 {
    overlap / union
  } else {
    0.0
  }
}

fn snap_bounds(boxes: &[Rect], drag: Rect) -> Rect {
  let grown = Rect::from_xywh(
    drag.origin.x - CONTAINMENT_SLACK,
    drag.origin.y - CONTAINMENT_SLACK,
    drag.size.width + CONTAINMENT_SLACK * 2.0,
    drag.size.height + CONTAINMENT_SLACK * 2.0,
  );
  let contained = boxes
    .iter()
    .copied()
    .filter(|candidate| contains(grown, *candidate))
    .reduce(union_rect);
  if let Some(bounds) = contained {
    return bounds;
  }
  if let Some((_, bounds)) = boxes
    .iter()
    .copied()
    .map(|candidate| (intersection_over(candidate, drag), candidate))
    .filter(|(score, _)| *score >= MINIMUM_IOU)
    .max_by(|(left, _), (right, _)| left.total_cmp(right))
  {
    return bounds;
  }
  snap_to_nearby_box_edges(boxes, drag)
}

fn union_rect(a: Rect, b: Rect) -> Rect {
  let left = a.origin.x.min(b.origin.x);
  let top = a.origin.y.min(b.origin.y);
  Rect::from_xywh(
    left,
    top,
    a.right().max(b.right()) - left,
    a.bottom().max(b.bottom()) - top,
  )
}

fn snap_to_nearby_box_edges(boxes: &[Rect], drag: Rect) -> Rect {
  let nearest = |target: f64, candidates: Vec<f64>| {
    candidates
      .into_iter()
      .filter(|edge| (edge - target).abs() <= EDGE_SEARCH)
      .min_by(|left, right| (left - target).abs().total_cmp(&(right - target).abs()))
      .unwrap_or(target)
  };
  let x_edges = boxes
    .iter()
    .filter(|item| item.bottom() >= drag.origin.y && item.origin.y <= drag.bottom())
    .flat_map(|item| [item.origin.x, item.right()])
    .collect::<Vec<_>>();
  let x0 = nearest(drag.origin.x, x_edges.clone());
  let x1 = nearest(drag.right(), x_edges);
  let y_edges = boxes
    .iter()
    .filter(|item| item.right() >= x0.min(x1) && item.origin.x <= x0.max(x1))
    .flat_map(|item| [item.origin.y, item.bottom()])
    .collect::<Vec<_>>();
  let y0 = nearest(drag.origin.y, y_edges.clone());
  let y1 = nearest(drag.bottom(), y_edges);
  Rect::from_xywh(x0.min(x1), y0.min(y1), (x1 - x0).abs(), (y1 - y0).abs())
}

fn settle_worthwhile(from: Rect, to: Rect) -> bool {
  (from.origin.x - to.origin.x).abs() >= 1.0
    || (from.origin.y - to.origin.y).abs() >= 1.0
    || (from.right() - to.right()).abs() >= 1.0
    || (from.bottom() - to.bottom()).abs() >= 1.0
}

fn sample(displays: &[DisplaySnapshot], point: Point) -> Option<[u8; 4]> {
  let snapshot = displays.iter().find(|snapshot| {
    let display = snapshot.display;
    point.x >= display.origin.x
      && point.y >= display.origin.y
      && point.x < display.origin.x + display.size.width
      && point.y < display.origin.y + display.size.height
  })?;
  let display = snapshot.display;
  let x = (((point.x - display.origin.x) / display.size.width) * f64::from(snapshot.image.width))
    .floor()
    .clamp(0.0, f64::from(snapshot.image.width.saturating_sub(1))) as usize;
  let y = (((point.y - display.origin.y) / display.size.height) * f64::from(snapshot.image.height))
    .floor()
    .clamp(0.0, f64::from(snapshot.image.height.saturating_sub(1))) as usize;
  let offset = (y * snapshot.image.width as usize + x) * 4;
  snapshot
    .image
    .rgba
    .get(offset..offset + 4)
    .and_then(|pixel| pixel.try_into().ok())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dismissal_invalidates_topology_restarts() {
    let state = RulerState::default();
    let generation = state.begin();
    assert_eq!(state.active_generation(), Some(generation));
    assert!(state.cancel());
    assert_eq!(state.active_generation(), None);
    assert!(!state.is_current(generation));
  }

  #[test]
  fn radius_candidates_are_cached_for_each_tolerance() {
    let width = 120u32;
    let height = 120u32;
    let mut rgba = vec![0xFF; (width * height * 4) as usize];
    for pixel in rgba.chunks_mut(4) {
      pixel[3] = 255;
    }
    let set = |rgba: &mut [u8], x: u32, y: u32, rgb: [u8; 3]| {
      let index = ((y * width + x) * 4) as usize;
      rgba[index..index + 3].copy_from_slice(&rgb);
    };
    for y in 30..70 {
      for x in 30..90 {
        set(&mut rgba, x, y, [0xF8, 0xF9, 0xFA]);
      }
    }
    let blend = [0xFC, 0xFC, 0xFD];
    for x in 30..90 {
      set(&mut rgba, x, 30, blend);
      set(&mut rgba, x, 69, blend);
    }
    for y in 30..70 {
      set(&mut rgba, 30, y, blend);
      set(&mut rgba, 89, y, blend);
    }
    let state = RulerState::default();
    let generation = state.begin();
    let display = DesktopDisplay {
      id: 1,
      origin: Point { x: 0.0, y: 0.0 },
      size: crate::osc::geometry::Size {
        width: f64::from(width),
        height: f64::from(height),
      },
      scale: 1.0,
    };
    assert!(state.install(
      generation,
      &[display],
      &[(
        1,
        CapturedImage {
          rgba,
          width,
          height,
        },
      )],
    ));
    let session = state.0.lock().unwrap();
    let snapshot = &session.displays[0];
    assert!(snapshot.boxes_by_tolerance[Tolerance::ClearEdges.index()].is_empty());
    assert!(snapshot.boxes_by_tolerance[Tolerance::Balanced.index()].is_empty());
    assert_eq!(
      snapshot.boxes_by_tolerance[Tolerance::SubtleEdges.index()].len(),
      1
    );
  }

  #[test]
  fn samples_each_display_in_its_own_pixel_density() {
    let state = RulerState::default();
    let generation = state.begin();
    let displays = [
      DesktopDisplay {
        id: 1,
        origin: Point { x: 0.0, y: 0.0 },
        size: crate::osc::geometry::Size {
          width: 2.0,
          height: 1.0,
        },
        scale: 2.0,
      },
      DesktopDisplay {
        id: 2,
        origin: Point { x: 2.0, y: 1.0 },
        size: crate::osc::geometry::Size {
          width: 1.0,
          height: 1.0,
        },
        scale: 1.0,
      },
    ];
    let snapshots = [
      (
        1,
        CapturedImage {
          rgba: vec![1, 2, 3, 255, 4, 5, 6, 255, 7, 8, 9, 255, 10, 11, 12, 255],
          width: 4,
          height: 1,
        },
      ),
      (
        2,
        CapturedImage {
          rgba: vec![20, 21, 22, 255],
          width: 1,
          height: 1,
        },
      ),
    ];
    assert!(state.install(generation, &displays, &snapshots));
    assert_eq!(
      state
        .hover(state.map_pointer(Point { x: 1.75, y: 0.5 }).unwrap())
        .unwrap()
        .rgba,
      [10, 11, 12, 255]
    );
    assert_eq!(
      state
        .hover(state.map_pointer(Point { x: 2.5, y: 1.5 }).unwrap())
        .unwrap()
        .rgba,
      [20, 21, 22, 255]
    );
    assert!(state.map_pointer(Point { x: 2.5, y: 0.5 }).is_none());
  }

  #[test]
  fn viewport_actions_change_only_the_target_display() {
    let state = RulerState::default();
    let generation = state.begin();
    let displays = [
      DesktopDisplay {
        id: 1,
        origin: Point { x: 0.0, y: 0.0 },
        size: crate::osc::geometry::Size {
          width: 100.0,
          height: 80.0,
        },
        scale: 1.0,
      },
      DesktopDisplay {
        id: 2,
        origin: Point { x: 100.0, y: 0.0 },
        size: crate::osc::geometry::Size {
          width: 100.0,
          height: 80.0,
        },
        scale: 2.0,
      },
    ];
    let image = CapturedImage {
      rgba: vec![0; 100 * 80 * 4],
      width: 100,
      height: 80,
    };
    assert!(state.install(generation, &displays, &[(1, image.clone()), (2, image)]));
    assert!(state
      .hover(state.map_pointer(Point { x: 150.0, y: 40.0 }).unwrap())
      .is_some());
    state.update_viewport(
      2,
      ViewportAction::Zoom {
        anchor: Point { x: 50.0, y: 40.0 },
        factor: 2.0,
      },
    );
    let viewports = state.viewports();
    assert_eq!(viewports[0].viewport, Viewport::default());
    assert_eq!(viewports[1].viewport.zoom, 2.0);
    assert_eq!(
      state.map_pointer(Point { x: 100.0, y: 0.0 }).unwrap().world,
      Point { x: 125.0, y: 20.0 }
    );
    state.update_viewport(
      2,
      ViewportAction::Reset {
        anchor: Point { x: 0.0, y: 0.0 },
      },
    );
    assert!(state
      .viewports()
      .iter()
      .all(|item| item.viewport == Viewport::default()));
  }

  #[test]
  fn live_probes_use_frozen_source_edges_and_hide_during_pointer_gestures() {
    let state = RulerState::default();
    let generation = state.begin();
    let display = DesktopDisplay {
      id: 7,
      origin: Point { x: 100.0, y: 50.0 },
      size: crate::osc::geometry::Size {
        width: 10.0,
        height: 5.0,
      },
      scale: 2.0,
    };
    let mut rgba = vec![0; 20 * 10 * 4];
    for y in 2..9 {
      for x in 3..16 {
        let offset = (y * 20 + x) * 4;
        rgba[offset..offset + 4].copy_from_slice(&[255, 255, 255, 255]);
      }
    }
    let image = CapturedImage {
      rgba,
      width: 20,
      height: 10,
    };
    assert!(state.install(generation, &[display], &[(7, image)]));
    let pointer = state.map_pointer(Point { x: 105.2, y: 52.75 }).unwrap();
    assert!(state.hover(pointer).is_some());
    let probes = state.probes();
    assert_eq!(probes.len(), 2);
    assert_eq!(probes[0].display_id, 7);
    assert_eq!(probes[0].axis, ProbeAxis::Horizontal);
    assert_eq!((probes[0].start, probes[0].end), (101.5, 108.0));
    assert_eq!(probes[0].position, 52.75);
    assert_eq!(probes[1].axis, ProbeAxis::Vertical);
    assert_eq!((probes[1].start, probes[1].end), (51.0, 54.5));
    assert_eq!(probes[1].position, 105.2);

    assert!(state.pointer_down(pointer).is_some());
    assert!(state.probes().is_empty());
    assert!(state.cancel_pointer().is_some());
    assert_eq!(state.probes().len(), 2);

    let end = state.map_pointer(Point { x: 109.5, y: 54.5 }).unwrap();
    assert!(state.pointer_down(pointer).is_some());
    assert!(state.pointer_drag(end).is_some());
    assert!(state.pointer_up(end).is_some());
    assert_eq!(state.measurements().len(), 1);
  }

  #[test]
  fn held_range_stamps_a_probe_and_joins_history_copy_and_deletion() {
    let state = RulerState::default();
    let generation = state.begin();
    let display = DesktopDisplay {
      id: 1,
      origin: Point { x: 0.0, y: 0.0 },
      size: crate::osc::geometry::Size {
        width: 10.0,
        height: 5.0,
      },
      scale: 2.0,
    };
    let mut rgba = vec![0; 20 * 10 * 4];
    for y in 2..9 {
      for x in 3..16 {
        let offset = (y * 20 + x) * 4;
        rgba[offset..offset + 4].copy_from_slice(&[255, 255, 255, 255]);
      }
    }
    let image = CapturedImage {
      rgba,
      width: 20,
      height: 10,
    };
    assert!(state.install(generation, &[display], &[(1, image)]));
    let start = state.map_pointer(Point { x: 4.0, y: 2.25 }).unwrap();
    assert!(state.hover(start).is_some());
    assert!(state.begin_range(RangeAxis::Horizontal).is_some());
    let draft = state
      .probes()
      .into_iter()
      .find(|probe| probe.draft)
      .unwrap();
    assert_eq!(draft.axis, ProbeAxis::Horizontal);
    assert_eq!(draft.position, 2.25);

    let end = state.map_pointer(Point { x: 7.0, y: 3.25 }).unwrap();
    assert!(state.hover(end).is_some());
    assert!(state.finish_range().is_some());
    let stamped = state
      .probes()
      .into_iter()
      .find(|probe| probe.id != 0)
      .unwrap();
    assert!(!stamped.draft);
    assert_eq!(
      (stamped.start, stamped.end, stamped.position),
      (1.5, 8.0, 3.25)
    );
    assert_eq!(state.copy_latest_artifact().unwrap().1, "7 px");

    assert!(state.undo().is_some());
    assert!(state.probes().iter().all(|probe| probe.id == 0));
    assert!(state.redo().is_some());
    assert!(state.probes().iter().any(|probe| probe.id != 0));
    assert!(state.delete_targeted_artifact().is_some());
    assert!(state.probes().iter().all(|probe| probe.id == 0));
  }

  #[test]
  fn held_guides_snap_with_hysteresis_stamp_and_join_history() {
    let state = RulerState::default();
    let generation = state.begin();
    let display = DesktopDisplay {
      id: 7,
      origin: Point { x: 100.0, y: 50.0 },
      size: crate::osc::geometry::Size {
        width: 100.0,
        height: 80.0,
      },
      scale: 1.0,
    };
    let mut rgba = vec![0; 100 * 80 * 4];
    for y in 0..80 {
      for x in 0..100 {
        let value = if x >= 40 || y >= 25 { 255 } else { 0 };
        let offset = (y * 100 + x) * 4;
        rgba[offset..offset + 4].copy_from_slice(&[value, value, value, 255]);
      }
    }
    let image = CapturedImage {
      rgba,
      width: 100,
      height: 80,
    };
    assert!(state.install(generation, &[display], &[(7, image)]));

    let start = state.map_pointer(Point { x: 135.0, y: 70.0 }).unwrap();
    assert!(state.hover(start).is_some());
    assert!(state.begin_guide(GuideAxis::Vertical).is_some());
    assert_eq!(state.guides()[0].position, 140.0);

    let retained = state.map_pointer(Point { x: 150.0, y: 70.0 }).unwrap();
    assert!(state.hover(retained).is_some());
    assert_eq!(state.guides()[0].position, 140.0);

    let released = state.map_pointer(Point { x: 158.0, y: 70.0 }).unwrap();
    assert!(state.hover(released).is_some());
    assert_eq!(state.guides()[0].position, 158.0);

    assert!(state.hover(start).is_some());
    assert_eq!(state.guides()[0].position, 140.0);
    assert!(state.pointer_down(start).is_some());
    let guides = state.guides();
    assert_eq!(guides.len(), 2);
    assert!(guides.iter().any(|guide| guide.id != 0 && !guide.draft));
    assert!(state.cancel_guide().is_some());
    assert_eq!(state.guides().len(), 1);

    assert!(state.undo().is_some());
    assert!(state.guides().is_empty());
    assert!(state.redo().is_some());
    assert_eq!(state.guides().len(), 1);

    assert!(state.begin_guide(GuideAxis::Horizontal).is_some());
    assert_eq!(state.guides().last().unwrap().position, 75.0);
    assert!(state.cancel_guide().is_some());
  }

  #[test]
  fn adjacent_guides_create_a_movable_gap_with_single_step_history() {
    let state = RulerState::default();
    let generation = state.begin();
    let display = DesktopDisplay {
      id: 1,
      origin: Point { x: 0.0, y: 0.0 },
      size: crate::osc::geometry::Size {
        width: 100.0,
        height: 100.0,
      },
      scale: 1.0,
    };
    let image = CapturedImage {
      rgba: vec![0; 100 * 100 * 4],
      width: 100,
      height: 100,
    };
    assert!(state.install(generation, &[display], &[(1, image)]));
    {
      let mut session = state.0.lock().unwrap();
      session.document.guides = vec![
        GuideArtifact {
          id: 1,
          display_id: 1,
          axis: GuideAxis::Vertical,
          position: 20.0,
          anchor: 40.0,
        },
        GuideArtifact {
          id: 2,
          display_id: 1,
          axis: GuideAxis::Vertical,
          position: 60.0,
          anchor: 45.0,
        },
      ];
      session.document.next_id = 2;
      reconcile_guide_gaps(&mut session.document);
    }
    let gap = state.guide_gaps()[0];
    assert_eq!((gap.start, gap.end, gap.position), (20.0, 60.0, 42.5));
    assert_eq!(gap.axis, ProbeAxis::Horizontal);

    let start = state.map_pointer(Point { x: 60.0, y: 70.0 }).unwrap();
    assert!(state.hover(start).is_some());
    assert!(state.pointer_down(start).is_some());
    assert!(state.pointer_up(start).is_some());
    assert!(state.0.lock().unwrap().undo.is_empty());
    assert_eq!(state.guides()[1].position, 60.0);

    assert!(state.hover(start).is_some());
    assert!(state.pointer_down(start).is_some());
    let moved = state.map_pointer(Point { x: 75.0, y: 70.0 }).unwrap();
    assert!(state.pointer_drag(moved).is_some());
    assert!(state.pointer_up(moved).is_some());
    assert_eq!(state.guides()[1].position, 75.0);
    assert_eq!(state.guide_gaps()[0].end, 75.0);
    assert_eq!(state.0.lock().unwrap().undo.len(), 1);
    assert!(state.undo().is_some());
    assert_eq!(state.guides()[1].position, 60.0);
    assert!(state.redo().is_some());
    assert_eq!(state.guides()[1].position, 75.0);

    let gap = state.guide_gaps()[0];
    let label_center = state
      .map_pointer(Point {
        x: (gap.start + gap.end) * 0.5,
        y: gap.position,
      })
      .unwrap();
    assert!(state
      .begin_label_drag(LabelKind::GuideGap, gap.id, label_center, label_center)
      .is_some());
    let label_end = state.map_pointer(Point { x: 58.0, y: 58.0 }).unwrap();
    assert!(state.update_label_drag(label_end).is_some());
    assert!(state.finish_label_drag(label_end).is_some());
    assert_eq!(state.guide_gaps()[0].label_anchor, Some(label_end.world));
    assert_eq!(state.guide_gaps()[0].position, label_end.world.y);
    assert!(state.hide_label(LabelKind::GuideGap, gap.id).is_some());
    assert!(state.guide_gaps()[0].label_hidden);
    assert!(state.undo().is_some());
    assert!(!state.guide_gaps()[0].label_hidden);
  }

  #[test]
  fn deleting_a_guide_gap_removes_its_newer_guide() {
    let state = RulerState::default();
    {
      let mut session = state.0.lock().unwrap();
      session.active = true;
      session.visual = Some(RulerVisual {
        point: Point { x: 40.0, y: 50.0 },
        screen_point: Point { x: 40.0, y: 50.0 },
        display_id: 1,
        zoom: 1.0,
        rgba: [0, 0, 0, 255],
        crosshair: false,
        copied: false,
      });
      session.document.guides = vec![
        GuideArtifact {
          id: 1,
          display_id: 1,
          axis: GuideAxis::Vertical,
          position: 20.0,
          anchor: 50.0,
        },
        GuideArtifact {
          id: 2,
          display_id: 1,
          axis: GuideAxis::Vertical,
          position: 60.0,
          anchor: 50.0,
        },
      ];
      session.document.next_id = 2;
      reconcile_guide_gaps(&mut session.document);
      let gap_id = session.document.guide_gaps[0].id;
      session.hovered_target = Some(HoverTarget::GuideGap(gap_id));
    }
    assert!(state.delete_targeted_artifact().is_some());
    assert_eq!(state.guides().len(), 1);
    assert_eq!(state.guides()[0].id, 1);
    assert!(state.guide_gaps().is_empty());
    assert!(state.undo().is_some());
    assert_eq!(state.guides().len(), 2);
    assert_eq!(state.guide_gaps().len(), 1);
  }

  #[test]
  fn option_clips_transient_probes_to_neighbouring_guides() {
    let state = RulerState::default();
    let generation = state.begin();
    let display = DesktopDisplay {
      id: 1,
      origin: Point { x: 0.0, y: 0.0 },
      size: crate::osc::geometry::Size {
        width: 100.0,
        height: 100.0,
      },
      scale: 1.0,
    };
    let image = CapturedImage {
      rgba: vec![0; 100 * 100 * 4],
      width: 100,
      height: 100,
    };
    assert!(state.install(generation, &[display], &[(1, image)]));
    {
      let mut session = state.0.lock().unwrap();
      session.document.guides = vec![
        GuideArtifact {
          id: 1,
          display_id: 1,
          axis: GuideAxis::Vertical,
          position: 30.0,
          anchor: 10.0,
        },
        GuideArtifact {
          id: 2,
          display_id: 1,
          axis: GuideAxis::Vertical,
          position: 70.0,
          anchor: 10.0,
        },
        GuideArtifact {
          id: 3,
          display_id: 1,
          axis: GuideAxis::Horizontal,
          position: 20.0,
          anchor: 10.0,
        },
        GuideArtifact {
          id: 4,
          display_id: 1,
          axis: GuideAxis::Horizontal,
          position: 80.0,
          anchor: 10.0,
        },
      ];
      session.document.next_id = 4;
      reconcile_guide_gaps(&mut session.document);
    }
    let pointer = state.map_pointer(Point { x: 50.0, y: 50.0 }).unwrap();
    assert!(state.hover(pointer).is_some());
    let probes = state.probes();
    assert_eq!((probes[0].start, probes[0].end), (0.0, 99.0));
    assert_eq!((probes[1].start, probes[1].end), (0.0, 99.0));
    assert!(state.set_option_active(true).is_some());
    let probes = state.probes();
    assert_eq!((probes[0].start, probes[0].end), (30.0, 70.0));
    assert_eq!((probes[1].start, probes[1].end), (20.0, 80.0));
    assert!(state.set_option_active(false).is_some());
    let probes = state.probes();
    assert_eq!((probes[0].start, probes[0].end), (0.0, 99.0));
    assert_eq!((probes[1].start, probes[1].end), (0.0, 99.0));
  }

  #[test]
  fn tolerance_changes_every_live_edge_detector_and_preserves_stamped_artifacts() {
    let state = RulerState::default();
    let generation = state.begin();
    let display = DesktopDisplay {
      id: 1,
      origin: Point { x: 0.0, y: 0.0 },
      size: crate::osc::geometry::Size {
        width: 100.0,
        height: 100.0,
      },
      scale: 1.0,
    };
    let mut rgba = vec![255; 100 * 100 * 4];
    for y in 30..70 {
      for x in 30..90 {
        let offset = (y * 100 + x) * 4;
        rgba[offset..offset + 4].copy_from_slice(&[248, 249, 250, 255]);
      }
    }
    let image = CapturedImage {
      rgba,
      width: 100,
      height: 100,
    };
    assert!(state.install(generation, &[display], &[(1, image)]));
    let balanced_boxes = state.0.lock().unwrap().boxes.clone();
    assert!(balanced_boxes.is_empty());
    let pointer = state.map_pointer(Point { x: 50.0, y: 50.0 }).unwrap();
    assert!(state.hover(pointer).is_some());
    let horizontal = state
      .probes()
      .into_iter()
      .find(|probe| probe.axis == ProbeAxis::Horizontal)
      .unwrap();
    assert_eq!((horizontal.start, horizontal.end), (0.0, 99.0));

    assert!(state.begin_guide(GuideAxis::Vertical).is_some());
    assert!(state.pointer_down(pointer).is_some());
    assert!(state.cancel_guide().is_some());
    let stamped = state
      .guides()
      .into_iter()
      .map(|guide| (guide.id, guide.display_id, guide.axis, guide.position))
      .collect::<Vec<_>>();
    let probe_pointer = state.map_pointer(Point { x: 60.0, y: 50.0 }).unwrap();
    assert!(state.hover(probe_pointer).is_some());
    {
      let mut session = state.0.lock().unwrap();
      session.document.next_id += 1;
      let id = session.document.next_id;
      session.document.measurements.push(Measurement {
        id,
        bounds: Rect::from_xywh(0.0, 0.0, 10.0, 10.0),
        label: ArtifactLabel::default(),
      });
    }
    let _ = state.center_aids();
    assert!(state.0.lock().unwrap().center_aid_cache.is_some());
    let stamped_document = state.0.lock().unwrap().document.clone();

    assert!(state.cycle_tolerance().is_some());
    assert_eq!(state.tolerance_notice(), Some(Tolerance::SubtleEdges));
    let session = state.0.lock().unwrap();
    let subtle_boxes = session.boxes.clone();
    assert_eq!(subtle_boxes.len(), 1);
    assert_ne!(subtle_boxes, balanced_boxes);
    assert_eq!(session.document, stamped_document);
    assert!(session.center_aid_cache.is_none());
    drop(session);
    assert_eq!(
      state
        .guides()
        .into_iter()
        .map(|guide| (guide.id, guide.display_id, guide.axis, guide.position))
        .collect::<Vec<_>>(),
      stamped
    );
    let horizontal = state
      .probes()
      .into_iter()
      .find(|probe| probe.axis == ProbeAxis::Horizontal)
      .unwrap();
    assert_eq!((horizontal.start, horizontal.end), (30.0, 90.0));

    assert!(state.cycle_tolerance().is_some());
    assert_eq!(state.tolerance_notice(), Some(Tolerance::ClearEdges));
    assert!(state.0.lock().unwrap().boxes.is_empty());
    assert_eq!(
      state
        .guides()
        .into_iter()
        .map(|guide| (guide.id, guide.display_id, guide.axis, guide.position))
        .collect::<Vec<_>>(),
      stamped
    );
    let horizontal = state
      .probes()
      .into_iter()
      .find(|probe| probe.axis == ProbeAxis::Horizontal)
      .unwrap();
    assert_eq!((horizontal.start, horizontal.end), (0.0, 99.0));

    assert!(state.cycle_tolerance().is_some());
    assert_eq!(state.tolerance_notice(), Some(Tolerance::Balanced));
    assert_eq!(state.0.lock().unwrap().boxes, balanced_boxes);
  }

  #[test]
  fn guide_snapping_uses_the_same_tolerance_as_other_live_edges() {
    let state = RulerState::default();
    let generation = state.begin();
    let display = DesktopDisplay {
      id: 1,
      origin: Point { x: 0.0, y: 0.0 },
      size: crate::osc::geometry::Size {
        width: 100.0,
        height: 100.0,
      },
      scale: 1.0,
    };
    let mut rgba = vec![255; 100 * 100 * 4];
    for y in 30..70 {
      for x in 30..90 {
        let offset = (y * 100 + x) * 4;
        rgba[offset..offset + 4].copy_from_slice(&[248, 249, 250, 255]);
      }
    }
    assert!(state.install(
      generation,
      &[display],
      &[(
        1,
        CapturedImage {
          rgba,
          width: 100,
          height: 100,
        },
      )],
    ));
    let pointer = state.map_pointer(Point { x: 31.0, y: 50.0 }).unwrap();
    let session = state.0.lock().unwrap();
    let snapshot = &session.displays[0];
    assert_eq!(
      snap_guide(snapshot, GuideAxis::Vertical, pointer, Tolerance::Balanced),
      None
    );
    assert_eq!(
      snap_guide(
        snapshot,
        GuideAxis::Vertical,
        pointer,
        Tolerance::SubtleEdges,
      ),
      Some(30.0)
    );
  }

  #[test]
  fn crosshair_and_copy_feedback_share_the_current_visual() {
    let visual = RulerVisual {
      point: Point { x: 1.0, y: 2.0 },
      screen_point: Point { x: 1.0, y: 2.0 },
      display_id: 1,
      zoom: 1.0,
      rgba: [0x12, 0xAB, 0xEF, 0xFF],
      crosshair: false,
      copied: false,
    };
    assert_eq!(visual.hex(), "#12ABEF");
    assert_eq!(visual.packed_rgba(), 0x12AB_EFFF);
  }

  #[test]
  fn snapping_prefers_the_components_circled_by_the_drag() {
    let boxes = [
      Rect::from_xywh(20.0, 20.0, 30.0, 20.0),
      Rect::from_xywh(60.0, 20.0, 20.0, 20.0),
    ];
    assert_eq!(
      snap_bounds(&boxes, Rect::from_xywh(15.0, 15.0, 70.0, 30.0)),
      Rect::from_xywh(20.0, 20.0, 60.0, 20.0)
    );
  }

  #[test]
  fn snapping_uses_the_best_overlapping_container() {
    let boxes = [Rect::from_xywh(20.0, 20.0, 80.0, 60.0)];
    assert_eq!(
      snap_bounds(&boxes, Rect::from_xywh(24.0, 24.0, 70.0, 50.0)),
      boxes[0]
    );
  }

  #[test]
  fn settle_animation_starts_at_the_raw_drag_and_lands_on_the_snap() {
    let from = Rect::from_xywh(10.0, 10.0, 30.0, 30.0);
    let to = Rect::from_xywh(8.0, 8.0, 34.0, 34.0);
    let mut session = Session {
      active: true,
      document: Document {
        measurements: vec![Measurement {
          id: 1,
          bounds: to,
          label: ArtifactLabel::default(),
        }],
        probes: Vec::new(),
        guides: Vec::new(),
        guide_gaps: Vec::new(),
        radii: Vec::new(),
        next_id: 1,
      },
      settle: Some(Settle {
        id: 1,
        from,
        to,
        started: Instant::now(),
      }),
      ..Default::default()
    };
    let first = measurement_visuals(&mut session, Instant::now())[0];
    assert!(first.animating);
    assert!((first.bounds.origin.x - from.origin.x).abs() < 0.1);
    let landed = measurement_visuals(
      &mut session,
      Instant::now() + SETTLE_DURATION + Duration::from_millis(1),
    )[0];
    assert_eq!(landed.bounds, to);
    assert!(!landed.animating);
  }

  #[test]
  fn artifact_hover_targets_the_latest_overlapping_border() {
    let measurements = [
      Measurement {
        id: 1,
        bounds: Rect::from_xywh(10.0, 10.0, 50.0, 50.0),
        label: ArtifactLabel::default(),
      },
      Measurement {
        id: 2,
        bounds: Rect::from_xywh(10.0, 10.0, 50.0, 50.0),
        label: ArtifactLabel::default(),
      },
    ];
    assert_eq!(
      hit_test_measurement(&measurements, Point { x: 10.0, y: 30.0 }, 6.0),
      Some(2)
    );
    assert_eq!(
      hit_test_measurement(&measurements, Point { x: 30.0, y: 30.0 }, 6.0),
      None
    );
  }

  #[test]
  fn hover_target_clears_immediately_while_its_halo_fades_out() {
    let now = Instant::now();
    let target = HoverTarget::Measurement(1);
    let mut session = Session {
      hovered_target: Some(target),
      document: Document {
        measurements: vec![Measurement {
          id: 1,
          bounds: Rect::from_xywh(10.0, 10.0, 20.0, 20.0),
          label: ArtifactLabel::default(),
        }],
        next_id: 1,
        ..Default::default()
      },
      ..Default::default()
    };

    update_hover_target(&mut session, None, now);
    assert_eq!(session.hovered_target, None);
    assert!(session.hover_exit.is_some());
    assert_eq!(hover_alpha(&session, target, now), 1.0);
    let halfway = hover_alpha(&session, target, now + HOVER_EXIT_DURATION / 2);
    assert!(halfway > 0.0 && halfway < 1.0);
    assert_eq!(measurement_visuals(&mut session, now)[0].hover_alpha, 1.0);

    expire_hover_exit(&mut session, now + HOVER_EXIT_DURATION);
    assert!(session.hover_exit.is_none());
    assert_eq!(
      hover_alpha(&session, target, now + HOVER_EXIT_DURATION),
      0.0
    );
  }

  #[test]
  fn label_hover_targets_delete_without_changing_latest_copy() {
    let state = RulerState::default();
    {
      let mut session = state.0.lock().unwrap();
      session.active = true;
      session.visual = Some(RulerVisual {
        point: Point { x: 0.0, y: 0.0 },
        screen_point: Point { x: 0.0, y: 0.0 },
        display_id: 1,
        zoom: 1.0,
        rgba: [0, 0, 0, 255],
        crosshair: false,
        copied: false,
      });
      session.document.measurements = vec![
        Measurement {
          id: 1,
          bounds: Rect::from_xywh(0.0, 0.0, 20.0, 30.0),
          label: ArtifactLabel::default(),
        },
        Measurement {
          id: 2,
          bounds: Rect::from_xywh(40.0, 40.0, 50.0, 60.0),
          label: ArtifactLabel::default(),
        },
      ];
      session.document.next_id = 2;
    }
    assert!(state.hover_measurement_label(1).is_some());
    let visuals = state.measurements();
    assert!(visuals.iter().find(|item| item.id == 1).unwrap().hovered);
    assert_eq!(state.copy_latest_artifact().unwrap().1, "50 × 60 px");
    assert!(state.delete_targeted_artifact().is_some());
    assert_eq!(
      state
        .measurements()
        .into_iter()
        .map(|item| item.id)
        .collect::<Vec<_>>(),
      vec![2]
    );
  }

  #[test]
  fn label_drag_and_visibility_share_document_history() {
    let state = RulerState::default();
    let generation = state.begin();
    let display = DesktopDisplay {
      id: 1,
      origin: Point { x: 0.0, y: 0.0 },
      size: crate::osc::geometry::Size {
        width: 100.0,
        height: 100.0,
      },
      scale: 1.0,
    };
    let image = CapturedImage {
      rgba: vec![0, 0, 0, 255],
      width: 1,
      height: 1,
    };
    assert!(state.install(generation, &[display], &[(1, image)]));
    let initial = state.map_pointer(Point { x: 30.0, y: 30.0 }).unwrap();
    assert!(state.hover(initial).is_some());
    {
      let mut session = state.0.lock().unwrap();
      session.document.measurements.push(Measurement {
        id: 1,
        bounds: Rect::from_xywh(20.0, 20.0, 40.0, 40.0),
        label: ArtifactLabel::default(),
      });
      session.document.next_id = 1;
    }
    let center = state.map_pointer(Point { x: 40.0, y: 40.0 }).unwrap();
    assert!(state
      .begin_label_drag(LabelKind::Measurement, 1, initial, center)
      .is_some());
    let below_threshold = state.map_pointer(Point { x: 32.0, y: 30.0 }).unwrap();
    assert!(state.update_label_drag(below_threshold).is_some());
    assert!(state.measurements()[0].label_anchor.is_none());
    let moved = state.map_pointer(Point { x: 40.0, y: 30.0 }).unwrap();
    assert!(state.update_label_drag(moved).is_some());
    assert!(state.finish_label_drag(moved).is_some());
    assert_eq!(
      state.measurements()[0].label_anchor,
      Some(Point { x: 50.0, y: 40.0 })
    );
    assert!(state.hide_label(LabelKind::Measurement, 1).is_some());
    assert!(state.measurements()[0].label_hidden);
    let border = state.map_pointer(Point { x: 20.0, y: 40.0 }).unwrap();
    assert!(state.toggle_label_at(border).is_some());
    assert!(!state.measurements()[0].label_hidden);
    assert!(state.toggle_label_at(border).is_some());
    assert!(state.measurements()[0].label_hidden);
    assert!(state.undo().is_some());
    assert!(!state.measurements()[0].label_hidden);
    assert!(state.undo().is_some());
    assert!(state.measurements()[0].label_hidden);
    assert!(state.undo().is_some());
    assert!(!state.measurements()[0].label_hidden);
    assert!(state.undo().is_some());
    assert!(state.measurements()[0].label_anchor.is_none());
  }

  #[test]
  fn history_is_bounded_and_new_edits_clear_redo() {
    let mut session = Session::default();
    for id in 1..=105 {
      record_history(&mut session);
      session.document.measurements.push(Measurement {
        id,
        bounds: Rect::from_xywh(id as f64, 0.0, 10.0, 10.0),
        label: ArtifactLabel::default(),
      });
    }
    assert_eq!(session.undo.len(), HISTORY_LIMIT);
    session.redo.push(Document::default());
    record_history(&mut session);
    assert!(session.redo.is_empty());
  }

  #[test]
  fn undo_redo_and_delete_operate_on_the_artifact_document() {
    let state = RulerState::default();
    {
      let mut session = state.0.lock().unwrap();
      session.active = true;
      session.visual = Some(RulerVisual {
        point: Point { x: 10.0, y: 10.0 },
        screen_point: Point { x: 10.0, y: 10.0 },
        display_id: 1,
        zoom: 1.0,
        rgba: [0, 0, 0, 255],
        crosshair: false,
        copied: false,
      });
      record_history(&mut session);
      session.document.measurements.push(Measurement {
        id: 1,
        bounds: Rect::from_xywh(1.0, 2.0, 30.0, 40.0),
        label: ArtifactLabel::default(),
      });
      session.document.next_id = 1;
      session.hovered_target = Some(HoverTarget::Measurement(1));
    }
    assert_eq!(state.copy_latest_artifact().unwrap().1, "30 × 40 px");
    assert!(state.delete_targeted_artifact().is_some());
    assert!(state.measurements().is_empty());
    assert!(state.undo().is_some());
    assert_eq!(state.measurements().len(), 1);
    assert!(state.redo().is_some());
    assert!(state.measurements().is_empty());
  }

  #[test]
  fn radius_artifacts_share_hit_text_label_and_history_semantics() {
    let radius = RadiusArtifact {
      id: 7,
      display_id: 1,
      bounds: Rect::from_xywh(10.0, 10.0, 80.0, 60.0),
      corner: Corner::TopLeft,
      radius: 12.0,
      low_confidence: true,
      label: ArtifactLabel::default(),
    };
    let mut document = Document {
      radii: vec![radius],
      next_id: 7,
      ..Default::default()
    };
    assert_eq!(latest_target(&document), Some(HoverTarget::Radius(7)));
    assert_eq!(
      artifact_text(&document, HoverTarget::Radius(7)).as_deref(),
      Some("≈ 12 px")
    );
    assert_eq!(
      hit_test_radius(&document.radii, Point { x: 14.0, y: 14.0 }, 3.0),
      Some(7)
    );
    assert_eq!(
      hit_test_radius(&document.radii, Point { x: 60.0, y: 40.0 }, 3.0),
      None
    );
    label_state_mut(&mut document, HoverTarget::Radius(7))
      .unwrap()
      .anchor = Some(Point { x: 30.0, y: 25.0 });
    assert_eq!(
      label_state(&document, HoverTarget::Radius(7))
        .unwrap()
        .anchor,
      Some(Point { x: 30.0, y: 25.0 })
    );
  }

  #[test]
  fn center_aids_are_document_cached_and_toggle_as_one_native_view() {
    let state = RulerState::default();
    {
      let mut session = state.0.lock().unwrap();
      session.active = true;
      session.centerlines_visible = true;
      session.visual = Some(RulerVisual {
        point: Point { x: 40.0, y: 40.0 },
        screen_point: Point { x: 40.0, y: 40.0 },
        display_id: 1,
        zoom: 1.0,
        rgba: [0, 0, 0, 255],
        crosshair: false,
        copied: false,
      });
      session.boxes = vec![
        Rect::from_xywh(10.0, 10.0, 80.0, 60.0),
        Rect::from_xywh(35.0, 25.0, 30.0, 30.0),
      ];
      session.document.measurements.push(Measurement {
        id: 1,
        bounds: Rect::from_xywh(10.0, 10.0, 80.0, 60.0),
        label: ArtifactLabel::default(),
      });
      session.document.next_id = 1;
    }
    let (lines, objects) = state.center_aids();
    assert_eq!(lines.len(), 1);
    assert!(lines[0].x_accent);
    assert!(lines[0].y_accent);
    assert_eq!(objects.len(), 1);
    assert!(state.0.lock().unwrap().center_aid_cache.is_some());

    assert!(state.toggle_centerlines().is_some());
    assert_eq!(state.center_aids(), (Vec::new(), Vec::new()));
    assert!(state.toggle_centerlines().is_some());
    assert_eq!(state.center_aids(), (lines, objects));
  }
}
