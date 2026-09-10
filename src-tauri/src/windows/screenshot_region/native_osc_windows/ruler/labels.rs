// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

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

/// Port of `measurement_text`. A flat measurement reports only the long side.
/// A readout shows the digits it has; the label resizes when a number gains
/// one, which the surface it lives on follows.
pub(crate) fn measurement_text(global: Rect) -> String {
  let width = (global.size.width.round() as i64).max(0);
  let height = (global.size.height.round() as i64).max(0);
  if global.size.height < 8.0 {
    return format!("{width} px");
  }
  if global.size.width < 8.0 {
    return format!("{height} px");
  }
  format!("{width} × {height} px")
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

/// Port of `probe_dimensions_text`: the loupe's dimensions field, from the one
/// horizontal and one vertical live probe of this surface's display.
pub(crate) fn probe_dimensions_text(probes: &[ProbePacket], display_id: u32) -> Option<String> {
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
  Some(format!("{width} × {height} px"))
}
