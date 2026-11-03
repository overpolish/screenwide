// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use libc::qos_class_t;
use objc2::rc::Retained;
use objc2::runtime::{NSObjectProtocol, ProtocolObject};
use objc2_foundation::{NSActivityOptions, NSProcessInfo, NSString};

use super::Urgency;

pub(super) struct Activity(Retained<ProtocolObject<dyn NSObjectProtocol>>);

// SAFETY: NSProcessInfo is thread-safe, and the token is an opaque handle
// only ever handed back to it, so the activity may end on any thread.
unsafe impl Send for Activity {}

impl Activity {
  pub(super) fn begin(reason: &str, keep_awake: bool) -> Self {
    let options = if keep_awake {
      NSActivityOptions::UserInitiated
    } else {
      NSActivityOptions::UserInitiatedAllowingIdleSystemSleep
    };
    Self(
      NSProcessInfo::processInfo()
        .beginActivityWithOptions_reason(options, &NSString::from_str(reason)),
    )
  }
}

impl Drop for Activity {
  fn drop(&mut self) {
    // SAFETY: the token came from `beginActivityWithOptions_reason` and is
    // ended once.
    unsafe { NSProcessInfo::processInfo().endActivity(&self.0) };
  }
}

pub(super) struct Thread {
  class: qos_class_t,
  relative_priority: libc::c_int,
}

impl Thread {
  pub(super) fn set(urgency: Urgency) -> Self {
    let mut previous = Self {
      class: qos_class_t::QOS_CLASS_UNSPECIFIED,
      relative_priority: 0,
    };
    // SAFETY: both out-pointers are valid for the call, for this thread.
    unsafe {
      libc::pthread_get_qos_class_np(
        libc::pthread_self(),
        &mut previous.class,
        &mut previous.relative_priority,
      )
    };
    set_self(
      match urgency {
        Urgency::Waited => qos_class_t::QOS_CLASS_USER_INITIATED,
        // Utility rather than background, which keeps a thread to the
        // efficiency cores and throttles its disk reads: anything that waits
        // on what it prepares would wait on that too.
        Urgency::Ahead => qos_class_t::QOS_CLASS_UTILITY,
      },
      0,
    );
    previous
  }
}

impl Drop for Thread {
  fn drop(&mut self) {
    // A thread nobody gave a class runs at the default one, which it cannot
    // ask for back as unspecified.
    let class = match self.class {
      qos_class_t::QOS_CLASS_UNSPECIFIED => qos_class_t::QOS_CLASS_DEFAULT,
      class => class,
    };
    set_self(class, self.relative_priority);
  }
}

fn set_self(class: qos_class_t, relative_priority: libc::c_int) {
  // SAFETY: changes only the calling thread's own class.
  let error = unsafe { libc::pthread_set_qos_class_self_np(class, relative_priority) };
  if error != 0 {
    eprintln!("Could not set this thread's QoS class to {class:?}: error {error}");
  }
}
