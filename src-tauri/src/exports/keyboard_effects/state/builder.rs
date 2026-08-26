// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Physical-event builder for deterministic keyboard-chord display state.

use std::collections::HashMap;

use super::{role, KeyboardStateTimeline, LayoutTrack, TransitionKind, VisualKey, VisualRole};
use crate::exports::keyboard_effects::HOLD_US;
use crate::exports::keyboard_effects::{KeyPress, Shortcut};

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
  role: VisualRole,
  current: Option<usize>,
}

#[derive(Default)]
struct Builder {
  timeline: KeyboardStateTimeline,
  slots: Vec<Slot>,
  held: HashMap<u16, usize>,
  next_slot_id: u32,
}

impl KeyboardStateTimeline {
  pub(in crate::exports::keyboard_effects) fn from_shortcuts(shortcuts: &[Shortcut]) -> Self {
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
    let mut builder = Builder::default();
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

impl Builder {
  fn key_down(&mut self, event: PhysicalEvent) {
    self.clear_finished(event.at);
    if self.held.contains_key(&event.key_code) {
      return;
    }
    let role = role(event.key_code);
    let new_physical_chord = self.held.is_empty();
    let mut reusable_slot = None;
    let mut replacement_enter = false;
    let mut replacement_candidates = Vec::new();
    if new_physical_chord {
      for index in 0..self.slots.len() {
        if let Some(visual) = self.slots[index].current.take() {
          self.start_exit(visual, event.at, TransitionKind::Replacement);
          replacement_candidates.push((self.slots[index].id, visual));
        }
      }
    } else {
      replacement_candidates = self.retire_released(event.at);
      reusable_slot = replacement_candidates.iter().find_map(|(slot, _)| {
        self
          .slot(*slot)
          .is_some_and(|known| known.role.same_slot_kind(role))
          .then_some(*slot)
      });
      if role == VisualRole::Primary {
        if let Some(slot) = self
          .slots
          .iter()
          .find(|slot| slot.role == VisualRole::Primary && slot.current.is_some())
          .map(|slot| slot.id)
        {
          if let Some(visual) = self.slot_mut(slot).and_then(|known| known.current.take()) {
            self.start_exit(visual, event.at, TransitionKind::Replacement);
          }
          reusable_slot = Some(slot);
          replacement_enter = true;
        }
      }
    }
    reusable_slot = reusable_slot.or_else(|| {
      replacement_candidates.iter().find_map(|(slot, _)| {
        self
          .slot(*slot)
          .is_some_and(|known| known.role.same_slot_kind(role))
          .then_some(*slot)
      })
    });
    if reusable_slot.is_some_and(|slot| {
      replacement_candidates
        .iter()
        .any(|(candidate, _)| *candidate == slot)
    }) {
      replacement_enter = true;
    }
    if reusable_slot.is_none() {
      let inactive = self
        .slots
        .iter()
        .filter(|slot| slot.current.is_none() && slot.role.same_slot_kind(role))
        .map(|slot| (slot.id, self.latest_visible_in_slot(slot.id, event.at)))
        .min_by_key(|(slot, visible)| (visible.is_none(), *slot));
      if let Some((slot, outgoing)) = inactive {
        reusable_slot = Some(slot);
        replacement_enter = outgoing.is_some();
        if let Some(outgoing) = outgoing {
          self.start_exit(outgoing, event.at, TransitionKind::Replacement);
        }
      }
    }
    if let Some(slot_id) = reusable_slot {
      replacement_candidates.retain(|(candidate, _)| *candidate != slot_id);
    }
    let slot_id = if let Some(slot_id) = reusable_slot {
      if let Some(slot) = self.slot_mut(slot_id) {
        slot.role = role;
      }
      slot_id
    } else {
      let slot_id = self.next_slot_id;
      self.next_slot_id += 1;
      self.slots.push(Slot {
        id: slot_id,
        role,
        current: None,
      });
      slot_id
    };
    let visual = self.timeline.visuals.len();
    self.timeline.visuals.push(VisualKey {
      source_shortcut: event.shortcut,
      key_code: event.key_code,
      modifier_mask: event.modifier_mask,
      role,
      slot_id,
      enter_us: event.at,
      animation_enter_us: event.at,
      replacement_enter,
      layout_exit_us: None,
      layout_anchor_until_us: None,
      exit: None,
      layout: LayoutTrack::default(),
    });
    self
      .slot_mut(slot_id)
      .expect("new slots remain present")
      .current = Some(visual);
    self.held.insert(event.key_code, visual);
    for (_, outgoing) in replacement_candidates {
      let visual = &mut self.timeline.visuals[outgoing];
      visual.layout_exit_us = Some(event.at);
      if let Some((exit_us, _)) = visual.exit {
        visual.exit = Some((exit_us, TransitionKind::Detached));
      }
    }
  }

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
    let finished = self
      .slots
      .iter()
      .filter_map(|slot| {
        let visual = slot.current?;
        (!self.timeline.visuals[visual].visible_at(at)).then_some(slot.id)
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
      .filter(|(_, visual)| visual.slot_id == slot && visual.visible_at(at))
      .max_by_key(|(_, visual)| visual.enter_us)
      .map(|(index, _)| index)
  }
}
