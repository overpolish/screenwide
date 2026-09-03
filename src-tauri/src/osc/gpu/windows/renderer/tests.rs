// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

const VIEW: Size = Size {
  width: 400.0,
  height: 200.0,
};

fn quad_kinds(vertices: &[Vertex]) -> Vec<u32> {
  vertices.chunks_exact(6).map(|quad| quad[0].kind).collect()
}

#[test]
fn shared_osc_shader_is_embedded_as_compiled_bytecode() {
  assert_eq!(&VERTEX_SHADER[..4], b"DXBC");
  assert_eq!(&PIXEL_SHADER[..4], b"DXBC");
}

#[test]
fn constants_match_the_shader_register_packing() {
  use std::mem::offset_of;
  assert_eq!(size_of::<RenderConstants>(), 448);
  assert_eq!(offset_of!(RenderConstants, light_mode), 0);
  assert_eq!(offset_of!(RenderConstants, magnifier_box), 16);
  assert_eq!(offset_of!(RenderConstants, action_fills), 32);
  assert_eq!(offset_of!(RenderConstants, control_colors), 64);
  assert_eq!(offset_of!(RenderConstants, ocr_colors), 96);
  assert_eq!(offset_of!(RenderConstants, overlay_shade), 224);
  assert_eq!(offset_of!(RenderConstants, ruler_colors), 240);
  assert_eq!(offset_of!(RenderConstants, ruler_sample), 272);
  assert_eq!(offset_of!(RenderConstants, ruler_animation), 288);
  assert_eq!(offset_of!(RenderConstants, magnifier_source), 304);
  assert_eq!(offset_of!(RenderConstants, magnifier_sample), 320);
  assert_eq!(offset_of!(RenderConstants, magnifier_source_range), 336);
  assert_eq!(offset_of!(RenderConstants, magnifier_flags), 352);
  // Appended, never renumbered: every offset above is unchanged.
  assert_eq!(offset_of!(RenderConstants, chrome), 368);
  assert_eq!(offset_of!(RenderConstants, chrome_outline), 384);
  assert_eq!(offset_of!(RenderConstants, chrome_backdrop), 400);
  assert_eq!(offset_of!(RenderConstants, chrome_source), 416);
  assert_eq!(offset_of!(RenderConstants, outlined_label), 432);
}

#[test]
fn chrome_plates_and_labels_use_the_appended_kinds() {
  let mut out = Vec::new();
  add_plate(&mut out, VIEW, Rect::from_xywh(10.0, 10.0, 80.0, 24.0));
  add_label(&mut out, VIEW, Rect::from_xywh(20.0, 14.0, 40.0, 16.0));
  assert_eq!(quad_kinds(&out), vec![46, 47]);

  // A degenerate rect emits nothing rather than a zero-area quad the SDF
  // would have to divide through.
  let mut empty = Vec::new();
  add_plate(&mut empty, VIEW, Rect::from_xywh(0.0, 0.0, 0.0, 24.0));
  add_label(&mut empty, VIEW, Rect::from_xywh(0.0, 0.0, 40.0, 0.0));
  assert!(empty.is_empty());
}

#[test]
fn constants_take_their_colors_from_the_shared_tokens() {
  let dark = RenderConstants::new(false);
  assert_eq!(dark.light_mode, [0, 0, 0, 0]);
  assert_eq!(dark.control_colors[0], control_palette(false).fill);
  assert_eq!(dark.control_colors[1], control_palette(false).outline);
  assert_eq!(dark.overlay_shade, overlay_palette().shade);
  assert_eq!(
    dark.ruler_colors,
    [ruler_palette(false).primary, ruler_palette(false).info]
  );
  assert_eq!(dark.ocr_colors[0], ocr_palette(false).primary_fill);
  assert_eq!(dark.ocr_colors[7], ocr_palette(false).selection_outline);
  assert_eq!(dark.action_fills, [[0.0; 4]; 2]);

  let light = RenderConstants::new(true);
  assert_eq!(light.light_mode, [1, 0, 0, 0]);
  assert_eq!(light.control_colors[0], control_palette(true).fill);
}

#[test]
fn vertices_land_in_normalized_device_coordinates() {
  let mut out = Vec::new();
  add_quad(&mut out, VIEW, Rect::from_xywh(0.0, 0.0, 400.0, 200.0), 6);

  assert_eq!(out.len(), 6);
  assert_eq!(out[0].position, [-1.0, 1.0]);
  assert_eq!(out[1].position, [1.0, 1.0]);
  assert_eq!(out[2].position, [1.0, -1.0]);
  assert_eq!(out[0].uv, [0.0, 0.0]);
  assert_eq!(out[5].uv, [0.0, 1.0]);
  assert!(out
    .iter()
    .all(|vertex| vertex.kind == 6 && vertex.padding == 0));
}

#[test]
fn lines_extend_by_a_half_width_at_both_ends() {
  let mut out = Vec::new();
  add_line(
    &mut out,
    VIEW,
    Point { x: 100.0, y: 100.0 },
    Point { x: 300.0, y: 100.0 },
    4.0,
    0,
  );

  assert_eq!(out.len(), 6);
  // 98px and 302px map to the extended horizontal span.
  assert!((f64::from(out[0].position[0]) - (2.0 * 98.0 / 400.0 - 1.0)).abs() < 1e-6);
  assert!((f64::from(out[1].position[0]) - (2.0 * 302.0 / 400.0 - 1.0)).abs() < 1e-6);

  let mut degenerate = Vec::new();
  add_line(
    &mut degenerate,
    VIEW,
    Point { x: 1.0, y: 1.0 },
    Point { x: 1.0, y: 1.0 },
    4.0,
    0,
  );
  add_line(
    &mut degenerate,
    VIEW,
    Point { x: 1.0, y: 1.0 },
    Point { x: 9.0, y: 1.0 },
    0.0,
    0,
  );
  assert!(degenerate.is_empty());
}

#[test]
fn snapping_separates_hairlines_from_handle_centers() {
  assert_eq!(snap(10.2, 2.0), 10.25);
  assert_eq!(snap(10.0, 1.0), 10.5);
  assert_eq!(snap_handle_center(10.2, 2.0), 10.0);
  assert_eq!(snap_handle_center(10.4, 1.0), 10.0);
  assert_eq!(snap_handle_center(10.6, 1.0), 11.0);
}

#[test]
fn marquee_edges_form_one_closed_boundary_aware_pattern() {
  let mut out = Vec::new();
  let scale = 2.0;
  let half = 1.5 / scale;
  add_crop_with_handles(
    &mut out,
    VIEW,
    Rect::from_xywh(100.0, 50.0, 96.0, 48.0),
    Rect::from_xywh(0.0, 0.0, 400.0, 200.0),
    scale,
    true,
    false,
  );

  // Four dim rects precede the four marching-ant edges.
  let ants = &out[4 * 6..8 * 6];
  assert_eq!(quad_kinds(ants), vec![8, 8, 10, 10]);
  // The marquee is exactly four edge quads. Corner circles would hide an
  // incorrectly clipped dash instead of giving it a true terminal cap.
  assert_eq!(out.len(), 8 * 6);
  let min_x = snap(100.0, scale);
  let max_x = snap(196.0, scale);
  let min_y = snap(50.0, scale);
  let max_y = snap(98.0, scale);
  let width = ((max_x - min_x) * scale) as f32;
  let height = ((max_y - min_y) * scale) as f32;
  assert_eq!(ants[0].uv, [0.0, 0.0]);
  assert_eq!(ants[1].uv, [width, 0.0]);
  assert_eq!(ants[0].aux, [0.0, width]);
  assert_eq!(ants[0].position, ndc(VIEW, min_x, min_y - half));
  assert_eq!(ants[1].position, ndc(VIEW, max_x, min_y - half));

  // The bottom and left run backwards so all four phases follow a single
  // clockwise perimeter. The final phase is an exact twelve-pixel cycle,
  // hence a dash cannot be accidentally cut at the closing corner.
  assert_eq!(ants[6].uv, [width, 0.0]);
  assert_eq!(ants[7].uv, [0.0, 0.0]);
  assert_eq!(ants[6].aux, [width + height, width]);
  assert_eq!(ants[12].uv, [0.0, height]);
  assert_eq!(ants[14].uv, [1.0, 0.0]);
  assert_eq!(ants[12].aux, [width * 2.0 + height, height]);
  assert_eq!(ants[12].position, ndc(VIEW, min_x - half, min_y));
  assert_eq!(ants[14].position, ndc(VIEW, min_x + half, max_y));
  assert_eq!(ants[18].aux, [width, height]);
  assert_eq!((width + height) * 2.0 % 12.0, 0.0);
}

#[test]
fn arc_uv_is_a_unit_quadrant_regardless_of_corner() {
  let mut out = Vec::new();
  add_ruler_arc(
    &mut out,
    VIEW,
    Point { x: 120.0, y: 80.0 },
    20.0,
    4,
    1.0,
    false,
    0.0,
    false,
  );

  assert_eq!(out.len(), 6);
  assert_eq!(quad_kinds(&out), vec![39]);
  // The quad spans -margin/radius .. (radius + margin)/radius on both axes,
  // so the arc itself always sits at unit distance from the uv origin.
  let margin = 1.5 / 20.0;
  assert_eq!(out[0].uv, [-margin as f32, -margin as f32]);
  assert_eq!(out[2].uv, [(1.0 + margin) as f32, (1.0 + margin) as f32]);

  let mut hovered = Vec::new();
  add_ruler_arc(
    &mut hovered,
    VIEW,
    Point { x: 120.0, y: 80.0 },
    20.0,
    1,
    1.0,
    true,
    5.0,
    true,
  );
  assert_eq!(quad_kinds(&hovered), vec![40, 41]);
}

#[test]
fn selection_emits_the_frame_passes_then_eight_handles() {
  let mut out = Vec::new();
  add_selection(
    &mut out,
    VIEW,
    Rect::from_xywh(40.0, 30.0, 200.0, 100.0),
    2.0,
    0.0,
    false,
  );

  assert_eq!(out.len(), 96);
  assert_eq!(
    quad_kinds(&out),
    vec![2, 2, 2, 2, 0, 0, 0, 0, 3, 16, 3, 16, 3, 16, 3, 16]
  );

  let mut rounded = Vec::new();
  add_selection(
    &mut rounded,
    VIEW,
    Rect::from_xywh(40.0, 30.0, 200.0, 100.0),
    2.0,
    50.0,
    true,
  );
  assert_eq!(rounded.len(), 102);
  assert_eq!(quad_kinds(&rounded).last(), Some(&3));
}

#[test]
fn selection_and_crop_share_identical_handle_primitives() {
  let frame = Rect::from_xywh(40.0, 30.0, 200.0, 100.0);
  let mut selection = Vec::new();
  add_selection(&mut selection, VIEW, frame, 2.0, 0.0, false);
  let mut crop = Vec::new();
  // Image equal to crop emits no shade, leaving four marquee quads followed
  // by the same shared handle run Export and Region both consume.
  add_crop(&mut crop, VIEW, frame, frame, 2.0);

  assert_eq!(&selection[8 * 6..16 * 6], &crop[4 * 6..12 * 6]);
  assert_eq!(
    quad_kinds(&selection[8 * 6..16 * 6]),
    vec![3, 16, 3, 16, 3, 16, 3, 16]
  );
}

#[test]
fn shared_text_primitives_distinguish_actions_and_readouts() {
  let rect = Rect::from_xywh(10.0, 12.0, 80.0, 24.0);
  let mut out = Vec::new();
  add_coverage_label(&mut out, VIEW, rect, false);
  add_coverage_label(&mut out, VIEW, rect, true);
  add_outlined_label(&mut out, VIEW, rect);
  assert_eq!(quad_kinds(&out), vec![49, 50, 51]);
}

#[test]
fn ruler_box_pairs_a_halo_with_four_hairlines() {
  let mut out = Vec::new();
  add_ruler_box(
    &mut out,
    VIEW,
    Rect::from_xywh(20.0, 20.0, 60.0, 40.0),
    2.0,
    false,
    0.0,
  );
  assert_eq!(quad_kinds(&out), vec![35, 28, 28, 28, 28]);

  let mut hovered = Vec::new();
  add_ruler_box(
    &mut hovered,
    VIEW,
    Rect::from_xywh(20.0, 20.0, 60.0, 40.0),
    2.0,
    true,
    6.0,
  );
  assert_eq!(quad_kinds(&hovered)[0], 34);

  // A frame with no interior height drops the two vertical hairlines.
  let mut flat = Vec::new();
  add_ruler_box(
    &mut flat,
    VIEW,
    Rect::from_xywh(20.0, 20.0, 60.0, 0.0),
    2.0,
    false,
    0.0,
  );
  assert_eq!(quad_kinds(&flat), vec![35, 28, 28]);
}

#[test]
fn crop_shade_skips_empty_bands_and_keeps_handles_optional() {
  let mut flush = Vec::new();
  add_crop_with_handles(
    &mut flush,
    VIEW,
    Rect::from_xywh(0.0, 0.0, 400.0, 100.0),
    Rect::from_xywh(0.0, 0.0, 400.0, 200.0),
    1.0,
    false,
    false,
  );
  assert_eq!(quad_kinds(&flush), vec![6]);

  let mut full = Vec::new();
  add_crop(
    &mut full,
    VIEW,
    Rect::from_xywh(50.0, 40.0, 200.0, 80.0),
    Rect::from_xywh(0.0, 0.0, 400.0, 200.0),
    1.0,
  );
  assert_eq!(
    quad_kinds(&full),
    vec![6, 6, 6, 6, 8, 8, 10, 10, 3, 16, 3, 16, 3, 16, 3, 16]
  );
}

#[test]
fn icons_offset_the_atlas_cell_by_twenty_one() {
  let mut out = Vec::new();
  add_icon(&mut out, VIEW, 0, 0.0, 0.0, 16.0);
  assert!(out.is_empty());

  add_icon(&mut out, VIEW, 3, 10.0, 10.0, 16.0);
  assert_eq!(quad_kinds(&out), vec![24]);
}

#[test]
fn texture_quads_carry_the_explicit_uv_rect() {
  let mut out = Vec::new();
  add_texture_quad(
    &mut out,
    VIEW,
    Rect::from_xywh(0.0, 0.0, 10.0, 10.0),
    Rect::from_xywh(0.25, 0.5, 0.25, 0.5),
    11,
  );

  assert_eq!(out[0].uv, [0.25, 0.5]);
  assert_eq!(out[2].uv, [0.5, 1.0]);
}

#[test]
fn magnifier_box_centers_a_ninety_six_point_lens_in_pixels() {
  let mut constants = RenderConstants::new(false);
  constants.set_magnifier(
    Point { x: 100.0, y: 60.0 },
    2.0,
    3,
    (1920, 1080),
    (0.5, 0.25),
    (0.0, 0.0),
    (2.0, 1.0),
  );

  assert_eq!(constants.magnifier_box, [104.0, 24.0, 192.0, 192.0]);
  assert_eq!(constants.magnifier_source, [1920.0, 1080.0, 2.0, 0.0]);
  assert_eq!(constants.magnifier_sample, [0.5, 0.25, 0.0, 0.0]);
  // Source bounds are clamped into the unit range like the macOS builder.
  assert_eq!(constants.magnifier_source_range, [0.0, 0.0, 1.0, 1.0]);
  assert_eq!(constants.magnifier_flags, [3, 1, 0, 0]);

  let mut out = Vec::new();
  add_magnifier(&mut out, VIEW, &constants);
  assert_eq!(quad_kinds(&out), vec![45]);

  constants.clear_magnifier();
  out.clear();
  add_magnifier(&mut out, VIEW, &constants);
  assert!(out.is_empty());
}

#[test]
fn magnifier_anchor_snaps_to_the_dragged_edge() {
  let frame = Rect::from_xywh(10.0, 20.0, 100.0, 50.0);
  let point = Point { x: 500.0, y: 5.0 };

  assert_eq!(magnifier_anchor(point, frame, 1).x, 10.0);
  assert_eq!(magnifier_anchor(point, frame, 2).x, 110.0);
  assert_eq!(magnifier_anchor(point, frame, 4).y, 20.0);
  assert_eq!(magnifier_anchor(point, frame, 8).y, 70.0);
  // Without an edge the pointer is used, clamped into the frame.
  assert_eq!(
    magnifier_anchor(point, frame, 0),
    Point { x: 110.0, y: 20.0 }
  );
}
