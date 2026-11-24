// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The phone a short video is previewed on. Every current phone but the
//! iPhone SE is taller than 9:16, and the apps zoom a 9:16 video until it
//! fills the screen, cutting its sides; they lay their buttons against the
//! phone's edges, not the video's. The phones are measured from their
//! makers' panel sizes; the system's own bars from the safe areas each
//! platform lays apps out in.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A phone the video is previewed on.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum Phone {
  #[serde(rename = "iphone-17-pro")]
  Iphone17Pro,
  #[serde(rename = "iphone-17-pro-max")]
  Iphone17ProMax,
  IphoneSe,
  #[serde(rename = "pixel-10")]
  Pixel10,
  GalaxyS26,
  GalaxyFoldCover,
}

/// The phone a format is previewed on. An object rather than the bare phone,
/// as projects store it: older ones also hold a `fit`, from when the video
/// could be shown whole with bars, which reading them now leaves out.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PhoneView {
  pub phone: Phone,
}

/// What the system draws round an app on a phone.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum SystemBars {
  /// The clock beside the Dynamic Island, the home indicator at the foot.
  Island,
  /// The clock above the screen's top edge and a home button below it.
  HomeButton,
  /// The clock beside a punch-hole camera, the gesture pill at the foot.
  PunchHole,
}

impl Phone {
  /// The panel's width and height in pixels, as its maker gives them.
  pub(crate) fn panel(self) -> (f64, f64) {
    match self {
      Self::Iphone17Pro => (1206.0, 2622.0),
      Self::Iphone17ProMax => (1320.0, 2868.0),
      Self::IphoneSe => (750.0, 1334.0),
      Self::Pixel10 => (1080.0, 2424.0),
      Self::GalaxyS26 => (1080.0, 2340.0),
      Self::GalaxyFoldCover => (968.0, 2376.0),
    }
  }

  pub(crate) fn bars(self) -> SystemBars {
    match self {
      Self::Iphone17Pro | Self::Iphone17ProMax => SystemBars::Island,
      Self::IphoneSe => SystemBars::HomeButton,
      Self::Pixel10 | Self::GalaxyS26 | Self::GalaxyFoldCover => SystemBars::PunchHole,
    }
  }

  /// The room the system keeps at the top and at the foot of the screen, as
  /// shares of its height: the status bar, and the home indicator or
  /// gesture pill. iPhone's from its safe area (62 and 34 points of 874 or
  /// 956); Android's from its 24 and 16 dp bars over a 923 dp screen.
  pub(crate) fn insets(self) -> (f64, f64) {
    match self {
      Self::Iphone17Pro => (62.0 / 874.0, 34.0 / 874.0),
      Self::Iphone17ProMax => (62.0 / 956.0, 34.0 / 956.0),
      Self::IphoneSe => (20.0 / 667.0, 0.0),
      Self::Pixel10 | Self::GalaxyS26 | Self::GalaxyFoldCover => (36.0 / 923.0, 20.0 / 923.0),
    }
  }

  /// How round the screen's corners are, as a share of its width.
  pub(crate) fn corner(self) -> f64 {
    match self {
      Self::Iphone17Pro | Self::Iphone17ProMax => 0.14,
      Self::IphoneSe => 0.0,
      Self::Pixel10 | Self::GalaxyS26 => 0.1,
      Self::GalaxyFoldCover => 0.08,
    }
  }
}

/// A rectangle in the canvas's pixels, its corner where the canvas's is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
  pub x: f64,
  pub y: f64,
  pub width: f64,
  pub height: f64,
}

impl Rect {
  pub(crate) fn union(self, other: Self) -> Self {
    let x = self.x.min(other.x);
    let y = self.y.min(other.y);
    Self {
      x,
      y,
      width: (self.x + self.width).max(other.x + other.width) - x,
      height: (self.y + self.height).max(other.y + other.height) - y,
    }
  }
}

/// Where `view`'s screen lies over a canvas `canvas` pixels: the largest
/// screen of the phone's shape inside the canvas, centred, so the canvas
/// overhangs it where the app cuts the video.
pub fn screen_on(canvas: (f64, f64), view: PhoneView) -> Rect {
  let (panel_width, panel_height) = view.phone.panel();
  let shape = panel_width / panel_height;
  let (canvas_width, canvas_height) = canvas;
  let (width, height) = if canvas_height * shape <= canvas_width {
    (canvas_height * shape, canvas_height)
  } else {
    (canvas_width, canvas_width / shape)
  };
  Rect {
    x: (canvas_width - width) / 2.0,
    y: (canvas_height - height) / 2.0,
    width,
    height,
  }
}
