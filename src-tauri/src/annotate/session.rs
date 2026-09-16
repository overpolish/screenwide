// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The overlay's session identity.
//!
//! A display change, a second toggle or another capture tool can land while a
//! start is still building its host windows. The generation counter is what
//! lets the late half of a superseded start recognise that the session it was
//! building for is gone.

use std::sync::Mutex;

/// What the overlay is doing.
///
/// Drawing and showing are deliberately separate: annotations kept after exiting
/// stay on screen, drawn by hosts that no longer take input, so what the
/// screen shows and what a recording counts as visible are the same thing.
#[derive(Clone, Copy, Default, Eq, PartialEq)]
enum Phase {
  #[default]
  Idle,
  Drawing,
  Showing,
}

#[derive(Default)]
struct Session {
  generation: u64,
  phase: Phase,
}

#[derive(Default)]
pub struct AnnotateState(Mutex<Session>);

impl AnnotateState {
  fn session(&self) -> std::sync::MutexGuard<'_, Session> {
    self
      .0
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
  }

  pub(super) fn begin(&self) -> u64 {
    let mut session = self.session();
    session.generation = session.generation.wrapping_add(1);
    session.phase = Phase::Drawing;
    session.generation
  }

  /// Whether `generation` is still the live session, which is what decides if
  /// built host windows are shown or thrown away.
  pub(super) fn install(&self, generation: u64) -> bool {
    let session = self.session();
    session.phase == Phase::Drawing && session.generation == generation
  }

  /// Drawing ends and the annotations stay on screen. Reports whether there was a
  /// drawing session to end.
  pub(super) fn show(&self) -> bool {
    let mut session = self.session();
    session.generation = session.generation.wrapping_add(1);
    std::mem::replace(&mut session.phase, Phase::Showing) == Phase::Drawing
  }

  /// Everything ends, reporting whether anything was up.
  pub(super) fn cancel(&self) -> bool {
    let mut session = self.session();
    session.generation = session.generation.wrapping_add(1);
    std::mem::replace(&mut session.phase, Phase::Idle) != Phase::Idle
  }

  /// The live drawing session. Showing annotations are not one: they take no input,
  /// so they own neither Escape nor the pointer.
  pub(super) fn active_generation(&self) -> Option<u64> {
    let session = self.session();
    (session.phase == Phase::Drawing).then_some(session.generation)
  }

  pub(super) fn is_showing(&self) -> bool {
    self.session().phase == Phase::Showing
  }
}
