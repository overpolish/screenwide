// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The menu bar and Dock of a display, from what AppKit leaves free of them
//! (`visibleFrame`): the menu bar along the top, the Dock along whichever
//! other edge it sits on.

use objc2::MainThreadMarker;
use objc2_app_kit::NSScreen;
use tauri::AppHandle;

use super::{DisplayBars, Insets};

/// AppKit's screens are read on the main thread; a recording starts off it.
pub(super) fn display_bars(app: &AppHandle, monitor_id: u32) -> Option<DisplayBars> {
  if let Some(mtm) = MainThreadMarker::new() {
    return bars_of(mtm, monitor_id);
  }
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  app
    .run_on_main_thread(move || {
      let mtm = MainThreadMarker::new().expect("run on the main thread");
      let _ = sender.send(bars_of(mtm, monitor_id));
    })
    .ok()?;
  receiver.recv().ok().flatten()
}

fn bars_of(mtm: MainThreadMarker, monitor_id: u32) -> Option<DisplayBars> {
  let screen = NSScreen::screens(mtm)
    .iter()
    .find(|screen| crate::app_windows::screen_display_id(screen) == Some(monitor_id))?;
  // Both in AppKit's coordinates, which count up from the bottom.
  let frame = screen.frame();
  let visible = screen.visibleFrame();
  let top = (frame.origin.y + frame.size.height) - (visible.origin.y + visible.size.height);
  Some(DisplayBars {
    width: frame.size.width,
    height: frame.size.height,
    menu_bar: Insets {
      top,
      ..Insets::default()
    },
    dock: Insets {
      top: 0.0,
      bottom: visible.origin.y - frame.origin.y,
      left: visible.origin.x - frame.origin.x,
      right: (frame.origin.x + frame.size.width) - (visible.origin.x + visible.size.width),
    },
  })
}
