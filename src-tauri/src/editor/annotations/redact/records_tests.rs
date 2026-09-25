// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use crate::editor::annotations::reveal::AnnotationReveal;

fn redaction(p0: [f32; 2], p2: [f32; 2], flags: u32) -> NativeAnnotation {
  NativeAnnotation {
    kind: AnnotationKind::Redact.raw(),
    flags,
    params: [8.0, 0.0, 1.0],
    color: [0.2, 0.4, 0.6, 1.0],
    p0,
    p1: p2,
    p2,
    reveal: AnnotationReveal::WHOLE,
    ..NativeAnnotation::default()
  }
}

#[test]
fn a_box_covers_every_pixel_it_touches_and_stops_at_the_source() {
  let items = [redaction([1.5, 2.2], [10.1, 300.0], 0)];
  let list = redact_records(&items, &[], 64, 100);
  assert_eq!(list.records[0].bounds, [1, 2, 11, 100]);
  // A box wholly outside the source covers nothing, so it is no pass.
  let outside = [redaction([70.0, 10.0], [90.0, 20.0], 0)];
  assert!(redact_records(&outside, &[], 64, 100).records.is_empty());
}

#[test]
fn each_pixelated_box_reads_only_its_own_zones() {
  let points = [[1.0, -1.0], [2.0, -1.0], [3.0, 4.0]];
  let mut first = redaction([0.0, 0.0], [16.0, 8.0], PIXELATE);
  (first.p3, first.data_offset, first.data_count) = ([2.0, 1.0], 0, 2);
  let mut second = redaction([20.0, 0.0], [28.0, 8.0], PIXELATE);
  (second.p3, second.data_offset, second.data_count) = ([1.0, 1.0], 2, 1);
  let list = redact_records(&[first, second], &points, 64, 64);
  let zones = |record: &RedactRecord| {
    let first = record.zone_first as usize;
    list.zones[first..first + record.entry_count as usize].to_vec()
  };
  assert_eq!(zones(&list.records[0]), points[..2]);
  assert_eq!(zones(&list.records[1]), points[2..]);
}

#[test]
fn an_arriving_box_fades_in_or_grows_its_cells_from_a_pixel() {
  let arriving = AnnotationReveal {
    opacity: 0.5,
    ..AnnotationReveal::WHOLE
  };
  let mut flat = redaction([0.0, 0.0], [32.0, 32.0], 0);
  flat.reveal = arriving;
  let mut blur = redaction([0.0, 0.0], [32.0, 32.0], BLUR);
  blur.reveal = arriving;
  let list = redact_records(&[flat, blur], &[], 64, 64);
  assert_eq!(list.records[0].color[3], 0.5);
  // Halfway from a pixel to eight.
  assert_eq!(list.records[1].size, 4.5);
  assert_eq!(list.records[1].grid, [8, 8]);
  assert_eq!(list.records[1].color[3], 1.0);
}
