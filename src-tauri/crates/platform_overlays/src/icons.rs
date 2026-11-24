// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The platforms' interface glyphs, each on a 24-unit square as the apps draw
//! them: TikTok's and Shorts' filled, Instagram's outlined. Path data only;
//! [`super::svg`] places, sizes and paints them.

/// A glyph's outline, filled or stroked.
pub(crate) enum Glyph {
  /// Filled with the colour it is drawn in.
  Fill(&'static str),
  /// Stroked 2 units wide, round joins and caps, with no fill.
  Stroke(&'static str),
  /// Several parts, each filled or stroked.
  Parts(&'static [Glyph]),
}

use Glyph::{Fill, Parts, Stroke};

pub(crate) const HEART: Glyph = Fill(
  "M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z",
);

/// TikTok's comment: a round bubble whose three dots are cut out of it.
pub(crate) const TIKTOK_COMMENT: Glyph = Fill(
  "M12 2.6C6.5 2.6 2 6.3 2 10.9c0 2.5 1.3 4.7 3.4 6.2l-.6 3.9c-.1.5.4.9.9.6l4.3-2.4c.6.1 1.3.1 2 .1 5.5 0 10-3.7 10-8.3S17.5 2.6 12 2.6zM7.6 12.3a1.4 1.4 0 1 1 0-2.8 1.4 1.4 0 0 1 0 2.8zm4.4 0a1.4 1.4 0 1 1 0-2.8 1.4 1.4 0 0 1 0 2.8zm4.4 0a1.4 1.4 0 1 1 0-2.8 1.4 1.4 0 0 1 0 2.8z",
);

pub(crate) const BOOKMARK: Glyph = Fill(
  "M6.5 2h11A1.5 1.5 0 0 1 19 3.5v17.3c0 .8-.9 1.3-1.6.8L12 17.8l-5.4 3.8c-.7.5-1.6 0-1.6-.8V3.5A1.5 1.5 0 0 1 6.5 2z",
);

/// TikTok's share: an arrow bent back on itself, pointing right.
pub(crate) const TIKTOK_SHARE: Glyph =
  Fill("M13.4 3.6c0-.6.7-.9 1.1-.5l7.7 7.5c.4.4.4 1 0 1.4l-7.7 7.5c-.4.4-1.1.1-1.1-.5v-3.9C7.6 15.1 4.4 16.9 2.5 20.4c-.2.4-.8.2-.8-.2.4-6.6 4.5-11.2 11.7-11.8z");

pub(crate) const SEARCH: Glyph =
  Stroke("M10.8 18.1a7.3 7.3 0 1 0 0-14.6 7.3 7.3 0 0 0 0 14.6zM16.2 16.2l4.8 4.8");

pub(crate) const MUSIC_NOTE: Glyph =
  Fill("M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z");

pub(crate) const TIKTOK_LIVE: Glyph = Parts(&[
  Stroke("M4.5 7h15a1.5 1.5 0 0 1 1.5 1.5v10a1.5 1.5 0 0 1-1.5 1.5h-15A1.5 1.5 0 0 1 3 18.5v-10A1.5 1.5 0 0 1 4.5 7zM8 3l4 4 4-4"),
  Fill("M6.6 10.6h1.1v4.2h2.1v.9H6.6zM10.7 10.6h1.1v5.1h-1.1zM12.4 10.6h1.2l1 3.5 1-3.5h1.2l-1.6 5.1h-1.2zM17.3 10.6h3v.9h-1.9v1.2h1.8v.9h-1.8v1.2h1.9v.9h-3z"),
]);

pub(crate) const TIKTOK_HOME: Glyph =
  Fill("M3 10.2 11.3 3.3c.4-.3 1-.3 1.4 0L21 10.2V20a1.5 1.5 0 0 1-1.5 1.5H15v-6.2h-6v6.2H4.5A1.5 1.5 0 0 1 3 20z");

pub(crate) const FRIENDS: Glyph = Stroke(
  "M9 11.2a3.8 3.8 0 1 0 0-7.6 3.8 3.8 0 0 0 0 7.6zM2.2 20.4c.6-3.6 3.4-6 6.8-6s6.2 2.4 6.8 6M16.2 4a3.6 3.6 0 0 1 0 7M18.6 14.6c1.7.9 2.9 2.9 3.2 5.4",
);

pub(crate) const INBOX: Glyph = Stroke(
  "M4.5 3.5h15A1.5 1.5 0 0 1 21 5v11a1.5 1.5 0 0 1-1.5 1.5h-4.2L12 20.8l-3.3-3.3H4.5A1.5 1.5 0 0 1 3 16V5a1.5 1.5 0 0 1 1.5-1.5zM8 10.5h8",
);

pub(crate) const PROFILE: Glyph =
  Stroke("M12 11.5a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM4 21a8 8 0 0 1 16 0");

pub(crate) const THUMB_UP: Glyph = Fill(
  "M1 21h4V9H1v12zm22-11c0-1.1-.9-2-2-2h-6.31l.95-4.57.03-.32c0-.41-.17-.79-.44-1.06L14.17 1 7.59 7.59C7.22 7.95 7 8.45 7 9v10c0 1.1.9 2 2 2h9c.83 0 1.54-.5 1.84-1.22l3.02-7.05c.09-.23.14-.47.14-.73v-2z",
);

/// The silhouette in a placeholder avatar.
pub(crate) const PROFILE_SOLID: Glyph = Fill("M12 11.5a4.2 4.2 0 1 0 0-8.4 4.2 4.2 0 0 0 0 8.4zM3.8 21c.4-4.4 3.9-7.5 8.2-7.5s7.8 3.1 8.2 7.5z");

pub(crate) const THUMB_DOWN: Glyph = Fill(
  "M15 3H6c-.83 0-1.54.5-1.84 1.22l-3.02 7.05c-.09.23-.14.47-.14.73v2c0 1.1.9 2 2 2h6.31l-.95 4.57-.03.32c0 .41.17.79.44 1.06L9.83 23l6.59-6.59c.36-.36.58-.86.58-1.41V5c0-1.1-.9-2-2-2zm4 0v12h4V3h-4z",
);

/// Shorts' comment: a squared bubble with its tail at the bottom right.
pub(crate) const SHORTS_COMMENT: Glyph =
  Fill("M5 3h14a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2h-1v4.2c0 .5-.6.7-.9.4L12.6 17H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z");

pub(crate) const SHORTS_SHARE: Glyph =
  Fill("M14 9V5l7 7-7 7v-4.1c-5 0-8.5 1.6-11 5.1 1-5 4-10 11-11z");

pub(crate) const REMIX: Glyph = Fill(
  "M12 6v3l4-4-4-4v3c-4.42 0-8 3.58-8 8 0 1.57.46 3.03 1.24 4.26L6.7 14.8c-.45-.83-.7-1.79-.7-2.8 0-3.31 2.69-6 6-6zm6.76 1.74L17.3 9.2c.44.84.7 1.79.7 2.8 0 3.31-2.69 6-6 6v-3l-4 4 4 4v-3c4.42 0 8-3.58 8-8 0-1.57-.46-3.03-1.24-4.26z",
);

pub(crate) const MORE_VERTICAL: Glyph = Fill(
  "M12 7.2a2 2 0 1 0 0-4 2 2 0 0 0 0 4zm0 6.8a2 2 0 1 0 0-4 2 2 0 0 0 0 4zm0 6.8a2 2 0 1 0 0-4 2 2 0 0 0 0 4z",
);

pub(crate) const MORE_HORIZONTAL: Glyph = Fill(
  "M5.2 13.8a1.8 1.8 0 1 0 0-3.6 1.8 1.8 0 0 0 0 3.6zm6.8 0a1.8 1.8 0 1 0 0-3.6 1.8 1.8 0 0 0 0 3.6zm6.8 0a1.8 1.8 0 1 0 0-3.6 1.8 1.8 0 0 0 0 3.6z",
);

pub(crate) const YOUTUBE_HOME: Glyph =
  Stroke("M4 10.3 12 4l8 6.3V20a1 1 0 0 1-1 1h-4.5v-6h-5v6H5a1 1 0 0 1-1-1z");

/// The Shorts tab: its rounded, slanted frame with the play mark in it.
pub(crate) const SHORTS_TAB: Glyph = Parts(&[
  Fill("M17.8 1.9c2.3 1.2 3.2 4 2 6.3-.4.8-1.1 1.5-1.9 1.9l-1.3.7c1.6.1 3 1.1 3.6 2.6.9 2.4-.2 5-2.5 6.1l-8.3 4.3c-2.3 1.2-5.1.3-6.3-2-1.2-2.3-.3-5.1 2-6.3l1.3-.7c-1.6-.1-3-1.1-3.6-2.6-.9-2.4.2-5 2.5-6.1l8.3-4.3c1.3-.6 2.9-.6 4.2.1z"),
  Fill("M10 8.8v6.4l5.4-3.2z"),
]);

pub(crate) const CREATE_CIRCLE: Glyph =
  Stroke("M12 21.5a9.5 9.5 0 1 0 0-19 9.5 9.5 0 0 0 0 19zM12 7.5v9M7.5 12h9");

pub(crate) const SUBSCRIPTIONS: Glyph = Stroke(
  "M5 3.5h14M3.5 7h17M4.5 10.5h15a1.5 1.5 0 0 1 1.5 1.5v7.5a1.5 1.5 0 0 1-1.5 1.5h-15A1.5 1.5 0 0 1 3 19.5V12a1.5 1.5 0 0 1 1.5-1.5zM10.2 13.3v4.4l3.8-2.2z",
);

pub(crate) const INSTAGRAM_HEART: Glyph = Stroke(
  "M12 20.2c-.4-.2-2.3-1.7-4.4-3.6C5.3 14.5 3 12.6 3 9.4A4.7 4.7 0 0 1 7.6 4.6c1.8 0 3.3 1 4.4 2.6 1.1-1.6 2.6-2.6 4.4-2.6A4.7 4.7 0 0 1 21 9.4c0 3.2-2.3 5.1-4.6 7.2-2.1 1.9-4 3.4-4.4 3.6z",
);

pub(crate) const INSTAGRAM_COMMENT: Glyph =
  Stroke("M20.656 17.008a9.993 9.993 0 1 0-3.59 3.615L22 22Z");

pub(crate) const INSTAGRAM_SEND: Glyph =
  Stroke("M22 3 9.218 10.083M11.698 20.334 22 3.001H2l7.218 7.083z");

pub(crate) const INSTAGRAM_CAMERA: Glyph = Stroke(
  "M5 6.5h2.8l1.4-2.2h5.6l1.4 2.2H19a2 2 0 0 1 2 2V18a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8.5a2 2 0 0 1 2-2zM12 16.6a3.6 3.6 0 1 0 0-7.2 3.6 3.6 0 0 0 0 7.2z",
);

pub(crate) const INSTAGRAM_HOME: Glyph = Stroke(
  "M9.005 16.545a2.997 2.997 0 0 1 2.997-2.997A2.997 2.997 0 0 1 15 16.545V22h7V11.543L12 2 2 11.543V22h7.005Z",
);

pub(crate) const INSTAGRAM_NEW: Glyph = Stroke(
  "M8 2.5h8A5.5 5.5 0 0 1 21.5 8v8a5.5 5.5 0 0 1-5.5 5.5H8A5.5 5.5 0 0 1 2.5 16V8A5.5 5.5 0 0 1 8 2.5zM12 7.5v9M7.5 12h9",
);

pub(crate) const INSTAGRAM_REELS: Glyph = Parts(&[
  Stroke("M7.5 2.5h9a5 5 0 0 1 5 5v9a5 5 0 0 1-5 5h-9a5 5 0 0 1-5-5v-9a5 5 0 0 1 5-5zM2.5 7.5h19M8 2.7l3 4.8M13.8 2.7l3 4.8"),
  Fill("M10 11.2v6.2l5.3-3.1z"),
]);

pub(crate) const CHEVRON_DOWN: Glyph = Stroke("M6 9l6 6 6-6");
