// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

mod lock;

use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};

pub(super) use lock::LockWatch;

/// `kIOPMAssertionLevelOn`.
const LEVEL_ON: u32 = 255;
/// `kIOReturnSuccess`.
const SUCCESS: i32 = 0;

#[link(name = "IOKit", kind = "framework")]
unsafe extern "C" {
  fn IOPMAssertionCreateWithName(
    kind: CFStringRef,
    level: u32,
    name: CFStringRef,
    id: *mut u32,
  ) -> i32;
  fn IOPMAssertionRelease(id: u32) -> i32;
}

pub(super) struct Hold(u32);

impl Hold {
  /// The display sleeps as usual.
  pub(super) fn system(reason: &str) -> Result<Self, String> {
    Self::begin("PreventUserIdleSystemSleep", reason)
  }

  /// A display kept on keeps the system awake with it.
  pub(super) fn display(reason: &str) -> Result<Self, String> {
    Self::begin("PreventUserIdleDisplaySleep", reason)
  }

  fn begin(kind: &'static str, reason: &str) -> Result<Self, String> {
    let kind = CFString::from_static_string(kind);
    let name = CFString::new(reason);
    let mut id = 0;
    // SAFETY: both strings outlive the call, which copies them, and `id` is
    // a valid out-pointer.
    let result = unsafe {
      IOPMAssertionCreateWithName(
        kind.as_concrete_TypeRef(),
        LEVEL_ON,
        name.as_concrete_TypeRef(),
        &mut id,
      )
    };
    if result != SUCCESS {
      return Err(format!(
        "Could not take a {kind} assertion for {reason}: IOReturn {result:#x}"
      ));
    }
    Ok(Self(id))
  }
}

impl Drop for Hold {
  fn drop(&mut self) {
    // SAFETY: the assertion came from `IOPMAssertionCreateWithName` and is
    // released once.
    unsafe { IOPMAssertionRelease(self.0) };
  }
}
