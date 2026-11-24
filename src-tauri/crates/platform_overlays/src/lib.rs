// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The interface a short-video app lays over a video (TikTok's, YouTube
//! Shorts', Instagram Reels'), drawn over the editor's picture so what the
//! app will cover can be seen while editing, on the phone it is previewed
//! on where one is chosen. It is only ever drawn there: the compositor and
//! the export never see it.
//!
//! Each is written as SVG from the apps' own glyphs and measures, taken on a
//! 1080 by 1920 frame, and drawn to a picture as large as the canvas is on
//! screen, so it is sharp at any zoom.

mod icons;
mod phone;
mod reels;
mod scene;
mod shorts;
mod svg;
mod tiktok;

use std::sync::{Arc, LazyLock};

use resvg::{tiny_skia, usvg};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use phone::{screen_on, Phone, PhoneView, Rect};
use svg::{Svg, WIDTH};

/// Whose interface is laid over the picture.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum PlatformOverlay {
  Tiktok,
  YoutubeShorts,
  InstagramReels,
}

/// What is laid over the canvas: an app's interface, a phone, or both.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct OverlayRequest {
  pub platform: Option<PlatformOverlay>,
  pub phone: Option<PhoneView>,
}

impl OverlayRequest {
  pub fn is_empty(self) -> bool {
    self.platform.is_none() && self.phone.is_none()
  }
}

/// An overlay drawn: premultiplied RGBA, row by row, and where its corner
/// lies from the canvas's, in the canvas's pixels; up and to the left of it
/// where a phone's bars reach past the canvas.
pub struct OverlayPicture {
  pub size: (u32, u32),
  pub origin: (f64, f64),
  pub pixels: Vec<u8>,
}

/// `request` drawn over a canvas `canvas` pixels wide and high, or `None`
/// for nothing to draw, a size too small to draw or a document the renderer
/// refuses.
pub fn draw(request: OverlayRequest, canvas: (u32, u32)) -> Option<OverlayPicture> {
  if request.is_empty() || canvas.0 < 8 || canvas.1 < 8 {
    return None;
  }
  let (document, bounds) = scene::document(request, (f64::from(canvas.0), f64::from(canvas.1)));
  let options = usvg::Options {
    fontdb: Arc::clone(&FONTS),
    ..Default::default()
  };
  let tree = usvg::Tree::from_str(&document, &options).ok()?;
  let size = (bounds.width.round() as u32, bounds.height.round() as u32);
  let mut pixmap = tiny_skia::Pixmap::new(size.0, size.1)?;
  resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
  Some(OverlayPicture {
    size,
    origin: (bounds.x, bounds.y),
    pixels: pixmap.take(),
  })
}

/// The computer's fonts, read once: the apps' words are set in its sans
/// serif.
static FONTS: LazyLock<Arc<usvg::fontdb::Database>> = LazyLock::new(|| {
  let mut fonts = usvg::fontdb::Database::new();
  fonts.load_system_fonts();
  Arc::new(fonts)
});

/// Reads the computer's fonts on a thread of their own, once, so the first
/// overlay drawn does not wait for them: reading them takes from a few
/// hundred milliseconds to a couple of seconds.
pub fn warm() {
  static STARTED: std::sync::Once = std::sync::Once::new();
  STARTED.call_once(|| {
    std::thread::spawn(|| LazyLock::force(&FONTS));
  });
}

/// The words the overlays show, in the app's language.
pub(crate) struct Words {
  caption: String,
  dislike: String,
  follow: String,
  following: String,
  for_you: String,
  friends: String,
  handle: String,
  handle_bare: String,
  home: String,
  inbox: String,
  likes: String,
  original_audio: String,
  original_sound: String,
  profile: String,
  reels: String,
  remix: String,
  share: String,
  shorts: String,
  sound: String,
  subscribe: String,
  subscriptions: String,
  you: String,
}

impl Words {
  fn new() -> Self {
    use screenwide_i18n::t;
    Self {
      caption: t!("editor-overlay-caption"),
      dislike: t!("editor-overlay-dislike"),
      follow: t!("editor-overlay-follow"),
      following: t!("editor-overlay-following"),
      for_you: t!("editor-overlay-for-you"),
      friends: t!("editor-overlay-friends"),
      handle: t!("editor-overlay-handle"),
      handle_bare: t!("editor-overlay-handle-bare"),
      home: t!("editor-overlay-home"),
      inbox: t!("editor-overlay-inbox"),
      likes: t!("editor-overlay-likes"),
      original_audio: t!("editor-overlay-original-audio"),
      original_sound: t!("editor-overlay-original-sound"),
      profile: t!("editor-overlay-profile"),
      reels: t!("editor-overlay-reels"),
      remix: t!("editor-overlay-remix"),
      share: t!("editor-overlay-share"),
      shorts: t!("editor-overlay-shorts"),
      sound: t!("editor-overlay-sound"),
      subscribe: t!("editor-overlay-subscribe"),
      subscriptions: t!("editor-overlay-subscriptions"),
      you: t!("editor-overlay-you"),
    }
  }
}

/// Where a tab bar `height` units tall at the bottom puts its five tabs.
pub(crate) struct TabBar {
  /// The middle of each tab's glyph, and the baseline of its label.
  icon: f64,
  label: f64,
}

impl TabBar {
  fn slot(&self, index: usize) -> f64 {
    WIDTH / 5.0 * (index as f64 + 0.5)
  }
}

/// Paints the tab bar's ground in `color`, reaching under the system's home
/// indicator to the screen's foot, and says where its tabs go.
pub(crate) fn tab_bar(svg: &mut Svg, height: f64, color: &str) -> TabBar {
  let top = svg.up(height);
  svg.rect((0.0, top, WIDTH, svg.foot() - top), 0.0, color);
  TabBar {
    icon: top + 46.0,
    label: top + 104.0,
  }
}

#[cfg(test)]
mod tests;
