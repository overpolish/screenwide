// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(in crate::editor::keyboard_effects) fn attach_tracks(
  visuals: &mut [VisualKey],
  ranges: Option<&[crate::editor::timeline_edit::TimelineRange]>,
) {
  // Layout is a per-badge concern: keys of one badge-continuity group never
  // shift, anchor, or morph in response to another group's keys, so each
  // group's tracks are attached in isolation.
  let clock = ranges.map_or_else(AnimationClock::source, AnimationClock::edited);
  let mut groups = visuals
    .iter()
    .map(|visual| visual.group)
    .collect::<Vec<_>>();
  groups.sort_unstable();
  groups.dedup();
  for group in groups {
    attach_group_tracks(visuals, group, clock);
  }
}

pub(super) fn attach_group_tracks(
  visuals: &mut [VisualKey],
  group: u32,
  clock: AnimationClock<'_>,
) {
  // Coalesce each chord's rolled presses: a key entering within the assembly
  // window of the previous key of its chord shares that roll's layout event.
  let mut chord_enters = HashMap::<usize, Vec<(u64, usize)>>::new();
  for (index, visual) in visuals
    .iter()
    .enumerate()
    .filter(|(_, visual)| visual.group == group)
  {
    chord_enters
      .entry(visual.source_shortcut)
      .or_default()
      .push((visual.enter_us, index));
  }
  let mut layout_enters = HashMap::<usize, u64>::new();
  for enters in chord_enters.values_mut() {
    enters.sort_unstable();
    let mut roll_start = None;
    let mut previous = None;
    for &(enter_us, index) in enters.iter() {
      // The roll is what the viewer perceives, so the window is measured on
      // the output clock: at 2x a press half a source-second later still
      // reads as one gesture.
      let rolled =
        previous.is_some_and(|previous: u64| clock.elapsed_us(previous, enter_us) < ASSEMBLY_US);
      if !rolled {
        roll_start = Some(enter_us);
      }
      layout_enters.insert(index, roll_start.unwrap_or(enter_us));
      previous = Some(enter_us);
    }
  }
  // A rolled key claims its slot from the roll's first press: it is present
  // (at zero entrance progress) from then on, so the keys before it never
  // shift over when it pops in.
  for (&index, &layout_enter) in &layout_enters {
    if layout_enter < visuals[index].enter_us {
      visuals[index].reserve_from_us = Some(layout_enter);
    }
  }
  let mut events = BTreeMap::<u64, Events>::new();
  for (index, visual) in visuals
    .iter()
    .enumerate()
    .filter(|(_, visual)| visual.group == group)
  {
    let layout_enter = layout_enters
      .get(&index)
      .copied()
      .unwrap_or(visual.enter_us);
    events.entry(layout_enter).or_default().enters.push(index);
    if let Some((at, kind)) = visual.exit {
      if kind != TransitionKind::Replacement {
        events.entry(at).or_default().freezes.push(index);
      }
      let remove_at = if kind == TransitionKind::Detached {
        visual
          .layout_exit_us
          .unwrap_or_else(|| at.saturating_add(MOTION_US))
      } else {
        at.saturating_add(MOTION_US)
      };
      events.entry(remove_at).or_default().removes.push(index);
    }
  }
  let mut active = HashMap::<u32, usize>::new();
  for (at, event) in events {
    for &index in &event.freezes {
      visuals[index].layout.freeze(at);
    }
    let mut removed = Vec::new();
    for &index in &event.removes {
      if active.get(&visuals[index].slot_id) == Some(&index) {
        active.remove(&visuals[index].slot_id);
        removed.push(index);
      }
    }
    let mut existing = tracked_indices(visuals, &active, at, clock);
    for &index in &existing {
      visuals[index].layout.cancel_future(at);
    }
    let barrier = existing.iter().fold(at, |available, index| {
      available.max(visuals[*index].layout.available_at(at))
    });
    let mut base = existing.first().map_or_else(
      || ordered_slots(visuals, active.values().copied()),
      |index| visuals[*index].layout.representative_snapshot(barrier),
    );
    let entering = event.enters.to_vec();
    for &index in &entering {
      active.insert(visuals[index].slot_id, index);
    }
    let target = ordered_slots(visuals, active.values().copied());
    if base.is_empty() && removed.is_empty() {
      base.clone_from(&target);
    }
    for &index in &entering {
      let replacement = if visuals[index].replacement_enter {
        replacement_track(visuals, index, at, clock)
      } else {
        None
      };
      visuals[index].layout = replacement.unwrap_or_else(|| LayoutTrack::new(base.clone()));
    }
    existing = tracked_indices(visuals, &active, at, clock);
    for &index in &existing {
      visuals[index].layout.cancel_future(at);
      if !base.contains(&visuals[index].slot_id) {
        visuals[index].animation_enter_us = barrier;
      }
    }
    for &index in &removed {
      if visuals[index]
        .exit
        .is_some_and(|(_, kind)| kind == TransitionKind::Detached)
      {
        visuals[index].exit = Some((barrier, TransitionKind::Detached));
      }
      visuals[index].layout.freeze(barrier);
      if !active.is_empty() {
        visuals[index].layout_anchor_until_us = Some(barrier.saturating_add(MOTION_US));
      }
    }
    for index in existing {
      visuals[index].layout.schedule(barrier, target.clone());
    }
  }
}

pub(super) fn ordered_slots(
  visuals: &[VisualKey],
  indices: impl Iterator<Item = usize>,
) -> Vec<u32> {
  let mut indices = indices.collect::<Vec<_>>();
  indices.sort_by_key(|index| (visuals[*index].role.order(), visuals[*index].slot_id));
  indices
    .into_iter()
    .map(|index| visuals[index].slot_id)
    .collect()
}

pub(super) fn replacement_track(
  visuals: &[VisualKey],
  incoming: usize,
  at: u64,
  clock: AnimationClock<'_>,
) -> Option<LayoutTrack> {
  visuals
    .iter()
    .enumerate()
    .filter(|(index, visual)| {
      *index != incoming
        && visual.slot_id == visuals[incoming].slot_id
        && visual.enter_us < at
        && visual.visible_at(at, clock)
    })
    .max_by_key(|(_, visual)| visual.enter_us)
    .map(|(_, visual)| visual.layout.clone())
}

pub(super) fn tracked_indices(
  visuals: &[VisualKey],
  active: &HashMap<u32, usize>,
  at: u64,
  clock: AnimationClock<'_>,
) -> Vec<usize> {
  let mut tracked = active.values().copied().collect::<Vec<_>>();
  tracked.extend(visuals.iter().enumerate().filter_map(|(index, visual)| {
    visual
      .exit
      .is_some_and(|(exit_us, kind)| {
        kind == TransitionKind::Replacement
          && exit_us <= at
          && visual.visible_at(at, clock)
          && active.contains_key(&visual.slot_id)
      })
      .then_some(index)
  }));
  tracked.sort_unstable();
  tracked.dedup();
  tracked
}
