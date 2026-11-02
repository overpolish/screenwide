// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The platform's place for a preview in its window: a GPU surface beside
//! the webview, placed, clipped and rounded to a box in the page.

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
use macos as platform;
#[cfg(target_os = "windows")]
use windows as platform;

pub use platform::HostWindow;

use crate::placement::PreviewPlacement;

/// Which side of the page a preview is drawn on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stacking {
  /// Under the page, showing through a transparent box.
  Below,
  /// Over the page, letting every event through.
  Above,
}

/// One preview's view or visuals.
pub(crate) struct Host {
  #[cfg(target_os = "macos")]
  slot: dispatch2::MainThreadBound<macos::Slot>,
  #[cfg(target_os = "windows")]
  slot: windows::Slot,
}

impl Host {
  /// Makes the preview's place in `window`, hidden, and the surface on it.
  pub(crate) fn open(
    window: &HostWindow,
    stacking: Stacking,
  ) -> Result<(Self, screenwide_gpu::surface::Surface), String> {
    let (slot, surface) = platform::create(window, stacking)?;
    Ok((Self { slot }, surface))
  }

  pub(crate) fn place(&self, placement: PreviewPlacement) {
    #[cfg(target_os = "macos")]
    dispatch2::run_on_main(|mtm| self.slot.get(mtm).place(placement));
    #[cfg(target_os = "windows")]
    self.slot.place(placement);
  }
}
