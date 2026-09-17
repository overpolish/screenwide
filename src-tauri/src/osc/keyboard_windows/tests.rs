// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{routes_to_overlay, Overlay};

#[test]
fn routes_plain_latched_and_command_shortcuts() {
  assert!(routes_to_overlay(
    Overlay::Ruler,
    0x31,
    0,
    false,
    true,
    false
  ));
  assert!(routes_to_overlay(
    Overlay::Ruler,
    0x31,
    0,
    true,
    true,
    false
  ));
  assert!(routes_to_overlay(
    Overlay::Ruler,
    0x31,
    0,
    false,
    false,
    true
  ));
  assert!(routes_to_overlay(
    Overlay::Ruler,
    0x43,
    2,
    false,
    true,
    false
  ));
  assert!(routes_to_overlay(
    Overlay::Ruler,
    0x43,
    0,
    false,
    false,
    true
  ));
  assert!(!routes_to_overlay(
    Overlay::Ruler,
    0x43,
    0,
    false,
    true,
    false
  ));
  assert!(!routes_to_overlay(
    Overlay::Ruler,
    0x41,
    2,
    false,
    true,
    false
  ));
  assert!(routes_to_overlay(
    Overlay::Ruler,
    0x12,
    0,
    false,
    true,
    false
  ));
  assert!(routes_to_overlay(
    Overlay::Ruler,
    0x12,
    0,
    false,
    false,
    true
  ));
}

#[test]
fn consumed_key_release_is_matched_after_modifiers_are_released() {
  assert!(routes_to_overlay(
    Overlay::Ruler,
    0x31,
    0,
    false,
    true,
    false
  ));
  assert!(routes_to_overlay(
    Overlay::Ruler,
    0x31,
    8,
    false,
    false,
    true
  ));
  assert!(!routes_to_overlay(
    Overlay::Ruler,
    0x31,
    0,
    false,
    false,
    true
  ));
}

#[test]
fn text_recognition_routes_only_control_a_and_control_c_down() {
  assert!(routes_to_overlay(
    Overlay::TextRecognition,
    0x41,
    2,
    false,
    true,
    false
  ));
  assert!(routes_to_overlay(
    Overlay::TextRecognition,
    0x43,
    2,
    false,
    true,
    false
  ));
  assert!(!routes_to_overlay(
    Overlay::TextRecognition,
    0x41,
    0,
    false,
    true,
    false
  ));
  assert!(!routes_to_overlay(
    Overlay::TextRecognition,
    0x41,
    2,
    false,
    false,
    true
  ));
}

#[test]
fn annotate_routes_its_own_keys_and_leaves_the_way_out_alone() {
  let down =
    |vk, modifiers| routes_to_overlay(Overlay::Annotate, vk, modifiers, false, true, false);
  // Undo, the two clears and the arrow tool.
  assert!(down(0x5a, 2));
  assert!(down(0x08, 0));
  assert!(down(0x2e, 0));
  assert!(down(0x41, 0));
  // Redo is not the overlay's, so Ctrl+Shift+Z keeps reaching the app behind.
  assert!(!down(0x5a, 2 | 8));
  // Escape and the activation shortcut belong to their own global
  // registrations; consuming either would leave no way out of the overlay.
  assert!(!down(0x1b, 0));
  assert!(!down(0x41, 2 | 8));
  // Nothing is consumed on release: only the press is a command.
  assert!(!routes_to_overlay(
    Overlay::Annotate,
    0x5a,
    2,
    false,
    false,
    true
  ));
}
