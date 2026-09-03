// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn packet(x: f64, y: f64, width: f64, height: f64, kind: u8) -> OcrRectPacket {
  OcrRectPacket {
    x,
    y,
    width,
    height,
    kind,
    padding: [0; 7],
  }
}

const BOUNDS: Size = Size {
  width: 1920.0,
  height: 1080.0,
};

#[test]
fn highlight_kinds_follow_the_shared_visual_kinds() {
  assert_eq!(rect_kind(1), 17);
  assert_eq!(rect_kind(2), 18);
  assert_eq!(rect_kind(3), 19);
  assert_eq!(rect_kind(4), 20);
  // An unknown kind falls back to a text line, as the macOS ladder did.
  assert_eq!(rect_kind(9), 17);
}

#[test]
fn rects_are_translated_into_the_surface_and_filtered_to_it() {
  let packets = [
    packet(2000.0, 100.0, 200.0, 40.0, 1),
    packet(1000.0, 100.0, 200.0, 40.0, 2),
    // Entirely on the display to the left of this one.
    packet(100.0, 100.0, 200.0, 40.0, 1),
  ];
  let offset = Point { x: 1920.0, y: 0.0 };

  let kept = local_rects(&packets, offset, BOUNDS);
  assert_eq!(kept.len(), 1);
  assert_eq!(kept[0].rect, Rect::from_xywh(80.0, 100.0, 200.0, 40.0));
  assert_eq!(kept[0].kind, 1);

  // Without an offset the first display keeps the other two.
  let anchor = local_rects(&packets, Point::default(), BOUNDS);
  assert_eq!(anchor.len(), 2);
  assert_eq!(anchor[0].kind, 2);
}

#[test]
fn a_zero_area_rect_never_lands_on_a_surface() {
  let packets = [packet(10.0, 10.0, 0.0, 40.0, 1)];
  assert!(local_rects(&packets, Point::default(), BOUNDS).is_empty());
}

#[test]
fn the_target_surface_is_the_one_showing_most_of_the_selection() {
  // A selection straddling a seam: 300pt on the anchor, 100pt beyond it.
  let anchor = overlap_area(Rect::from_xywh(1620.0, 0.0, 400.0, 100.0), BOUNDS);
  let peer = overlap_area(Rect::from_xywh(-300.0, 0.0, 400.0, 100.0), BOUNDS);
  assert_eq!(anchor, 300.0 * 100.0);
  assert_eq!(peer, 100.0 * 100.0);
  assert!(anchor > peer);
  // A selection entirely off the surface contributes nothing.
  assert_eq!(
    overlap_area(Rect::from_xywh(-500.0, 0.0, 100.0, 100.0), BOUNDS),
    0.0
  );
}

#[test]
fn the_status_pill_stays_inside_its_surface() {
  let view = Size {
    width: 800.0,
    height: 600.0,
  };
  let centred = status_rect(100.0, view, Rect::from_xywh(300.0, 200.0, 200.0, 200.0));
  assert_eq!(centred.size.width, 128.0);
  assert_eq!(centred.origin.x, 400.0 - 64.0);
  assert_eq!(centred.origin.y, 300.0 - 14.0);

  // A selection in the corner pushes the pill back inside the margins.
  let clamped = status_rect(100.0, view, Rect::from_xywh(780.0, 590.0, 10.0, 10.0));
  assert_eq!(clamped.origin.x, 800.0 - 128.0 - 8.0);
  assert_eq!(clamped.origin.y, 600.0 - 28.0 - 8.0);

  // A long message widens the pill instead of clipping it.
  let wide = status_rect(400.0, view, Rect::from_xywh(0.0, 0.0, 800.0, 600.0));
  assert_eq!(wide.size.width, 424.0);
}

#[test]
fn toolbar_activation_maps_onto_the_command_phases() {
  let mut chrome = Chrome {
    phase: PHASE_READY,
    ..Chrome::default()
  };
  chrome.set_target(true);
  chrome
    .toolbar
    .layout(&std::array::from_fn::<_, CONTROL_COUNT, _>(|index| {
      ControlSpec {
        rect: Rect::from_xywh(index as f64 * 100.0, 0.0, 90.0, 24.0),
        style: if index < 2 {
          ControlStyle::button(ControlColor::Neutral, ControlSize::Compact)
        } else {
          ControlStyle::icon_button(ControlColor::Neutral, ControlSize::Compact)
        },
        icon: toolbar_icon(index),
      }
    }));

  // Copy all is control 1, so it dispatches phase 9.
  let inside = Point { x: 10.0, y: 10.0 };
  assert!(chrome.control_input(inside, PHASE_DOWN).consumed);
  assert_eq!(chrome.control_input(inside, PHASE_UP).dispatch, Some(9));

  // "Recognize another area" is control 3 → phase 11.
  let reset = Point { x: 210.0, y: 10.0 };
  chrome.control_input(reset, PHASE_DOWN);
  assert_eq!(chrome.control_input(reset, PHASE_UP).dispatch, Some(11));

  // Close arms first and only confirms on the second activation.
  let close = Point { x: 310.0, y: 10.0 };
  chrome.control_input(close, PHASE_DOWN);
  let armed = chrome.control_input(close, PHASE_UP);
  assert_eq!(armed.dispatch, None);
  assert!(armed.arm_confirm);
  chrome.control_input(close, PHASE_DOWN);
  let confirmed = chrome.control_input(close, PHASE_UP);
  assert_eq!(confirmed.dispatch, Some(12));

  // A point outside every control is left for the region gesture.
  assert!(
    !chrome
      .control_input(Point { x: 600.0, y: 600.0 }, PHASE_DOWN)
      .consumed
  );
}

#[test]
fn the_cancel_button_is_only_offered_events_while_it_is_visible() {
  let mut chrome = Chrome::default();
  chrome.cancel.layout(&[ControlSpec {
    rect: Rect::from_xywh(0.0, 0.0, 100.0, 36.0),
    style: ControlStyle::button(ControlColor::Neutral, ControlSize::Default),
    icon: ControlIcon::X,
  }]);
  let inside = Point { x: 10.0, y: 10.0 };
  assert!(!chrome.control_input(inside, PHASE_DOWN).consumed);

  chrome.set_cancel_visible(true);
  assert!(chrome.control_input(inside, PHASE_DOWN).consumed);
  assert_eq!(chrome.control_input(inside, PHASE_UP).dispatch, Some(8));
}

#[test]
fn the_ready_phase_hides_the_cancel_button_and_disarms_the_close() {
  let mut chrome = Chrome::default();
  chrome.set_cancel_visible(true);
  chrome.apply(PHASE_READY, &[], "", Point::default(), BOUNDS);
  assert!(!chrome.cancel_visible);

  chrome.close_armed = true;
  chrome.apply(PHASE_LOADING, &[], "Finding text", Point::default(), BOUNDS);
  assert!(!chrome.close_armed);
}

#[test]
fn an_early_platform_timer_keeps_the_close_confirmation_armed() {
  let now = Instant::now();
  let mut chrome = Chrome::default();
  let armed = chrome.confirm.press(now);
  chrome.close_armed = armed.armed;

  let early = chrome.expire_confirm_at(now + std::time::Duration::from_millis(1999));
  assert!(early.arm_confirm);
  assert!(!early.redraw);

  let expired = chrome.expire_confirm_at(now + std::time::Duration::from_millis(2000));
  assert!(!expired.arm_confirm);
  assert!(expired.redraw);
  assert!(!chrome.close_armed);
}

#[test]
fn only_the_target_surface_carries_the_pill_and_the_toolbar() {
  let mut chrome = Chrome::default();
  chrome.apply(PHASE_LOADING, &[], "Finding text", Point::default(), BOUNDS);
  chrome.set_target(true);
  assert!(chrome.status_visible && !chrome.toolbar_visible);

  chrome.set_target(false);
  assert!(!chrome.status_visible);

  chrome.apply(PHASE_READY, &[], "", Point::default(), BOUNDS);
  chrome.set_target(true);
  assert!(chrome.toolbar_visible && !chrome.status_visible);
}

#[test]
fn the_selection_border_only_draws_while_a_recognition_is_live() {
  let view = Size {
    width: 800.0,
    height: 600.0,
  };
  let region = Rect::from_xywh(100.0, 100.0, 200.0, 100.0);
  let mut chrome = Chrome::default();
  chrome.apply(
    PHASE_LOADING,
    &[packet(120.0, 120.0, 40.0, 12.0, 1)],
    "",
    Point::default(),
    view,
  );

  let mut out = Vec::new();
  chrome.add_world_vertices(&mut out, view, region, 1.0);
  let kinds = out
    .chunks_exact(6)
    .map(|quad| quad[0].kind)
    .collect::<Vec<_>>();
  assert_eq!(kinds, vec![18, 18, 18, 18, 17]);

  // The idle phase keeps the highlights out of the frame entirely.
  chrome.apply(0, &[], "", Point::default(), view);
  out.clear();
  chrome.add_world_vertices(&mut out, view, region, 1.0);
  assert!(out.is_empty());
}
