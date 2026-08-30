// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Platform-neutral Button and IconButton OSC state. Native renderers consume
//! the resolved visuals while Rust remains the single owner of component
//! metrics, semantic colours, hit testing and pointer transitions.

mod confirm;
mod confirm_ffi;
mod ffi;
mod icons;
mod style;

use std::time::{Duration, Instant};

use super::geometry::{Point, Rect};

pub use confirm::{ConfirmAction, ConfirmActionSpec, ConfirmLayer, ConfirmUpdate};
pub use icons::ControlIcon;
pub use style::{
  control_metrics, control_visual, Appearance, ControlColor, ControlKind, ControlMetrics,
  ControlSize, ControlStyle, ControlVisual, Interaction,
};

const TRANSITION_DURATION: Duration = Duration::from_millis(150);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlSpec {
  pub rect: Rect,
  pub style: ControlStyle,
  pub icon: ControlIcon,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ControlUpdate {
  pub activated: usize,
  pub changed: bool,
  pub consumed: bool,
}

#[derive(Clone, Copy, Debug)]
struct Transition {
  from: [ControlVisual; 2],
  started: Instant,
}

#[derive(Clone, Copy, Debug)]
struct Control {
  interaction: Interaction,
  spec: ControlSpec,
  transition: Option<Transition>,
}

impl Control {
  fn new(spec: ControlSpec) -> Self {
    let interaction = if spec.style.disabled {
      Interaction::Disabled
    } else {
      Interaction::Normal
    };
    Self {
      interaction,
      spec,
      transition: None,
    }
  }

  fn visuals_at(&self, now: Instant) -> [ControlVisual; 2] {
    let target = [
      control_visual(self.spec.style, self.interaction, Appearance::Light),
      control_visual(self.spec.style, self.interaction, Appearance::Dark),
    ];
    let Some(transition) = self.transition else {
      return target;
    };
    let progress = (now
      .saturating_duration_since(transition.started)
      .as_secs_f32()
      / TRANSITION_DURATION.as_secs_f32())
    .min(1.0);
    [
      transition.from[0].mix(target[0], progress),
      transition.from[1].mix(target[1], progress),
    ]
  }

  fn set_interaction(&mut self, interaction: Interaction, now: Instant) -> bool {
    if self.interaction == interaction {
      return false;
    }
    let from = self.visuals_at(now);
    self.interaction = interaction;
    self.transition = Some(Transition { from, started: now });
    true
  }

  fn update_spec(&mut self, spec: ControlSpec, now: Instant) {
    if self.spec.style != spec.style {
      self.spec = spec;
      self.interaction = if spec.style.disabled {
        Interaction::Disabled
      } else {
        Interaction::Normal
      };
      self.transition = None;
    } else {
      self.spec.rect = spec.rect;
      if spec.style.disabled {
        self.set_interaction(Interaction::Disabled, now);
      }
    }
  }
}

#[derive(Default)]
pub struct ControlGroup {
  controls: Vec<Control>,
  hovered: usize,
  pressed: usize,
}

impl ControlGroup {
  pub fn layout(&mut self, specs: &[ControlSpec]) {
    let now = Instant::now();
    for (index, spec) in specs.iter().copied().enumerate() {
      if let Some(control) = self.controls.get_mut(index) {
        control.update_spec(spec, now);
      } else {
        self.controls.push(Control::new(spec));
      }
    }
    self.controls.truncate(specs.len());
    if self.hovered > specs.len() {
      self.hovered = 0;
    }
    if self.pressed > specs.len() {
      self.pressed = 0;
    }
    self.apply_interactions(now);
  }

  pub fn hit_index(&self, point: (f64, f64)) -> usize {
    let point = Point {
      x: point.0,
      y: point.1,
    };
    self
      .controls
      .iter()
      .position(|control| !control.spec.style.disabled && control.spec.rect.contains(point))
      .map_or(0, |index| index + 1)
  }

  pub fn down(&mut self, point: (f64, f64)) -> ControlUpdate {
    let hit = self.hit_index(point);
    self.hovered = hit;
    self.pressed = hit;
    let changed = self.apply_interactions(Instant::now());
    ControlUpdate {
      changed,
      consumed: hit != 0,
      ..Default::default()
    }
  }

  pub fn move_to(&mut self, point: (f64, f64)) -> ControlUpdate {
    let hovered = self.hit_index(point);
    let hover_changed = hovered != self.hovered;
    self.hovered = hovered;
    let changed = self.apply_interactions(Instant::now()) || hover_changed;
    ControlUpdate {
      changed,
      consumed: hovered != 0 || self.pressed != 0,
      ..Default::default()
    }
  }

  pub fn up(&mut self, point: (f64, f64)) -> ControlUpdate {
    let hovered = self.hit_index(point);
    let activated = usize::from(self.pressed != 0 && self.pressed == hovered) * self.pressed;
    let had_press = self.pressed != 0;
    self.hovered = hovered;
    self.pressed = 0;
    let changed = self.apply_interactions(Instant::now()) || had_press;
    ControlUpdate {
      activated,
      changed,
      consumed: had_press,
    }
  }

  pub fn clear_hover(&mut self) -> ControlUpdate {
    let hover_changed = self.hovered != 0;
    self.hovered = 0;
    let changed = self.apply_interactions(Instant::now()) || hover_changed;
    ControlUpdate {
      changed,
      consumed: self.pressed != 0,
      ..Default::default()
    }
  }

  pub fn visuals(&self, appearance: Appearance) -> Vec<ControlVisual> {
    let now = Instant::now();
    let index = usize::from(appearance == Appearance::Dark);
    self
      .controls
      .iter()
      .map(|control| control.visuals_at(now)[index])
      .collect()
  }

  pub fn is_animating(&self) -> bool {
    let now = Instant::now();
    self.controls.iter().any(|control| {
      control.transition.is_some_and(|transition| {
        now.saturating_duration_since(transition.started) < TRANSITION_DURATION
      })
    })
  }

  fn apply_interactions(&mut self, now: Instant) -> bool {
    let mut changed = false;
    for (index, control) in self.controls.iter_mut().enumerate() {
      let item = index + 1;
      let interaction = if control.spec.style.disabled {
        Interaction::Disabled
      } else if item == self.pressed {
        Interaction::Pressed
      } else if item == self.hovered {
        Interaction::Hovered
      } else {
        Interaction::Normal
      };
      changed |= control.set_interaction(interaction, now);
    }
    changed
  }
}

#[cfg(test)]
mod tests;
