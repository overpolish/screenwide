// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What a preview shows, and what drives it. A tool's scene is driven by the
//! page's pointer, sent as it moves; a recording's pointer is scripted up
//! front, as a recording's cursor is known whole before its zooms are
//! planned.

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod clock;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod content;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod driver;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod render;
#[cfg(any(target_os = "macos", target_os = "windows"))]
mod script;
#[cfg(all(test, any(target_os = "macos", target_os = "windows")))]
mod tests;

use serde::Deserialize;
use ts_rs::TS;

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use render::{Moment, SceneRenderer};

/// What a preview shows: one of the tools at work on the page's picture, or a
/// recording of the page's pictures composed by the editor's compositor.
#[derive(Clone, Debug, Deserialize, PartialEq, TS)]
#[serde(
  tag = "kind",
  rename_all = "camelCase",
  rename_all_fields = "camelCase"
)]
#[ts(export)]
pub enum PreviewScene {
  /// The Ruler measuring the picture.
  Ruler,
  /// OCR reading the picture.
  TextRecognition,
  /// Annotate drawing on glass over the page's own picture, in `color`
  /// (`#rrggbb`), its annotations outlasting the session where `keep`. Its
  /// counter is `counter_size` points across with its tail at
  /// `counter_angle`, as the settings set them: the page's pointer drops
  /// the counter where its tail reaches what it marks, so it has to know.
  Annotate {
    color: String,
    keep: bool,
    counter_size: f64,
    counter_angle: f64,
  },
  /// The picture recorded with the scripted pointer, zoomed where `on` as
  /// auto zoom plans it.
  AutoZoom { on: bool, script: PointerScript },
  /// The picture recorded with the scripted pointer at `fps` frames a
  /// second.
  FrameRate { fps: u32, script: PointerScript },
  /// The screen, the first picture, recorded with Screenwide's windows, the
  /// second, in it where `included`.
  IncludeScreenwide { included: bool },
}

impl PreviewScene {
  /// A tool's scene, which the app's own tool renderers draw; the others
  /// are recordings this crate composes.
  pub fn is_tool(&self) -> bool {
    matches!(
      self,
      Self::Ruler | Self::TextRecognition | Self::Annotate { .. }
    )
  }

  /// Whether the drawing sits over the page rather than under it. Ruler and
  /// OCR draw their whole screen and show through a transparent box, so the
  /// page can paint over them; Annotate draws on glass over the page's own
  /// picture, and a recording covers its box whole.
  pub fn over_page(&self) -> bool {
    !matches!(self, Self::Ruler | Self::TextRecognition)
  }

  /// Whether `other` is the same scene with other values, which a running
  /// preview can take without reopening.
  pub fn same_kind(&self, other: &Self) -> bool {
    std::mem::discriminant(self) == std::mem::discriminant(other)
  }
}

/// A pointer played on a loop, in the first picture's points, and the
/// moments typing is heard, as a recording's keyboard track tells them.
#[derive(Clone, Debug, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PointerScript {
  /// How long one pass takes, in milliseconds.
  pub duration_ms: u32,
  /// Where the pointer is at moments in the pass, in order. It glides
  /// between them, and is pressed from a key that says so until one that
  /// does not.
  pub keys: Vec<PointerKey>,
  /// When a key is typed, in milliseconds, in order: auto zoom holds on
  /// where typing goes.
  #[serde(default)]
  pub typing_ms: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PointerKey {
  pub at_ms: u32,
  pub x: f64,
  pub y: f64,
  pub pressed: bool,
}

/// The page's pointer for a tool's scene.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum PreviewInput {
  Hover,
  Down,
  Drag,
  Up,
  /// Back to how the tool starts: no measurements, no area chosen, nothing
  /// drawn.
  Reset,
  /// Annotate: what the next press draws.
  PickArrow,
  PickCounter,
  /// Annotate: the session ends, and the next starts.
  Exit,
  Enter,
}
