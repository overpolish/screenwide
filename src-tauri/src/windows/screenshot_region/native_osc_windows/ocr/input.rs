// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Chrome {
  /// The 2s armed-close timeout (`+ocr_toolbar_input.m:20-34`).
  pub(crate) fn expire_confirm(&mut self) -> ControlOutcome {
    self.expire_confirm_at(Instant::now())
  }

  pub(super) fn expire_confirm_at(&mut self, now: Instant) -> ControlOutcome {
    if !self.close_armed {
      return ControlOutcome::default();
    }
    let update = self.confirm.expire(now);
    self.close_armed = update.armed;
    ControlOutcome {
      redraw: update.changed,
      animating: update.animating,
      // A platform timer can arrive just before the monotonic deadline. Keep
      // the one-shot alive until ConfirmAction actually reports expiry.
      arm_confirm: update.armed,
      ..Default::default()
    }
  }

  /// Port of `screenwide_region_osc_ocr_control_input` (`+ocr_toolbar_input.m:69`):
  /// the toolbar is offered the event first, then the cancel button.
  pub(crate) fn control_input(&mut self, point: Point, phase: u32) -> ControlOutcome {
    let toolbar = self.toolbar_input(point, phase);
    if toolbar.consumed {
      return toolbar;
    }
    let cancel = self.cancel_input(point, phase);
    ControlOutcome {
      redraw: toolbar.redraw || cancel.redraw,
      animating: toolbar.animating || cancel.animating,
      ..cancel
    }
  }

  pub(super) fn toolbar_input(&mut self, point: Point, phase: u32) -> ControlOutcome {
    if !self.toolbar_visible {
      return ControlOutcome::default();
    }
    let update = dispatch_phase(&mut self.toolbar, point, phase);
    let mut outcome = ControlOutcome {
      consumed: update.consumed,
      redraw: update.changed,
      animating: self.toolbar.is_animating(),
      ..Default::default()
    };
    if update.activated == CONTROL_COUNT {
      let confirm = self.confirm.press(Instant::now());
      self.close_armed = confirm.armed;
      outcome.redraw |= confirm.changed;
      outcome.animating |= confirm.animating;
      outcome.arm_confirm = confirm.armed;
      outcome.disarm_confirm = !confirm.armed;
      if confirm.confirmed {
        outcome.dispatch = Some(12);
      }
    } else if update.activated != 0 {
      outcome.dispatch = Some(8 + update.activated as u32);
    }
    outcome
  }

  pub(super) fn cancel_input(&mut self, point: Point, phase: u32) -> ControlOutcome {
    if !self.cancel_visible {
      return ControlOutcome::default();
    }
    let update = dispatch_phase(&mut self.cancel, point, phase);
    ControlOutcome {
      consumed: update.consumed,
      dispatch: (update.activated != 0).then_some(8),
      redraw: update.changed,
      animating: self.cancel.is_animating(),
      ..Default::default()
    }
  }
}

fn dispatch_phase(
  group: &mut ControlGroup,
  point: Point,
  phase: u32,
) -> crate::osc::controls::ControlUpdate {
  match phase {
    PHASE_HOVER | PHASE_DRAG => group.move_to((point.x, point.y)),
    PHASE_DOWN => group.down((point.x, point.y)),
    PHASE_UP => group.up((point.x, point.y)),
    _ => group.clear_hover(),
  }
}
