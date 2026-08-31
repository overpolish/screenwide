// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{
  probe::ProbeAxis,
  render::{MeasurementPacket, ProbePacket},
  snapshot::{RulerMeasurementVisual, RulerProbeVisual},
};
use crate::osc::geometry::{Point, Rect};

#[test]
fn measurement_packet_preserves_geometry_flags_hover_and_label() {
  let packet = MeasurementPacket::from(&RulerMeasurementVisual {
    id: 7,
    bounds: Rect::from_xywh(10.0, 20.0, 30.0, 40.0),
    draft: true,
    animating: false,
    hovered: true,
    hover_alpha: 0.5,
    label_anchor: Some(Point { x: 50.0, y: 60.0 }),
    label_hidden: true,
  });

  assert_eq!(
    (packet.x, packet.y, packet.width, packet.height),
    (10.0, 20.0, 30.0, 40.0)
  );
  assert_eq!(packet.flags, 0b1101);
  assert_eq!(packet.padding[0], 128);
  assert_eq!((packet.label_anchor_x, packet.label_anchor_y), (50.0, 60.0));
}

#[test]
fn probe_packet_uses_the_shared_axis_and_transient_encoding() {
  let packet = ProbePacket::from(&RulerProbeVisual {
    id: 0,
    display_id: 2,
    axis: ProbeAxis::Vertical,
    start: 1.0,
    end: 9.0,
    position: 4.0,
    draft: false,
    hovered: false,
    hover_alpha: 0.0,
    label_anchor: None,
    label_hidden: false,
  });

  assert_eq!(packet.axis, 2);
  assert_eq!(packet.flags, 0b0100);
  assert!(packet.label_anchor_x.is_nan() && packet.label_anchor_y.is_nan());
}
