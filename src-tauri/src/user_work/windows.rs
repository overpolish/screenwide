// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{Mutex, PoisonError};

use windows::Win32::System::Threading::{
  GetCurrentProcess, GetCurrentThread, ProcessPowerThrottling, SetProcessInformation,
  SetThreadInformation, ThreadPowerThrottling, PROCESS_POWER_THROTTLING_CURRENT_VERSION,
  PROCESS_POWER_THROTTLING_EXECUTION_SPEED, PROCESS_POWER_THROTTLING_STATE,
  THREAD_POWER_THROTTLING_CURRENT_VERSION, THREAD_POWER_THROTTLING_EXECUTION_SPEED,
  THREAD_POWER_THROTTLING_STATE,
};

use super::Urgency;

/// Activities under way. Windows keeps one throttling setting per process, so
/// the first activity turns throttling off and the last hands it back.
static ACTIVITIES: Mutex<usize> = Mutex::new(0);

pub(super) struct Activity;

impl Activity {
  pub(super) fn begin(_reason: &str, _keep_awake: bool) -> Self {
    let mut activities = ACTIVITIES.lock().unwrap_or_else(PoisonError::into_inner);
    if *activities == 0 {
      set_process_throttling(PROCESS_POWER_THROTTLING_EXECUTION_SPEED);
    }
    *activities += 1;
    Self
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
