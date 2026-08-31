// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Platform-neutral routing for normalized Ruler input.

use crate::osc::{
  geometry::Point,
  protocol::{CursorIcon, InputModifiers, InputPhase, OscResult, ResultStatus},
};

use super::{
  snapshot::{GuideAxis, LabelKind, RangeAxis, RulerVisual, ViewportAction},
  RulerState,
};

pub(crate) struct RulerDispatch {
  pub visual: RulerVisual,
  pub copy: Option<String>,
  pub cursor: CursorIcon,
  pub handle: u8,
}

pub(crate) fn visual_result(
  state: &RulerState,
  visual: RulerVisual,
  cursor: CursorIcon,
  handle: u8,
) -> OscResult {
  let tolerance = state.tolerance_notice();
  let tolerance_mode = tolerance.map_or(0, |value| match value {
    super::snapshot::Tolerance::ClearEdges => 1,
    super::snapshot::Tolerance::Balanced => 2,
    super::snapshot::Tolerance::SubtleEdges => 3,
  });
  OscResult {
    status: ResultStatus::Changed as u8,
    cursor: cursor as u8,
    handle,
    x: visual.screen_point.x,
    y: visual.screen_point.y,
    ruler_color: visual.packed_rgba(),
    ruler_flags: 1
      | u8::from(visual.crosshair) << 1
      | u8::from(visual.copied) << 2
      | u8::from(tolerance.is_some()) << 3
      | tolerance_mode << 4
      | u8::from(state.interaction_active()) << 6
      | u8::from(state.hover_fade_active()) << 7,
    ..Default::default()
  }
}

pub(crate) fn dispatch_input(
  state: &RulerState,
  phase: InputPhase,
  point: Point,
  modifiers: InputModifiers,
) -> Option<RulerDispatch> {
  if phase.pointer() {
    let _ = state.set_option_active(modifiers.option);
  }
  let (visual, copy) = match phase {
    InputPhase::Hover => (
      state
        .map_pointer(point)
        .and_then(|value| state.hover(value)),
      None,
    ),
    InputPhase::Down => (
      state
        .map_pointer(point)
        .and_then(|value| state.pointer_down(value)),
      None,
    ),
    InputPhase::Drag => (
      state
        .map_pointer(point)
        .and_then(|value| state.pointer_drag(value)),
      None,
    ),
    InputPhase::Up => (
      state
        .map_pointer(point)
        .and_then(|value| state.pointer_up(value)),
      None,
    ),
    InputPhase::Cancel => (state.cancel_pointer(), None),
    InputPhase::RulerToggleCrosshair => (state.toggle_crosshair(), None),
    InputPhase::RulerCopyColour => state
      .copy_colour()
      .map_or((None, None), |(visual, text)| (Some(visual), Some(text))),
    InputPhase::RulerAnimationFrame => (state.animation_frame(), None),
    InputPhase::RulerDeleteMeasurement => (state.delete_targeted_artifact(), None),
    InputPhase::RulerCopyMeasurement => state
      .copy_latest_artifact()
      .map_or((None, None), |(visual, text)| (Some(visual), Some(text))),
    InputPhase::RulerUndo => (state.undo(), None),
    InputPhase::RulerRedo => (state.redo(), None),
    InputPhase::RulerBeginHorizontalRange => (state.begin_range(RangeAxis::Horizontal), None),
    InputPhase::RulerBeginVerticalRange => (state.begin_range(RangeAxis::Vertical), None),
    InputPhase::RulerFinishRange => (state.finish_range(), None),
    InputPhase::RulerCancelRange => (state.cancel_range(), None),
    InputPhase::RulerHoverProbeLabel => (state.hover_probe_label(point.x.max(0.0) as u64), None),
    InputPhase::RulerHoverMeasurementLabel => {
      (state.hover_measurement_label(point.x.max(0.0) as u64), None)
    }
    InputPhase::RulerBeginVerticalGuide => (state.begin_guide(GuideAxis::Vertical), None),
    InputPhase::RulerBeginHorizontalGuide => (state.begin_guide(GuideAxis::Horizontal), None),
    InputPhase::RulerCancelGuide => (state.cancel_guide(), None),
    InputPhase::RulerCycleTolerance => (state.cycle_tolerance(), None),
    InputPhase::RulerSetOptionActive => (state.set_option_active(modifiers.option), None),
    InputPhase::RulerBeginRadius => (state.begin_radius(), None),
    InputPhase::RulerCancelRadius => (state.cancel_radius(), None),
    InputPhase::RulerToggleCenterlines => (state.toggle_centerlines(), None),
    _ => return None,
  };
  let visual = visual?;
  let (cursor, handle) = if matches!(
    phase,
    InputPhase::RulerHoverProbeLabel | InputPhase::RulerHoverMeasurementLabel
  ) {
    (CursorIcon::OpenHand, 0)
  } else if let Some(axis) = state.hovered_guide_axis() {
    match axis {
      GuideAxis::Vertical => (CursorIcon::HorizontalResize, 4),
      GuideAxis::Horizontal => (CursorIcon::VerticalResize, 2),
    }
  } else {
    (CursorIcon::Crosshair, 0)
  };
  Some(RulerDispatch {
    visual,
    copy,
    cursor,
    handle,
  })
}

pub(crate) fn dispatch_viewport(
  state: &RulerState,
  display_id: u32,
  operation: u32,
  anchor: Point,
  delta: Point,
) -> Option<RulerVisual> {
  let action = match operation {
    1 => ViewportAction::Zoom {
      anchor,
      factor: delta.x,
    },
    2 => ViewportAction::Pan { anchor, delta },
    3 => ViewportAction::Reset { anchor },
    _ => return None,
  };
  state.update_viewport(display_id, action)
}

pub(crate) fn dispatch_label(
  state: &RulerState,
  operation: u32,
  kind: u8,
  id: u64,
  pointer: Point,
  label_center: Point,
) -> Option<RulerDispatch> {
  let label_kind = match kind {
    1 => Some(LabelKind::Measurement),
    2 => Some(LabelKind::Probe),
    3 => Some(LabelKind::GuideGap),
    4 => Some(LabelKind::Radius),
    _ => None,
  };
  let visual = match operation {
    1 => label_kind.and_then(|kind| {
      state
        .map_pointer(pointer)
        .zip(state.map_pointer(label_center))
        .and_then(|(pointer, center)| state.begin_label_drag(kind, id, pointer, center))
    }),
    2 => state
      .map_pointer(pointer)
      .and_then(|pointer| state.update_label_drag(pointer)),
    3 => state
      .map_pointer(pointer)
      .and_then(|pointer| state.finish_label_drag(pointer)),
    4 => state.cancel_label_drag(),
    5 => label_kind.and_then(|kind| state.hide_label(kind, id)),
    6 => state
      .map_pointer(pointer)
      .and_then(|pointer| state.toggle_label_at(pointer)),
    7 => match label_kind {
      Some(LabelKind::Measurement) => state.hover_measurement_label(id),
      Some(LabelKind::Probe) => state.hover_probe_label(id),
      Some(LabelKind::GuideGap) => state.hover_guide_gap_label(id),
      Some(LabelKind::Radius) => state.hover_radius_label(id),
      None => None,
    },
    _ => None,
  }?;
  Some(RulerDispatch {
    visual,
    copy: None,
    cursor: match operation {
      1 | 2 => CursorIcon::ClosedHand,
      3 | 7 => CursorIcon::OpenHand,
      _ => CursorIcon::Crosshair,
    },
    handle: 0,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn visual_result_has_one_platform_independent_wire_encoding() {
    let state = RulerState::default();
    let visual = RulerVisual {
      point: Point { x: 10.0, y: 20.0 },
      screen_point: Point { x: 30.0, y: 40.0 },
      display_id: 2,
      zoom: 2.0,
      rgba: [0x12, 0x34, 0x56, 0xff],
      crosshair: true,
      copied: true,
    };

    let result = visual_result(&state, visual, CursorIcon::OpenHand, 4);

    assert_eq!(result.status, ResultStatus::Changed as u8);
    assert_eq!(result.cursor, CursorIcon::OpenHand as u8);
    assert_eq!(result.handle, 4);
    assert_eq!((result.x, result.y), (30.0, 40.0));
    assert_eq!(result.ruler_color, 0x1234_56ff);
    assert_eq!(result.ruler_flags, 0b0000_0111);
  }
}
