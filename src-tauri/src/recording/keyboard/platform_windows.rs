// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};
use std::sync::{mpsc, Arc};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[path = "classifier_windows.rs"]
mod classifier;

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
  CallNextHookEx, DispatchMessageW, GetMessageW, PostThreadMessageW, SetWindowsHookExW,
  TranslateMessage, UnhookWindowsHookEx, HC_ACTION, KBDLLHOOKSTRUCT, LLKHF_EXTENDED, MSG,
  WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_QUIT, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use super::{EventSink, FocusContext, RawKeyboardEvent};

const EVENT_QUEUE_CAPACITY: usize = 128;
const POLL: Duration = Duration::from_millis(25);
const START_TIMEOUT: Duration = Duration::from_secs(2);

/// Only one recorder may hold the hook: a second one would double every event.
static HOOK_INSTALLED: AtomicBool = AtomicBool::new(false);

struct HookState {
  events: mpsc::SyncSender<classifier::PendingEvent>,
  focus: Arc<AtomicU8>,
}

thread_local! {
  /// Low-level hook callbacks run on the thread that installed the hook, so
  /// its state needs no lock. The callback delays every keystroke in the
  /// system until it returns, and Windows silently drops the hook if it
  /// exceeds LowLevelHooksTimeout.
  static HOOK_STATE: RefCell<Option<HookState>> = const { RefCell::new(None) };
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
  if code == HC_ACTION as i32 {
    // SAFETY: for HC_ACTION the system passes a KBDLLHOOKSTRUCT it owns for
    // the duration of this call.
    let data = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
    let is_down = matches!(wparam.0 as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
    let is_up = matches!(wparam.0 as u32, WM_KEYUP | WM_SYSKEYUP);
    // LLKHF_INJECTED is deliberately not filtered: remote-desktop and
    // streaming hosts (Parsec, Sunshine, RDP) deliver all of the user's real
    // typing as injected input, and the macOS HID tap records synthetic
    // events too.
    if is_down || is_up {
      HOOK_STATE.with(|state| {
        if let Some(state) = state.borrow().as_ref() {
          let _ = state.events.try_send(classifier::PendingEvent {
            at: Instant::now(),
            extended: data.flags.contains(LLKHF_EXTENDED),
            focus: state.focus.load(Ordering::Acquire),
            is_down,
            virtual_key: data.vkCode,
          });
        }
      });
    }
  }
  // SAFETY: passing the callback's own arguments along the hook chain.
  unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// The hook thread. It installs the hook and pumps messages so the callback can
/// run; it does no other work.
fn run_hook(
  events: mpsc::SyncSender<classifier::PendingEvent>,
  focus: Arc<AtomicU8>,
  thread_id: Arc<AtomicU32>,
  installed: mpsc::Sender<Result<(), String>>,
) {
  // SAFETY: reads this thread's own identifier.
  thread_id.store(unsafe { GetCurrentThreadId() }, Ordering::Release);
  HOOK_STATE.with(|state| *state.borrow_mut() = Some(HookState { events, focus }));
  // SAFETY: a low-level keyboard hook takes no module handle, and zero asks
  // for every thread in the session.
  let hook = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), None, 0) };
  let hook = match hook {
    Ok(hook) => hook,
    Err(error) => {
      HOOK_STATE.with(|state| *state.borrow_mut() = None);
      let _ = installed.send(Err(format!(
        "Could not listen for keyboard events: {error}"
      )));
      return;
    }
  };
  let _ = installed.send(Ok(()));

  let mut message = MSG::default();
  // GetMessageW returns zero for the WM_QUIT that stop() posts, and -1 on
  // failure; both end the pump.
  while unsafe { GetMessageW(&mut message, None, 0, 0) }.0 > 0 {
    unsafe {
      let _ = TranslateMessage(&message);
      DispatchMessageW(&message);
    }
  }

  // SAFETY: unhooking from the thread that installed the hook, once.
  let _ = unsafe { UnhookWindowsHookEx(hook) };
  HOOK_STATE.with(|state| *state.borrow_mut() = None);
}

fn run(stop: &AtomicBool, sink: &EventSink, ready: mpsc::Sender<Result<(), String>>) {
  if HOOK_INSTALLED.swap(true, Ordering::AcqRel) {
    let _ = ready.send(Err(
      "Keyboard shortcut recording is already running".to_owned(),
    ));
    return;
  }

  let (events, pending) = mpsc::sync_channel::<classifier::PendingEvent>(EVENT_QUEUE_CAPACITY);
  let cached_focus = Arc::new(AtomicU8::new(classifier::encode_focus(
    FocusContext::Unknown,
  )));
  let thread_id = Arc::new(AtomicU32::new(0));
  let (installed, did_install) = mpsc::channel();
  let hook_focus = Arc::clone(&cached_focus);
  let hook_thread_id = Arc::clone(&thread_id);
  let hook = std::thread::Builder::new()
    .name("screenwide-keyboard-hook".to_owned())
    .spawn(move || run_hook(events, hook_focus, hook_thread_id, installed));
  let hook = match hook {
    Ok(hook) => hook,
    Err(error) => {
      HOOK_INSTALLED.store(false, Ordering::Release);
      let _ = ready.send(Err(format!(
        "Could not start the keyboard shortcut recorder: {error}"
      )));
      return;
    }
  };

  let started = match did_install.recv_timeout(START_TIMEOUT) {
    Ok(Ok(())) => Ok(()),
    Ok(Err(error)) => Err(error),
    Err(_) => Err("Keyboard shortcut recording did not start in time".to_owned()),
  };
  if let Err(error) = started {
    let _ = ready.send(Err(error));
    quit(&thread_id);
    let _ = hook.join();
    HOOK_INSTALLED.store(false, Ordering::Release);
    return;
  }
  let _ = ready.send(Ok(()));

  let focus_source = classifier::Focus::enter();
  let mut tracker = classifier::KeyTracker::default();
  while !stop.load(Ordering::Acquire) {
    let fresh = focus_source.context();
    cached_focus.store(classifier::encode_focus(fresh), Ordering::Release);
    while let Ok(event) = pending.try_recv() {
      let focus = FocusContext::conservative(classifier::decode_focus(event.focus), fresh);
      let Some(classified) = tracker.classify(&event) else {
        continue;
      };
      sink(RawKeyboardEvent {
        at: event.at,
        focus,
        kind: classified.kind,
        key_code: classified.key_code,
        modifiers: classified.modifiers,
      });
    }
    std::thread::sleep(POLL);
  }

  quit(&thread_id);
  let _ = hook.join();
  HOOK_INSTALLED.store(false, Ordering::Release);
}

fn quit(thread_id: &AtomicU32) {
  let thread_id = thread_id.load(Ordering::Acquire);
  if thread_id != 0 {
    // SAFETY: WM_QUIT to the hook thread's own message queue ends its pump.
    let _ = unsafe { PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) };
  }
}

pub(super) fn start(stop: Arc<AtomicBool>, sink: EventSink) -> Result<JoinHandle<()>, String> {
  let (ready, did_start) = mpsc::channel();
  let worker_stop = Arc::clone(&stop);
  let worker = std::thread::Builder::new()
    .name("screenwide-keyboard-recorder".to_owned())
    .spawn(move || run(&worker_stop, &sink, ready))
    .map_err(|error| error.to_string())?;
  match did_start.recv_timeout(START_TIMEOUT) {
    Ok(Ok(())) => Ok(worker),
    Ok(Err(error)) => {
      let _ = worker.join();
      Err(error)
    }
    Err(_) => {
      stop.store(true, Ordering::Release);
      let _ = worker.join();
      Err("Keyboard shortcut recording did not start in time".to_owned())
    }
  }
}
