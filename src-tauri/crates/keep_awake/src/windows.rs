// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::time::Duration;

use windows::core::PWSTR;
use windows::Win32::System::Power::{
  SetThreadExecutionState, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED,
};
use windows::Win32::System::RemoteDesktop::{
  WTSFreeMemory, WTSQuerySessionInformationW, WTSSessionInfoEx, WTSINFOEXW,
  WTS_CURRENT_SERVER_HANDLE, WTS_CURRENT_SESSION, WTS_SESSIONSTATE_LOCK,
};
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::UI::Input::KeyboardAndMouse::{
  GetLastInputInfo, SendInput, INPUT, INPUT_0, INPUT_MOUSE, LASTINPUTINFO, MOUSEEVENTF_MOVE,
  MOUSEINPUT,
};

use super::EVENT_TAG;

pub(super) fn permitted() -> bool {
  true
}

pub(super) fn idle() -> Duration {
  let mut info = LASTINPUTINFO {
    cbSize: size_of::<LASTINPUTINFO>() as u32,
    dwTime: 0,
  };
  // SAFETY: `info` is sized as the call requires.
  if !unsafe { GetLastInputInfo(&mut info) }.as_bool() {
    return Duration::ZERO;
  }
  // Both count milliseconds since boot and wrap together every 49.7 days.
  // SAFETY: takes nothing.
  let now = unsafe { GetTickCount() };
  Duration::from_millis(u64::from(now.wrapping_sub(info.dwTime)))
}

/// Whether this session is unlocked. Input cannot reach a locked session's
/// desktop anyway; asking keeps a lock from costing a wake-up for nothing.
pub(super) fn screen_in_use() -> bool {
  let mut buffer = PWSTR::null();
  let mut bytes = 0;
  // SAFETY: both out-pointers are valid; the buffer is freed below.
  let asked = unsafe {
    WTSQuerySessionInformationW(
      Some(WTS_CURRENT_SERVER_HANDLE),
      WTS_CURRENT_SESSION,
      WTSSessionInfoEx,
      &mut buffer,
      &mut bytes,
    )
  };
  if asked.is_err() || buffer.is_null() {
    return true;
  }
  // SAFETY: `WTSSessionInfoEx` answers with a `WTSINFOEXW`, level 1 holding
  // the session's flags.
  let locked = unsafe {
    let info = &*buffer.0.cast::<WTSINFOEXW>();
    info.Level == 1 && info.Data.WTSInfoExLevel1.SessionFlags == WTS_SESSIONSTATE_LOCK as i32
  };
  // SAFETY: the buffer came from the query above and is freed once.
  unsafe { WTSFreeMemory(buffer.0.cast()) };
  !locked
}

/// Sends a mouse move of no distance, which the cursor does not follow, and
/// restarts the display-off and sleep timers. Windows drops the move without
/// saying so while an elevated app is in front (UIPI); the timers keep the
/// computer awake then, though chat apps may show the user away.
pub(super) fn nudge() {
  let input = INPUT {
    r#type: INPUT_MOUSE,
    Anonymous: INPUT_0 {
      mi: MOUSEINPUT {
        dx: 0,
        dy: 0,
        mouseData: 0,
        dwFlags: MOUSEEVENTF_MOVE,
        time: 0,
        dwExtraInfo: EVENT_TAG as usize,
      },
    },
  };
  // SAFETY: one fully initialised input, sized as the call requires.
  unsafe { SendInput(&[input], size_of::<INPUT>() as i32) };
  // Without `ES_CONTINUOUS` this holds nothing: it restarts the timers once,
  // as input would, and leaves no request in `powercfg /requests`.
  // SAFETY: takes only flags.
  unsafe { SetThreadExecutionState(ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED) };
}
