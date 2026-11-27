// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

mod lock;

use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Power::{
  PowerClearRequest, PowerCreateRequest, PowerRequestDisplayRequired, PowerRequestSystemRequired,
  PowerSetRequest, POWER_REQUEST_TYPE,
};
use windows::Win32::System::SystemServices::POWER_REQUEST_CONTEXT_VERSION;
use windows::Win32::System::Threading::{
  POWER_REQUEST_CONTEXT_SIMPLE_STRING, REASON_CONTEXT, REASON_CONTEXT_0,
};

pub(super) use lock::LockWatch;

/// A power request, which belongs to no thread, unlike
/// `SetThreadExecutionState`, so it may be dropped on another thread than the
/// one that made it.
pub(super) struct Hold {
  request: HANDLE,
  kinds: &'static [POWER_REQUEST_TYPE],
}

// SAFETY: the handle names a kernel object, usable from any thread.
unsafe impl Send for Hold {}

impl Hold {
  /// The display turns off as usual.
  pub(super) fn system(reason: &str) -> Result<Self, String> {
    Self::begin(reason, &[PowerRequestSystemRequired])
  }

  /// A display request alone does not keep the system awake, so it takes
  /// both.
  pub(super) fn display(reason: &str) -> Result<Self, String> {
    Self::begin(
      reason,
      &[PowerRequestSystemRequired, PowerRequestDisplayRequired],
    )
  }

  fn begin(reason: &str, kinds: &'static [POWER_REQUEST_TYPE]) -> Result<Self, String> {
    let mut text: Vec<u16> = reason.encode_utf16().chain([0]).collect();
    let context = REASON_CONTEXT {
      Version: POWER_REQUEST_CONTEXT_VERSION,
      Flags: POWER_REQUEST_CONTEXT_SIMPLE_STRING,
      Reason: REASON_CONTEXT_0 {
        SimpleReasonString: PWSTR(text.as_mut_ptr()),
      },
    };
    // SAFETY: `context` and the NUL-terminated string it points to outlive
    // the call, which copies the reason.
    let request = unsafe { PowerCreateRequest(&context) }
      .map(|request| Self { request, kinds })
      .map_err(|error| format!("Could not create a power request for {reason}: {error}"))?;
    for &kind in kinds {
      // SAFETY: the handle was just created; dropping `request` clears what
      // was set and closes it.
      unsafe { PowerSetRequest(request.request, kind) }
        .map_err(|error| format!("Could not keep the system awake for {reason}: {error}"))?;
    }
    Ok(request)
  }
}

impl Drop for Hold {
  fn drop(&mut self) {
    // SAFETY: the handle came from `PowerCreateRequest` and is closed once.
    // Clearing a request never set fails harmlessly.
    unsafe {
      for &kind in self.kinds {
        let _ = PowerClearRequest(self.request, kind);
      }
      let _ = CloseHandle(self.request);
    }
  }
}
