// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pasting on macOS. The text goes on the general pasteboard as a promise
//! (`declareTypes:owner:`), which AppKit asks the owner to keep when an app
//! reads it: that call is the receipt. The pasteboard is AppKit's, so
//! publishing and putting back run on the main thread, whose run loop also
//! delivers the receipt; the wait between runs on the caller's.

use std::cell::RefCell;
use std::sync::{mpsc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, NSObjectProtocol, ProtocolObject};
use objc2::{define_class, msg_send, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
  NSApplication, NSApplicationActivationOptions, NSPasteboard, NSPasteboardItem,
  NSPasteboardTypeString, NSRunningApplication, NSWorkspace,
};
use objc2_foundation::{NSArray, NSData, NSInteger, NSObject, NSString};

use super::{settled, OnMain, Refusal};

/// nspasteboard.org's markers: clipboard managers keep nothing marked with
/// them, the text is not the person's to keep, and it is only passing.
const CONCEALED: &str = "org.nspasteboard.ConcealedType";
const TRANSIENT: &str = "org.nspasteboard.TransientType";

/// Where V is on an ANSI keyboard. Command's shortcuts follow it on every
/// layout that keeps them on QWERTY's keys, which macOS's own do.
const V_KEY: u16 = 9;

/// How often the wait looks for a read.
const POLL: Duration = Duration::from_millis(20);

#[link(name = "Carbon", kind = "framework")]
extern "C" {
  /// Whether a password field, or anything else, has asked that no other
  /// program see what is typed.
  fn IsSecureEventInputEnabled() -> u8;
}

/// When the text being pasted was last read.
static LAST_READ: Mutex<Option<Instant>> = Mutex::new(None);

/// A paste under way: the promise, which the pasteboard does not keep, what
/// the pasteboard held, and its change count once it held the text.
struct Pending {
  _promise: Retained<Promise>,
  held: Vec<Vec<(Retained<NSString>, Retained<NSData>)>>,
  count: NSInteger,
}

thread_local! {
  static PENDING: RefCell<Option<Pending>> = const { RefCell::new(None) };
}

define_class!(
  // Safety: NSObject has no subclassing requirements, and this implements no
  // `Drop`.
  #[unsafe(super(NSObject))]
  #[thread_kind = MainThreadOnly]
  #[name = "ScreenwidePastePromise"]
  #[ivars = Retained<NSString>]
  struct Promise;

  unsafe impl NSObjectProtocol for Promise {}

  impl Promise {
    /// An app is reading the text: it is handed over, and the read noted.
    #[unsafe(method(pasteboard:provideDataForType:))]
    fn provide(&self, pasteboard: &NSPasteboard, kind: &NSString) {
      // Safety: the type is one of AppKit's own constants.
      let string = unsafe { NSPasteboardTypeString };
      if !kind.isEqualToString(string) {
        return;
      }
      pasteboard.setString_forType(self.ivars(), string);
      *LAST_READ.lock().unwrap_or_else(PoisonError::into_inner) = Some(Instant::now());
    }
  }
);

impl Promise {
  fn new(marker: MainThreadMarker, text: &str) -> Retained<Self> {
    let this = Self::alloc(marker).set_ivars(NSString::from_str(text));
    unsafe { msg_send![super(this), init] }
  }
}

pub(super) fn paste(text: &str, on_main: OnMain<'_>) -> Result<(), Refusal> {
  // Safety: a plain query of the system's state.
  if unsafe { IsSecureEventInputEnabled() } != 0 {
    return Err(Refusal::SecureField);
  }
  let sent = on_main_thread(on_main, {
    let text = text.to_owned();
    move || publish(&text)
  })??;
  while !settled(
    Instant::now(),
    sent,
    *LAST_READ.lock().unwrap_or_else(PoisonError::into_inner),
  ) {
    std::thread::sleep(POLL);
  }
  on_main_thread(on_main, put_back)
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

/// Puts the text on the pasteboard in place of what it held, which is kept,
/// and presses the paste keys. Answers when they were pressed.
fn publish(text: &str) -> Result<Instant, Refusal> {
  let marker = MainThreadMarker::new()
    .ok_or_else(|| Refusal::Failed("The pasteboard was used off the main thread".to_owned()))?;
  let pasteboard = NSPasteboard::generalPasteboard();
  let held = held(&pasteboard);
  let promise = Promise::new(marker, text);
  // Safety: as above.
  let string = unsafe { NSPasteboardTypeString };
  let concealed = NSString::from_str(CONCEALED);
  let transient = NSString::from_str(TRANSIENT);
  let owner: &AnyObject = &promise;
  // Safety: the owner answers `pasteboard:provideDataForType:`.
  unsafe {
    pasteboard.declareTypes_owner(
      &NSArray::from_slice(&[string, &concealed, &transient]),
      Some(owner),
    );
  }
  for marker in [&concealed, &transient] {
    pasteboard.setData_forType(Some(&NSData::new()), marker);
  }
  *LAST_READ.lock().unwrap_or_else(PoisonError::into_inner) = None;
  PENDING.with(|pending| {
    *pending.borrow_mut() = Some(Pending {
      _promise: promise,
      held,
      count: pasteboard.changeCount(),
    });
  });
  let sent = Instant::now();
  press_paste()?;
  Ok(sent)
}

/// Every item on the pasteboard, as each type's data.
fn held(pasteboard: &NSPasteboard) -> Vec<Vec<(Retained<NSString>, Retained<NSData>)>> {
  let Some(items) = pasteboard.pasteboardItems() else {
    return Vec::new();
  };
  items
    .iter()
    .map(|item| {
      item
        .types()
        .iter()
        .filter_map(|kind| item.dataForType(&kind).map(|data| (kind, data)))
        .collect()
    })
    .collect()
}

/// Puts back what the pasteboard held, unless something was copied since.
fn put_back() {
  let Some(pending) = PENDING.with(|pending| pending.borrow_mut().take()) else {
    return;
  };
  let pasteboard = NSPasteboard::generalPasteboard();
  if pasteboard.changeCount() != pending.count {
    return;
  }
  pasteboard.clearContents();
  let items: Vec<Retained<NSPasteboardItem>> = pending
    .held
    .iter()
    .map(|types| {
      let item = NSPasteboardItem::new();
      for (kind, data) in types {
        item.setData_forType(data, kind);
      }
      item
    })
    .collect();
  if items.is_empty() {
    return;
  }
  let writers: Vec<&ProtocolObject<dyn objc2_app_kit::NSPasteboardWriting>> = items
    .iter()
    .map(|item| ProtocolObject::from_ref(&**item))
    .collect();
  pasteboard.writeObjects(&NSArray::from_slice(&writers));
}

/// Command-V, with no other modifier whatever keys are held.
fn press_paste() -> Result<(), Refusal> {
  let failed = || Refusal::Failed("Could not press the paste keys".to_owned());
  let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).map_err(|()| failed())?;
  for down in [true, false] {
    let event = CGEvent::new_keyboard_event(source.clone(), V_KEY, down).map_err(|()| failed())?;
    event.set_flags(CGEventFlags::CGEventFlagCommand);
    event.post(CGEventTapLocation::HID);
  }
  Ok(())
}
