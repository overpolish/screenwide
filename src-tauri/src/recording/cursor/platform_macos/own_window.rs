// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::mpsc;
use std::time::Duration;

use dispatch2::DispatchQueue;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSScreen, NSWindow};
use objc2_foundation::NSPoint;

/// How long a press waits for AppKit to say which window it landed on. A
/// main thread too busy to answer leaves the press in: a click on Screenwide
/// kept by mistake costs less than a click on the work lost.
const ANSWER_TIMEOUT: Duration = Duration::from_millis(100);

/// Whether a press at a point, in global top-left points, lands on one of
/// Screenwide's own windows. AppKit's hit test passes through windows that
/// ignore the mouse, as the press does, but answers only on the main thread;
/// the event tap runs on its own thread, so it asks and waits.
pub(in super::super) fn on_own_window(x: f64, y: f64) -> bool {
  let (answer, verdict) = mpsc::sync_channel(1);
  DispatchQueue::main().exec_async(move || {
    let Some(mtm) = MainThreadMarker::new() else {
      return;
    };
    let Some(primary) = NSScreen::screens(mtm).firstObject() else {
      return;
    };
    // AppKit measures the desktop up from the primary display's bottom edge;
    // the event, down from its top edge.
    let point = NSPoint::new(x, primary.frame().size.height - y);
    let number = NSWindow::windowNumberAtPoint_belowWindowWithWindowNumber(point, 0, mtm);
    let own = NSApplication::sharedApplication(mtm)
      .windowWithWindowNumber(number)
      .is_some();
    let _ = answer.send(own);
  });
  verdict.recv_timeout(ANSWER_TIMEOUT).unwrap_or(false)
}
