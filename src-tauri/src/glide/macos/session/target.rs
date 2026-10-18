// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use cidre::cg;
use core_graphics::geometry::CGPoint;
use tauri::AppHandle;

use super::super::{
  own_window::own_window_at,
  titlebar::{ax_titlebar_at, AxTitlebar},
  tween::WindowTarget,
};

/// What a gesture at this anchor would carry: the window as something the tween
/// can drive, the frame it was read at, and the process whose icon the preview
/// should show. One of Screenwide's own windows is resolved natively and names
/// this very process; everything else comes from the Accessibility hit test,
/// which rejects our own windows outright.
pub(in crate::glide::macos) fn target_at(
  app: &AppHandle,
  anchor: CGPoint,
) -> Option<(WindowTarget, cg::Rect, Option<u32>)> {
  match ax_titlebar_at(anchor) {
    AxTitlebar::Titlebar(window, frame) => {
      // The owning process, read before the target takes the element over, so
      // the preview can be told which application it is carrying.
      let pid = window.pid().ok().and_then(|pid| u32::try_from(pid).ok());
      Some((WindowTarget::new(window), frame, pid))
    }
    // Our own pid under the cursor: only now are the native resolver's
    // main-thread hops paid, and never on samples over other applications.
    AxTitlebar::OwnProcess => own_window_at(app, anchor)
      .map(|(window, frame)| (WindowTarget::own(window), frame, Some(std::process::id()))),
    AxTitlebar::Miss => None,
  }
}
