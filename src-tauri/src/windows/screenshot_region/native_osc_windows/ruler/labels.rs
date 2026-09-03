// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Port of `decimal_digit_count` (`:461-469`): the width every number in a
/// label is padded to, so a readout never resizes as the pointer moves.
pub(crate) fn decimal_digit_count(value: f64) -> usize {
  let mut magnitude = value.abs().ceil().max(0.0) as u64;
  let mut digits = 1;
  while magnitude >= 10 {
    magnitude /= 10;
    digits += 1;
  }
  digits
}

/// Six characters cover `" × "` and `" px"` (`reserved_dimensions_length`).
pub(crate) fn reserved_dimensions_length(desktop: Size) -> usize {
  decimal_digit_count(desktop.width) + decimal_digit_count(desktop.height) + 6
}

/// The `%*ld` of the macOS format strings: right-aligned, space padded.
pub(super) fn padded(value: i64, digits: usize) -> String {
  format!("{value:>digits$}")
}

pub(crate) fn hex_text(color: u32) -> String {
  format!(
    "#{:02X}{:02X}{:02X}",
    (color >> 24) & 0xFF,
    (color >> 16) & 0xFF,
    (color >> 8) & 0xFF
  )
}

pub(crate) fn tolerance_text(mode: u8) -> &'static str {
  match mode {
    1 => "Clear edges",
    3 => "Subtle edges",
    _ => "Balanced",
  }
}

/// Port of `measurement_text` (`:1262-1285`). A flat measurement reports only
/// the long side; `reserve` pads to the desktop's digit count.
pub(crate) fn measurement_text(
  global: Rect,
  reserve: bool,
  width_digits: usize,
  height_digits: usize,
) -> String {
  let width = (global.size.width.round() as i64).max(0);
  let height = (global.size.height.round() as i64).max(0);
  if !reserve {
    if global.size.height < 8.0 {
      return format!("{width} px");
    }
    if global.size.width < 8.0 {
      return format!("{height} px");
    }
    return format!("{width} × {height} px");
  }
  if global.size.height < 8.0 {
    return format!("{} px", padded(width, width_digits));
  }
  if global.size.width < 8.0 {
    return format!("{} px", padded(height, height_digits));
  }
  format!(
    "{} × {} px",
    padded(width, width_digits),
    padded(height, height_digits)
  )
}

pub(crate) fn stamped_probe_text(probe: ProbePacket) -> String {
  let distance = ((probe.end - probe.start).abs().round() as i64).max(0);
  format!("{distance} px")
}

pub(crate) fn radius_text(radius: RadiusPacket) -> String {
  let value = (radius.radius.round() as i64).max(0);
  if radius.flags & 1 != 0 {
    format!("≈ {value} px")
  } else {
    format!("{value} px")
  }
}

/// Port of `probe_dimensions_text` (`:434-459`): the loupe's second row, from
/// the one horizontal and one vertical live probe of this surface's display.
pub(crate) fn probe_dimensions_text(
  probes: &[ProbePacket],
  display_id: u32,
  desktop: Size,
) -> Option<String> {
  let mut horizontal = None;
  let mut vertical = None;
  for probe in probes {
    if probe.flags & 4 == 0 || probe.display_id != display_id {
      continue;
    }
    if probe.axis == 1 {
      horizontal = Some(*probe);
    } else if probe.axis == 2 {
      vertical = Some(*probe);
    }
  }
  let horizontal = horizontal?;
  let vertical = vertical?;
  let width = (((horizontal.end - horizontal.start).abs()).round() as i64).max(0);
  let height = (((vertical.end - vertical.start).abs()).round() as i64).max(0);
  Some(format!(
    "{} × {} px",
    padded(width, decimal_digit_count(desktop.width)),
    padded(height, decimal_digit_count(desktop.height))
  ))
}
