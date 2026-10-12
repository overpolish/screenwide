// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn button(
  event_type: CGEventType,
  event: &CGEvent,
) -> Option<(CursorButton, ButtonState)> {
  let state = match event_type {
    CGEventType::LeftMouseDown | CGEventType::RightMouseDown | CGEventType::OtherMouseDown => {
      ButtonState::Down
    }
    CGEventType::LeftMouseUp | CGEventType::RightMouseUp | CGEventType::OtherMouseUp => {
      ButtonState::Up
    }
    _ => return None,
  };
  let number = event.get_integer_value_field(EventField::MOUSE_EVENT_BUTTON_NUMBER);
  let button = match number {
    0 => CursorButton::Left,
    1 => CursorButton::Right,
    2 => CursorButton::Middle,
    other => CursorButton::Other(u8::try_from(other).unwrap_or(u8::MAX)),
  };
  Some((button, state))
}

pub(super) fn raw_event(
  catalog: &CursorCatalog,
  event_type: CGEventType,
  event: &CGEvent,
  at: Instant,
) -> RawCursorEvent {
  let point = event.location();
  let kind = button(event_type, event).map_or(RawCursorEventKind::Move, |(button, state)| {
    let click_count = event
      .get_integer_value_field(EventField::MOUSE_EVENT_CLICK_STATE)
      .clamp(0, i64::from(u8::MAX)) as u8;
    RawCursorEventKind::Button {
      button,
      click_count,
      state,
    }
  });
  RawCursorEvent {
    appearance: catalog.current(),
    at,
    kind,
    x: point.x,
    y: point.y,
  }
}

pub(super) fn current_event(
  catalog: &CursorCatalog,
  kind: RawCursorEventKind,
) -> Option<RawCursorEvent> {
  let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState).ok()?;
  let event = CGEvent::new(source).ok()?;
  let mut event = raw_event(catalog, CGEventType::MouseMoved, &event, Instant::now());
  event.kind = kind;
  Some(event)
}

pub(super) fn is_motion(event_type: CGEventType) -> bool {
  matches!(
    event_type,
    CGEventType::MouseMoved
      | CGEventType::LeftMouseDragged
      | CGEventType::RightMouseDragged
      | CGEventType::OtherMouseDragged
  )
}
