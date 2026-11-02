// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A scripted pointer written as the cursor track a recording saves beside
//! its movie, and its typing as the keyboard track, so the cursor's look,
//! its smoothing and the auto zooms all come from the code that reads a
//! real one.

use screenwide_records::cursor::{
  ButtonState, CursorButton, CursorRecord, CursorSource, CursorSourceKind, CursorStyle,
};
use screenwide_records::keyboard::KeyboardRecord;

use super::PointerScript;

/// `script`'s typing as a keyboard track: when, and nothing of what, as a
/// recording keeps typing into a text field.
pub(super) fn keyboard_records(script: &PointerScript) -> Vec<KeyboardRecord> {
  std::iter::once(KeyboardRecord::Header {
    platform: "settings-preview".to_owned(),
    timebase: "recording-microseconds".to_owned(),
    version: screenwide_records::keyboard::FORMAT_VERSION,
  })
  .chain(
    script
      .typing_ms
      .iter()
      .map(|&at_ms| KeyboardRecord::Typing {
        timestamp_us: micros(at_ms),
      }),
  )
  .collect()
}

/// How often the track samples the pointer between keys: about the rate
/// the system reports a moving pointer at.
const SAMPLE_MS: u32 = 8;

/// The arrow's box and hotspot in points, which the cursor's artwork refits
/// to its own.
const ARROW: [f64; 4] = [17.0, 23.0, 4.0, 4.0];

/// `script` as a cursor track over a screen `points` in size, recorded at
/// `pixels`.
pub(super) fn cursor_records(
  script: &PointerScript,
  points: (f64, f64),
  pixels: (u32, u32),
) -> Vec<CursorRecord> {
  let mut records = vec![
    CursorRecord::Header {
      coordinate_space: "global-points".to_owned(),
      platform: "settings-preview".to_owned(),
      source: CursorSource {
        height: points.1,
        kind: CursorSourceKind::Screen,
        platform_id: String::new(),
        video_height: pixels.1,
        video_width: pixels.0,
        width: points.0,
        x: 0.0,
        y: 0.0,
      },
      timebase: "recording-microseconds".to_owned(),
      version: screenwide_records::cursor::FORMAT_VERSION,
    },
    CursorRecord::Appearance {
      width: ARROW[0],
      height: ARROW[1],
      hotspot_x: ARROW[2],
      hotspot_y: ARROW[3],
      style: CursorStyle::Arrow,
      timestamp_us: 0,
    },
  ];
  let mut pressed = false;
  for (index, key) in script.keys.iter().enumerate() {
    if let Some(previous) = index.checked_sub(1).map(|index| &script.keys[index]) {
      let span = key.at_ms.saturating_sub(previous.at_ms);
      let moved = previous.x != key.x || previous.y != key.y;
      let mut at = SAMPLE_MS;
      while moved && at < span {
        let share = ease(f64::from(at) / f64::from(span));
        records.push(CursorRecord::Position {
          timestamp_us: micros(previous.at_ms + at),
          x: lerp(previous.x, key.x, share),
          y: lerp(previous.y, key.y, share),
        });
        at += SAMPLE_MS;
      }
    }
    let timestamp_us = micros(key.at_ms);
    if key.pressed != pressed {
      pressed = key.pressed;
      records.push(CursorRecord::Button {
        button: CursorButton::Left,
        click_count: 1,
        state: if pressed {
          ButtonState::Down
        } else {
          ButtonState::Up
        },
        timestamp_us,
        x: key.x,
        y: key.y,
      });
    } else {
      records.push(CursorRecord::Position {
        timestamp_us,
        x: key.x,
        y: key.y,
      });
    }
  }
  records
}

fn micros(ms: u32) -> u64 {
  u64::from(ms) * 1_000
}

fn lerp(from: f64, to: f64, share: f64) -> f64 {
  from + (to - from) * share
}

/// A hand's move: gathering speed, then settling on the target.
fn ease(share: f64) -> f64 {
  let share = share.clamp(0.0, 1.0);
  share * share * (3.0 - 2.0 * share)
}
