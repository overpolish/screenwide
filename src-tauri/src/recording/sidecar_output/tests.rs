// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::recording::cursor::{ButtonState, CursorButton, CursorRecord, CursorStyle};
use crate::recording::keyboard::KeyboardRecord;

type Cursor = RollingRecords<CursorRecord, crate::recording::cursor::CursorBaseline>;
type Keys = RollingRecords<KeyboardRecord, crate::recording::keyboard::KeyboardBaseline>;

fn position(timestamp_us: u64, x: f64) -> CursorRecord {
  CursorRecord::Position {
    timestamp_us,
    x,
    y: 0.0,
  }
}

fn appearance(timestamp_us: u64, style: CursorStyle) -> CursorRecord {
  CursorRecord::Appearance {
    height: 16.0,
    hotspot_x: 0.0,
    hotspot_y: 0.0,
    style,
    timestamp_us,
    width: 16.0,
  }
}

fn button(timestamp_us: u64, state: ButtonState) -> CursorRecord {
  CursorRecord::Button {
    button: CursorButton::Left,
    click_count: 1,
    state,
    timestamp_us,
    x: 0.0,
    y: 0.0,
  }
}

fn key(timestamp_us: u64, key_code: u16, down: bool) -> KeyboardRecord {
  if down {
    KeyboardRecord::KeyDown {
      key_code,
      modifiers: Vec::new(),
      timestamp_us,
    }
  } else {
    KeyboardRecord::KeyUp {
      key_code,
      modifiers: Vec::new(),
      timestamp_us,
    }
  }
}

fn push_all<R: TimedRecord, B: Baseline<R>>(rolling: &mut RollingRecords<R, B>, records: &[R]) {
  for record in records {
    rolling.push(record.clone());
  }
}

#[test]
fn a_clip_opens_with_the_cursor_it_inherited_and_rebases_what_follows() {
  let mut rolling = Cursor::new(10_000_000);
  push_all(
    &mut rolling,
    &[
      appearance(0, CursorStyle::Arrow),
      position(100, 1.0),
      appearance(2_000, CursorStyle::IBeam),
      position(3_000, 2.0),
      position(5_000, 3.0),
      position(9_000, 4.0),
    ],
  );

  assert_eq!(
    rolling.clip(4_000, 6_000),
    [
      appearance(0, CursorStyle::IBeam),
      position(0, 2.0),
      position(1_000, 3.0),
    ]
  );
}

#[test]
fn state_from_records_past_the_horizon_survives_into_a_later_clip() {
  let mut rolling = Cursor::new(1_000);
  push_all(
    &mut rolling,
    &[
      appearance(0, CursorStyle::PointingHand),
      button(10, ButtonState::Down),
      position(20, 5.0),
      // Everything above is now older than the horizon and folded away.
      position(5_000, 6.0),
    ],
  );

  assert_eq!(
    rolling.clip(4_500, 5_000),
    [
      appearance(0, CursorStyle::PointingHand),
      position(0, 5.0),
      button(0, ButtonState::Down),
      position(500, 6.0),
    ]
  );
}

#[test]
fn a_released_button_is_not_restated() {
  let mut rolling = Cursor::new(10_000);
  push_all(
    &mut rolling,
    &[
      button(10, ButtonState::Down),
      button(20, ButtonState::Up),
      position(30, 1.0),
    ],
  );

  assert_eq!(rolling.clip(100, 200), [position(0, 1.0)]);
}

#[test]
fn a_hidden_cursor_stays_hidden_but_a_shown_one_needs_no_record() {
  let hidden = CursorRecord::Visibility {
    timestamp_us: 10,
    visible: false,
    x: 1.0,
    y: 1.0,
  };
  let shown = CursorRecord::Visibility {
    timestamp_us: 20,
    visible: true,
    x: 1.0,
    y: 1.0,
  };

  let mut rolling = Cursor::new(10_000);
  push_all(&mut rolling, std::slice::from_ref(&hidden));
  assert_eq!(rolling.clip(100, 200), [hidden.at_timestamp_us(0)]);

  push_all(&mut rolling, &[shown]);
  assert!(rolling.clip(100, 200).is_empty());
}

#[test]
fn keys_held_across_the_start_are_restated_and_released_ones_are_not() {
  let mut rolling = Keys::new(10_000);
  push_all(
    &mut rolling,
    &[
      key(10, 55, true),
      key(20, 56, true),
      key(30, 56, false),
      KeyboardRecord::Typing { timestamp_us: 40 },
      key(150, 55, false),
    ],
  );

  assert_eq!(
    rolling.clip(100, 200),
    [key(0, 55, true), key(50, 55, false)]
  );
}

#[test]
fn records_after_the_clip_are_left_out() {
  let mut rolling = Keys::new(10_000);
  push_all(&mut rolling, &[key(100, 1, true), key(300, 1, false)]);

  assert_eq!(rolling.clip(0, 200), [key(100, 1, true)]);
}
