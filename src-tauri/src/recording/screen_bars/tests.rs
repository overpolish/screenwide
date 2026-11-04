// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// A 1800 by 1169 point display with a 39 point menu bar and a Dock
/// `dock_at` points along its bottom.
fn display(dock_at: f64) -> DisplayBars {
  DisplayBars {
    width: 1_800.0,
    height: 1_169.0,
    menu_bar: Insets {
      top: 39.0,
      ..Insets::default()
    },
    dock: Insets {
      bottom: dock_at,
      ..Insets::default()
    },
  }
}

fn region(x: f64, y: f64, width: f64, height: f64) -> Option<(f64, f64, f64, f64)> {
  Some((x, y, width, height))
}

fn kept(bars: &DisplayBars, menu_bar: bool, dock: bool) -> Option<(f64, f64, f64, f64)> {
  kept_region(bars, menu_bar, dock).map(|region| {
    (
      region.position.x,
      region.position.y,
      region.size.width,
      region.size.height,
    )
  })
}

#[test]
fn each_bar_is_left_out_only_when_chosen() {
  let bars = display(80.0);
  assert_eq!(
    kept(&bars, true, false),
    region(0.0, 39.0, 1_800.0, 1_130.0)
  );
  assert_eq!(kept(&bars, false, true), region(0.0, 0.0, 1_800.0, 1_089.0));
  assert_eq!(kept(&bars, true, true), region(0.0, 39.0, 1_800.0, 1_050.0));
  assert_eq!(kept(&bars, false, false), None);
}

#[test]
fn a_dock_on_the_side_narrows_the_picture() {
  let bars = DisplayBars {
    dock: Insets {
      left: 64.0,
      ..Insets::default()
    },
    ..display(0.0)
  };
  assert_eq!(
    kept(&bars, false, true),
    region(64.0, 0.0, 1_736.0, 1_169.0)
  );
}

#[test]
fn a_hidden_bar_takes_no_room_and_leaves_the_display_whole() {
  // A hidden Dock still keeps a few points free to slide out from.
  assert_eq!(kept(&display(4.0), false, true), None);
  let hidden_menu_bar = DisplayBars {
    menu_bar: Insets::default(),
    ..display(0.0)
  };
  assert_eq!(kept(&hidden_menu_bar, true, true), None);
}

#[test]
fn a_taskbar_is_left_out_from_whichever_edge_it_is_on() {
  let taskbar = DisplayBars {
    width: 1_536.0,
    height: 864.0,
    menu_bar: Insets {
      right: 48.0,
      ..Insets::default()
    },
    dock: Insets::default(),
  };
  assert_eq!(
    kept(&taskbar, true, false),
    region(0.0, 0.0, 1_488.0, 864.0)
  );
}
