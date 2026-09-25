// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::gesture::{EDGE_BOTTOM, EDGE_LEFT, EDGE_RIGHT, EDGE_TOP};
use super::native::{fill, source_per_capture_point, RedactPicture, RedactSource};
use super::palette::{packed, zones, Palette};
use crate::editor::annotations::edit::AnnotationEdit;
use crate::editor::annotations::flags::PIXELATE;
use crate::editor::annotations::gesture::{AnnotationGestureTarget, AnnotationHandle, BOX_HANDLES};
use crate::editor::annotations::snap::SnapModifiers;
use crate::editor::annotations::{
  Annotation, AnnotationKind, AnnotationPoint, AnnotationRedaction, AnnotationShape,
};

fn point(x: f64, y: f64) -> AnnotationPoint {
  AnnotationPoint { x, y }
}

fn corners(annotation: &Annotation) -> (AnnotationPoint, AnnotationPoint) {
  let AnnotationShape::Redact { start, end, .. } = annotation.shape else {
    unreachable!()
  };
  (start, end)
}

fn held(shift: bool) -> SnapModifiers {
  SnapModifiers {
    shift,
    position: false,
  }
}

/// Draws a fresh box from `from` to `to` and returns the list it lands in.
fn drawn(from: AnnotationPoint, to: AnnotationPoint, shift: bool) -> Vec<Annotation> {
  let mut annotations = Vec::new();
  let edit = AnnotationEdit::begin(
    &mut annotations,
    AnnotationGestureTarget::New,
    from,
    None,
    Some(AnnotationKind::Redact),
    None,
    0.0,
  )
  .unwrap();
  edit.update(&mut annotations, to, held(shift), None);
  annotations
}

/// Drags grip `handle` of `annotations[0]` from `from` to `to`.
fn dragged(
  annotations: &mut Vec<Annotation>,
  handle: AnnotationHandle,
  from: AnnotationPoint,
  to: AnnotationPoint,
  shift: bool,
) {
  let edit = AnnotationEdit::begin(
    annotations,
    AnnotationGestureTarget::Existing { index: 0, handle },
    from,
    None,
    None,
    None,
    0.0,
  )
  .unwrap();
  edit.update(annotations, to, held(shift), None);
}

#[test]
fn a_box_drawn_up_and_left_is_stored_top_left_first() {
  let annotations = drawn(point(80.0, 60.0), point(20.0, 10.0), false);
  assert_eq!(
    corners(&annotations[0]),
    (point(20.0, 10.0), point(80.0, 60.0))
  );
  assert!(!annotations[0].animated);
}

#[test]
fn shift_pulls_a_fresh_box_out_square() {
  let annotations = drawn(point(10.0, 10.0), point(50.0, 30.0), true);
  assert_eq!(
    corners(&annotations[0]),
    (point(10.0, 10.0), point(50.0, 50.0))
  );
}

#[test]
fn an_edge_dragged_past_its_opposite_turns_the_box_inside_out() {
  let mut annotations = drawn(point(10.0, 10.0), point(50.0, 40.0), false);
  dragged(
    &mut annotations,
    AnnotationHandle::Edges(EDGE_RIGHT),
    point(50.0, 25.0),
    point(0.0, 99.0),
    false,
  );
  // Only the right side moved; the vertical travel was ignored.
  assert_eq!(
    corners(&annotations[0]),
    (point(0.0, 10.0), point(10.0, 40.0))
  );
}

#[test]
fn shift_on_a_corner_keeps_the_box_proportions() {
  let mut annotations = drawn(point(0.0, 0.0), point(40.0, 20.0), false);
  dragged(
    &mut annotations,
    AnnotationHandle::Edges(EDGE_RIGHT | EDGE_BOTTOM),
    point(40.0, 20.0),
    point(100.0, 25.0),
    true,
  );
  assert_eq!(
    corners(&annotations[0]),
    (point(0.0, 0.0), point(100.0, 50.0))
  );
}

#[test]
fn the_body_carries_the_box_without_resizing_it() {
  let mut annotations = drawn(point(10.0, 10.0), point(50.0, 40.0), false);
  dragged(
    &mut annotations,
    AnnotationHandle::Body,
    point(30.0, 20.0),
    point(35.0, 12.0),
    false,
  );
  assert_eq!(
    corners(&annotations[0]),
    (point(15.0, 2.0), point(55.0, 32.0))
  );
}

#[test]
fn box_grips_read_back_from_their_native_numbers() {
  let grip = |edges: u32| AnnotationHandle::from_raw(BOX_HANDLES + edges);
  assert_eq!(
    grip(EDGE_TOP | EDGE_LEFT),
    Some(AnnotationHandle::Edges(EDGE_TOP | EDGE_LEFT))
  );
  assert_eq!(
    grip(EDGE_BOTTOM),
    Some(AnnotationHandle::Edges(EDGE_BOTTOM))
  );
  assert_eq!(grip(0), None);
  assert_eq!(grip(EDGE_LEFT | EDGE_RIGHT), None);
  assert_eq!(grip(EDGE_TOP | EDGE_BOTTOM | EDGE_LEFT), None);
}

#[test]
fn round_trips_a_redaction_in_camel_case() {
  let mut annotation = drawn(point(1.0, 2.0), point(30.0, 40.0), false).remove(0);
  annotation.style.redaction = AnnotationRedaction::Pixelate;
  let json = serde_json::to_string(&annotation).unwrap();
  assert!(json.contains("\"kind\":\"redact\""), "{json}");
  assert!(json.contains("\"redaction\":\"pixelate\""), "{json}");
  assert_eq!(
    serde_json::from_str::<Annotation>(&json).unwrap(),
    annotation
  );
}

/// A 20 by 20 grey picture whose middle 10 by 10 holds `secret`.
fn picture_with(secret: u8) -> Vec<u8> {
  let mut rgba = [200, 200, 200, 255].repeat(400);
  for y in 5..15 {
    for x in 5..15 {
      rgba[(y * 20 + x) * 4..][..3].fill(secret);
    }
  }
  rgba
}

#[test]
fn erase_takes_the_surrounding_colour_and_never_the_covered_one() {
  let mut annotation = drawn(point(5.0, 5.0), point(15.0, 15.0), false).remove(0);
  annotation.style.color = "#ff000080".to_owned();
  let AnnotationShape::Redact { start, end, seed } = annotation.shape else {
    unreachable!()
  };
  let filled = |style: &crate::editor::annotations::AnnotationStyle, secret: u8| {
    let rgba = picture_with(secret);
    let picture = RedactPicture::new(&rgba, 20, 20, 20.0);
    fill(
      start,
      end,
      seed,
      style,
      RedactSource::Picture(&picture),
      None,
    )
  };
  let grey = 200.0 / 255.0;
  assert_eq!(filled(&annotation.style, 0).color, [grey, grey, grey, 1.0]);
  assert_eq!(filled(&annotation.style, 0), filled(&annotation.style, 255));
  // The colour mode ignores the picture and drops the style's alpha.
  annotation.style.redaction = AnnotationRedaction::Color;
  assert_eq!(filled(&annotation.style, 0).color, [1.0, 0.0, 0.0, 1.0]);
}

#[test]
fn pixelate_carries_its_seed_and_its_block_in_source_pixels() {
  let mut annotation = drawn(point(5.0, 5.0), point(15.0, 15.0), false).remove(0);
  annotation.style.redaction = AnnotationRedaction::Pixelate;
  annotation.style.width = 12.0;
  let AnnotationShape::Redact { start, end, seed } = annotation.shape else {
    unreachable!()
  };
  let rgba = picture_with(0);
  // A 2x capture: 20 pixels across are 10 points, each point two pixels.
  let picture = RedactPicture::new(&rgba, 20, 20, 10.0);
  let filled = fill(
    start,
    end,
    seed,
    &annotation.style,
    RedactSource::Picture(&picture),
    None,
  );
  assert_eq!(filled.flags, PIXELATE);
  assert_eq!(filled.params[0], 24.0);
  let halves = (filled.params[1] as u32) << 16 | filled.params[2] as u32;
  assert_eq!(halves, seed);
}

#[test]
fn a_decoded_proxy_sizes_cells_to_the_same_share_of_the_capture() {
  // A 2x recording 1,920 pixels wide is 960 points: its own frames and a
  // half-size proxy of them put the same points under every cell.
  let full = source_per_capture_point(1_920, 960.0);
  let proxy = source_per_capture_point(960, 960.0);
  assert_eq!(full, 2.0);
  assert_eq!(proxy, 1.0);
  // Unknown is one point per pixel, never a zero cell.
  assert_eq!(source_per_capture_point(640, 0.0), 1.0);
}

/// A 20 by 10 grey box, `first` painted over the leading `first_count`
/// pixels and `second` over the next `second_count`.
fn covered(first: [u8; 3], first_count: usize, second: [u8; 3], second_count: usize) -> Vec<u8> {
  let mut rgba = [200, 200, 200, 255].repeat(200);
  for (index, pixel) in rgba.as_chunks_mut::<4>().0.iter_mut().enumerate() {
    if index < first_count {
      pixel[..3].copy_from_slice(&first);
    } else if index < first_count + second_count {
      pixel[..3].copy_from_slice(&second);
    }
  }
  rgba
}

#[test]
fn inks_are_the_covered_colours_in_an_order_that_tells_nothing() {
  let (navy, red) = ([10, 20, 80], [220, 30, 30]);
  let mut palette = Palette::default();
  let mostly_navy = palette.inks(
    &covered(navy, 60, red, 20),
    20,
    [0, 0, 20, 10],
    [200; 3],
    1_000,
  );
  let mostly_red = palette.inks(
    &covered(red, 60, navy, 20),
    20,
    [0, 0, 20, 10],
    [200; 3],
    1_000,
  );
  assert_eq!(mostly_navy, vec![navy, red]);
  assert_eq!(mostly_red, mostly_navy);
}

#[test]
fn a_colour_close_to_the_surface_or_too_rare_is_no_ink() {
  let rgba = covered([190, 190, 190], 100, [0, 0, 0], 2);
  assert!(Palette::default()
    .inks(&rgba, 20, [0, 0, 20, 10], [200; 3], 1_000)
    .is_empty());
}

/// Each zone keeps its own colours: navy text on the left, red on the right,
/// and a zone with nothing but surface holds no ink.
#[test]
fn each_zone_takes_the_colours_under_it() {
  let (navy, red) = ([10, 20, 80], [220, 30, 30]);
  let mut rgba = [200, 200, 200, 255].repeat(48 * 8);
  for (index, pixel) in rgba.as_chunks_mut::<4>().0.iter_mut().enumerate() {
    match index % 48 {
      0..16 if index % 3 == 0 => pixel[..3].copy_from_slice(&navy),
      16..32 if index % 3 == 0 => pixel[..3].copy_from_slice(&red),
      _ => {}
    }
  }
  // Blocks of eight pixels, zones of two blocks: three zones across one row.
  let cut = zones(&rgba, 48, [0, 0, 48, 8], [200; 3], 8.0);
  assert_eq!((cut.columns, cut.blocks), (3, 2));
  assert_eq!(
    cut.inks,
    vec![
      [packed(Some(&navy)), -1.0],
      [packed(Some(&red)), -1.0],
      [-1.0, -1.0],
    ]
  );
}

/// A zone is as tall as the box, so large type over one line still has more
/// than a character in every zone.
#[test]
fn a_zone_is_as_tall_as_its_box() {
  let rgba = [200, 200, 200, 255].repeat(400 * 40);
  let cut = zones(&rgba, 400, [0, 0, 400, 40], [200; 3], 8.0);
  // Forty pixels high is five blocks of eight: one row of five-block zones.
  assert_eq!((cut.columns, cut.blocks), (10, 5));
}

/// A box too long for its blocks takes larger zones rather than more of
/// them.
#[test]
fn a_long_box_takes_larger_zones() {
  let rgba = [200, 200, 200, 255].repeat(20_000);
  let cut = zones(&rgba, 20_000, [0, 0, 20_000, 1], [200; 3], 1.0);
  assert!(cut.inks.len() <= 4_096, "{}", cut.inks.len());
  assert_eq!(cut.blocks, 8);
}
