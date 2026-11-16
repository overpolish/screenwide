// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Whether the app in front runs as administrator, which Windows shields
//! from the keys of anything less privileged.

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::Threading::{
  GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

/// Whether the app in front runs elevated while Screenwide does not. One
/// Windows will not let Screenwide look into is taken as elevated: an app of
/// the same user, at the same level, can always be looked into.
pub(super) fn is_elevated_above_us() -> bool {
  // Safety: the window and process are only queried, and every handle opened
  // here is closed.
  unsafe {
    if is_elevated(GetCurrentProcess()).unwrap_or(false) {
      return false;
    }
    let mut process_id = 0;
    GetWindowThreadProcessId(GetForegroundWindow(), Some(&raw mut process_id));
    if process_id == 0 {
      return false;
    }
    let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) else {
      return true;
    };
    let theirs = is_elevated(process);
    let _ = CloseHandle(process);
    theirs.unwrap_or(true)
  }
}

/// # Safety
///
/// `process` must be a process handle open for limited queries.
unsafe fn is_elevated(process: HANDLE) -> Option<bool> {
  let mut token = HANDLE::default();
  unsafe { OpenProcessToken(process, TOKEN_QUERY, &raw mut token) }.ok()?;
  let mut elevation = TOKEN_ELEVATION::default();
  let mut length = 0;
  #[allow(clippy::cast_possible_truncation)]
  let size = size_of::<TOKEN_ELEVATION>() as u32;
  let result = unsafe {
    GetTokenInformation(
      token,
      TokenElevation,
      Some((&raw mut elevation).cast()),
      size,
      &raw mut length,
    )
  };
  let _ = unsafe { CloseHandle(token) };
  result.ok().map(|()| elevation.TokenIsElevated != 0)
}
