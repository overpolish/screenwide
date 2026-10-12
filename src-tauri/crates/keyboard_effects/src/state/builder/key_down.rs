// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Builder<'_> {
  pub(super) fn key_down(&mut self, event: PhysicalEvent) {
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
      // The new chord continues the badge on screen only when it will draw
      // in the same place at the same size; otherwise that badge finishes
      // the exit it already has scheduled, completely untouched, and this
      // chord starts a fresh badge with fresh slots.
      let predecessor = self
        .slots
        .iter()
        .filter_map(|slot| slot.current)
        .max_by_key(|visual| self.timeline.visuals[*visual].enter_us);
      // A badge that has begun its exit fade is already leaving, so a new
      // chord never continues it: the fade is finished by this press below
      // and the chord pops in as a fresh badge instead.
      let chains = predecessor.is_some_and(|visual| {
        let known = &self.timeline.visuals[visual];
        known.exit.is_none_or(|(exit_us, _)| event.at < exit_us)
          && self
            .context
            .chains(known.source_shortcut, event.shortcut, event.at)
      });
      if chains {
        for index in 0..self.slots.len() {
          if self.slots[index].group != self.current_group {
            continue;
          }
          if let Some(visual) = self.slots[index].current.take() {
            self.start_exit(visual, event.at, TransitionKind::Replacement);
            replacement_candidates.push((self.slots[index].id, visual));
          }
        }
      } else {
        self.current_group += 1;
        // Every key is physically released by now, so lingering badges are in
        // their post-release hold. A new visible badge makes that hold a lie
        // (the same key may already be down again), so the fade is pulled
        // forward far enough to be FINISHED by this press - while never
        // starting before the badge's own keys were actually released.
        if !self.context.deleted_at(event.shortcut, event.at) {
          let fade_start = self.context.fade_finished_by(event.at);
          let lingering = self
            .slots
            .iter()
            .filter_map(|slot| slot.current)
            .collect::<Vec<_>>();
          for visual in lingering {
            let visual = &mut self.timeline.visuals[visual];
            if let Some((exit_us, kind)) = visual.exit {
              let release_us = exit_us.saturating_sub(HOLD_US);
              let finished_by_press = fade_start.max(release_us);
              visual.exit = Some((exit_us.min(finished_by_press), kind));
            }
          }
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
          .find(|slot| {
            slot.group == self.current_group
              && slot.role == VisualRole::Primary
              && slot.current.is_some()
          })
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
        .filter(|slot| {
          slot.group == self.current_group
            && slot.current.is_none()
            && slot.role.same_slot_kind(role)
        })
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
        group: self.current_group,
        role,
        current: None,
      });
      slot_id
    };
    let visual = self.timeline.visuals.len();
    self.timeline.visuals.push(VisualKey {
      source_shortcut: event.shortcut,
      group: self.current_group,
      key_code: event.key_code,
      modifier_mask: event.modifier_mask,
      role,
      slot_id,
      enter_us: event.at,
      animation_enter_us: event.at,
      reserve_from_us: None,
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
}
