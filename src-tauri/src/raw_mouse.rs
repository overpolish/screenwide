// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Raw mouse input for whichever feature needs it for a while. Windows lets a
//! process register only one window for raw mouse input, and removing the
//! registration removes it for the whole process, so a feature that
//! registered its own window took the input from the others and, when it
//! finished, left nobody with any. One window holds the registration for the
//! life of the process (Glide's input window, which starts with the app) and
//! hands every raw mouse report here; features subscribe for as long as they
//! need the reports.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard};

use windows::Win32::UI::Input::{MOUSE_MOVE_ABSOLUTE, RAWMOUSE};
use windows::Win32::UI::WindowsAndMessaging::RI_MOUSE_LEFT_BUTTON_UP;

/// One raw mouse report, as subscribers read it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MouseReport {
  /// Relative travel in mouse counts; zero for a device that reports
  /// positions instead, such as a tablet or a remote session.
  pub(crate) delta_x: i32,
  pub(crate) delta_y: i32,
  pub(crate) left_released: bool,
}

impl MouseReport {
  pub(crate) fn of(mouse: &RAWMOUSE) -> Self {
    let relative = mouse.usFlags.0 & MOUSE_MOVE_ABSOLUTE.0 == 0;
    // SAFETY: the button fields are the variant every mouse report fills.
    let buttons = u32::from(unsafe { mouse.Anonymous.Anonymous.usButtonFlags });
    Self {
      delta_x: if relative { mouse.lLastX } else { 0 },
      delta_y: if relative { mouse.lLastY } else { 0 },
      left_released: buttons & RI_MOUSE_LEFT_BUTTON_UP != 0,
    }
  }
}

type Subscriber = Box<dyn FnMut(&MouseReport) + Send>;

static SUBSCRIBERS: Mutex<Vec<(u64, Subscriber)>> = Mutex::new(Vec::new());
static NEXT_ID: AtomicU64 = AtomicU64::new(1);
/// Whether the window holding the registration has it.
static REGISTERED: AtomicBool = AtomicBool::new(false);

fn subscribers() -> MutexGuard<'static, Vec<(u64, Subscriber)>> {
  SUBSCRIBERS
    .lock()
    .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// A subscription, ended when dropped.
#[must_use = "dropping the subscription ends it"]
pub(crate) struct Subscription {
  id: u64,
}

impl Drop for Subscription {
  fn drop(&mut self) {
    subscribers().retain(|(id, _)| *id != self.id);
  }
}

/// Hands every raw mouse report to `subscriber` until the subscription is
/// dropped. It runs on the registered window's thread, which also drives
/// Glide, so it must not wait on anything; nor may it subscribe or drop a
/// subscription. Fails when no window holds the registration.
pub(crate) fn subscribe(
  subscriber: impl FnMut(&MouseReport) + Send + 'static,
) -> Result<Subscription, String> {
  if !REGISTERED.load(Ordering::Acquire) {
    return Err("Raw mouse input is unavailable".to_owned());
  }
  let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
  subscribers().push((id, Box::new(subscriber)));
  Ok(Subscription { id })
}

/// Called by the window holding the registration once it has it.
pub(crate) fn set_registered(registered: bool) {
  REGISTERED.store(registered, Ordering::Release);
}

/// Called by the window holding the registration for every raw mouse report.
pub(crate) fn deliver(report: &MouseReport) {
  for (_, subscriber) in subscribers().iter_mut() {
    subscriber(report);
  }
}

#[cfg(test)]
mod tests {
  use std::sync::{Arc, Mutex};

  use super::*;

  #[test]
  fn an_ended_subscription_leaves_the_others_receiving() {
    set_registered(true);
    let first = Arc::new(Mutex::new(0));
    let second = Arc::new(Mutex::new(0));
    let count = |counter: &Arc<Mutex<i32>>| {
      let counter = Arc::clone(counter);
      move |_: &MouseReport| *counter.lock().unwrap() += 1
    };
    let first_subscription = subscribe(count(&first)).unwrap();
    let second_subscription = subscribe(count(&second)).unwrap();

    deliver(&MouseReport::default());
    drop(first_subscription);
    deliver(&MouseReport::default());

    assert_eq!(*first.lock().unwrap(), 1);
    assert_eq!(*second.lock().unwrap(), 2);
    drop(second_subscription);
  }

  #[test]
  fn a_position_report_carries_no_travel() {
    let mut mouse = RAWMOUSE {
      lLastX: 30_000,
      lLastY: 12_000,
      ..Default::default()
    };
    mouse.usFlags.0 = MOUSE_MOVE_ABSOLUTE.0;
    mouse.Anonymous.Anonymous.usButtonFlags = RI_MOUSE_LEFT_BUTTON_UP as u16;
    assert_eq!(
      MouseReport::of(&mouse),
      MouseReport {
        delta_x: 0,
        delta_y: 0,
        left_released: true,
      }
    );
  }
}
