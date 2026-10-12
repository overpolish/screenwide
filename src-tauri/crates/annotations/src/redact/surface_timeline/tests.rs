// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{surface_at, timeline, SAMPLE_MS};

const BLUE: [u8; 3] = [20, 60, 200];
const WHITE: [u8; 3] = [250, 250, 250];
const RED: [u8; 3] = [220, 30, 30];

/// A clip's samples, one a [`SAMPLE_MS`], from `colours`.
fn sampled(colours: &[[u8; 3]]) -> Vec<(u64, Option<[u8; 3]>)> {
  colours
    .iter()
    .enumerate()
    .map(|(index, colour)| (index as u64 * SAMPLE_MS, Some(*colour)))
    .collect()
}

fn surface(entries: &[[f32; 2]], elapsed_ms: f32) -> [u8; 3] {
  surface_at(entries, elapsed_ms)
    .unwrap()
    .map(|channel| (channel * 255.0).round() as u8)
}

#[test]
fn a_background_that_changes_is_followed_from_the_moment_it_changed() {
  let mut colours = vec![BLUE; 10];
  colours.extend([WHITE; 10]);
  let entries = timeline(&sampled(&colours));
  assert_eq!(surface(&entries, 500.0), BLUE);
  // The white is known ahead, so the fade starts where it appeared.
  let halfway = surface(&entries, 1_075.0);
  assert!(halfway[0] > BLUE[0] && halfway[0] < WHITE[0], "{halfway:?}");
  assert_eq!(surface(&entries, 1_200.0), WHITE);
}

#[test]
fn something_busy_beside_the_box_is_not_followed() {
  let mut colours = vec![BLUE; 5];
  colours.extend([RED, BLUE, WHITE, RED, BLUE, WHITE, BLUE, RED]);
  colours.extend([BLUE; 5]);
  let entries = timeline(&sampled(&colours));
  for elapsed in (0..colours.len() as u64 * SAMPLE_MS).step_by(50) {
    assert_eq!(surface(&entries, elapsed as f32), BLUE, "at {elapsed} ms");
  }
}

#[test]
fn a_change_too_brief_to_hold_is_not_followed() {
  let mut colours = vec![BLUE; 5];
  colours.extend([WHITE; 3]);
  colours.extend([BLUE; 5]);
  let entries = timeline(&sampled(&colours));
  assert_eq!(entries.len(), 1);
}

#[test]
fn compression_noise_is_one_surface() {
  let colours: Vec<_> = (0..20)
    .map(|index| BLUE.map(|channel| channel.saturating_add((index % 3) * 4)))
    .collect();
  assert_eq!(timeline(&sampled(&colours)).len(), 1);
}

#[test]
fn a_change_near_the_clips_end_is_followed() {
  let mut colours = vec![BLUE; 10];
  colours.extend([WHITE; 3]);
  let entries = timeline(&sampled(&colours));
  assert_eq!(surface(&entries, 1_250.0), WHITE);
}
