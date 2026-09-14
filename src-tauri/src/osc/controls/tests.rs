// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::icons::{icon_atlas, ICON_ATLAS_CELL_SIZE, ICON_ATLAS_COLUMNS};
use super::*;
use crate::osc::geometry::Rect;

fn spec(x: f64, disabled: bool) -> ControlSpec {
  ControlSpec {
    rect: Rect::from_xywh(x, 20.0, 60.0, 24.0),
    icon: ControlIcon::None,
    style: ControlStyle {
      disabled,
      ..ControlStyle::button(ControlColor::Neutral, ControlSize::Regular)
    },
  }
}

#[test]
fn lucide_icons_share_one_supersampled_alpha_atlas() {
  let atlas = icon_atlas();
  assert_eq!(atlas.width, ICON_ATLAS_COLUMNS * ICON_ATLAS_CELL_SIZE);
  assert_eq!(atlas.height, ICON_ATLAS_CELL_SIZE);
  assert_eq!(atlas.pixels().len(), (atlas.width * atlas.height) as usize);

  let cell_has_coverage = |cell: u32| {
    (0..ICON_ATLAS_CELL_SIZE).any(|y| {
      let start = (y * atlas.width + cell * ICON_ATLAS_CELL_SIZE) as usize;
      atlas.pixels()[start..start + ICON_ATLAS_CELL_SIZE as usize]
        .iter()
        .any(|alpha| *alpha != 0)
    })
  };
  assert!(!cell_has_coverage(ControlIcon::None as u32));
  for cell in 1..ICON_ATLAS_COLUMNS {
    assert!(cell_has_coverage(cell));
  }
  assert!(atlas.pixels().iter().any(|alpha| (1..=254).contains(alpha)));
}

/// The values the tests expect for the build's platform: AppKit's on macOS,
/// Fluent's on the Windows skin. Both come from the same tokens the page uses.
const fn skin<T: Copy>(appkit: T, fluent: T) -> T {
  if super::style::WINDOWS_SKIN {
    fluent
  } else {
    appkit
  }
}

#[test]
fn one_regular_size_matches_the_platform_control_metrics() {
  let button = control_metrics(ControlKind::Button, ControlSize::Regular);
  assert_eq!(
    (button.height, button.radius, button.padding_x, button.gap),
    skin((24.0, 8.0, 12.0, 4.0), (32.0, 4.0, 12.0, 4.0))
  );
  assert_eq!(
    (button.font_size, button.line_height, button.icon_size),
    skin((13.0, 16.0, 16.0), (14.0, 20.0, 16.0))
  );
  // Readouts assembled from the glyph atlas sit one tier below their labels.
  assert_eq!(
    (button.readout_font_size, button.readout_line_height),
    skin((11.0, 14.0), (14.0, 18.0))
  );
  // The ruler callout is the capture-size ToggleMenuButton: two readout lines
  // with the same 6pt breathing room above and below. macOS rounds the
  // taller control more; Fluent keeps the control corner.
  assert_eq!(
    (button.callout_height, button.callout_radius),
    skin((40.0, 11.0), (40.0, 4.0))
  );

  let icon = control_metrics(ControlKind::IconButton, ControlSize::Regular);
  assert_eq!(
    (icon.height, icon.radius, icon.padding_x, icon.icon_size),
    skin((24.0, 8.0, 4.0, 16.0), (32.0, 4.0, 8.0, 16.0))
  );
  assert_eq!(icon.gap, 0.0);
  assert_eq!(
    (icon.readout_font_size, icon.readout_line_height),
    skin((11.0, 14.0), (14.0, 18.0))
  );
  assert_eq!(
    (icon.callout_height, icon.callout_radius),
    skin((40.0, 11.0), (40.0, 4.0))
  );
}

#[test]
fn spacing_matches_the_semantic_css_tokens() {
  let spacing = super::style::control_spacing();
  assert_eq!(spacing.tight, 2.0);
  assert_eq!(spacing.control, 4.0);
  assert_eq!(spacing.control_inset, 8.0);
  assert_eq!(spacing.section, 12.0);
  assert_eq!(spacing.layout, 24.0);
  assert_eq!(spacing.window_inset, 14.0);
}

#[test]
fn neutral_labels_use_the_content_tier_over_the_system_fill() {
  let light = control_visual(
    ControlStyle::button(ControlColor::Neutral, ControlSize::Regular),
    Interaction::Normal,
    Appearance::Light,
  );
  assert_eq!(
    light.fill,
    skin([0.0, 0.0, 0.0, 0.10], [1.0, 1.0, 1.0, 0.70])
  );
  assert_eq!(
    light.foreground,
    skin([0.0, 0.0, 0.0, 0.85], [0.0, 0.0, 0.0, 0.896])
  );

  let dark = control_visual(
    ControlStyle::button(ControlColor::Neutral, ControlSize::Regular),
    Interaction::Normal,
    Appearance::Dark,
  );
  assert_eq!(
    dark.fill,
    skin([1.0, 1.0, 1.0, 0.10], [1.0, 1.0, 1.0, 0.06])
  );
  assert_eq!(
    dark.foreground,
    skin([1.0, 1.0, 1.0, 0.85], [1.0, 1.0, 1.0, 1.0])
  );
}

/// A macOS bezel ignores hover and darkens when pressed; a Fluent bezel
/// changes fill on both, and is lighter than its rest state while pressed.
#[test]
fn bezeled_controls_follow_the_platform_hover_and_press_fills() {
  let style = ControlStyle::icon_button(ControlColor::Neutral, ControlSize::Regular);
  let normal = control_visual(style, Interaction::Normal, Appearance::Light);
  let hovered = control_visual(style, Interaction::Hovered, Appearance::Light);
  let pressed = control_visual(style, Interaction::Pressed, Appearance::Light);

  if super::style::WINDOWS_SKIN {
    let near_white = 249.0 / 255.0;
    assert_eq!(hovered.fill, [near_white, near_white, near_white, 0.50]);
    assert_eq!(pressed.fill, [near_white, near_white, near_white, 0.30]);
  } else {
    assert_eq!(hovered.fill, normal.fill);
    assert_eq!(pressed.fill, [0.0, 0.0, 0.0, 0.172]);
  }
}

#[test]
fn disabled_controls_drop_to_the_disabled_label_and_fill() {
  let style = ControlStyle {
    disabled: true,
    ..ControlStyle::button(ControlColor::Primary, ControlSize::Regular)
  };
  let visual = control_visual(style, Interaction::Disabled, Appearance::Dark);
  assert_eq!(
    visual.fill,
    skin([1.0, 1.0, 1.0, 0.03], [1.0, 1.0, 1.0, 0.158])
  );
  assert_eq!(
    visual.foreground,
    skin([1.0, 1.0, 1.0, 0.25], [1.0, 1.0, 1.0, 0.363])
  );
}

/// macOS paints the accent opaque and mixes it toward black under the
/// pointer and pressed; Fluent paints the accent's light-appearance tone and
/// fades it to 90% and 80%.
#[test]
fn primary_fills_paint_the_accent_and_step_it_on_hover_and_press() {
  let style = ControlStyle::button(ControlColor::Primary, ControlSize::Regular);
  let normal = control_visual(style, Interaction::Normal, Appearance::Light);
  let hovered = control_visual(style, Interaction::Hovered, Appearance::Light);
  let pressed = control_visual(style, Interaction::Pressed, Appearance::Light);

  if super::style::WINDOWS_SKIN {
    let accent = crate::system_accent::accent_fill_rgb(true);
    assert_eq!(normal.fill, [accent[0], accent[1], accent[2], 1.0]);
    assert_eq!(hovered.fill, [accent[0], accent[1], accent[2], 0.90]);
    assert_eq!(pressed.fill, [accent[0], accent[1], accent[2], 0.80]);
  } else {
    let accent = crate::system_accent::accent_rgb();
    assert_eq!(normal.fill, [accent[0], accent[1], accent[2], 1.0]);
    for (channel, expected) in accent.iter().enumerate() {
      assert!((hovered.fill[channel] - expected * 0.90).abs() < 1e-6);
      assert!((pressed.fill[channel] - expected * 0.80).abs() < 1e-6);
    }
  }
  // White on a light-appearance accent fill on every platform.
  assert_eq!(normal.foreground, [1.0; 4]);
}

#[test]
fn armed_icon_button_keeps_neutral_chrome_with_error_foreground() {
  let style = ControlStyle::icon_button(ControlColor::Error, ControlSize::Regular);
  let normal = control_visual(style, Interaction::Normal, Appearance::Light);
  let hovered = control_visual(style, Interaction::Hovered, Appearance::Dark);

  assert_eq!(
    normal.fill,
    skin([0.0, 0.0, 0.0, 0.10], [1.0, 1.0, 1.0, 0.70])
  );
  assert_eq!(normal.foreground, [1.0, 56.0 / 255.0, 60.0 / 255.0, 1.0]);
  assert_eq!(
    hovered.fill,
    skin([1.0, 1.0, 1.0, 0.10], [1.0, 1.0, 1.0, 0.084])
  );
  assert_eq!(hovered.foreground, [1.0, 66.0 / 255.0, 69.0 / 255.0, 1.0]);
}

#[test]
fn group_has_independent_buttons_and_no_gap_hit() {
  let mut group = ControlGroup::default();
  group.layout(&[spec(10.0, false), spec(80.0, false)]);

  assert_eq!(group.hit_index((20.0, 30.0)), 1);
  assert_eq!(group.hit_index((75.0, 30.0)), 0);
  assert_eq!(group.hit_index((90.0, 30.0)), 2);

  assert!(group.down((20.0, 30.0)).consumed);
  group.move_to((90.0, 30.0));
  assert_eq!(group.up((90.0, 30.0)).activated, 0);
}

#[test]
fn disabled_controls_neither_hit_nor_activate() {
  let mut group = ControlGroup::default();
  group.layout(&[spec(10.0, true)]);
  assert_eq!(group.hit_index((20.0, 30.0)), 0);
  assert!(!group.down((20.0, 30.0)).consumed);
  assert_eq!(group.up((20.0, 30.0)).activated, 0);
}
