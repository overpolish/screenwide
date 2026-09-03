// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn measurement(id: u64, x: f64, y: f64, width: f64, height: f64) -> MeasurementPacket {
  MeasurementPacket {
    id,
    x,
    y,
    width,
    height,
    flags: 0,
    padding: [0; 7],
    label_anchor_x: f64::NAN,
    label_anchor_y: f64::NAN,
  }
}

fn probe(id: u64, axis: u8, start: f64, end: f64, position: f64) -> ProbePacket {
  ProbePacket {
    id,
    display_id: 1,
    axis,
    flags: 0,
    padding: [0; 2],
    start,
    end,
    position,
    label_anchor_x: f64::NAN,
    label_anchor_y: f64::NAN,
  }
}

#[test]
fn unanchored_labels_compare_by_bytes_so_a_nan_anchor_is_not_a_change() {
  let left = vec![measurement(1, 0.0, 0.0, 10.0, 10.0)];
  let right = left.clone();
  // `PartialEq` would call these different because an absent anchor is NaN.
  assert!(left[0] != right[0]);
  assert!(same(&left, &right));

  let mut moved = right.clone();
  moved[0].x = 1.0;
  assert!(!same(&left, &moved));
}

#[test]
fn the_labelled_projection_masks_everything_but_text_and_placement() {
  let mut moved = vec![measurement(1, 0.0, 0.0, 10.0, 10.0)];
  // Bit 2 is "hovered" and padding[0] is the hover opacity: neither changes
  // the label, so neither may re-rasterise it.
  moved[0].flags = 0b100;
  moved[0].padding[0] = 200;
  let plain = vec![measurement(1, 0.0, 0.0, 10.0, 10.0)];
  assert!(same(
    &labelled_measurements(&plain),
    &labelled_measurements(&moved)
  ));
  assert!(!same(&plain, &moved));

  // Bit 0 is "reserve width", which does change the text.
  let mut reserved = plain.clone();
  reserved[0].flags = 1;
  assert!(!same(
    &labelled_measurements(&plain),
    &labelled_measurements(&reserved)
  ));
}

#[test]
fn live_and_anonymous_probes_carry_no_label_at_all() {
  let mut live = probe(0, 1, 0.0, 10.0, 5.0);
  live.flags = 4;
  let anonymous = probe(0, 1, 0.0, 10.0, 5.0);
  let mut draft = probe(0, 1, 0.0, 10.0, 5.0);
  draft.flags = 1;
  let stamped = probe(7, 1, 0.0, 10.0, 5.0);

  let projected = labelled_probes(&[live, anonymous, draft, stamped]);
  assert_eq!(projected.len(), 2);
  assert_eq!(projected[0].flags, 1);
  assert_eq!(projected[1].id, 7);
}

#[test]
fn the_hover_key_packs_the_artifact_id_with_its_kind_tag() {
  let mut data = RulerData::default();
  let mut hovered = probe(9, 1, 0.0, 10.0, 5.0);
  hovered.padding[0] = 128;
  data.probes = vec![probe(8, 1, 0.0, 1.0, 1.0), hovered];
  let (key, opacity) = hovered_artifact_key(&data);
  assert_eq!(key, (9 << 3) | 2);
  assert!((opacity - 128.0 / 255.0).abs() < 1e-9);

  // A measurement outranks the probe, matching the macOS scan order.
  let mut measurement = measurement(3, 0.0, 0.0, 1.0, 1.0);
  measurement.padding[0] = 255;
  data.measurements = vec![measurement];
  assert_eq!(hovered_artifact_key(&data).0, (3 << 3) | 1);

  assert_eq!(hovered_artifact_key(&RulerData::default()), (0, 0.0));
}

#[test]
fn an_animating_measurement_or_a_flagged_result_asks_for_a_settle_frame() {
  let mut data = RulerData::default();
  assert!(!animation_active(&data, 1));
  assert!(animation_active(&data, 1 | 128));
  let mut animating = measurement(1, 0.0, 0.0, 4.0, 4.0);
  animating.flags = 2;
  data.measurements = vec![animating];
  assert!(animation_active(&data, 1));
}

#[test]
fn projection_subtracts_the_desktop_offset_and_the_viewport_origin() {
  let projected = project_world_rect(
    500.0,
    300.0,
    100.0,
    50.0,
    Point { x: 400.0, y: 0.0 },
    Point { x: 20.0, y: 10.0 },
    2.0,
  );
  assert_eq!(projected, Rect::from_xywh(160.0, 580.0, 200.0, 100.0));

  // At identity the world point lands at its surface-local position.
  let identity = project_world_rect(
    500.0,
    300.0,
    10.0,
    10.0,
    Point { x: 400.0, y: 0.0 },
    Point::default(),
    1.0,
  );
  assert_eq!(identity.origin, Point { x: 100.0, y: 300.0 });
}

#[test]
fn the_snapshot_uv_window_is_the_viewport_expressed_in_texture_space() {
  let mut ruler = Ruler {
    viewport_zoom: 4.0,
    viewport_origin: Point { x: 480.0, y: 270.0 },
    ..Ruler::default()
  };
  let uv = ruler.snapshot_uv(Size {
    width: 1920.0,
    height: 1080.0,
  });
  assert_eq!(uv, Rect::from_xywh(0.25, 0.25, 0.25, 0.25));

  // A zoom below one never shrinks the sampled window.
  ruler.viewport_zoom = 0.5;
  ruler.viewport_origin = Point::default();
  assert_eq!(
    ruler.snapshot_uv(Size {
      width: 100.0,
      height: 100.0
    }),
    Rect::from_xywh(0.0, 0.0, 1.0, 1.0)
  );
}

#[test]
fn label_widths_are_padded_to_the_desktops_digit_count() {
  assert_eq!(decimal_digit_count(0.0), 1);
  assert_eq!(decimal_digit_count(9.0), 1);
  assert_eq!(decimal_digit_count(10.0), 2);
  assert_eq!(decimal_digit_count(3840.0), 4);
  assert_eq!(
    reserved_dimensions_length(Size {
      width: 3840.0,
      height: 2160.0
    }),
    14
  );

  let global = Rect::from_xywh(0.0, 0.0, 42.0, 300.0);
  assert_eq!(measurement_text(global, false, 4, 4), "42 × 300 px");
  assert_eq!(measurement_text(global, true, 4, 4), "  42 ×  300 px");
  // A flat measurement reports only its long side.
  assert_eq!(
    measurement_text(Rect::from_xywh(0.0, 0.0, 42.0, 2.0), false, 4, 4),
    "42 px"
  );
  assert_eq!(
    measurement_text(Rect::from_xywh(0.0, 0.0, 2.0, 42.0), false, 4, 4),
    "42 px"
  );
}

#[test]
fn readout_text_covers_only_characters_the_atlas_has_cells_for() {
  let text = format!(
    "{}{}{}{}",
    hex_text(0x12AB_34FF),
    measurement_text(Rect::from_xywh(0.0, 0.0, 42.0, 300.0), true, 4, 4),
    stamped_probe_text(probe(1, 1, 0.0, 96.0, 0.0)),
    radius_text(RadiusPacket {
      radius: 12.0,
      flags: 1,
      ..Default::default()
    })
  );
  assert!(text
    .chars()
    .all(|glyph| super::super::text::glyph_index(glyph).is_some()));
  assert_eq!(hex_text(0x12AB_34FF), "#12AB34");
  assert_eq!(
    radius_text(RadiusPacket {
      radius: 12.0,
      flags: 1,
      ..Default::default()
    }),
    "≈ 12 px"
  );
}

#[test]
fn the_loupe_flips_to_the_other_side_of_the_pointer_at_the_edges() {
  let view = Size {
    width: 800.0,
    height: 600.0,
  };
  // Room below and to the right: the readout trails the pointer.
  assert_eq!(
    loupe_origin(Point { x: 100.0, y: 100.0 }, 200.0, 40.0, view, 8.0),
    Point { x: 108.0, y: 108.0 }
  );
  // Against the far corner it flips to the near side instead.
  assert_eq!(
    loupe_origin(Point { x: 780.0, y: 580.0 }, 200.0, 40.0, view, 8.0),
    Point { x: 572.0, y: 532.0 }
  );
  // And it never leaves the inset.
  let clamped = loupe_origin(Point { x: 0.0, y: 0.0 }, 200.0, 40.0, view, 8.0);
  assert!(clamped.x >= 8.0 && clamped.y >= 8.0);
}

#[test]
fn labels_go_to_the_surface_whose_viewport_shows_them() {
  let world = vec![
    Rect::from_xywh(0.0, 0.0, 1920.0, 1080.0),
    Rect::from_xywh(1920.0, 0.0, 1920.0, 1080.0),
  ];
  let mut data = RulerData {
    measurements: vec![
      measurement(1, 100.0, 100.0, 200.0, 100.0),
      measurement(2, 2400.0, 100.0, 200.0, 100.0),
    ],
    probes: vec![probe(3, 1, 2000.0, 2200.0, 400.0)],
    ..RulerData::default()
  };

  // Compared by identity, because an absent anchor is NaN and no packet is
  // ever equal to itself under `PartialEq`.
  let tags = |owned: &[LabelItem]| {
    owned
      .iter()
      .map(|item| match item {
        LabelItem::Measurement(value) => (1, value.id),
        LabelItem::Probe(value) => (2, value.id),
        LabelItem::GuideGap(value) => (3, value.id),
        LabelItem::Radius(value) => (4, value.id),
      })
      .collect::<Vec<_>>()
  };
  let owned = assign_labels(&world, &data);
  assert_eq!(tags(&owned[0]), vec![(1, 1)]);
  assert_eq!(tags(&owned[1]), vec![(1, 2), (2, 3)]);

  // A hidden label is owned by nobody.
  data.measurements[0].flags = 8;
  assert!(assign_labels(&world, &data)[0].is_empty());
}

#[test]
fn label_hit_testing_walks_the_four_pools_in_order() {
  let rects = vec![
    LabelRect {
      id: 5,
      kind: 2,
      rect: Rect::from_xywh(0.0, 0.0, 100.0, 24.0),
    },
    LabelRect {
      id: 9,
      kind: 1,
      rect: Rect::from_xywh(0.0, 0.0, 100.0, 24.0),
    },
  ];
  // Overlapping labels resolve to the measurement, as on macOS.
  let hit = label_hit(&rects, Point { x: 10.0, y: 10.0 }).unwrap();
  assert_eq!((hit.id, hit.kind), (9, 1));
  assert_eq!(hit.center, Point { x: 50.0, y: 12.0 });
  assert!(label_hit(&rects, Point { x: 200.0, y: 10.0 }).is_none());
}

#[test]
fn held_range_guide_and_radius_keys_latch_and_report_a_release_phase() {
  assert_eq!(
    key_command(0x31, false, false, false, false),
    Some(KeyCommand {
      phase: 20,
      release: Some(22)
    })
  );
  assert_eq!(
    key_command(0x48, false, false, false, false),
    Some(KeyCommand {
      phase: 27,
      release: Some(28)
    })
  );
  assert_eq!(
    key_command(0x52, false, false, false, false),
    Some(KeyCommand {
      phase: 31,
      release: Some(32)
    })
  );
  // A held key does not re-fire, and neither does an auto-repeat.
  assert_eq!(key_command(0x52, false, false, false, true), None);
  assert_eq!(key_command(0x52, false, false, true, false), None);
}

#[test]
fn the_command_keys_match_the_macos_keycode_table() {
  let phase =
    |vk, command, shift| key_command(vk, command, shift, false, false).map(|command| command.phase);
  assert_eq!(phase(0x58, false, false), Some(13));
  assert_eq!(phase(0x09, false, false), Some(14));
  assert_eq!(phase(0x08, false, false), Some(16));
  assert_eq!(phase(0x2E, false, false), Some(16));
  assert_eq!(phase(0x43, true, false), Some(17));
  assert_eq!(phase(0x5A, true, false), Some(18));
  assert_eq!(phase(0x5A, true, true), Some(19));
  assert_eq!(phase(0x59, true, false), Some(19));
  assert_eq!(phase(0x54, false, false), Some(29));
  assert_eq!(phase(0x4D, false, false), Some(33));
  assert_eq!(phase(0x56, false, false), Some(26));
  // The chorded phases need their modifier, and unknown keys stay unmapped.
  assert_eq!(phase(0x43, false, false), None);
  assert_eq!(phase(0x51, false, false), None);
}

#[test]
fn animations_ease_from_where_they_were_and_stop_when_they_settle() {
  // `now` is read after construction: the default back-dates `started` by
  // the full duration relative to its own clock read, not an earlier one.
  let mut animation = Animation::default();
  let now = Instant::now();
  assert_eq!(animation.amount(now), 0.0);
  assert!(!animation.running(now));

  animation.set(true, false, now);
  assert!(animation.running(now));
  assert_eq!(animation.amount(now), 0.0);
  let settled = now + ANIMATION_DURATION;
  assert!((animation.amount(settled) - 1.0).abs() < 1e-9);
  assert!(!animation.running(settled));

  // Reversing mid-flight starts from the current value, not from 1.
  let midway = now + ANIMATION_DURATION / 2;
  let value = animation.amount(midway);
  animation.set(false, false, midway);
  assert!((animation.amount(midway) - value).abs() < 1e-9);

  // A restart replays from zero even though the target is unchanged.
  animation.set(true, false, midway);
  animation.set(true, true, midway);
  assert_eq!(animation.amount(midway), 0.0);
}

#[test]
fn tolerance_notice_keeps_its_animation() {
  let mut ruler = Ruler::default();
  let now = Instant::now();
  ruler.visible = true;

  ruler.set_tolerance(true, true, now);
  assert_eq!(ruler.tolerance_amount(now), 0.0);
  assert!(ruler.tolerance_amount(now + ANIMATION_DURATION / 2) > 0.0);
  assert!(ruler.is_animating(now));

  let settled = now + ANIMATION_DURATION;
  assert_eq!(ruler.tolerance_amount(settled), 1.0);
  ruler.set_tolerance(false, false, settled);
  assert_eq!(ruler.tolerance_amount(settled), 1.0);
}

#[test]
fn the_hover_halo_widens_over_the_transition_and_holds_its_alpha() {
  let now = Instant::now();
  let mut ruler = Ruler::default();
  assert_eq!(ruler.hover_alpha(), 0.0);

  ruler.set_hover((4 << 3) | 1, 1.0, now);
  assert_eq!(ruler.hover_width(now), HOVER_WIDTH_MIN);
  assert!((ruler.hover_width(now + ANIMATION_DURATION) - HOVER_WIDTH_MAX).abs() < 1e-9);
  assert!((ruler.hover_alpha() - HOVER_ALPHA).abs() < 1e-9);

  // Half opacity halves the halo without touching its width.
  ruler.set_hover((4 << 3) | 1, 0.5, now);
  assert!((ruler.hover_alpha() - HOVER_ALPHA * 0.5).abs() < 1e-9);
}

#[test]
fn the_vertex_budget_matches_the_macos_capacity_formula() {
  let mut ruler = Ruler::default();
  assert_eq!(ruler.vertex_capacity(), 0);

  ruler.visible = true;
  ruler.crosshair = true;
  ruler.data.measurements = vec![measurement(1, 0.0, 0.0, 10.0, 10.0)];
  ruler.data.probes = vec![probe(2, 1, 0.0, 10.0, 5.0)];
  ruler.data.guides = vec![GuidePacket::default()];
  ruler.data.guide_gaps = vec![GuideGapPacket::default()];
  ruler.data.radii = vec![RadiusPacket::default()];
  ruler.data.centerlines = vec![CenterlinePacket::default()];
  ruler.data.inner_objects = vec![InnerObjectPacket::default()];
  assert_eq!(
    ruler.vertex_capacity(),
    12 + 48 + 24 + 12 + 24 + 12 + 12 + 36
  );
}

#[test]
fn the_world_pass_emits_its_primitives_in_the_macos_order() {
  let view = Size {
    width: 400.0,
    height: 200.0,
  };
  let now = Instant::now();
  let mut ruler = Ruler {
    visible: true,
    crosshair: true,
    point: Point { x: 100.0, y: 60.0 },
    data: RulerData {
      probes: vec![probe(1, 1, 20.0, 120.0, 60.0)],
      guides: vec![GuidePacket {
        id: 2,
        display_id: 1,
        axis: 1,
        flags: 0,
        padding: [0; 2],
        position: 200.0,
      }],
      centerlines: vec![CenterlinePacket {
        id: 3,
        x: 10.0,
        y: 10.0,
        width: 40.0,
        height: 40.0,
        flags: 0,
        padding: [0; 7],
      }],
      measurements: vec![measurement(4, 10.0, 10.0, 60.0, 40.0)],
      ..RulerData::default()
    },
    ..Ruler::default()
  };

  let mut out = Vec::new();
  ruler.add_world_vertices(&mut out, view, 1.0, 1, Point::default(), now);
  let kinds = out
    .chunks_exact(6)
    .map(|quad| quad[0].kind)
    .collect::<Vec<_>>();
  assert_eq!(
    kinds,
    vec![
      28, 28, // crosshair
      28, 28, 28, // probe span and its two ticks
      36, // guide
      42, 42, // centerlines
      35, 28, 28, 28, 28 // measurement box
    ]
  );

  // A hidden ruler emits nothing at all.
  ruler.visible = false;
  out.clear();
  ruler.add_world_vertices(&mut out, view, 1.0, 1, Point::default(), now);
  assert!(out.is_empty());
}

#[test]
fn hovered_artifacts_add_their_halo_before_the_line() {
  let view = Size {
    width: 400.0,
    height: 200.0,
  };
  let mut hovered = probe(1, 1, 20.0, 120.0, 60.0);
  hovered.padding[0] = 255;
  let ruler = Ruler {
    visible: true,
    data: RulerData {
      probes: vec![hovered],
      ..RulerData::default()
    },
    ..Ruler::default()
  };

  let mut out = Vec::new();
  ruler.add_world_vertices(&mut out, view, 1.0, 1, Point::default(), Instant::now());
  let kinds = out
    .chunks_exact(6)
    .map(|quad| quad[0].kind)
    .collect::<Vec<_>>();
  assert_eq!(kinds, vec![32, 28, 28, 28]);
}

#[test]
fn live_probes_only_draw_on_their_own_display_and_with_chrome_shown() {
  let view = Size {
    width: 400.0,
    height: 200.0,
  };
  let mut live = probe(0, 1, 20.0, 120.0, 60.0);
  live.flags = 4;
  live.display_id = 2;
  let mut ruler = Ruler {
    visible: true,
    data: RulerData {
      probes: vec![live],
      ..RulerData::default()
    },
    ..Ruler::default()
  };

  let mut out = Vec::new();
  ruler.add_world_vertices(&mut out, view, 1.0, 1, Point::default(), Instant::now());
  assert!(out.is_empty(), "a live probe belongs to its own display");

  out.clear();
  ruler.add_world_vertices(&mut out, view, 1.0, 2, Point::default(), Instant::now());
  assert_eq!(out.len(), 18);

  // Screenshot mode hides the transient chrome, live probes included.
  ruler.transient_chrome = false;
  out.clear();
  ruler.add_world_vertices(&mut out, view, 1.0, 2, Point::default(), Instant::now());
  assert!(out.is_empty());
}

#[test]
fn the_dimensions_row_needs_both_live_probes() {
  let desktop = Size {
    width: 1920.0,
    height: 1080.0,
  };
  let mut horizontal = probe(0, 1, 100.0, 300.0, 50.0);
  horizontal.flags = 4;
  let mut vertical = probe(0, 2, 20.0, 80.0, 200.0);
  vertical.flags = 4;

  assert_eq!(
    probe_dimensions_text(&[horizontal, vertical], 1, desktop),
    Some(" 200 ×   60 px".to_owned())
  );
  assert_eq!(probe_dimensions_text(&[horizontal], 1, desktop), None);
  // Another display's live probes never reach this readout.
  assert_eq!(
    probe_dimensions_text(&[horizontal, vertical], 2, desktop),
    None
  );
}
