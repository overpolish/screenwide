// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pasting on macOS. The pasteboard is AppKit's, so the text is put on it on
//! the main thread; the paste keys and the look at what has the keyboard
//! run on the caller's.

mod focus;

use std::sync::mpsc;

use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use objc2::MainThreadMarker;
use objc2_app_kit::{
  NSApplication, NSApplicationActivationOptions, NSPasteboard, NSPasteboardTypeString,
  NSRunningApplication, NSWorkspace,
};
use objc2_foundation::NSString;

use super::{OnMain, Pasted, Refusal};

/// Where V is on an ANSI keyboard. Command's shortcuts follow it on every
/// layout that keeps them on QWERTY's keys, which macOS's own do.
const V_KEY: u16 = 9;

#[link(name = "Carbon", kind = "framework")]
extern "C" {
  /// Whether a password field, or anything else, has asked that no other
  /// program see what is typed.
  fn IsSecureEventInputEnabled() -> u8;
}

pub(super) fn paste(text: &str, on_main: OnMain<'_>) -> Result<Pasted, Refusal> {
  // Safety: a plain query of the system's state.
  if unsafe { IsSecureEventInputEnabled() } != 0 {
    return Err(Refusal::SecureField);
  }
  let front = on_main_thread(on_main, {
    let text = text.to_owned();
    move || copy(&text).map(|()| frontmost())
  })??;
  let focus = front.map_or(crate::Focus::Unknown, focus::focus);
  if press_paste().is_err() {
    return Ok(Pasted::OnClipboard);
  }
  Ok(focus.pasted())
}

/// An app, by its process.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Target(i32);

impl Target {
  pub(super) fn current() -> Option<Self> {
    frontmost().filter(|&pid| !is_ours(pid)).map(Self)
  }
}

fn frontmost() -> Option<i32> {
  NSWorkspace::sharedWorkspace()
    .frontmostApplication()
    .map(|app| app.processIdentifier())
}

fn is_ours(pid: i32) -> bool {
  u32::try_from(pid).is_ok_and(|pid| pid == std::process::id())
}

/// Brings `target` in front if Screenwide is: activation is handed over, as
/// macOS asks of an app making another active. False when nothing was done.
pub(super) fn bring_back(target: Target, on_main: OnMain<'_>) -> bool {
  on_main_thread(on_main, move || {
    let Some(marker) = MainThreadMarker::new() else {
      return false;
    };
    if !frontmost().is_some_and(is_ours) {
      return false;
    }
    let Some(app) = NSRunningApplication::runningApplicationWithProcessIdentifier(target.0) else {
      return false;
    };
    NSApplication::sharedApplication(marker).yieldActivationToApplication(&app);
    app.activateWithOptions(NSApplicationActivationOptions::empty())
  })
  .unwrap_or(false)
}

pub(super) fn is_in_front(target: Target, on_main: OnMain<'_>) -> bool {
  on_main_thread(on_main, move || frontmost() == Some(target.0)).unwrap_or(false)
}

/// Runs `work` on the main thread and waits for its answer.
fn on_main_thread<T: Send + 'static>(
  on_main: OnMain<'_>,
  work: impl FnOnce() -> T + Send + 'static,
) -> Result<T, Refusal> {
  let (answer, answered) = mpsc::channel();
  on_main(Box::new(move || {
    let _ = answer.send(work());
  }));
  answered
    .recv()
    .map_err(|_| Refusal::Failed("The main thread did not run the paste".to_owned()))
}

/// Puts the text on the pasteboard in place of what it held.
fn copy(text: &str) -> Result<(), Refusal> {
  if MainThreadMarker::new().is_none() {
    return Err(Refusal::Failed(
      "The pasteboard was used off the main thread".to_owned(),
    ));
  }
  let pasteboard = NSPasteboard::generalPasteboard();
  pasteboard.clearContents();
  // Safety: the type is one of AppKit's own constants.
  let string = unsafe { NSPasteboardTypeString };
  if pasteboard.setString_forType(&NSString::from_str(text), string) {
    Ok(())
  } else {
    Err(Refusal::Failed(
      "Could not put the text on the pasteboard".to_owned(),
    ))
  }
}

/// Command-V, with no other modifier whatever keys are held.
fn press_paste() -> Result<(), ()> {
  let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)?;
  for down in [true, false] {
    let event = CGEvent::new_keyboard_event(source.clone(), V_KEY, down)?;
    event.set_flags(CGEventFlags::CGEventFlagCommand);
    event.post(CGEventTapLocation::HID);
  }
  Ok(())
}
