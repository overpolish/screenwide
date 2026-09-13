// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl KeyboardCompositor {
  pub(in crate::editor::keyboard_effects) fn evaluate_with_ranges(
    &self,
    position_ms: u64,
    settings: KeyboardEffectSettings,
    ranges: Option<&[crate::editor::timeline_edit::TimelineRange]>,
  ) -> Option<KeyboardOverlay> {
    let settings = settings.normalized();
    if !settings.bake {
      return None;
    }
    self.set_animation_timeline(ranges);
    let now = position_ms.saturating_mul(1_000);
    let baked = self
      .baked
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let clock = ranges.map_or_else(AnimationClock::source, AnimationClock::edited);
    let animation = match settings.animation {
      KeyboardAnimation::Pop => KeyboardOverlay::ANIMATION_POP,
      KeyboardAnimation::Fade => KeyboardOverlay::ANIMATION_FADE,
      KeyboardAnimation::None => KeyboardOverlay::ANIMATION_NONE,
    };
    let size_scale = (settings.size_percent / 100.0) as f32;
    let appearance = match settings.appearance {
      KeyboardAppearance::Dark => KeyboardOverlay::APPEARANCE_DARK,
      KeyboardAppearance::Light => KeyboardOverlay::APPEARANCE_LIGHT,
    };
    let mut overlay = KeyboardOverlay {
      key_count: 0,
      animation,
      appearance,
      scale: size_scale,
      progress: 1.0,
      maximum_width: baked.maximum_width as f32,
      requested_scale: size_scale,
      center_x: settings
        .position_x_percent
        .map_or(-1.0, |position| (position / 100.0) as f32),
      center_y: settings
        .position_y_percent
        .map_or(-1.0, |position| (position / 100.0) as f32),
      ..Default::default()
    };
    let deleted_ids = self
      .deleted_shortcut_ids
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let deleted_ranges = self
      .deleted_shortcut_ranges
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    let visible = baked
      .timeline
      .visuals
      .iter()
      .filter(|key| {
        let shortcut_id = key.source_shortcut as u64;
        !deleted_ids.contains(&shortcut_id)
          && !deleted_ranges.iter().any(|range| {
            range.shortcut_id == shortcut_id
              && position_ms >= range.start_ms
              && position_ms < range.end_ms
          })
      })
      .filter(|key| key.visible_at(now, clock))
      .collect::<Vec<_>>();
    // Manual placements are per shortcut, so during a transition two groups
    // can occupy different spots. The overlay centre follows the newest
    // visible group as before; every key additionally carries its own
    // group's centre so a differently placed badge finishes its animation
    // where the user put it instead of teleporting to the successor.
    let positions = self
      .shortcut_positions
      .read()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .clone();
    let placement_at = |shortcut: usize| {
      positions
        .iter()
        .find(|position| {
          position.shortcut_id == shortcut as u64
            && position_ms >= position.start_ms
            && position_ms < position.end_ms
        })
        .map(|position| (position.center_x, position.center_y, position.size_percent))
    };
    let base_center = (overlay.center_x, overlay.center_y);
    self.apply_shortcut_position(&mut overlay, &visible, position_ms);
    let overlay_placement = visible
      .iter()
      .max_by_key(|key| key.enter_us)
      .and_then(|key| placement_at(key.source_shortcut));
    // Wire slot indices and layout masks are bit positions, so they must be
    // compact. Fresh badge groups keep allocating new slot ids for the whole
    // recording; only the slots this frame actually references are wired,
    // in the global ordering so rows stay stable.
    let mut frame_slots: Vec<u32> = Vec::new();
    for key in &visible {
      let sample = key.layout.sample(now, clock);
      for slot in std::iter::once(&key.slot_id)
        .chain(sample.from)
        .chain(sample.to)
      {
        if !frame_slots.contains(slot) {
          frame_slots.push(*slot);
        }
      }
    }
    frame_slots.sort_by_key(|slot| {
      baked
        .slots
        .iter()
        .position(|known| known == slot)
        .unwrap_or(usize::MAX)
    });
    let slots = &frame_slots;
    let mut visible = visible;
    visible.sort_by_key(|key| {
      (
        slots
          .iter()
          .position(|slot| *slot == key.slot_id)
          .unwrap_or(usize::MAX),
        key.enter_us,
      )
    });
    for key in visible {
      let exit = key.exit.filter(|(exit_us, _)| now >= *exit_us);
      let exit_progress = exit.map(|(exit_us, _)| {
        ((clock.elapsed_us(exit_us, now) as f64 / MICROS_PER_SECOND) / EXIT_SECONDS).clamp(0.0, 1.0)
          as f32
      });
      let layout_tail_visible = key.layout_anchor_until_us.is_some_and(|until_us| {
        let anchor_us = until_us.saturating_sub(layout::MOTION_US);
        clock.elapsed_us(anchor_us, now) < layout::MOTION_US
      });
      let artwork_hidden = exit_progress
        .is_some_and(|progress| settings.animation == KeyboardAnimation::None || progress >= 1.0);
      if artwork_hidden && !layout_tail_visible {
        continue;
      }
      if overlay.key_count as usize >= MAX_KEYS {
        break;
      }
      // Inherit the overlay centre when this key's group placement matches
      // the group the overlay follows; otherwise pin the key to its own
      // group's spot so it animates there.
      let placement = placement_at(key.source_shortcut);
      let (center_x, center_y, group_scale) = if placement == overlay_placement {
        (
          KEY_CENTER_INHERIT,
          KEY_CENTER_INHERIT,
          overlay.requested_scale,
        )
      } else if let Some((x, y, size)) = placement {
        (
          x as f32,
          y as f32,
          size.map_or(size_scale, |size| (size / 100.0) as f32),
        )
      } else {
        (
          if base_center.0 >= 0.0 {
            base_center.0
          } else {
            KEY_CENTER_DEFAULT
          },
          if base_center.1 >= 0.0 {
            base_center.1
          } else {
            KEY_CENTER_DEFAULT
          },
          size_scale,
        )
      };
      let scale_ratio = group_scale / overlay.requested_scale.max(0.001);
      let entrance_seconds = if key.replacement_enter {
        EXIT_SECONDS
      } else {
        ENTRANCE_SECONDS
      };
      let raw_entrance_progress = ((clock.elapsed_us(key.animation_enter_us, now) as f64
        / MICROS_PER_SECOND)
        / entrance_seconds)
        .clamp(0.0, 1.0) as f32;
      let entrance_progress = if key.replacement_enter {
        replacement_enter_progress(raw_entrance_progress)
      } else {
        raw_entrance_progress
      };
      // The exit composes with a still-running entrance instead of replacing
      // it: the exit curve starts at 1.0, so a key whose compressed lifetime
      // never finished entering hands over continuously at whatever point it
      // reached, rather than popping to full size to begin its exit.
      let key_progress = exit_progress.map_or(entrance_progress, |progress| {
        let exiting = if exit.is_some_and(|(_, kind)| kind == TransitionKind::Replacement) {
          1.0 - replacement_exit_progress(progress)
        } else {
          1.0 - progress
        };
        entrance_progress.min(exiting)
      });
      let detached_amount = exit
        .filter(|(_, kind)| *kind == TransitionKind::Detached)
        .and(exit_progress)
        .map(|progress| entrance_progress.min(1.0 - pop_spring(progress).clamp(0.0, 1.0)));
      // A slot reserved by a roll renders nothing until its own entrance,
      // in every animation mode: it only holds the row geometry stable.
      let pending_entrance = now < key.animation_enter_us;
      let key_alpha = if pending_entrance || artwork_hidden {
        0.0
      } else if let Some(amount) = detached_amount {
        amount
      } else if animation == KeyboardOverlay::ANIMATION_FADE
        || key.replacement_enter
        || exit.is_some_and(|(_, kind)| kind == TransitionKind::Replacement)
      {
        ease_out(key_progress)
      } else {
        1.0
      };
      let key_scale = (if pending_entrance || artwork_hidden {
        0.0
      } else if animation == KeyboardOverlay::ANIMATION_POP {
        detached_amount.unwrap_or_else(|| pop_spring(key_progress))
      } else {
        1.0
      }) * overlay.requested_scale;
      let index = overlay.key_count as usize;
      let layout = key.layout.sample(now, clock);
      overlay.keys[index] = KeyboardKey {
        key_code: key.key_code,
        modifier_mask: key.modifier_mask,
        visible: if exit_progress.is_some() { 2 } else { 1 },
        progress: key_progress,
        alpha: key_alpha,
        scale: key_scale,
        layout_progress: layout.progress,
        slot: slots
          .iter()
          .position(|slot| *slot == key.slot_id)
          .unwrap_or_default() as u32,
        layout_from_mask: layout::mask(layout.from, slots),
        layout_to_mask: layout::mask(layout.to, slots),
        center_x,
        center_y,
        scale_ratio,
      };
      overlay.key_count += 1;
    }
    if overlay.key_count == 0 {
      return None;
    }
    Some(overlay)
  }
}
