// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::super::*;
use super::*;

pub(in crate::ruler::snapshot) fn update_range(session: &mut Session, pointer: RulerPointer) {
  let Some(mut range) = session.range else {
    return;
  };
  let Some(end_probe) = automatic_probes(session, pointer)
    .and_then(|probes| probes.into_iter().find(|probe| probe.axis == range.axis))
  else {
    return;
  };
  let tracking_world = match range.axis {
    ProbeAxis::Horizontal => Point {
      x: range.start_pointer.world.x,
      y: pointer.world.y,
    },
    ProbeAxis::Vertical => Point {
      x: pointer.world.x,
      y: range.start_pointer.world.y,
    },
  };
  let tracking_pointer = session
    .displays
    .iter()
    .find(|snapshot| {
      let display = snapshot.display;
      tracking_world.x >= display.origin.x
        && tracking_world.y >= display.origin.y
        && tracking_world.x < display.origin.x + display.size.width
        && tracking_world.y < display.origin.y + display.size.height
    })
    .map(|snapshot| RulerPointer {
      world: tracking_world,
      screen: pointer.screen,
      display_id: snapshot.display.id,
      zoom: snapshot.viewport.zoom,
    });
  let start_probe = tracking_pointer
    .and_then(|tracking| automatic_probes(session, tracking))
    .and_then(|probes| probes.into_iter().find(|probe| probe.axis == range.axis))
    .unwrap_or(range.start_probe);
  let forward = match range.axis {
    ProbeAxis::Horizontal => pointer.world.x >= range.start_pointer.world.x,
    ProbeAxis::Vertical => pointer.world.y >= range.start_pointer.world.y,
  };
  range.draft = RulerProbeVisual {
    id: 0,
    display_id: 0,
    axis: range.axis,
    start: if forward {
      start_probe.start
    } else {
      end_probe.start
    },
    end: if forward {
      end_probe.end
    } else {
      start_probe.end
    },
    position: match range.axis {
      ProbeAxis::Horizontal => pointer.world.y,
      ProbeAxis::Vertical => pointer.world.x,
    },
    draft: true,
    hovered: false,
    hover_alpha: 0.0,
    label_anchor: None,
    label_hidden: false,
  };
  session.range = Some(range);
}

pub(in crate::ruler::snapshot) fn settle_worthwhile(from: Rect, to: Rect) -> bool {
  (from.origin.x - to.origin.x).abs() >= 1.0
    || (from.origin.y - to.origin.y).abs() >= 1.0
    || (from.right() - to.right()).abs() >= 1.0
    || (from.bottom() - to.bottom()).abs() >= 1.0
}

pub(in crate::ruler::snapshot) fn sample(
  displays: &[DisplaySnapshot],
  point: Point,
) -> Option<[u8; 4]> {
  let snapshot = displays.iter().find(|snapshot| {
    let display = snapshot.display;
    point.x >= display.origin.x
      && point.y >= display.origin.y
      && point.x < display.origin.x + display.size.width
      && point.y < display.origin.y + display.size.height
  })?;
  let display = snapshot.display;
  let x = (((point.x - display.origin.x) / display.size.width) * f64::from(snapshot.image.width))
    .floor()
    .clamp(0.0, f64::from(snapshot.image.width.saturating_sub(1))) as usize;
  let y = (((point.y - display.origin.y) / display.size.height) * f64::from(snapshot.image.height))
    .floor()
    .clamp(0.0, f64::from(snapshot.image.height.saturating_sub(1))) as usize;
  let offset = (y * snapshot.image.width as usize + x) * 4;
  snapshot
    .image
    .rgba
    .get(offset..offset + 4)
    .and_then(|pixel| pixel.try_into().ok())
}
