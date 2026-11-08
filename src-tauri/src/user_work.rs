// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keeps work someone is waiting for at full speed.
//!
//! Screenwide is an accessory app (`LSUIElement`), mostly without a visible
//! window, so macOS may put it in App Nap: the whole process is clamped to
//! background priority on the efficiency cores, which no thread can escape by
//! raising its own QoS. Windows likewise power-throttles processes it judges
//! to be in the background. An activity lifts that for as long as the work
//! lasts.
//!
//! Threads Rust spawns, tokio's included, start at the default QoS whatever
//! their spawner's, so a thread doing work someone waits for says so itself.

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as platform;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as platform;

/// Other systems neither nap apps nor throttle threads by their QoS, so there
/// is nothing to lift.
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform {
  pub(super) struct Activity;

  impl Activity {
    pub(super) fn begin(_: &str, _: bool) -> Self {
      Self
    }
  }

  pub(super) struct Thread;

  impl Thread {
    pub(super) fn set(_: super::Urgency) -> Self {
      Self
    }
  }
}

#[derive(Clone, Copy)]
enum Urgency {
  /// Someone is waiting for the result now.
  Waited,
  /// Ahead of need, so it never delays work someone waits for.
  Ahead,
}

/// Keeps the app out of App Nap and power throttling while alive. Activities
/// nest: the app is throttled again only once every one has ended.
#[must_use = "the app may be throttled again as soon as the activity drops"]
pub(crate) struct UserActivity {
  _platform: platform::Activity,
}

impl UserActivity {
  /// For work someone waits on while at the Mac. The Mac may still sleep when
  /// idle, as one can be left waiting with nobody there, such as an open
  /// text-recognition overlay.
  pub(crate) fn begin(reason: &str) -> Self {
    Self {
      _platform: platform::Activity::begin(reason, false),
    }
  }

  /// For long work someone started and expects finished on their return,
  /// such as an export: the computer does not sleep when idle until it ends.
  pub(crate) fn begin_keeping_awake(reason: &str) -> Self {
    Self {
      _platform: platform::Activity::begin(reason, true),
    }
  }
}

/// Sets the calling thread's urgency until dropped, then restores what it
/// had: pool threads go on to run other work afterwards.
#[must_use = "the thread's urgency is restored as soon as this drops"]
pub(crate) struct ThreadUrgency {
  _platform: platform::Thread,
}

impl ThreadUrgency {
  /// The thread works on something someone is waiting for now.
  pub(crate) fn waited() -> Self {
    Self {
      _platform: platform::Thread::set(Urgency::Waited),
    }
  }

  /// The thread prepares something ahead of need.
  #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
  pub(crate) fn ahead() -> Self {
    Self {
      _platform: platform::Thread::set(Urgency::Ahead),
    }
  }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
  use super::ThreadUrgency;

  fn class() -> libc::qos_class_t {
    let mut class = libc::qos_class_t::QOS_CLASS_UNSPECIFIED;
    let mut relative = 0;
    // SAFETY: both out-pointers are valid for the call.
    unsafe { libc::pthread_get_qos_class_np(libc::pthread_self(), &mut class, &mut relative) };
    class
  }

  #[test]
  fn a_thread_gets_its_class_back() {
    std::thread::spawn(|| {
      let before = class() as u32;
      {
        let _urgency = ThreadUrgency::waited();
        assert_eq!(
          class() as u32,
          libc::qos_class_t::QOS_CLASS_USER_INITIATED as u32
        );
        let _ahead = ThreadUrgency::ahead();
        assert_eq!(class() as u32, libc::qos_class_t::QOS_CLASS_UTILITY as u32);
      }
      let after = class() as u32;
      let default = libc::qos_class_t::QOS_CLASS_DEFAULT as u32;
      assert!(after == before || (before == 0 && after == default));
    })
    .join()
    .unwrap();
  }
}
