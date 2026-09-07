// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Monitor identities, directional selection and preview geometry are OS-neutral.
use super::GlideFrame;

#[derive(Clone, Debug)]
pub(crate) struct Monitor {
  pub id: String,
  pub bounds: GlideFrame,
  pub layout: GlideFrame,
  pub work: GlideFrame,
}
fn center(frame: GlideFrame) -> (f64, f64) {
  (frame.x + frame.width / 2.0, frame.y + frame.height / 2.0)
}

/// Eight directional sectors prevent a horizontal push from choosing a screen
/// directly above. Among screens in the requested sector choose the nearest.
pub(crate) fn neighbour(
  monitors: &[Monitor],
  current: usize,
  direction: (i8, i8),
) -> Option<usize> {
  let (x, y) = center(monitors.get(current)?.layout);
  monitors
    .iter()
    .enumerate()
    .filter_map(|(index, monitor)| {
      if index == current {
        return None;
      }
      let (cx, cy) = center(monitor.layout);
      let (dx, dy) = (cx - x, cy - y);
      if dx.abs() + dy.abs() < 1.0 {
        return None;
      } // mirrored displays
      let sector = if dx.abs() > dy.abs() * 2.0 {
        (dx.signum() as i8, 0)
      } else if dy.abs() > dx.abs() * 2.0 {
        (0, dy.signum() as i8)
      } else {
        (dx.signum() as i8, dy.signum() as i8)
      };
      (sector == direction).then_some((index, dx.hypot(dy)))
    })
    .min_by(|a, b| a.1.total_cmp(&b.1))
    .map(|(index, _)| index)
}

/// Uniform 48×32 previews retain relative centre positions, scaled just enough
/// to leave an 8px gap between every pair. Display sizes never size the tiles.
pub(crate) fn preview_offsets(monitors: &[Monitor]) -> Vec<(f64, f64)> {
  let centers: Vec<_> = monitors
    .iter()
    .map(|monitor| center(monitor.layout))
    .collect();
  let mut scale: f64 = 0.0;
  for (i, (x, y)) in centers.iter().enumerate() {
    for (cx, cy) in centers.iter().skip(i + 1) {
      let dx = (cx - x).abs();
      let dy = (cy - y).abs();
      if dx + dy >= 1.0 {
        scale = scale.max((56.0 / dx).min(40.0 / dy));
      }
    }
  }
  let min_x = centers
    .iter()
    .map(|(x, _)| *x)
    .reduce(f64::min)
    .unwrap_or(0.0);
  let min_y = centers
    .iter()
    .map(|(_, y)| *y)
    .reduce(f64::min)
    .unwrap_or(0.0);
  centers
    .into_iter()
    .map(|(x, y)| ((x - min_x) * scale, (y - min_y) * scale))
    .collect()
}

/// An unsnapped window keeps its size and relative placement, clamped to fit.
pub(crate) fn carry_frame(
  frame: GlideFrame,
  source: GlideFrame,
  destination: GlideFrame,
) -> GlideFrame {
  let fraction = |value: f64, origin: f64, extent: f64, size: f64| {
    ((value - origin) / (extent - size).max(1.0)).clamp(0.0, 1.0)
  };
  let travel_x = (destination.width - frame.width).max(0.0);
  let travel_y = (destination.height - frame.height).max(0.0);
  GlideFrame {
    x: destination.x + fraction(frame.x, source.x, source.width, frame.width) * travel_x,
    y: destination.y + fraction(frame.y, source.y, source.height, frame.height) * travel_y,
    width: frame.width,
    height: frame.height,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  fn screen(x: f64, y: f64) -> Monitor {
    let bounds = GlideFrame {
      x,
      y,
      width: 1000.0,
      height: 800.0,
    };
    Monitor {
      id: format!("{x}:{y}"),
      bounds,
      layout: bounds,
      work: bounds,
    }
  }
  #[test]
  fn separates_cardinal_diagonal_and_missing_neighbours() {
    let monitors = vec![
      screen(0.0, 0.0),
      screen(-1000.0, 0.0),
      screen(-1000.0, -800.0),
      screen(0.0, -800.0),
    ];
    assert_eq!(neighbour(&monitors, 0, (-1, 0)), Some(1));
    assert_eq!(neighbour(&monitors, 0, (-1, -1)), Some(2));
    assert_eq!(neighbour(&monitors, 0, (0, -1)), Some(3));
    assert_eq!(neighbour(&monitors, 0, (1, 0)), None);
    let positions = preview_offsets(&monitors);
    assert!(positions[2].0 < positions[0].0 && positions[2].1 < positions[0].1);
    for (i, (x, y)) in positions.iter().enumerate() {
      for (cx, cy) in positions.iter().skip(i + 1) {
        assert!((cx - x).abs() >= 56.0 - 0.001 || (cy - y).abs() >= 40.0 - 0.001);
      }
    }
  }
  #[test]
  fn navigation_uses_selector_layout_instead_of_per_display_logical_coordinates() {
    let mut above = screen(1000.0, 0.0);
    above.layout = screen(0.0, -800.0).layout;
    let displays = vec![screen(0.0, 0.0), above];
    assert_eq!(neighbour(&displays, 0, (0, -1)), Some(1));
    assert_eq!(neighbour(&displays, 0, (1, 0)), None);
    let offsets = preview_offsets(&displays);
    assert_eq!(offsets[0].0, offsets[1].0);
    assert!(offsets[1].1 < offsets[0].1);
  }

  #[test]
  fn singleton_display_stays_as_one_source_preview_without_destination() {
    let displays = vec![screen(0.0, 0.0)];
    assert_eq!(neighbour(&displays, 0, (1, 0)), None);
    assert_eq!(preview_offsets(&displays), vec![(0.0, 0.0)]);
  }
  #[test]
  fn preserves_large_windows_and_clamps_their_destination_travel() {
    let result = carry_frame(
      GlideFrame {
        x: -500.0,
        y: 0.0,
        width: 500.0,
        height: 400.0,
      },
      screen(0.0, 0.0).work,
      GlideFrame {
        x: 0.0,
        y: 0.0,
        width: 500.0,
        height: 400.0,
      },
    );
    assert_eq!(
      result,
      GlideFrame {
        x: 0.0,
        y: 0.0,
        width: 500.0,
        height: 400.0
      }
    );
  }

  #[test]
  fn carries_relative_placement_between_displays() {
    let result = carry_frame(
      GlideFrame {
        x: 250.0,
        y: 100.0,
        width: 500.0,
        height: 400.0,
      },
      GlideFrame {
        x: 0.0,
        y: 0.0,
        width: 1000.0,
        height: 800.0,
      },
      GlideFrame {
        x: 1000.0,
        y: 200.0,
        width: 1200.0,
        height: 1000.0,
      },
    );
    assert_eq!(result.x, 1350.0);
    assert_eq!(result.y, 350.0);
    assert_eq!(result.width, 500.0);
    assert_eq!(result.height, 400.0);
  }
}
