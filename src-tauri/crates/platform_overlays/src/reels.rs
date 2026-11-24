// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Instagram Reels over a video: the title and camera at the top, the
//! outlined actions down the right, the account, caption and audio at the
//! bottom left, and the tab bar.

use super::icons::{
  CHEVRON_DOWN, INSTAGRAM_CAMERA, INSTAGRAM_COMMENT, INSTAGRAM_HEART, INSTAGRAM_HOME,
  INSTAGRAM_NEW, INSTAGRAM_REELS, INSTAGRAM_SEND, MORE_HORIZONTAL, MUSIC_NOTE, SEARCH,
};
use super::svg::{Anchor, Svg};
use super::{tab_bar, Words};

const WHITE: &str = "#ffffff";
const RAIL_X: f64 = 1010.0;
const TAB_BAR: f64 = 140.0;

pub(crate) fn draw(svg: &mut Svg, words: &Words) {
  svg.bottom_shade(700.0, TAB_BAR);
  svg.text(
    &words.reels,
    (40.0, svg.down(126.0)),
    46.0,
    700,
    WHITE,
    Anchor::Start,
    true,
  );
  svg.glyph_at(
    &CHEVRON_DOWN,
    (
      40.0 + words.reels.chars().count() as f64 * 26.0 + 26.0,
      svg.down(112.0),
    ),
    36.0,
    WHITE,
    true,
  );
  svg.glyph_at(
    &INSTAGRAM_CAMERA,
    (1018.0, svg.down(110.0)),
    62.0,
    WHITE,
    true,
  );

  let actions = [
    (&INSTAGRAM_HEART, Some("84.1K"), 880.0),
    (&INSTAGRAM_COMMENT, Some("612"), 720.0),
    (&INSTAGRAM_SEND, Some("9,027"), 560.0),
    (&MORE_HORIZONTAL, None, 420.0),
  ];
  for (glyph, count, from_bottom) in actions {
    let y = svg.up(from_bottom);
    svg.glyph_at(glyph, (RAIL_X, y), 66.0, WHITE, true);
    if let Some(count) = count {
      svg.text(
        count,
        (RAIL_X, y + 70.0),
        25.0,
        500,
        WHITE,
        Anchor::Middle,
        true,
      );
    }
  }
  // The audio's picture, a rounded square with a white edge.
  let audio = svg.up(268.0);
  svg.rect((RAIL_X - 32.0, audio - 32.0, 64.0, 64.0), 12.0, WHITE);
  svg.rect((RAIL_X - 28.0, audio - 28.0, 56.0, 56.0), 9.0, "#6b6f78");

  // The account with its Follow button, the caption, then the audio.
  let account = svg.up(340.0);
  svg.avatar((58.0, account), 30.0);
  svg.text(
    &words.handle_bare,
    (104.0, account + 11.0),
    30.0,
    600,
    WHITE,
    Anchor::Start,
    true,
  );
  let button = 104.0 + words.handle_bare.chars().count() as f64 * 16.5 + 22.0;
  svg.outline(
    (button, account - 26.0, 124.0, 52.0),
    12.0,
    "rgba(255,255,255,0.8)",
    2.5,
  );
  svg.text(
    &words.follow,
    (button + 62.0, account + 10.0),
    26.0,
    600,
    WHITE,
    Anchor::Middle,
    false,
  );
  svg.text(
    &words.caption,
    (32.0, svg.up(268.0)),
    30.0,
    400,
    WHITE,
    Anchor::Start,
    true,
  );
  svg.glyph_at(&MUSIC_NOTE, (48.0, svg.up(216.0)), 28.0, WHITE, true);
  svg.text(
    &words.original_audio,
    (72.0, svg.up(206.0)),
    26.0,
    400,
    WHITE,
    Anchor::Start,
    true,
  );

  let bar = tab_bar(svg, TAB_BAR, "#000000");
  let tabs = [&INSTAGRAM_HOME, &SEARCH, &INSTAGRAM_NEW, &INSTAGRAM_REELS];
  for (index, glyph) in tabs.into_iter().enumerate() {
    svg.glyph_at(
      glyph,
      (bar.slot(index), bar.icon + 18.0),
      54.0,
      WHITE,
      false,
    );
  }
  let profile = (bar.slot(4), bar.icon + 18.0);
  svg.avatar(profile, 26.0);
}
