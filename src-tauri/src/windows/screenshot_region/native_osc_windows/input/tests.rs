// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

#[test]
fn hit_testing_declines_everything_the_webview_still_owns() {
  let exclusion = Rect::from_xywh(10.0, 10.0, 100.0, 40.0);
  let inside = Point { x: 20.0, y: 20.0 };
  let outside = Point { x: 200.0, y: 200.0 };

  assert!(hit_transparent(false, true, exclusion, outside));
  assert!(hit_transparent(true, false, exclusion, outside));
  assert!(hit_transparent(true, true, exclusion, inside));
  assert!(!hit_transparent(true, true, exclusion, outside));
  // An unset exclusion rect never swallows the pointer.
  assert!(!hit_transparent(
    true,
    true,
    Rect::default(),
    Point::default()
  ));
}

#[test]
fn modifier_bits_carry_shift_control_double_click_and_alt() {
  assert_eq!(modifier_bits(0, false, false), 0);
  assert_eq!(modifier_bits(MK_SHIFT, false, false), 1);
  assert_eq!(modifier_bits(MK_CONTROL, false, false), 2);
  assert_eq!(modifier_bits(0, true, false), 4);
  assert_eq!(modifier_bits(0, false, true), 8);
  assert_eq!(modifier_bits(MK_SHIFT | MK_CONTROL, true, true), 15);
}

#[test]
fn ocr_shortcuts_map_control_a_and_control_c_once() {
  assert_eq!(ocr_keyboard_phase(0x41, true, false), Some(6));
  assert_eq!(ocr_keyboard_phase(0x43, true, false), Some(7));
  assert_eq!(ocr_keyboard_phase(0x41, false, false), None);
  assert_eq!(ocr_keyboard_phase(0x41, true, true), None);
  assert_eq!(ocr_keyboard_phase(0x56, true, false), None);
}

#[test]
fn notched_vertical_wheels_zoom_while_precision_deltas_pan() {
  assert!(vertical_wheel_zooms(120, false));
  assert!(vertical_wheel_zooms(-120, false));
  assert!(vertical_wheel_zooms(240, false));
  assert!(!vertical_wheel_zooms(30, false));
  assert!(!vertical_wheel_zooms(-45, false));
  assert!(!vertical_wheel_zooms(0, false));
  assert!(vertical_wheel_zooms(30, true));
}

#[test]
fn cursor_icons_map_onto_system_cursors_with_diagonals_from_the_handle() {
  assert_eq!(cursor_shape(0, 0), CursorShape::None);
  assert_eq!(cursor_shape(1, 0), CursorShape::Crosshair);
  assert_eq!(cursor_shape(2, 0), CursorShape::Move);
  assert_eq!(cursor_shape(3, 0), CursorShape::Move);
  assert_eq!(cursor_shape(4, 0), CursorShape::ResizeHorizontal);
  assert_eq!(cursor_shape(5, 0), CursorShape::ResizeVertical);
  // 6 = north-east, 9 = south-west share one diagonal; 7/8 the other.
  assert_eq!(cursor_shape(6, 6), CursorShape::ResizeNesw);
  assert_eq!(cursor_shape(6, 9), CursorShape::ResizeNesw);
  assert_eq!(cursor_shape(6, 7), CursorShape::ResizeNwse);
  assert_eq!(cursor_shape(6, 8), CursorShape::ResizeNwse);
  assert_eq!(cursor_shape(7, 0), CursorShape::Arrow);
  assert_eq!(cursor_shape(8, 0), CursorShape::IBeam);
  assert_eq!(cursor_shape(9, 0), CursorShape::Hand);
}

#[test]
fn handle_edges_match_the_shared_bitmask() {
  assert_eq!(edges_for_handle(1), 0);
  assert_eq!(edges_for_handle(2), 4);
  assert_eq!(edges_for_handle(3), 8);
  assert_eq!(edges_for_handle(4), 2);
  assert_eq!(edges_for_handle(5), 1);
  assert_eq!(edges_for_handle(6), 6);
  assert_eq!(edges_for_handle(7), 5);
  assert_eq!(edges_for_handle(8), 10);
  assert_eq!(edges_for_handle(9), 9);
}

#[test]
fn the_lens_only_follows_a_live_resize() {
  assert_eq!(magnifier_for(PHASE_DRAG, GESTURE_RESIZING, 1, 6), Some(6));
  assert_eq!(magnifier_for(PHASE_MOVE, GESTURE_RESIZING, 1, 6), None);
  assert_eq!(magnifier_for(PHASE_DRAG, 1, 1, 6), None);
  assert_eq!(magnifier_for(PHASE_DRAG, GESTURE_RESIZING, 0, 6), None);
  assert_eq!(magnifier_for(PHASE_DRAG, GESTURE_RESIZING, 1, 0), None);
}
