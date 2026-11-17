// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{mpsc, LazyLock, Mutex, MutexGuard};
use std::time::Duration;

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{
  GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
  VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
  CallNextHookEx, DispatchMessageW, GetMessageW, PostThreadMessageW, SetWindowsHookExW,
  TranslateMessage, HC_ACTION, KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED, MSG,
  WH_KEYBOARD_LL, WM_APP, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use crate::router::{RawKey, Router};
use crate::{KeyEvent, Priority, Route};

const START_TIMEOUT: Duration = Duration::from_secs(2);
/// Asks the hook thread to send the mask key, outside the hook callback.
const WM_SEND_MASK: u32 = WM_APP + 1;
/// A key no keyboard has and nothing answers.
const MASK_KEY: VIRTUAL_KEY = VIRTUAL_KEY(0xE8);
/// Marks the keys this crate sends, so the hook passes them by. Spells "SWSK".
const EVENT_TAG: usize = 0x5357_534B;

static ROUTER: LazyLock<Mutex<Router>> = LazyLock::new(Mutex::default);
/// Whether the hook thread has started.
static HOOK_STARTED: Mutex<bool> = Mutex::new(false);

fn router() -> MutexGuard<'static, Router> {
  ROUTER
    .lock()
    .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// A handler's place in the routing, removed when dropped.
#[must_use = "dropping the registration removes the handler"]
pub struct Registration {
  id: u64,
}

impl Drop for Registration {
  fn drop(&mut self) {
    router().remove(self.id);
  }
}

/// Hears every key, before any handler takes it, and can take none. Runs on
/// the hook's thread, which holds up every keystroke in the system until it
/// returns, so it must not wait on anything; nor may it register a handler
/// or drop one.
pub fn observe(handler: impl FnMut(&KeyEvent) + Send + 'static) -> Result<Registration, String> {
  ensure_running()?;
  let id = router().add_observer(Box::new(handler));
  Ok(Registration { id })
}

/// Offers keys to `handler` in `priority`'s place: see [`crate::KeyEvent`]
/// for what it is told and [`observe`] for what it may do while it decides.
pub fn take(
  priority: Priority,
  handler: impl FnMut(&KeyEvent) -> Route + Send + 'static,
) -> Result<Registration, String> {
  ensure_running()?;
  let id = router().add_taker(priority, Box::new(handler));
  Ok(Registration { id })
}

/// Keeps every handler from taking a key while a shortcut field records one.
pub fn set_taking_paused(paused: bool) {
  router().set_paused(paused);
}

/// Sends a key no keyboard has, which the hook passes by. Windows opens Start
/// when Windows is released with nothing between its press and release, and
/// Alt held alone enters a window's menu mode; this stands in for the key
/// that was taken. Not to be called from a handler: the hook is waiting on it.
pub fn send_mask_key() {
  let key = |flags| INPUT {
    r#type: INPUT_KEYBOARD,
    Anonymous: INPUT_0 {
      ki: KEYBDINPUT {
        wVk: MASK_KEY,
        dwFlags: flags,
        dwExtraInfo: EVENT_TAG,
        ..Default::default()
      },
    },
  };
  let inputs = [key(Default::default()), key(KEYEVENTF_KEYUP)];
  // SAFETY: two keyboard inputs, sized as SendInput expects.
  unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
  if code == HC_ACTION as i32 {
    // SAFETY: for HC_ACTION the system passes a KBDLLHOOKSTRUCT it owns for
    // the duration of this call.
    let data = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
    let message = wparam.0 as u32;
    let pressed = matches!(message, WM_KEYDOWN | WM_SYSKEYDOWN);
    let released = matches!(message, WM_KEYUP | WM_SYSKEYUP);
    if (pressed || released) && data.dwExtraInfo != EVENT_TAG {
      let outcome = {
        let mut router = router();
        // SAFETY: reads the system's key state; any virtual key is valid.
        router.forget_released(data.vkCode, |key| unsafe {
          GetAsyncKeyState(key as i32) < 0
        });
        router.route(RawKey {
          key: data.vkCode,
          pressed,
          time: data.time,
          extended: data.flags.contains(LLKHF_EXTENDED),
          injected: data.flags.contains(LLKHF_INJECTED),
        })
      };
      if outcome.mask {
        // SAFETY: posts to this thread's own queue, read by its pump below.
        let _ =
          unsafe { PostThreadMessageW(GetCurrentThreadId(), WM_SEND_MASK, WPARAM(0), LPARAM(0)) };
      }
      if outcome.take {
        return LRESULT(1);
      }
    }
  }
  // SAFETY: passing the callback's own arguments along the hook chain.
  unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// Starts the hook thread the first time a handler registers. It runs for the
/// rest of the process: with no handlers the hook only passes keys on.
fn ensure_running() -> Result<(), String> {
  let mut started = HOOK_STARTED
    .lock()
    .unwrap_or_else(std::sync::PoisonError::into_inner);
  if *started {
    return Ok(());
  }
  let (installed, did_install) = mpsc::channel();
  std::thread::Builder::new()
    .name("screenwide-keyboard-hook".to_owned())
    .spawn(move || run(&installed))
    .map_err(|error| format!("Could not start the keyboard hook: {error}"))?;
  match did_install.recv_timeout(START_TIMEOUT) {
    Ok(Ok(())) => {
      *started = true;
      Ok(())
    }
    Ok(Err(error)) => Err(error),
    Err(_) => {
      // The thread may still install its hook; a second one would hand every
      // key to the handlers twice.
      *started = true;
      Err("The keyboard hook did not start in time".to_owned())
    }
  }
}

/// The hook thread: installs the hook and pumps messages so its callback can
/// run, sending the mask key when the callback asks.
fn run(installed: &mpsc::Sender<Result<(), String>>) {
  // SAFETY: a low-level keyboard hook takes no module handle, and zero asks
  // for every thread in the session.
  if let Err(error) = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), None, 0) } {
    let _ = installed.send(Err(format!("Could not install the keyboard hook: {error}")));
    return;
  }
  let _ = installed.send(Ok(()));
  let mut message = MSG::default();
  // The hook goes with the process; nothing posts WM_QUIT here.
  while unsafe { GetMessageW(&mut message, None, 0, 0) }.0 > 0 {
    if message.message == WM_SEND_MASK {
      send_mask_key();
      continue;
    }
    unsafe {
      let _ = TranslateMessage(&message);
      DispatchMessageW(&message);
    }
  }
}
