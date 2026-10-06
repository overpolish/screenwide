// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::c_void;
use std::sync::OnceLock;

use tauri::AppHandle;
use windows::Win32::Foundation::{ERROR_SUCCESS, HANDLE};
use windows::Win32::System::Power::{
  PowerRegisterSuspendResumeNotification, DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS,
};
use windows::Win32::UI::WindowsAndMessaging::{
  DEVICE_NOTIFY_CALLBACK, PBT_APMRESUMEAUTOMATIC, PBT_APMRESUMESUSPEND, PBT_APMSUSPEND,
};

struct Handlers {
  app: AppHandle,
  did_wake: fn(&AppHandle),
  will_sleep: fn(&AppHandle),
}

static HANDLERS: OnceLock<Handlers> = OnceLock::new();

/// The subscription the power manager is handed. Static, because it is
/// registered for the life of the process.
struct Subscription(DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS);

// SAFETY: never written after it is built; its context pointer is null and
// the callback reads its handlers from `HANDLERS`.
unsafe impl Sync for Subscription {}

static SUBSCRIPTION: Subscription = Subscription(DEVICE_NOTIFY_SUBSCRIBE_PARAMETERS {
  Callback: Some(power_event),
  Context: std::ptr::null_mut(),
});

/// Registers with the power manager, which calls back on a thread of its own
/// without needing a window. Windows waits briefly for the suspend callback,
/// so the computer does not sleep before the recording is paused.
pub(super) fn initialize(app: &AppHandle, will_sleep: fn(&AppHandle), did_wake: fn(&AppHandle)) {
  let handlers = Handlers {
    app: app.clone(),
    did_wake,
    will_sleep,
  };
  if HANDLERS.set(handlers).is_err() {
    return;
  }
  let mut registration = std::ptr::null_mut();
  let result = unsafe {
    PowerRegisterSuspendResumeNotification(
      DEVICE_NOTIFY_CALLBACK,
      HANDLE(std::ptr::from_ref(&SUBSCRIPTION.0).cast_mut().cast()),
      &mut registration,
    )
  };
  if result != ERROR_SUCCESS {
    eprintln!("Sleep and wake will not pause capture: {result:?}");
  }
}

/// A wake raises the automatic resume, and the user-present one as well when
/// someone woke the computer; the replay buffer only answers the first.
unsafe extern "system" fn power_event(_: *const c_void, kind: u32, _: *const c_void) -> u32 {
  let Some(handlers) = HANDLERS.get() else {
    return ERROR_SUCCESS.0;
  };
  match kind {
    PBT_APMSUSPEND => (handlers.will_sleep)(&handlers.app),
    PBT_APMRESUMEAUTOMATIC | PBT_APMRESUMESUSPEND => (handlers.did_wake)(&handlers.app),
    _ => {}
  }
  ERROR_SUCCESS.0
}
