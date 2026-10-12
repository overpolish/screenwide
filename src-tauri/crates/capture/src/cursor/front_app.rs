// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which app is in front, polled beside the cursor so the auto zoom knows
//! when the work moved to another app. Apps are told apart by process id and
//! nothing about them is kept.

#![cfg_attr(not(any(target_os = "macos", target_os = "windows")), allow(dead_code))]

/// The process in front now, where the system says.
#[cfg(target_os = "macos")]
fn front_process() -> Option<u32> {
  // The workspace's front app follows the main run loop, which the app keeps
  // running, so it can be read from the recorder's own thread.
  let app = objc2_app_kit::NSWorkspace::sharedWorkspace().frontmostApplication()?;
  u32::try_from(app.processIdentifier()).ok()
}

#[cfg(target_os = "windows")]
fn front_process() -> Option<u32> {
  use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

  let window = unsafe { GetForegroundWindow() };
  if window.is_invalid() {
    return None;
  }
  let mut process = 0;
  unsafe { GetWindowThreadProcessId(window, Some(&mut process)) };
  (process != 0).then_some(process)
}

/// Notices the moments another app comes to the front.
#[derive(Clone, Copy)]
pub(super) struct FrontApp {
  own: u32,
  last: Option<u32>,
}

impl FrontApp {
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(super) fn new() -> Self {
    let mut front = Self::with_own(std::process::id());
    front.changed(front_process());
    front
  }

  fn with_own(own: u32) -> Self {
    Self { own, last: None }
  }

  /// Whether the app in front is another one than when this last looked.
  #[cfg(any(target_os = "macos", target_os = "windows"))]
  pub(super) fn switched(&mut self) -> bool {
    self.changed(front_process())
  }

  /// Screenwide's own windows coming forward, its recording bar or a
  /// countdown, neither count nor end the app before them: going back to
  /// that app is no switch. The first app seen is where the recording
  /// starts, not a switch either.
  fn changed(&mut self, front: Option<u32>) -> bool {
    let Some(front) = front.filter(|&front| front != self.own) else {
      return false;
    };
    let switched = self.last.is_some_and(|last| last != front);
    self.last = Some(front);
    switched
  }
}

#[cfg(test)]
mod tests {
  use super::FrontApp;

  const OWN: u32 = 1;

  #[test]
  fn only_another_app_coming_forward_is_a_switch() {
    let mut front = FrontApp::with_own(OWN);
    assert!(!front.changed(Some(10)), "where the recording starts");
    assert!(!front.changed(Some(10)));
    assert!(!front.changed(Some(OWN)), "Screenwide's own bar");
    assert!(
      !front.changed(Some(10)),
      "back from the bar to the same app"
    );
    assert!(!front.changed(None));
    assert!(front.changed(Some(20)));
    assert!(!front.changed(Some(20)));
  }

  #[test]
  fn an_app_reached_through_screenwide_is_a_switch() {
    let mut front = FrontApp::with_own(OWN);
    front.changed(Some(OWN));
    assert!(!front.changed(Some(10)), "the first app seen");
    front.changed(Some(OWN));
    assert!(front.changed(Some(20)));
  }
}
