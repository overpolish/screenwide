// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Physical-event builder for deterministic keyboard-chord display state.

#[path = "builder/key_down.rs"]
mod key_down;

use std::collections::HashMap;

use super::{role, KeyboardStateTimeline, LayoutTrack, TransitionKind, VisualKey, VisualRole};
use crate::editor::keyboard_effects::clock::AnimationClock;
use crate::editor::keyboard_effects::HOLD_US;
use crate::editor::keyboard_effects::{KeyPress, Shortcut};
use crate::editor::timeline_edit::{
  source_before_output_duration_us, DeletedKeyboardShortcutRange, KeyboardShortcutPositionRange,
  TimelineRange,
};

/// The timeline-edit state a chord's badge continuity depends on. A chord
/// continues its predecessor's badge only when both are visible then and
/// occupy the same place at the same size.
#[derive(Clone, Copy, Default)]
pub(in crate::editor::keyboard_effects) struct ChainContext<'a> {
  pub deleted_ids: &'a [u64],
  pub deleted_ranges: &'a [DeletedKeyboardShortcutRange],
  pub positions: &'a [KeyboardShortcutPositionRange],
  /// The edit's retained playback ranges; animation durations run on the
  /// output clock, so bake-time scheduling maps through them.
  pub ranges: Option<&'a [TimelineRange]>,
}

impl<'a> ChainContext<'a> {
  pub(super) fn clock(&self) -> AnimationClock<'a> {
    self
      .ranges
      .map_or_else(AnimationClock::source, AnimationClock::edited)
  }

  /// The source instant at which an exit fade must start so it has fully
  /// played out - in output time - by `press_us`.
  fn fade_finished_by(&self, press_us: u64) -> u64 {
    source_before_output_duration_us(self.ranges, press_us, super::EXIT_US)
      .unwrap_or_else(|| press_us.saturating_sub(super::EXIT_US))
  }
}

impl ChainContext<'_> {
  fn deleted(&self, shortcut: usize, at_ms: u64) -> bool {
    let shortcut = shortcut as u64;
    self.deleted_ids.contains(&shortcut)
      || self.deleted_ranges.iter().any(|range| {
        range.shortcut_id == shortcut && at_ms >= range.start_ms && at_ms < range.end_ms
      })
  }

  fn placement(&self, shortcut: usize, at_ms: u64) -> Option<(f64, f64, Option<f64>)> {
    self
      .positions
      .iter()
      .find(|position| {
        position.shortcut_id == shortcut as u64
          && at_ms >= position.start_ms
          && at_ms < position.end_ms
      })
      .map(|position| (position.center_x, position.center_y, position.size_percent))
  }

  fn deleted_at(&self, shortcut: usize, at_us: u64) -> bool {
    self.deleted(shortcut, at_us / 1_000)
  }

  fn chains(&self, previous: usize, next: usize, at_us: u64) -> bool {
    let at_ms = at_us / 1_000;
    if self.deleted(previous, at_ms) || self.deleted(next, at_ms) {
      return false;
    }
    self.placement(previous, at_ms) == self.placement(next, at_ms)
  }
}

#[derive(Clone, Copy, Debug)]
struct PhysicalEvent {
  shortcut: usize,
  at: u64,
  key_code: u16,
  modifier_mask: u32,
  down: bool,
}

#[derive(Clone, Copy, Debug)]
struct Slot {
  id: u32,
  group: u32,
  role: VisualRole,
  current: Option<usize>,
}

#[derive(Default)]
struct Builder<'a> {
  context: ChainContext<'a>,
  timeline: KeyboardStateTimeline,
  slots: Vec<Slot>,
  held: HashMap<u16, usize>,
  next_slot_id: u32,
  current_group: u32,
}

impl KeyboardStateTimeline {
  pub(in crate::editor::keyboard_effects) fn from_shortcuts(
    shortcuts: &[Shortcut],
    context: ChainContext<'_>,
  ) -> Self {
    let mut events = shortcuts
      .iter()
      .enumerate()
      .flat_map(|(shortcut, shortcut_data)| {
        shortcut_data
          .keys
          .iter()
          .flat_map(move |press| events_for_press(shortcut, press))
      })
      .flatten()
      .collect::<Vec<_>>();
    events.sort_by_key(|event| (event.at, event.down));
    let mut builder = Builder {
      context,
      ..Builder::default()
    };
    for event in events {
      if event.down {
        builder.key_down(event);
      } else {
        builder.key_up(event);
      }
    }
    builder.timeline
  }
}

fn events_for_press(shortcut: usize, press: &KeyPress) -> [Option<PhysicalEvent>; 2] {
  [
    Some(PhysicalEvent {
      shortcut,
      at: press.down_us,
      key_code: press.key_code,
      modifier_mask: press.modifier_mask,
      down: true,
    }),
    press.up_us.map(|at| PhysicalEvent {
      shortcut,
      at,
      key_code: press.key_code,
      modifier_mask: press.modifier_mask,
      down: false,
    }),
  ]
}

impl Builder<'_> {
  fn key_up(&mut self, event: PhysicalEvent) {
    self.clear_finished(event.at);
    let Some(visual) = self.held.remove(&event.key_code) else {
      return;
    };
    if self.held.is_empty() {
      let deadline = event.at.saturating_add(HOLD_US);
      let currents = self
        .slots
        .iter()
        .filter(|slot| slot.group == self.current_group)
        .filter_map(|slot| slot.current)
        .collect::<Vec<_>>();
      for current in currents {
        self.schedule_release(current, deadline, TransitionKind::GroupRelease);
      }
    } else {
      self.schedule_release(
        visual,
        event.at.saturating_add(HOLD_US),
        TransitionKind::Release,
      );
    }
  }

  fn retire_released(&mut self, at: u64) -> Vec<(u32, usize)> {
    let held = self.held.values().copied().collect::<Vec<_>>();
    let released = self
      .slots
      .iter()
      .filter(|slot| slot.group == self.current_group)
      .filter_map(|slot| {
        slot
          .current
          .filter(|visual| !held.contains(visual))
          .map(|visual| (slot.id, visual))
      })
      .collect::<Vec<_>>();
    for (slot_id, visual) in &released {
      self.start_exit(*visual, at, TransitionKind::Replacement);
      if let Some(slot) = self.slot_mut(*slot_id) {
        slot.current = None;
      }
    }
    released
  }

  fn schedule_release(&mut self, visual: usize, deadline: u64, kind: TransitionKind) {
    if self.slots.iter().any(|slot| slot.current == Some(visual)) {
      self.timeline.visuals[visual].exit = Some((deadline, kind));
      if kind == TransitionKind::Release {
        self.timeline.visuals[visual].layout_exit_us = Some(deadline);
      } else if kind == TransitionKind::GroupRelease {
        self.timeline.visuals[visual].layout_exit_us = None;
      }
    }
  }

  fn start_exit(&mut self, visual: usize, at: u64, kind: TransitionKind) {
    let known = &mut self.timeline.visuals[visual];
    let exit_at = known.exit.map_or(at, |(scheduled, _)| scheduled.min(at));
    known.exit = Some((exit_at, kind));
    if kind == TransitionKind::Replacement {
      known.layout_exit_us = None;
    }
  }

  fn clear_finished(&mut self, at: u64) {
    let clock = self.context.clock();
    let finished = self
      .slots
      .iter()
      .filter_map(|slot| {
        let visual = slot.current?;
        (!self.timeline.visuals[visual].visible_at(at, clock)).then_some(slot.id)
      })
      .collect::<Vec<_>>();
    for slot_id in finished {
      if let Some(slot) = self.slot_mut(slot_id) {
        slot.current = None;
      }
    }
  }

  fn slot(&self, id: u32) -> Option<&Slot> {
    self.slots.iter().find(|slot| slot.id == id)
  }

  fn slot_mut(&mut self, id: u32) -> Option<&mut Slot> {
    self.slots.iter_mut().find(|slot| slot.id == id)
  }

  fn latest_visible_in_slot(&self, slot: u32, at: u64) -> Option<usize> {
    self
      .timeline
      .visuals
      .iter()
      .enumerate()
      .filter(|(_, visual)| visual.slot_id == slot && visual.visible_at(at, self.context.clock()))
      .max_by_key(|(_, visual)| visual.enter_us)
      .map(|(index, _)| index)
  }
}
