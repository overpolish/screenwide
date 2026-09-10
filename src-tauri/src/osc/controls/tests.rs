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

#[test]
fn one_regular_size_matches_the_appkit_control_metrics() {
  let button = control_metrics(ControlKind::Button, ControlSize::Regular);
  assert_eq!(
    (button.height, button.radius, button.padding_x, button.gap),
    (24.0, 8.0, 12.0, 4.0)
  );
  assert_eq!(
    (button.font_size, button.line_height, button.icon_size),
    (13.0, 16.0, 16.0)
  );
  // Readouts assembled from the glyph atlas sit one tier below their labels.
  assert_eq!(
    (button.readout_font_size, button.readout_line_height),
    (11.0, 14.0)
  );
  // The ruler callout is the capture-size ToggleMenuButton: two readout lines
  // with the same 6pt breathing room above and below.
  assert_eq!((button.callout_height, button.callout_radius), (40.0, 11.0));

  let icon = control_metrics(ControlKind::IconButton, ControlSize::Regular);
  assert_eq!(
    (icon.height, icon.radius, icon.padding_x, icon.icon_size),
    (24.0, 8.0, 4.0, 16.0)
  );
  assert_eq!(icon.gap, 0.0);
  assert_eq!(
    (icon.readout_font_size, icon.readout_line_height),
    (11.0, 14.0)
  );
  assert_eq!((icon.callout_height, icon.callout_radius), (40.0, 11.0));
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
  assert_eq!(light.fill, [0.0, 0.0, 0.0, 0.10]);
  assert_eq!(light.foreground, [0.0, 0.0, 0.0, 0.85]);

  let dark = control_visual(
    ControlStyle::button(ControlColor::Neutral, ControlSize::Regular),
    Interaction::Normal,
    Appearance::Dark,
  );
  assert_eq!(dark.fill, [1.0, 1.0, 1.0, 0.10]);
  assert_eq!(dark.foreground, [1.0, 1.0, 1.0, 0.85]);
}

#[test]
fn bezeled_controls_ignore_hover_and_darken_only_when_pressed() {
  let style = ControlStyle::icon_button(ControlColor::Neutral, ControlSize::Regular);
  let normal = control_visual(style, Interaction::Normal, Appearance::Light);
  let hovered = control_visual(style, Interaction::Hovered, Appearance::Light);
  let pressed = control_visual(style, Interaction::Pressed, Appearance::Light);

  assert_eq!(hovered.fill, normal.fill);
  assert_eq!(pressed.fill, [0.0, 0.0, 0.0, 0.172]);
}

#[test]
fn disabled_controls_drop_to_the_tertiary_label_and_quaternary_fill() {
  let style = ControlStyle {
    disabled: true,
    ..ControlStyle::button(ControlColor::Primary, ControlSize::Regular)
  };
  let visual = control_visual(style, Interaction::Disabled, Appearance::Dark);
  assert_eq!(visual.fill, [1.0, 1.0, 1.0, 0.03]);
  assert_eq!(visual.foreground, [1.0, 1.0, 1.0, 0.25]);
}

#[test]
fn primary_fills_paint_the_accent_opaque_and_darken_on_press() {
  let style = ControlStyle::button(ControlColor::Primary, ControlSize::Regular);
  let normal = control_visual(style, Interaction::Normal, Appearance::Light);
  let hovered = control_visual(style, Interaction::Hovered, Appearance::Light);
  let pressed = control_visual(style, Interaction::Pressed, Appearance::Light);

  let accent = crate::system_accent::accent_rgb();
  assert_eq!(normal.fill, [accent[0], accent[1], accent[2], 1.0]);
  assert_eq!(normal.fill[3], 1.0);
  for (channel, expected) in accent.iter().enumerate() {
    assert!((hovered.fill[channel] - expected * 0.90).abs() < 1e-6);
    assert!((pressed.fill[channel] - expected * 0.80).abs() < 1e-6);
  }
  assert_eq!(normal.foreground, [1.0; 4]);
}

#[test]
fn armed_icon_button_keeps_neutral_chrome_with_error_foreground() {
  let style = ControlStyle::icon_button(ControlColor::Error, ControlSize::Regular);
  let normal = control_visual(style, Interaction::Normal, Appearance::Light);
  let hovered = control_visual(style, Interaction::Hovered, Appearance::Dark);

  assert_eq!(normal.fill, [0.0, 0.0, 0.0, 0.10]);
  assert_eq!(normal.foreground, [1.0, 56.0 / 255.0, 60.0 / 255.0, 1.0]);
  assert_eq!(hovered.fill, [1.0, 1.0, 1.0, 0.10]);
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
