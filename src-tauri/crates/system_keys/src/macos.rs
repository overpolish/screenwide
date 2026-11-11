// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An event tap where key presses enter the window server, ahead of the
//! system's own hot keys, on a thread of its own. Unlike turning the hot keys
//! off in the system's settings, the tap keeps nothing once the process is
//! gone. Creating it needs Accessibility, which Screenwide asks for anyway.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread::JoinHandle;
use std::time::Duration;

use core_foundation::base::TCFType;
use core_foundation::mach_port::CFMachPortRef;
use core_foundation::runloop::{kCFRunLoopCommonModes, kCFRunLoopDefaultMode, CFRunLoop};
use core_graphics::event::{
  CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
  CGEventType, CallbackResult, EventField,
};

use super::{OnKey, SystemKey};

pub(super) const SUPPORTED: bool = true;

const START_TIMEOUT: Duration = Duration::from_secs(2);
/// How long the tap's run loop waits before looking for a stop it missed:
/// one asked for just before the loop starts running is lost.
const STOP_POLL: Duration = Duration::from_secs(1);
/// `kVK_ANSI_4` and `kVK_ANSI_5`. The system matches its screenshot keys by
/// key position, so these hold on every layout.
const KEY_4: i64 = 0x15;
const KEY_5: i64 = 0x17;

extern "C" {
  fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
}

#[derive(Clone, Copy, Default)]
struct Modifiers {
  command: bool,
  control: bool,
  option: bool,
  shift: bool,
}

impl Modifiers {
  fn of(flags: CGEventFlags) -> Self {
    Self {
      command: flags.contains(CGEventFlags::CGEventFlagCommand),
      control: flags.contains(CGEventFlags::CGEventFlagControl),
      option: flags.contains(CGEventFlags::CGEventFlagAlternate),
      shift: flags.contains(CGEventFlags::CGEventFlagShift),
    }
  }
}

/// The system's key `key` is part of under `held`, if any. Only the exact
/// combination counts: Cmd+Option+Shift+4 is not Cmd+Shift+4, and Cmd+Shift+3
/// stays the system's, as Screenwide has no picture of the whole screen.
fn matched(key: i64, held: Modifiers) -> Option<SystemKey> {
  if !held.command || !held.shift || held.option {
    return None;
  }
  match (key, held.control) {
    (KEY_4, false) => Some(SystemKey::AreaScreenshot),
    (KEY_4, true) => Some(SystemKey::AreaScreenshotToClipboard),
    (KEY_5, false) => Some(SystemKey::ScreenshotToolbar),
    _ => None,
  }
}

struct TapState {
  on_key: OnKey,
  /// Keys whose press was taken, so their repeats and release are too.
  taken: RefCell<HashSet<i64>>,
  /// The tap itself, for turning it back on when the system turns it off.
  port: Cell<CFMachPortRef>,
}

impl TapState {
  fn handle(&self, event_type: CGEventType, event: &CGEvent) -> CallbackResult {
    let pressed = match event_type {
      CGEventType::KeyDown => true,
      CGEventType::KeyUp => false,
      // The system turns a tap off that keeps input waiting too long, and
      // while secure input is on; it is turned back on, or the keys would
      // quietly go back to the system.
      CGEventType::TapDisabledByTimeout | CGEventType::TapDisabledByUserInput => {
        let port = self.port.get();
        if !port.is_null() {
          // SAFETY: the tap's port, alive while its run loop calls back.
          unsafe { CGEventTapEnable(port, true) };
        }
        return CallbackResult::Keep;
      }
      _ => return CallbackResult::Keep,
    };
    let key = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
    let mut taken = self.taken.borrow_mut();
    if !pressed {
      return if taken.remove(&key) {
        CallbackResult::Drop
      } else {
        CallbackResult::Keep
      };
    }
    if taken.contains(&key) {
      return CallbackResult::Drop;
    }
    let Some(system_key) = matched(key, Modifiers::of(event.get_flags())) else {
      return CallbackResult::Keep;
    };
    if !(self.on_key)(system_key) {
      return CallbackResult::Keep;
    }
    taken.insert(key);
    CallbackResult::Drop
  }
}

/// The tap's thread: creates the tap and runs its run loop until stopped.
fn run(on_key: OnKey, stopping: &AtomicBool, ready: &mpsc::Sender<Result<CFRunLoop, String>>) {
  let state = TapState {
    on_key,
    taken: RefCell::new(HashSet::new()),
    port: Cell::new(std::ptr::null_mut()),
  };
  // SAFETY: the tap only runs on this thread's run loop and is dropped below,
  // before `state` it borrows.
  let tap = unsafe {
    CGEventTap::new_unchecked(
      CGEventTapLocation::HID,
      CGEventTapPlacement::HeadInsertEventTap,
      CGEventTapOptions::Default,
      vec![CGEventType::KeyDown, CGEventType::KeyUp],
      |_, event_type, event| state.handle(event_type, event),
    )
  };
  let Ok(tap) = tap else {
    let _ = ready.send(Err(
      "Could not take the system's screenshot keys; Accessibility access is required".to_owned(),
    ));
    return;
  };
  let Ok(source) = tap.mach_port().create_runloop_source(0) else {
    let _ = ready.send(Err(
      "Could not take the system's screenshot keys: the event tap has no run loop source"
        .to_owned(),
    ));
    return;
  };
  state.port.set(tap.mach_port().as_concrete_TypeRef());
  let run_loop = CFRunLoop::get_current();
  // SAFETY: a constant the framework defines.
  run_loop.add_source(&source, unsafe { kCFRunLoopCommonModes });
  tap.enable();
  let _ = ready.send(Ok(run_loop));

  while !stopping.load(Ordering::Acquire) {
    // SAFETY: a constant the framework defines.
    CFRunLoop::run_in_mode(unsafe { kCFRunLoopDefaultMode }, STOP_POLL, false);
  }
  drop(tap);
}

pub(super) struct Hook {
  stopping: Arc<AtomicBool>,
  run_loop: Option<CFRunLoop>,
  worker: Option<JoinHandle<()>>,
}

impl Hook {
  pub(super) fn start(on_key: OnKey) -> Result<Self, String> {
    let stopping = Arc::new(AtomicBool::new(false));
    let (ready, is_ready) = mpsc::channel();
    let worker_stopping = Arc::clone(&stopping);
    let worker = std::thread::Builder::new()
      .name("screenwide-system-keys".to_owned())
      .spawn(move || run(on_key, &worker_stopping, &ready))
      .map_err(|error| format!("Could not take the system's screenshot keys: {error}"))?;
    let mut hook = Self {
      stopping,
      run_loop: None,
      worker: Some(worker),
    };
    match is_ready.recv_timeout(START_TIMEOUT) {
      Ok(Ok(run_loop)) => {
        hook.run_loop = Some(run_loop);
        Ok(hook)
      }
      Ok(Err(error)) => {
        hook.stop();
        Err(error)
      }
      Err(_) => {
        hook.stop();
        Err("Taking the system's screenshot keys did not start in time".to_owned())
      }
    }
  }

  fn stop(&mut self) {
    self.stopping.store(true, Ordering::Release);
    if let Some(run_loop) = self.run_loop.take() {
      run_loop.stop();
    }
    if let Some(worker) = self.worker.take() {
      let _ = worker.join();
    }
  }
}

impl Drop for Hook {
  fn drop(&mut self) {
    self.stop();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn only_the_exact_combinations_are_taken() {
    let command_shift = Modifiers {
      command: true,
      shift: true,
      ..Modifiers::default()
    };
    let with_control = Modifiers {
      control: true,
      ..command_shift
    };
    assert_eq!(
      matched(KEY_4, command_shift),
      Some(SystemKey::AreaScreenshot)
    );
    assert_eq!(
      matched(KEY_4, with_control),
      Some(SystemKey::AreaScreenshotToClipboard)
    );
    assert_eq!(
      matched(KEY_5, command_shift),
      Some(SystemKey::ScreenshotToolbar)
    );
    // Cmd+Ctrl+Shift+5 is no system key, and Option makes another combination.
    assert_eq!(matched(KEY_5, with_control), None);
    assert_eq!(
      matched(
        KEY_4,
        Modifiers {
          option: true,
          ..command_shift
        }
      ),
      None
    );
    // Cmd+Shift+3, a picture of the whole screen, stays the system's.
    assert_eq!(matched(0x14, command_shift), None);
    assert_eq!(
      matched(
        KEY_4,
        Modifiers {
          command: true,
          ..Modifiers::default()
        }
      ),
      None
    );
  }
}
