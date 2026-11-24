// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! YouTube Shorts over a video: search and more at the top, the actions on
//! their discs down the right, the channel, title and sound at the bottom
//! left, the red progress line and the tab bar.

use super::icons::{
  CREATE_CIRCLE, MORE_VERTICAL, MUSIC_NOTE, PROFILE, REMIX, SEARCH, SHORTS_COMMENT, SHORTS_SHARE,
  SHORTS_TAB, SUBSCRIPTIONS, THUMB_DOWN, THUMB_UP, YOUTUBE_HOME,
};
use super::svg::{Anchor, Svg, WIDTH};
use super::{tab_bar, Words};

const WHITE: &str = "#ffffff";
const RAIL_X: f64 = 1004.0;
const TAB_BAR: f64 = 140.0;

pub(crate) fn draw(svg: &mut Svg, words: &Words) {
  svg.bottom_shade(700.0, TAB_BAR);
  svg.glyph_at(&SEARCH, (920.0, svg.down(104.0)), 58.0, WHITE, true);
  svg.glyph_at(&MORE_VERTICAL, (1024.0, svg.down(104.0)), 58.0, WHITE, true);

  let actions = [
    (&THUMB_UP, words.likes.as_str(), 1010.0),
    (&THUMB_DOWN, words.dislike.as_str(), 850.0),
    (&SHORTS_COMMENT, "2,318", 690.0),
    (&SHORTS_SHARE, words.share.as_str(), 530.0),
    (&REMIX, words.remix.as_str(), 370.0),
  ];
  for (glyph, label, from_bottom) in actions {
    let y = svg.up(from_bottom);
    svg.circle((RAIL_X, y), 50.0, "rgba(40,40,40,0.55)");
    svg.glyph_at(glyph, (RAIL_X, y), 56.0, WHITE, false);
    svg.text(
      label,
      (RAIL_X, y + 88.0),
      25.0,
      500,
      WHITE,
      Anchor::Middle,
      true,
    );
  }
  // The sound's picture at the foot of the actions.
  let sound = svg.up(225.0);
  svg.rect((RAIL_X - 38.0, sound - 38.0, 76.0, 76.0), 14.0, WHITE);
  svg.rect((RAIL_X - 33.0, sound - 33.0, 66.0, 66.0), 11.0, "#6b6f78");

  // The channel and its button, the title, then the sound.
  let channel = svg.up(345.0);
  svg.avatar((62.0, channel), 32.0);
  svg.text(
    &words.handle,
    (110.0, channel + 11.0),
    31.0,
    600,
    WHITE,
    Anchor::Start,
    true,
  );
  let button = (
    110.0 + words.handle.chars().count() as f64 * 17.5 + 18.0,
    channel - 30.0,
  );
  svg.rect((button.0, button.1, 168.0, 60.0), 30.0, WHITE);
  svg.text(
    &words.subscribe,
    (button.0 + 84.0, button.1 + 40.0),
    27.0,
    600,
    "#0f0f0f",
    Anchor::Middle,
    false,
  );
  svg.text(
    &words.caption,
    (32.0, svg.up(268.0)),
    31.0,
    400,
    WHITE,
    Anchor::Start,
    true,
  );
  svg.glyph_at(&MUSIC_NOTE, (50.0, svg.up(214.0)), 30.0, WHITE, true);
  svg.text(
    &words.original_sound,
    (76.0, svg.up(204.0)),
    27.0,
    400,
    WHITE,
    Anchor::Start,
    true,
  );

  svg.rect(
    (0.0, svg.up(TAB_BAR + 4.0), WIDTH, 4.0),
    0.0,
    "rgba(255,255,255,0.3)",
  );
  svg.rect(
    (0.0, svg.up(TAB_BAR + 4.0), WIDTH * 0.42, 4.0),
    0.0,
    "#ff0033",
  );
  let bar = tab_bar(svg, TAB_BAR, "#0f0f0f");
  let tabs = [
    (&YOUTUBE_HOME, Some(&words.home)),
    (&SHORTS_TAB, Some(&words.shorts)),
    (&CREATE_CIRCLE, None),
    (&SUBSCRIPTIONS, Some(&words.subscriptions)),
    (&PROFILE, Some(&words.you)),
  ];
  for (index, (glyph, label)) in tabs.into_iter().enumerate() {
    let x = bar.slot(index);
    match label {
      Some(label) => {
        svg.glyph_at(glyph, (x, bar.icon), 46.0, WHITE, false);
        svg.text(
          label,
          (x, bar.label),
          20.0,
          400,
          WHITE,
          Anchor::Middle,
          false,
        );
      }
      None => svg.glyph_at(glyph, (x, bar.icon + 12.0), 76.0, WHITE, false),
    }
  }
}
