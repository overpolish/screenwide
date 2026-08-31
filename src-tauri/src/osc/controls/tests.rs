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
      ..ControlStyle::button(ControlColor::Neutral, ControlSize::Compact)
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
fn metrics_match_the_current_react_components() {
  let compact = control_metrics(ControlKind::Button, ControlSize::Compact);
  assert_eq!(
    (compact.height, compact.radius, compact.padding_x),
    (24.0, 8.0, 8.0)
  );
  assert_eq!(
    (compact.font_size, compact.line_height, compact.icon_size),
    (12.0, 16.0, 14.0)
  );

  let icon = control_metrics(ControlKind::IconButton, ControlSize::Default);
  assert_eq!(
    (icon.height, icon.radius, icon.padding_x, icon.icon_size),
    (36.0, 12.0, 6.0, 18.0)
  );
}

#[test]
fn spacing_matches_the_semantic_css_tokens() {
  let spacing = super::style::control_spacing();
  assert_eq!(spacing.tight, 2.0);
  assert_eq!(spacing.control, 4.0);
  assert_eq!(spacing.control_inset, 8.0);
  assert_eq!(spacing.section, 12.0);
  assert_eq!(spacing.window_inset, 24.0);
}

#[test]
fn neutral_tokens_match_the_translucent_css_values() {
  let light = control_visual(
    ControlStyle::button(ControlColor::Neutral, ControlSize::Compact),
    Interaction::Pressed,
    Appearance::Light,
  );
  assert_eq!(light.fill, [0.0, 0.0, 0.0, 0.17]);
  assert_eq!(
    light.foreground,
    [38.0 / 255.0, 38.0 / 255.0, 38.0 / 255.0, 1.0]
  );

  let dark = control_visual(
    ControlStyle::button(ControlColor::Neutral, ControlSize::Compact),
    Interaction::Pressed,
    Appearance::Dark,
  );
  assert_eq!(dark.fill, [1.0, 1.0, 1.0, 0.22]);
  assert_eq!(dark.foreground, [1.0; 4]);
}

#[test]
fn neutral_icon_buttons_keep_the_material_backing_fill() {
  let style = ControlStyle::icon_button(ControlColor::Neutral, ControlSize::Compact);
  let normal = control_visual(style, Interaction::Normal, Appearance::Light);
  let hovered = control_visual(style, Interaction::Hovered, Appearance::Light);

  assert_eq!(normal.fill, [0.0, 0.0, 0.0, 0.09]);
  assert_eq!(hovered.fill, [0.0, 0.0, 0.0, 0.13]);
}

#[test]
fn primary_tokens_match_the_translucent_css_values() {
  let style = ControlStyle::button(ControlColor::Primary, ControlSize::Compact);
  let light = control_visual(style, Interaction::Normal, Appearance::Light);
  let dark = control_visual(style, Interaction::Normal, Appearance::Dark);

  assert_eq!(
    light.fill,
    [216.0 / 255.0, 27.0 / 255.0, 96.0 / 255.0, 0.85]
  );
  assert_eq!(dark.fill, [1.0, 41.0 / 255.0, 112.0 / 255.0, 0.55]);
  assert_eq!(light.foreground, [1.0; 4]);
  assert_eq!(dark.foreground, [1.0; 4]);
}

#[test]
fn armed_icon_button_keeps_neutral_chrome_with_error_foreground() {
  let style = ControlStyle::icon_button(ControlColor::Error, ControlSize::Compact);
  let normal = control_visual(style, Interaction::Normal, Appearance::Light);
  let hovered = control_visual(style, Interaction::Hovered, Appearance::Dark);

  assert_eq!(normal.fill, [0.0, 0.0, 0.0, 0.09]);
  assert_eq!(normal.foreground, [215.0 / 255.0, 0.0, 21.0 / 255.0, 1.0]);
  assert_eq!(hovered.fill, [1.0, 1.0, 1.0, 0.17]);
  assert_eq!(hovered.foreground, [1.0, 105.0 / 255.0, 97.0 / 255.0, 1.0]);
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
