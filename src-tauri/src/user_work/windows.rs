// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{Mutex, PoisonError};

use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Power::{
  PowerClearRequest, PowerCreateRequest, PowerRequestSystemRequired, PowerSetRequest,
};
use windows::Win32::System::SystemServices::POWER_REQUEST_CONTEXT_VERSION;
use windows::Win32::System::Threading::{
  GetCurrentProcess, GetCurrentThread, ProcessPowerThrottling, SetProcessInformation,
  SetThreadInformation, ThreadPowerThrottling, POWER_REQUEST_CONTEXT_SIMPLE_STRING,
  PROCESS_POWER_THROTTLING_CURRENT_VERSION, PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
  PROCESS_POWER_THROTTLING_STATE, REASON_CONTEXT, REASON_CONTEXT_0,
  THREAD_POWER_THROTTLING_CURRENT_VERSION, THREAD_POWER_THROTTLING_EXECUTION_SPEED,
  THREAD_POWER_THROTTLING_STATE,
};

use super::Urgency;

/// Activities under way. Windows keeps one throttling setting per process, so
/// the first activity turns throttling off and the last hands it back.
static ACTIVITIES: Mutex<usize> = Mutex::new(0);

pub(super) struct Activity {
  _awake: Option<SystemRequired>,
}

impl Activity {
  pub(super) fn begin(reason: &str, keep_awake: bool) -> Self {
    let mut activities = ACTIVITIES.lock().unwrap_or_else(PoisonError::into_inner);
    if *activities == 0 {
      set_process_throttling(PROCESS_POWER_THROTTLING_EXECUTION_SPEED);
    }
    *activities += 1;
    Self {
      _awake: keep_awake.then(|| SystemRequired::hold(reason)).flatten(),
    }
  }
}

impl Drop for Activity {
  fn drop(&mut self) {
    let mut activities = ACTIVITIES.lock().unwrap_or_else(PoisonError::into_inner);
    *activities -= 1;
    if *activities == 0 {
      set_process_throttling(0);
    }
  }
}

/// Keeps the system from sleeping when idle while held; the display may still
/// turn off. A power request belongs to no thread, unlike
/// `SetThreadExecutionState`, so an activity may end on another thread than
/// the one that began it.
struct SystemRequired(HANDLE);

// SAFETY: the handle names a kernel object, usable from any thread.
unsafe impl Send for SystemRequired {}

impl SystemRequired {
  fn hold(reason: &str) -> Option<Self> {
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
    let request = match unsafe { PowerCreateRequest(&context) } {
      Ok(request) => Self(request),
      Err(error) => {
        eprintln!("Could not create a power request for {reason}: {error}");
        return None;
      }
    };
    // SAFETY: the handle was just created; dropping `request` closes it.
    if let Err(error) = unsafe { PowerSetRequest(request.0, PowerRequestSystemRequired) } {
      eprintln!("Could not keep the system awake for {reason}: {error}");
      return None;
    }
    Some(request)
  }
}

impl Drop for SystemRequired {
  fn drop(&mut self) {
    // SAFETY: the handle came from `PowerCreateRequest` and is closed once.
    // Clearing a request never set fails harmlessly.
    unsafe {
      let _ = PowerClearRequest(self.0, PowerRequestSystemRequired);
      let _ = CloseHandle(self.0);
    }
  }
}

/// Turns execution-speed throttling off when `control` names it, or hands the
/// decision back to Windows when it is zero.
fn set_process_throttling(control: u32) {
  let state = PROCESS_POWER_THROTTLING_STATE {
    Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
    ControlMask: control,
    StateMask: 0,
  };
  // SAFETY: `state` is a valid PROCESS_POWER_THROTTLING_STATE of the size
  // passed, for the current process's pseudo-handle.
  let result = unsafe {
    SetProcessInformation(
      GetCurrentProcess(),
      ProcessPowerThrottling,
      std::ptr::from_ref(&state).cast(),
      size_of::<PROCESS_POWER_THROTTLING_STATE>() as u32,
    )
  };
  if let Err(error) = result {
    eprintln!("Could not change the app's power throttling: {error}");
  }
}

pub(super) struct Thread;

impl Thread {
  pub(super) fn set(urgency: Urgency) -> Self {
    match urgency {
      Urgency::Waited => set_thread_throttling(THREAD_POWER_THROTTLING_EXECUTION_SPEED),
      // Left for Windows to throttle as it judges, as work nobody waits on.
      Urgency::Ahead => set_thread_throttling(0),
    }
    Self
  }
}

impl Drop for Thread {
  fn drop(&mut self) {
    set_thread_throttling(0);
  }
}

/// As [`set_process_throttling`], for the calling thread.
fn set_thread_throttling(control: u32) {
  let state = THREAD_POWER_THROTTLING_STATE {
    Version: THREAD_POWER_THROTTLING_CURRENT_VERSION,
    ControlMask: control,
    StateMask: 0,
  };
  // SAFETY: `state` is a valid THREAD_POWER_THROTTLING_STATE of the size
  // passed, for the calling thread's pseudo-handle.
  let result = unsafe {
    SetThreadInformation(
      GetCurrentThread(),
      ThreadPowerThrottling,
      std::ptr::from_ref(&state).cast(),
      size_of::<THREAD_POWER_THROTTLING_STATE>() as u32,
    )
  };
  if let Err(error) = result {
    eprintln!("Could not change this thread's power throttling: {error}");
  }
}
