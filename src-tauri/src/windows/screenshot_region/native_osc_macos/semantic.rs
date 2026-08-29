// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::osc::{
  controller::ControllerEvent,
  geometry::{Handle, Rect},
  gesture::GestureKind,
};

use super::{
  ffi, InputPhase, NativeHandle, NativeOscEvent, NativeOscResult, NativeRegion, ResultStatus,
  SemanticGesture, SemanticStatus, RESULT_GESTURE_DRAWING, RESULT_GESTURE_MOVING,
  RESULT_GESTURE_RESIZING,
};

pub(super) fn apply_phase_cursor(phase: u32, allow_drawing: bool, result: &mut NativeOscResult) {
  if phase == InputPhase::Hover as u32 && result.gesture != RESULT_GESTURE_RESIZING {
    result.cursor = if allow_drawing {
      ffi::CURSOR_CROSSHAIR
    } else if result.gesture == RESULT_GESTURE_MOVING {
      ffi::CURSOR_OPEN_HAND
    } else {
      ffi::CURSOR_ARROW
    };
  } else if phase == InputPhase::Down as u32 && result.gesture == RESULT_GESTURE_MOVING {
    result.cursor = ffi::CURSOR_CLOSED_HAND;
  }
}

pub(super) fn event_kind(event: &ControllerEvent) -> GestureKind {
  match event {
    ControllerEvent::Changed { kind, .. } | ControllerEvent::Finished { kind, .. } => *kind,
    ControllerEvent::Cancelled { .. } => GestureKind::Drawing,
  }
}

pub(super) fn region(rect: Rect) -> NativeRegion {
  NativeRegion {
    x: rect.origin.x,
    y: rect.origin.y,
    width: rect.size.width,
    height: rect.size.height,
  }
}

pub(super) fn native_handle(handle: Handle) -> NativeHandle {
  match handle {
    Handle::Body => NativeHandle::Body,
    Handle::North => NativeHandle::North,
    Handle::South => NativeHandle::South,
    Handle::East => NativeHandle::East,
    Handle::West => NativeHandle::West,
    Handle::NorthEast => NativeHandle::NorthEast,
    Handle::NorthWest => NativeHandle::NorthWest,
    Handle::SouthEast => NativeHandle::SouthEast,
    Handle::SouthWest => NativeHandle::SouthWest,
  }
}

pub(super) fn semantic_gesture(kind: GestureKind) -> SemanticGesture {
  match kind {
    GestureKind::Drawing => SemanticGesture::Drawing,
    GestureKind::Moving => SemanticGesture::Moving,
    GestureKind::Resizing(handle) => SemanticGesture::Resizing {
      handle: native_handle(handle),
    },
  }
}

pub(super) fn result_for(k: GestureKind, e: Option<&ControllerEvent>) -> NativeOscResult {
  let (status, rect) = match e {
    Some(ControllerEvent::Changed { draft, .. }) => (ResultStatus::Changed, *draft),
    Some(ControllerEvent::Finished { committed, .. }) => (ResultStatus::Finished, *committed),
    Some(ControllerEvent::Cancelled { committed }) => (ResultStatus::Cancelled, *committed),
    None => (ResultStatus::None, None),
  };
  let mut result = NativeOscResult {
    status: status as u8,
    ..Default::default()
  };
  result.gesture = match k {
    GestureKind::Drawing => {
      result.cursor = ffi::CURSOR_CROSSHAIR;
      RESULT_GESTURE_DRAWING
    }
    GestureKind::Moving => {
      result.cursor = ffi::CURSOR_OPEN_HAND;
      RESULT_GESTURE_MOVING
    }
    GestureKind::Resizing(handle) => {
      result.handle = native_handle(handle) as u8;
      result.cursor = match handle {
        Handle::North | Handle::South => ffi::CURSOR_VERTICAL,
        Handle::East | Handle::West => ffi::CURSOR_HORIZONTAL,
        Handle::NorthEast | Handle::NorthWest | Handle::SouthEast | Handle::SouthWest => {
          ffi::CURSOR_DIAGONAL
        }
        Handle::Body => ffi::CURSOR_OPEN_HAND,
      };
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

pub(super) fn payload_for(event: &ControllerEvent, monitor_id: Option<u32>) -> NativeOscEvent {
  match event {
    ControllerEvent::Changed { draft, kind } => NativeOscEvent {
      status: SemanticStatus::Changed,
      gesture: Some(semantic_gesture(*kind)),
      region: draft.map(region),
      monitor_id,
    },
    ControllerEvent::Finished { committed, kind } => NativeOscEvent {
      status: SemanticStatus::Finished,
      gesture: Some(semantic_gesture(*kind)),
      region: committed.map(region),
      monitor_id,
    },
    ControllerEvent::Cancelled { committed } => NativeOscEvent {
      status: SemanticStatus::Cancelled,
      gesture: None,
      region: committed.map(region),
      monitor_id,
    },
  }
}
