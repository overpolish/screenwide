// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the recording's cursor and key files say about where the work
//! happened, measured in shares of the visible picture and milliseconds of
//! the recording.

use super::{Area, VisibleArea};
use crate::recording::cursor::{ButtonState, CursorButton, CursorRecord};
use crate::recording::keyboard::KeyboardRecord;

/// How far a press must travel before it is a drag rather than a click, as a
/// share of the visible picture.
const DRAG_DISTANCE: f64 = 0.03;
/// How long a press must be held before it can be a drag: a click whose
/// pointer slips while it lands is still a click.
const DRAG_MS: u64 = 150;
/// How far past the picture's edge a press still counts, for one landing on
/// the very edge.
const EDGE: f64 = 0.01;
/// How long after a press another app may come to the front for that press
/// to have brought it there.
const SWITCH_CAUSE_MS: u64 = 300;

const CLICK_WEIGHT: f64 = 1.0;
const DOUBLE_CLICK_WEIGHT: f64 = 1.2;
const DRAG_WEIGHT: f64 = 1.5;

/// One thing done at a place: a click or a drag.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Activity {
  pub start_ms: u64,
  pub end_ms: u64,
  pub area: Area,
  /// How much it matters that this is seen up close.
  pub weight: f64,
  /// This press brought another app to the front.
  pub brought_forward: bool,
}

/// Where the pointer was from `ms` until the next sample, in shares of the
/// visible picture. It may lie outside the picture.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct CursorSample {
  pub ms: u64,
  pub x: f64,
  pub y: f64,
}

#[derive(Debug, Default)]
pub(super) struct Signals {
  /// Sorted by start.
  pub activities: Vec<Activity>,
  /// When a key was pressed, sorted.
  pub keys: Vec<u64>,
  /// Sorted by time.
  pub cursor: Vec<CursorSample>,
  /// When another app came to the front, sorted.
  pub switches: Vec<u64>,
}

struct Press {
  button: CursorButton,
  ms: u64,
  x: f64,
  y: f64,
  clicks: u8,
}

impl Signals {
  pub(super) fn read(
    cursor: &[CursorRecord],
    keyboard: &[KeyboardRecord],
    visible: VisibleArea,
  ) -> Self {
    let Some(source) = cursor.iter().find_map(|record| match record {
      CursorRecord::Header { source, .. } => Some(source),
      _ => None,
    }) else {
      return Self::default();
    };
    let width = source.width.max(f64::EPSILON) * visible.width.max(f64::EPSILON);
    let height = source.height.max(f64::EPSILON) * visible.height.max(f64::EPSILON);
    let origin = (
      source.x + visible.x * source.width,
      source.y + visible.y * source.height,
    );
    let share = |x: f64, y: f64| ((x - origin.0) / width, (y - origin.1) / height);
    let mut signals = Self::default();
    let mut pressed: Vec<Press> = Vec::new();
    for record in cursor {
      match *record {
        CursorRecord::Position { timestamp_us, x, y }
        | CursorRecord::Visibility {
          timestamp_us,
          x,
          y,
          visible: true,
        } => {
          let (x, y) = share(x, y);
          signals.cursor.push(CursorSample {
            ms: timestamp_us / 1_000,
            x,
            y,
          });
        }
        CursorRecord::Button {
          button,
          click_count,
          state,
          timestamp_us,
          x,
          y,
        } => {
          let ms = timestamp_us / 1_000;
          let (x, y) = share(x, y);
          signals.cursor.push(CursorSample { ms, x, y });
          match state {
            ButtonState::Down => pressed.push(Press {
              button,
              ms,
              x,
              y,
              clicks: click_count,
            }),
            ButtonState::Up => {
              if let Some(index) = pressed.iter().position(|press| press.button == button) {
                let press = pressed.remove(index);
                signals
                  .activities
                  .extend(activity(&press, Some((ms, x, y))));
              }
            }
          }
        }
        CursorRecord::AppSwitch { timestamp_us } => signals.switches.push(timestamp_us / 1_000),
        _ => {}
      }
    }
    // A press the recording stopped before letting go of still landed.
    for press in &pressed {
      signals.activities.extend(activity(press, None));
    }
    signals.switches.sort_unstable();
    for activity in &mut signals.activities {
      move_to_switch(activity, &signals.switches);
    }
    signals.activities.sort_by_key(|activity| activity.start_ms);
    signals.cursor.sort_by_key(|sample| sample.ms);
    signals.keys = keyboard
      .iter()
      .filter_map(|record| match record {
        KeyboardRecord::Shortcut { timestamp_us, .. }
        | KeyboardRecord::KeyDown { timestamp_us, .. } => Some(timestamp_us / 1_000),
        _ => None,
      })
      .collect();
    signals.keys.sort_unstable();
    signals
  }
}

/// A press that brought another app to the front is the first thing done in
/// that app, so it is moved to the switch it caused: the app reports coming
/// forward a moment after the press that did it.
fn move_to_switch(activity: &mut Activity, switches: &[u64]) {
  let next = switches.partition_point(|&switch| switch <= activity.start_ms);
  if let Some(&switch) = switches
    .get(next)
    .filter(|&&switch| switch <= activity.start_ms + SWITCH_CAUSE_MS)
  {
    activity.start_ms = switch;
    activity.end_ms = activity.end_ms.max(switch);
    activity.brought_forward = true;
  }
}

fn inside(x: f64, y: f64) -> bool {
  (-EDGE..=1.0 + EDGE).contains(&x) && (-EDGE..=1.0 + EDGE).contains(&y)
}

/// What a press let go at `release` did. A press landing off the visible
/// picture did nothing there is to zoom into; a drag is kept to the picture.
fn activity(press: &Press, release: Option<(u64, f64, f64)>) -> Option<Activity> {
  if !inside(press.x, press.y) {
    return None;
  }
  let clamp = |value: f64| value.clamp(0.0, 1.0);
  let start = Area::point(clamp(press.x), clamp(press.y));
  let drag = release.filter(|&(ms, x, y)| {
    ms.saturating_sub(press.ms) >= DRAG_MS && (x - press.x).hypot(y - press.y) >= DRAG_DISTANCE
  });
  Some(match drag {
    Some((ms, x, y)) => Activity {
      start_ms: press.ms,
      end_ms: ms,
      area: start.union(Area::point(clamp(x), clamp(y))),
      weight: DRAG_WEIGHT,
      brought_forward: false,
    },
    None => Activity {
      start_ms: press.ms,
      end_ms: release.map_or(press.ms, |(ms, ..)| ms.max(press.ms)),
      area: start,
      weight: if press.clicks >= 2 {
        DOUBLE_CLICK_WEIGHT
      } else {
        CLICK_WEIGHT
      },
      brought_forward: false,
    },
  })
}
