// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(Default)]
pub(super) struct OscAction {
  hovered: bool,
  pressed: bool,
  rect: [f32; 4],
}

impl OscAction {
  fn hit(&self, point: (f64, f64)) -> bool {
    let [x, y, width, height] = self.rect;
    point.0 >= f64::from(x)
      && point.0 <= f64::from(x + width)
      && point.1 >= f64::from(y)
      && point.1 <= f64::from(y + height)
  }

  pub(super) fn down(&mut self, point: (f64, f64)) -> bool {
    let hit = self.hit(point);
    self.hovered = hit;
    self.pressed = hit;
    hit
  }

  pub(super) fn move_to(&mut self, point: (f64, f64)) -> (bool, bool) {
    let hovered = self.hit(point);
    let changed = hovered != self.hovered;
    self.hovered = hovered;
    (hovered || self.pressed, changed)
  }

  pub(super) fn up(&mut self, point: (f64, f64)) -> (bool, bool) {
    let hovered = self.hit(point);
    let activate = self.pressed && hovered;
    let changed = self.pressed || hovered != self.hovered;
    self.hovered = hovered;
    self.pressed = false;
    (activate, changed)
  }

  pub(super) fn layout(&mut self, label: [f32; 4], scale: f32, visible: bool) -> f32 {
    if !visible || label[2] <= 0.0 {
      *self = Self::default();
      return 0.0;
    }
    self.rect = [
      label[0] - 7.0 * scale,
      label[1] - 4.0 * scale,
      label[2] + 14.0 * scale,
      label[3] + 8.0 * scale,
    ];
    if self.pressed {
      3.0
    } else if self.hovered {
      2.0
    } else {
      1.0
    }
  }
}
