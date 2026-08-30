// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::{Duration, Instant};

use super::{
  control_visual, Appearance, ControlColor, ControlIcon, ControlSize, ControlStyle, Interaction,
};

const ICON_SWAP_DURATION: Duration = Duration::from_millis(160);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConfirmActionSpec {
  pub idle_icon: ControlIcon,
  pub armed_icon: ControlIcon,
  pub idle_color: ControlColor,
  pub armed_color: ControlColor,
  pub timeout: Duration,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConfirmUpdate {
  pub confirmed: bool,
  pub changed: bool,
  pub animating: bool,
  pub armed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConfirmLayer {
  pub icon: ControlIcon,
  pub foreground: [f32; 4],
  pub opacity: f32,
  pub scale: f32,
}

#[derive(Clone, Copy, Debug)]
struct Transition {
  from_armed: bool,
  started: Instant,
}

pub struct ConfirmAction {
  spec: ConfirmActionSpec,
  armed: bool,
  armed_at: Option<Instant>,
  transition: Option<Transition>,
}

impl ConfirmAction {
  pub fn new(spec: ConfirmActionSpec) -> Self {
    Self {
      spec,
      armed: false,
      armed_at: None,
      transition: None,
    }
  }

  pub fn press(&mut self, now: Instant) -> ConfirmUpdate {
    if self.armed
      && self
        .armed_at
        .is_some_and(|armed| now.saturating_duration_since(armed) < self.spec.timeout)
    {
      self.set_armed(false, now);
      return ConfirmUpdate {
        confirmed: true,
        changed: true,
        animating: true,
        armed: false,
      };
    }
    // A delayed platform timer must not turn a press after the deadline into
    // a confirmation. Restart from the idle state even if expiry has not yet
    // been delivered.
    if self.armed {
      self.armed = false;
      self.armed_at = None;
      self.transition = None;
    }
    self.armed_at = Some(now);
    self.set_armed(true, now);
    ConfirmUpdate {
      changed: true,
      animating: true,
      armed: true,
      ..Default::default()
    }
  }

  pub fn expire(&mut self, now: Instant) -> ConfirmUpdate {
    let expired = self.armed
      && self
        .armed_at
        .is_some_and(|armed| now.saturating_duration_since(armed) >= self.spec.timeout);
    if !expired {
      return ConfirmUpdate {
        armed: self.armed,
        ..Default::default()
      };
    }
    self.set_armed(false, now);
    ConfirmUpdate {
      changed: true,
      animating: true,
      armed: false,
      ..Default::default()
    }
  }

  pub fn layers(&self, now: Instant, appearance: Appearance) -> [ConfirmLayer; 2] {
    let progress = self.transition.map_or(1.0, |transition| {
      (now
        .saturating_duration_since(transition.started)
        .as_secs_f32()
        / ICON_SWAP_DURATION.as_secs_f32())
      .min(1.0)
    });
    let eased = 1.0 - (1.0 - progress).powi(3);
    let armed_amount = self
      .transition
      .map_or(if self.armed { 1.0 } else { 0.0 }, |transition| {
        if transition.from_armed {
          1.0 - eased
        } else {
          eased
        }
      });
    [
      self.layer(
        self.spec.idle_icon,
        self.spec.idle_color,
        1.0 - armed_amount,
        appearance,
      ),
      self.layer(
        self.spec.armed_icon,
        self.spec.armed_color,
        armed_amount,
        appearance,
      ),
    ]
  }

  pub fn is_animating(&self, now: Instant) -> bool {
    self.transition.is_some_and(|transition| {
      now.saturating_duration_since(transition.started) < ICON_SWAP_DURATION
    })
  }

  fn set_armed(&mut self, armed: bool, now: Instant) {
    if self.armed == armed {
      return;
    }
    self.transition = Some(Transition {
      from_armed: self.armed,
      started: now,
    });
    self.armed = armed;
    if !armed {
      self.armed_at = None;
    }
  }

  fn layer(
    &self,
    icon: ControlIcon,
    color: ControlColor,
    amount: f32,
    appearance: Appearance,
  ) -> ConfirmLayer {
    let foreground = control_visual(
      ControlStyle::icon_button(color, ControlSize::Compact),
      Interaction::Normal,
      appearance,
    )
    .foreground;
    ConfirmLayer {
      icon,
      foreground,
      opacity: amount,
      scale: amount,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn action() -> ConfirmAction {
    ConfirmAction::new(ConfirmActionSpec {
      idle_icon: ControlIcon::X,
      armed_icon: ControlIcon::Trash2,
      idle_color: ControlColor::Neutral,
      armed_color: ControlColor::Error,
      timeout: Duration::from_secs(2),
    })
  }

  #[test]
  fn arbitrary_icons_arm_confirm_and_expire() {
    let now = Instant::now();
    let mut action = action();
    assert!(action.press(now).armed);
    assert!(action.press(now + Duration::from_millis(1900)).confirmed);
    assert!(action.press(now + Duration::from_secs(3)).armed);
    assert!(action.expire(now + Duration::from_secs(5)).changed);
  }

  #[test]
  fn icon_layers_crossfade_scale_and_use_configured_colors() {
    let now = Instant::now();
    let mut action = action();
    action.press(now);
    let start = action.layers(now, Appearance::Light);
    assert_eq!((start[0].opacity, start[1].opacity), (1.0, 0.0));
    let middle = action.layers(now + Duration::from_millis(80), Appearance::Light);
    assert!(middle[0].opacity < 1.0 && middle[0].opacity > 0.0);
    assert!(middle[1].scale > 0.0 && middle[1].scale < 1.0);
    assert_eq!(
      middle[1].foreground,
      [215.0 / 255.0, 0.0, 21.0 / 255.0, 1.0]
    );
  }

  #[test]
  fn press_after_timeout_rearms_instead_of_confirming() {
    let now = Instant::now();
    let mut action = action();
    action.press(now);
    let update = action.press(now + Duration::from_millis(2001));
    assert!(update.armed);
    assert!(!update.confirmed);
    let layers = action.layers(now + Duration::from_millis(2001), Appearance::Light);
    assert_eq!((layers[0].opacity, layers[1].opacity), (1.0, 0.0));
  }
}
