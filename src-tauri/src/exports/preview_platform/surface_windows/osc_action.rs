// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::{Duration, Instant};

const TRANSITION_DURATION: Duration = Duration::from_millis(150);
const NORMAL: [f32; 2] = [0.924, 0.226];
const HOVERED: [f32; 2] = [0.832, 0.322];
const PRESSED: [f32; 2] = [0.918, 0.231];

pub(super) struct OscAction {
  hovered: u8,
  pressed: u8,
  rect: [f32; 4],
  secondary_rect: [f32; 4],
  transition_from: [f32; 2],
  transition_started: Option<Instant>,
  transition_to: [f32; 2],
}

impl Default for OscAction {
  fn default() -> Self {
    Self {
      hovered: 0,
      pressed: 0,
      rect: [0.0; 4],
      secondary_rect: [0.0; 4],
      transition_from: NORMAL,
      transition_started: None,
      transition_to: NORMAL,
    }
  }
}

impl OscAction {
  fn target(&self) -> [f32; 2] {
    if self.pressed != 0 {
      PRESSED
    } else if self.hovered != 0 {
      HOVERED
    } else {
      NORMAL
    }
  }

  fn shades_at(&self, now: Instant) -> [f32; 2] {
    let Some(started) = self.transition_started else {
      return self.transition_to;
    };
    let progress = (now.saturating_duration_since(started).as_secs_f32()
      / TRANSITION_DURATION.as_secs_f32())
    .min(1.0);
    [
      self.transition_from[0] + (self.transition_to[0] - self.transition_from[0]) * progress,
      self.transition_from[1] + (self.transition_to[1] - self.transition_from[1]) * progress,
    ]
  }

  fn begin_transition(&mut self) {
    let now = Instant::now();
    let target = self.target();
    if target == self.transition_to {
      return;
    }
    self.transition_from = self.shades_at(now);
    self.transition_to = target;
    self.transition_started = Some(now);
  }

  pub(super) fn hit(&self, point: (f64, f64)) -> bool {
    self.hit_index(point) != 0
  }

  fn hit_index(&self, point: (f64, f64)) -> u8 {
    if self.hit_rect(point, self.rect) {
      1
    } else if self.hit_rect(point, self.secondary_rect) {
      2
    } else {
      0
    }
  }

  fn hit_rect(&self, point: (f64, f64), rect: [f32; 4]) -> bool {
    let [x, y, width, height] = rect;
    width > 0.0
      && height > 0.0
      && point.0 >= f64::from(x)
      && point.0 <= f64::from(x + width)
      && point.1 >= f64::from(y)
      && point.1 <= f64::from(y + height)
  }

  pub(super) fn down(&mut self, point: (f64, f64)) -> bool {
    let action = self.hit_index(point);
    self.hovered = action;
    self.pressed = action;
    self.begin_transition();
    action != 0
  }

  pub(super) fn move_to(&mut self, point: (f64, f64)) -> (bool, bool) {
    let hovered = self.hit_index(point);
    let changed = hovered != self.hovered;
    self.hovered = hovered;
    if changed {
      self.begin_transition();
    }
    (hovered != 0 || self.pressed != 0, changed)
  }

  pub(super) fn up(&mut self, point: (f64, f64)) -> (bool, bool) {
    let hovered = self.hit_index(point);
    let activate = self.pressed != 0 && self.pressed == hovered;
    let changed = self.pressed != 0 || hovered != self.hovered;
    self.hovered = hovered;
    self.pressed = 0;
    if changed {
      self.begin_transition();
    }
    (activate, changed)
  }

  pub(super) fn second_half(&self, point: (f64, f64)) -> bool {
    self.hit_rect(point, self.secondary_rect)
  }

  pub(super) fn layout(
    &mut self,
    label: [f32; 4],
    secondary_label: Option<[f32; 4]>,
    scale: f32,
    visible: bool,
  ) -> [f32; 4] {
    if !visible || label[2] <= 0.0 {
      *self = Self::default();
      return [0.0; 4];
    }
    self.rect = [
      label[0] - 6.0 * scale,
      label[1] - 4.0 * scale,
      label[2] + 12.0 * scale,
      label[3] + 8.0 * scale,
    ];
    self.secondary_rect = [0.0; 4];
    if let Some(label) = secondary_label {
      self.secondary_rect = [
        label[0] - 6.0 * scale,
        label[1] - 4.0 * scale,
        label[2] + 12.0 * scale,
        label[3] + 8.0 * scale,
      ];
    }
    let now = Instant::now();
    let shades = self.shades_at(now);
    if self
      .transition_started
      .is_some_and(|started| now.saturating_duration_since(started) >= TRANSITION_DURATION)
    {
      self.transition_started = None;
    }
    let mut actions = [NORMAL[0], NORMAL[1], NORMAL[0], NORMAL[1]];
    match if self.pressed != 0 {
      self.pressed
    } else {
      self.hovered
    } {
      1 => actions[..2].copy_from_slice(&shades),
      2 => actions[2..].copy_from_slice(&shades),
      _ => {}
    }
    actions
  }

  pub(super) fn is_animating(&self) -> bool {
    self.transition_started.is_some()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn action_layout_matches_the_compact_button_box() {
    let mut action = OscAction::default();
    let state = action.layout([100.0, 200.0, 50.0, 16.0], None, 1.0, true);

    assert_eq!(state, [NORMAL[0], NORMAL[1], NORMAL[0], NORMAL[1]]);
    assert!(action.hit((94.0, 196.0)));
    assert!(action.hit((156.0, 220.0)));
    assert!(!action.hit((93.9, 196.0)));
    assert!(!action.hit((156.1, 220.0)));
  }

  #[test]
  fn split_action_has_two_buttons_and_does_not_activate_the_gap() {
    let mut action = OscAction::default();
    action.layout(
      [100.0, 200.0, 50.0, 16.0],
      Some([166.0, 200.0, 90.0, 16.0]),
      1.0,
      true,
    );

    assert!(action.hit((100.0, 200.0)));
    assert!(!action.second_half((100.0, 200.0)));
    assert!(!action.hit((158.0, 200.0)));
    assert!(action.hit((166.0, 200.0)));
    assert!(action.second_half((166.0, 200.0)));

    assert!(action.down((100.0, 200.0)));
    assert_eq!(action.hovered, 1);
    action.move_to((166.0, 200.0));
    assert_eq!(action.hovered, 2);
    assert_eq!(action.up((166.0, 200.0)).0, false);
  }
}
