// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::osc::{
  controller::RegionController,
  geometry::{Handle, Monitor, Point, Rect, Size},
  gesture::GestureKind,
  protocol::CursorIcon,
};

fn controller() -> RegionController {
  RegionController::new(
    Monitor {
      size: Size {
        width: 100.,
        height: 80.,
      },
    },
    None,
    None,
  )
}

#[test]
fn null_out_is_safe() {
  unsafe { state::native_osc_input(std::ptr::null_mut(), 0, 0., 0., 0, std::ptr::null_mut()) }
}

#[test]
fn invalid_context_is_reported() {
  let mut out = NativeOscResult::default();
  unsafe { state::native_osc_input(std::ptr::null_mut(), 1, 0., 0., 0, &mut out) }
  assert_eq!(out.status, ResultStatus::Invalid as u8);
}

#[test]
fn hover_classifies_without_changing_state() {
  let c = controller();
  let before = (c.committed(), c.draft(), c.gesture_active());
  assert_eq!(c.hover_kind(Point { x: 4., y: 4. }), GestureKind::Drawing);
  assert_eq!((c.committed(), c.draft(), c.gesture_active()), before);
  assert_eq!(
    result_for(GestureKind::Drawing, None).status,
    ResultStatus::None as u8
  );
}

#[test]
fn idle_hover_keeps_crosshair_except_on_resize_handles() {
  let c = RegionController::new(
    Monitor {
      size: Size {
        width: 100.,
        height: 80.,
      },
    },
    Some(Rect {
      origin: Point { x: 20., y: 20. },
      size: Size {
        width: 30.,
        height: 20.,
      },
    }),
    None,
  );
  let kind = c.hover_kind(Point { x: 35., y: 30. });
  assert_eq!(kind, GestureKind::Moving);

  let mut result = result_for(kind, None);
  apply_phase_cursor(InputPhase::Hover, true, &mut result);
  assert_eq!(result.cursor, CursorIcon::Crosshair as u8);

  apply_phase_cursor(InputPhase::Down, true, &mut result);
  assert_eq!(result.cursor, ffi::CURSOR_CLOSED_HAND);

  let mut resize = result_for(GestureKind::Resizing(Handle::NorthEast), None);
  apply_phase_cursor(InputPhase::Hover, true, &mut resize);
  assert_eq!(resize.cursor, ffi::CURSOR_DIAGONAL);

  let mut recording_body = result_for(GestureKind::Moving, None);
  apply_phase_cursor(InputPhase::Hover, false, &mut recording_body);
  assert_eq!(recording_body.cursor, ffi::CURSOR_OPEN_HAND);

  let mut recording_outside = result_for(GestureKind::Drawing, None);
  apply_phase_cursor(InputPhase::Hover, false, &mut recording_outside);
  assert_eq!(recording_outside.cursor, ffi::CURSOR_ARROW);
}

#[test]
fn down_drag_and_up_return_regions_and_semantics() {
  let mut c = controller();
  assert_eq!(
    c.pointer_down(Point { x: 10., y: 10. }),
    GestureKind::Drawing
  );
  let changed = c.pointer_move(Point { x: 30., y: 25. }, false).unwrap();
  let result = result_for(event_kind(&changed), Some(&changed));
  assert_eq!(result.status, ResultStatus::Changed as u8);
  assert_eq!(
    (
      result.has_region,
      result.x,
      result.y,
      result.width,
      result.height
    ),
    (1, 10., 10., 20., 15.)
  );
  let finished = c.pointer_up(Point { x: 40., y: 30. }, false).unwrap();
  let result = result_for(event_kind(&finished), Some(&finished));
  assert_eq!(result.status, ResultStatus::Finished as u8);
  assert_eq!(result.gesture, RESULT_GESTURE_DRAWING);
}

#[test]
fn cancel_restores_region_and_payload() {
  let mut c = RegionController::new(
    Monitor {
      size: Size {
        width: 100.,
        height: 80.,
      },
    },
    Some(Rect {
      origin: Point { x: 20., y: 20. },
      size: Size {
        width: 30.,
        height: 20.,
      },
    }),
    None,
  );
  c.pointer_down(Point { x: 35., y: 30. });
  c.pointer_move(Point { x: 50., y: 40. }, false);
  let cancelled = c.cancel().unwrap();
  let result = result_for(GestureKind::Drawing, Some(&cancelled));
  assert_eq!(result.status, ResultStatus::Cancelled as u8);
  assert_eq!(
    payload_for(&cancelled, None).status,
    SemanticStatus::Cancelled
  );
  assert_eq!(c.committed().unwrap().origin, Point { x: 20., y: 20. });
}

#[test]
fn invalid_phase_is_neutral() {
  assert_eq!(invalid_result().status, ResultStatus::Invalid as u8);
}

#[test]
fn resize_handles_use_frontend_wire_names() {
  assert_eq!(
    serde_json::to_value(native_handle(Handle::NorthEast)).unwrap(),
    "northeast"
  );
}
