// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// Keep desktop origins physical: dividing each monitor's origin by its own
/// scale makes different-DPI displays disagree about where their edges meet.
pub(super) fn clamped_axis(
  origin: i32,
  logical_offset: f64,
  scale: f64,
  physical_size: u32,
  area_origin: i32,
  area_size: u32,
) -> i32 {
  let start = f64::from(area_origin);
  let end = start + f64::from(area_size.saturating_sub(physical_size));
  (f64::from(origin) + logical_offset * scale)
    .clamp(start, end)
    .round() as i32
}

#[cfg(test)]
mod tests {
  use super::clamped_axis;

  #[test]
  fn offsets_scale_without_scaling_the_desktop_origin() {
    assert_eq!(clamped_axis(1920, 100.0, 1.5, 400, 1920, 2560), 2070);
    assert_eq!(clamped_axis(-1920, 100.0, 2.0, 400, -1920, 1920), -1720);
  }

  #[test]
  fn panel_stays_inside_the_work_area() {
    assert_eq!(clamped_axis(2400, 300.0, 2.0, 600, 1920, 1280), 2600);
    assert_eq!(clamped_axis(0, -30.0, 1.0, 400, 40, 1280), 40);
    assert_eq!(clamped_axis(0, 30.0, 1.0, 1500, 40, 1280), 40);
  }
}
