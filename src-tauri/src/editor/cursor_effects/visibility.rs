// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn events(records: &[CursorRecord]) -> Vec<(u64, bool)> {
  let mut events: Vec<_> = records
    .iter()
    .filter_map(|record| match record {
      CursorRecord::Visibility {
        timestamp_us,
        visible,
        ..
      } => Some((*timestamp_us, *visible)),
      _ => None,
    })
    .collect();
  events.sort_by_key(|event| event.0);
  events
}

/// Explicit transitions prevent interpolation, lean and blur across a warp.
pub(super) fn segment(positions: &mut [Position], events: &[(u64, bool)]) {
  for position in positions {
    position.segment = position
      .segment
      .saturating_add(events.partition_point(|event| event.0 <= position.timestamp_us) as u32);
  }
}

impl CursorCompositor {
  pub(super) fn visibility_opacity(&self, timestamp_us: u64) -> f32 {
    const FADE_US: f32 = 120_000.0;
    let index = self
      .visibility
      .partition_point(|event| event.0 <= timestamp_us);
    let mut opacity = 1.0_f32;
    if let Some(&(at, visible)) = index.checked_sub(1).and_then(|i| self.visibility.get(i)) {
      if !visible {
        return 0.0;
      }
      opacity = (timestamp_us.saturating_sub(at) as f32 / FADE_US).clamp(0.0, 1.0);
    }
    if let Some(&(at, false)) = self.visibility.get(index) {
      opacity = opacity.min((at.saturating_sub(timestamp_us) as f32 / FADE_US).clamp(0.0, 1.0));
    }
    opacity * opacity * (3.0 - 2.0 * opacity)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn compositor() -> CursorCompositor {
    CursorCompositor::from_records(&[
      CursorRecord::Header {
        coordinate_space: "global-logical-points".into(),
        platform: "test".into(),
        source: CursorSource {
          height: 1000.0,
          width: 1000.0,
          x: 0.0,
          y: 0.0,
          kind: crate::recording::cursor::CursorSourceKind::Screen,
          platform_id: "test".into(),
          video_height: 1000,
          video_width: 1000,
        },
        timebase: "recording-microseconds".into(),
        version: cursor::FORMAT_VERSION,
      },
      CursorRecord::Appearance {
        height: 24.0,
        width: 16.0,
        hotspot_x: 0.0,
        hotspot_y: 0.0,
        style: CursorStyle::Arrow,
        timestamp_us: 0,
      },
      CursorRecord::Position {
        timestamp_us: 0,
        x: 10.0,
        y: 20.0,
      },
      CursorRecord::Visibility {
        timestamp_us: 200_000,
        visible: false,
        x: 10.0,
        y: 20.0,
      },
      // A short gesture still needs an explicit segment: no gap heuristic can detect it.
      CursorRecord::Visibility {
        timestamp_us: 250_000,
        visible: true,
        x: 900.0,
        y: 800.0,
      },
      CursorRecord::Position {
        timestamp_us: 260_000,
        x: 900.0,
        y: 800.0,
      },
    ])
    .unwrap()
  }

  #[test]
  fn fades_out_hides_and_fades_in_at_the_landing() {
    let compositor = compositor();
    assert_eq!(compositor.visibility_opacity(80_000), 1.0);
    assert_eq!(compositor.visibility_opacity(140_000), 0.5);
    assert_eq!(compositor.visibility_opacity(225_000), 0.0);
    assert_eq!(compositor.visibility_opacity(250_000), 0.0);
    assert_eq!(compositor.visibility_opacity(310_000), 0.5);
    assert_eq!(compositor.visibility_opacity(370_000), 1.0);
    assert!(compositor
      .evaluate(225_000, CursorEffectSettings::default())
      .is_none());
  }

  #[test]
  fn teleport_never_interpolates_or_blurs_between_endpoints() {
    let compositor = compositor();
    for smooth_movement in [false, true] {
      let settings = CursorEffectSettings {
        smooth_movement,
        ..Default::default()
      };
      let before = compositor.evaluate(190_000, settings).unwrap();
      let after = compositor.evaluate(251_000, settings).unwrap();
      assert!((before.x - 10.0).abs() < 0.001);
      assert!((after.x - 900.0).abs() < 0.001);
      assert_ne!(before.segment, after.segment);
      assert_eq!(after.rotation_degrees, 0.0);
      let gpu = compositor
        .gpu_cursor(
          (310_000 + SCREEN_REACTION_US).div_ceil(1000),
          (1000, 1000),
          settings,
        )
        .unwrap();
      assert!((gpu.opacity - 0.5).abs() < 0.01);
      assert!(gpu.blur_delta_x.abs() < 0.001);
      assert!(gpu.blur_delta_y.abs() < 0.001);
    }
  }

  #[test]
  fn old_recordings_remain_fully_visible() {
    let mut compositor = compositor();
    compositor.visibility.clear();
    assert_eq!(compositor.visibility_opacity(225_000), 1.0);
  }
}
