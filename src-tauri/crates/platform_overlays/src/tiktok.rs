// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! TikTok's For You feed over a video: the feeds' tabs at the top, the
//! actions down the right, who posted it, its caption and sound at the
//! bottom left, the progress line and the tab bar.

use super::icons::{
  BOOKMARK, FRIENDS, HEART, INBOX, MUSIC_NOTE, PROFILE, SEARCH, TIKTOK_COMMENT, TIKTOK_HOME,
  TIKTOK_LIVE, TIKTOK_SHARE,
};
use super::svg::{Anchor, Svg, WIDTH};
use super::{tab_bar, Words};

const WHITE: &str = "#ffffff";
const SOFT: &str = "rgba(255,255,255,0.62)";
const RAIL_X: f64 = 1000.0;
const TAB_BAR: f64 = 140.0;

pub(crate) fn draw(svg: &mut Svg, words: &Words) {
  svg.bottom_shade(760.0, TAB_BAR);
  // The feeds' tabs, LIVE before them and search after.
  svg.glyph_at(&TIKTOK_LIVE, (64.0, svg.down(112.0)), 62.0, WHITE, true);
  svg.text(
    &words.following,
    (440.0, svg.down(124.0)),
    36.0,
    600,
    SOFT,
    Anchor::Middle,
    true,
  );
  svg.text(
    &words.for_you,
    (640.0, svg.down(124.0)),
    36.0,
    700,
    WHITE,
    Anchor::Middle,
    true,
  );
  svg.rect((612.0, svg.down(146.0), 56.0, 6.0), 3.0, WHITE);
  svg.glyph_at(&SEARCH, (1018.0, svg.down(112.0)), 60.0, WHITE, true);

  // The actions, the sound's disc at their foot.
  let avatar = svg.up(1040.0);
  svg.circle((RAIL_X, avatar), 52.0, WHITE);
  svg.avatar((RAIL_X, avatar), 48.0);
  svg.circle((RAIL_X, avatar + 50.0), 20.0, "#fe2c55");
  svg.rect((RAIL_X - 9.0, avatar + 48.0, 18.0, 4.0), 2.0, WHITE);
  svg.rect((RAIL_X - 2.0, avatar + 41.0, 4.0, 18.0), 2.0, WHITE);
  let actions = [
    (&HEART, "248.6K", 870.0),
    (&TIKTOK_COMMENT, "1,904", 710.0),
    (&BOOKMARK, "31.2K", 550.0),
    (&TIKTOK_SHARE, "12.7K", 390.0),
  ];
  for (glyph, count, from_bottom) in actions {
    let y = svg.up(from_bottom);
    svg.glyph_at(glyph, (RAIL_X, y), 84.0, WHITE, true);
    svg.text(
      count,
      (RAIL_X, y + 76.0),
      26.0,
      600,
      WHITE,
      Anchor::Middle,
      true,
    );
  }
  let disc = (RAIL_X, svg.up(200.0));
  svg.circle(disc, 44.0, "#2b2b2f");
  svg.ring(disc, 36.0, "#3f3f45", 6.0);
  svg.avatar(disc, 22.0);

  // Who posted it, what they wrote and the sound it uses.
  svg.text(
    &words.handle,
    (32.0, svg.up(330.0)),
    34.0,
    700,
    WHITE,
    Anchor::Start,
    true,
  );
  svg.text(
    &words.caption,
    (32.0, svg.up(276.0)),
    31.0,
    400,
    WHITE,
    Anchor::Start,
    true,
  );
  svg.glyph_at(&MUSIC_NOTE, (50.0, svg.up(224.0)), 34.0, WHITE, true);
  svg.text(
    &words.sound,
    (78.0, svg.up(213.0)),
    29.0,
    400,
    WHITE,
    Anchor::Start,
    true,
  );

  // How far through it is, then the tab bar.
  svg.rect(
    (0.0, svg.up(TAB_BAR + 4.0), WIDTH, 4.0),
    0.0,
    "rgba(255,255,255,0.28)",
  );
  svg.rect((0.0, svg.up(TAB_BAR + 4.0), WIDTH * 0.34, 4.0), 0.0, WHITE);
  let bar = tab_bar(svg, TAB_BAR, "#000000");
  let tabs = [
    (&TIKTOK_HOME, &words.home, WHITE),
    (&FRIENDS, &words.friends, SOFT),
    (&INBOX, &words.inbox, SOFT),
    (&PROFILE, &words.profile, SOFT),
  ];
  for (index, (glyph, label, color)) in tabs.into_iter().enumerate() {
    let x = bar.slot(if index < 2 { index } else { index + 1 });
    svg.glyph_at(glyph, (x, bar.icon), 50.0, color, false);
    svg.text(
      label,
      (x, bar.label),
      21.0,
      500,
      color,
      Anchor::Middle,
      false,
    );
  }
  // The middle button: white over a cyan and a pink edge.
  let (x, y) = (bar.slot(2), bar.icon + 6.0);
  svg.rect((x - 46.0, y - 28.0, 84.0, 56.0), 14.0, "#25f4ee");
  svg.rect((x - 38.0, y - 28.0, 84.0, 56.0), 14.0, "#fe2c55");
  svg.rect((x - 42.0, y - 28.0, 84.0, 56.0), 14.0, WHITE);
  svg.rect((x - 14.0, y - 2.5, 28.0, 5.0), 2.5, "#000000");
  svg.rect((x - 2.5, y - 14.0, 5.0, 28.0), 2.5, "#000000");
}
