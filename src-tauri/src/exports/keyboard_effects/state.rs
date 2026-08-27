// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Deterministic keyboard-chord display state built from physical key events.

mod builder;
pub(super) use builder::ChainContext;

use super::layout::LayoutTrack;
#[path = "role.rs"]
mod role_model;
pub(super) use role_model::{role, VisualRole};

pub(super) const EXIT_US: u64 = (super::EXIT_SECONDS * super::MICROS_PER_SECOND) as u64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum TransitionKind {
  Release,
  GroupRelease,
  Replacement,
  Detached,
}

#[derive(Clone, Debug)]
pub(super) struct VisualKey {
  pub source_shortcut: usize,
  /// Badge-continuity group. A chord joins its predecessor's group only when
  /// it appears in the same place at the same size while that badge is still
  /// on screen; slot reuse, replacement morphs, layout motion and anchoring
  /// never cross group boundaries.
  pub group: u32,
  pub key_code: u16,
  pub modifier_mask: u32,
  pub role: VisualRole,
  pub slot_id: u32,
  pub enter_us: u64,
  pub animation_enter_us: u64,
  pub replacement_enter: bool,
  pub layout_exit_us: Option<u64>,
  pub layout_anchor_until_us: Option<u64>,
  pub exit: Option<(u64, TransitionKind)>,
  pub layout: LayoutTrack,
}

impl VisualKey {
  pub(super) fn visible_at(&self, now: u64) -> bool {
    let artwork_visible = self
      .exit
      .is_none_or(|(exit_us, _)| now < exit_us.saturating_add(EXIT_US));
    let layout_visible = self
      .layout_anchor_until_us
      .is_some_and(|until_us| now < until_us);
    self.animation_enter_us <= now && (artwork_visible || layout_visible)
  }
}

#[derive(Clone, Debug, Default)]
pub(super) struct KeyboardStateTimeline {
  pub visuals: Vec<VisualKey>,
}
