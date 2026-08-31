// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Shared Region session dispatch. Native hosts translate OS input into this
//! contract and apply the returned cursor and semantic event on their side.

use super::{
  controller::{ControllerEvent, RegionController},
  geometry::{Handle, Point},
  gesture::GestureKind,
  protocol::{
    CursorIcon, InputModifiers, InputPhase, OscResult, ResultStatus, RESULT_GESTURE_DRAWING,
    RESULT_GESTURE_MOVING, RESULT_GESTURE_RESIZING,
  },
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RegionDispatch {
  pub result: OscResult,
  pub event: Option<ControllerEvent>,
  pub gesture: GestureKind,
}

pub fn dispatch_region(
  controller: &mut RegionController,
  phase: InputPhase,
  point: Point,
  modifiers: InputModifiers,
  allow_drawing: bool,
) -> Option<RegionDispatch> {
  if phase == InputPhase::Down
    && !allow_drawing
    && controller.hover_kind(point) == GestureKind::Drawing
  {
    return None;
  }
  let (gesture, event) = match phase {
    InputPhase::Hover => (controller.hover_kind(point), None),
    InputPhase::Down => (controller.pointer_down(point), None),
    InputPhase::Drag => {
      let event = controller.pointer_move(point, modifiers.free_aspect);
      (
        event.as_ref().map_or(GestureKind::Drawing, event_kind),
        event,
      )
    }
    InputPhase::Up => {
      let event = controller.pointer_up(point, modifiers.free_aspect);
      (
        event.as_ref().map_or(GestureKind::Drawing, event_kind),
        event,
      )
    }
    InputPhase::Cancel => {
      let event = controller.cancel();
      (
        event.as_ref().map_or(GestureKind::Drawing, event_kind),
        event,
      )
    }
    _ => return None,
  };
  let mut result = result_for(gesture, event.as_ref());
  apply_phase_cursor(phase, allow_drawing, &mut result);
  Some(RegionDispatch {
    result,
    event,
    gesture,
  })
}

pub fn event_kind(event: &ControllerEvent) -> GestureKind {
  match event {
    ControllerEvent::Changed { kind, .. } | ControllerEvent::Finished { kind, .. } => *kind,
    ControllerEvent::Cancelled { .. } => GestureKind::Drawing,
  }
}

pub fn result_for(kind: GestureKind, event: Option<&ControllerEvent>) -> OscResult {
  let (status, rect) = match event {
    Some(ControllerEvent::Changed { draft, .. }) => (ResultStatus::Changed, *draft),
    Some(ControllerEvent::Finished { committed, .. }) => (ResultStatus::Finished, *committed),
    Some(ControllerEvent::Cancelled { committed }) => (ResultStatus::Cancelled, *committed),
    None => (ResultStatus::None, None),
  };
  let mut result = OscResult {
    status: status as u8,
    ..Default::default()
  };
  result.gesture = match kind {
    GestureKind::Drawing => {
      result.cursor = CursorIcon::Crosshair as u8;
      RESULT_GESTURE_DRAWING
    }
    GestureKind::Moving => {
      result.cursor = CursorIcon::OpenHand as u8;
      RESULT_GESTURE_MOVING
    }
    GestureKind::Resizing(handle) => {
      result.handle = handle_tag(handle);
      result.cursor = match handle {
        Handle::North | Handle::South => CursorIcon::VerticalResize,
        Handle::East | Handle::West => CursorIcon::HorizontalResize,
        Handle::NorthEast | Handle::NorthWest | Handle::SouthEast | Handle::SouthWest => {
          CursorIcon::DiagonalResize
        }
        Handle::Body => CursorIcon::OpenHand,
      } as u8;
      RESULT_GESTURE_RESIZING
    }
  };
  if let Some(rect) = rect {
    result.has_region = 1;
    result.x = rect.origin.x;
    result.y = rect.origin.y;
    result.width = rect.size.width;
    result.height = rect.size.height;
  }
  result
}

pub fn apply_phase_cursor(phase: InputPhase, allow_drawing: bool, result: &mut OscResult) {
  if phase == InputPhase::Hover && result.gesture != RESULT_GESTURE_RESIZING {
    result.cursor = if allow_drawing {
      CursorIcon::Crosshair
    } else if result.gesture == RESULT_GESTURE_MOVING {
      CursorIcon::OpenHand
    } else {
      CursorIcon::Arrow
    } as u8;
  } else if phase == InputPhase::Down && result.gesture == RESULT_GESTURE_MOVING {
    result.cursor = CursorIcon::ClosedHand as u8;
  }
}

pub fn handle_tag(handle: Handle) -> u8 {
  match handle {
    Handle::Body => 1,
    Handle::North => 2,
    Handle::South => 3,
    Handle::East => 4,
    Handle::West => 5,
    Handle::NorthEast => 6,
    Handle::NorthWest => 7,
    Handle::SouthEast => 8,
    Handle::SouthWest => 9,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::osc::geometry::{Monitor, Rect, Size};

  fn controller(committed: Option<Rect>) -> RegionController {
    RegionController::new(
      Monitor {
        size: Size {
          width: 100.0,
          height: 80.0,
        },
      },
      committed,
      None,
    )
  }

  #[test]
  fn dispatches_a_complete_draw_with_shared_cursor_and_result_semantics() {
    let mut controller = controller(None);
    let modifiers = InputModifiers::default();
    let down = dispatch_region(
      &mut controller,
      InputPhase::Down,
      Point { x: 10.0, y: 10.0 },
      modifiers,
      true,
    )
    .unwrap();
    assert_eq!(down.gesture, GestureKind::Drawing);
    assert_eq!(down.result.cursor, CursorIcon::Crosshair as u8);

    let drag = dispatch_region(
      &mut controller,
      InputPhase::Drag,
      Point { x: 30.0, y: 25.0 },
      modifiers,
      true,
    )
    .unwrap();
    assert_eq!(drag.result.status, ResultStatus::Changed as u8);
    assert_eq!((drag.result.width, drag.result.height), (20.0, 15.0));

    let up = dispatch_region(
      &mut controller,
      InputPhase::Up,
      Point { x: 40.0, y: 30.0 },
      modifiers,
      true,
    )
    .unwrap();
    assert_eq!(up.result.status, ResultStatus::Finished as u8);
  }

  #[test]
  fn disabled_drawing_rejects_an_outside_press_but_keeps_move_input() {
    let committed = Rect::from_xywh(20.0, 20.0, 30.0, 20.0);
    let mut controller = controller(Some(committed));
    assert!(dispatch_region(
      &mut controller,
      InputPhase::Down,
      Point { x: 5.0, y: 5.0 },
      InputModifiers::default(),
      false,
    )
    .is_none());
    assert_eq!(controller.committed(), Some(committed));
    assert_eq!(
      dispatch_region(
        &mut controller,
        InputPhase::Down,
        Point { x: 30.0, y: 30.0 },
        InputModifiers::default(),
        false,
      )
      .unwrap()
      .gesture,
      GestureKind::Moving
    );
  }
}
