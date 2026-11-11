// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{mpsc, Arc};
use std::thread::JoinHandle;
use std::time::Duration;

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{
  GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
  VIRTUAL_KEY, VK_CONTROL, VK_LWIN, VK_MENU, VK_R, VK_RWIN, VK_S, VK_SHIFT, VK_SNAPSHOT, VK_T,
};
use windows::Win32::UI::WindowsAndMessaging::{
  CallNextHookEx, DispatchMessageW, GetMessageW, PostThreadMessageW, SetWindowsHookExW,
  TranslateMessage, UnhookWindowsHookEx, HC_ACTION, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_APP,
  WM_KEYDOWN, WM_KEYUP, WM_QUIT, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use super::{OnKey, SystemKey};

pub(super) const SUPPORTED: bool = true;

const START_TIMEOUT: Duration = Duration::from_secs(2);
/// Asks the hook thread to send the mask key, outside the hook callback.
const WM_MASK_START: u32 = WM_APP + 1;
/// A key no keyboard has and nothing answers. Releasing the Windows key with
/// nothing between it and its press opens Start; the shell never saw the
/// letter that was taken, so this stands in for it.
const MASK_KEY: VIRTUAL_KEY = VIRTUAL_KEY(0xE8);
/// Marks the mask key, so the hook passes it by. Spells "SWSK".
const EVENT_TAG: usize = 0x5357_534B;

#[derive(Clone, Copy, Default)]
struct Modifiers {
  alt: bool,
  control: bool,
  shift: bool,
  win: bool,
}

impl Modifiers {
  /// What is held as a key arrives. The hook runs before that key's own
  /// state is recorded, but every modifier pressed before it has been.
  fn held() -> Self {
    // SAFETY: reads the system's key state; any virtual key is valid.
    let down = |key: VIRTUAL_KEY| unsafe { GetAsyncKeyState(i32::from(key.0)) } < 0;
    Self {
      alt: down(VK_MENU),
      control: down(VK_CONTROL),
      shift: down(VK_SHIFT),
      win: down(VK_LWIN) || down(VK_RWIN),
    }
  }
}

/// The system's key `key` is part of under `held`, if any. Only the exact
/// combination counts: Alt+Print Screen still captures the window in front,
/// and Win+Ctrl+Shift+S is not Win+Shift+S.
fn matched(key: u32, held: Modifiers) -> Option<SystemKey> {
  let win_shift = held.win && held.shift && !held.control && !held.alt;
  let alone = !held.win && !held.shift && !held.control && !held.alt;
  match VIRTUAL_KEY(key as u16) {
    VK_SNAPSHOT if alone => Some(SystemKey::PrintScreen),
    VK_S if win_shift => Some(SystemKey::ScreenClip),
    VK_R if win_shift => Some(SystemKey::ScreenRecord),
    VK_T if win_shift => Some(SystemKey::TextExtractor),
    _ => None,
  }
}

struct HookState {
  on_key: OnKey,
  /// Keys whose press was taken, so their repeats and release are too.
  taken: HashSet<u32>,
}

thread_local! {
  /// Low-level hook callbacks run on the thread that installed the hook.
  /// Windows drops a hook that keeps the keyboard waiting past
  /// LowLevelHooksTimeout, so the callback only decides and posts.
  static HOOK_STATE: RefCell<Option<HookState>> = const { RefCell::new(None) };
}

/// Whether the hook keeps this event from the system.
fn takes(key: u32, pressed: bool) -> bool {
  HOOK_STATE.with(|state| {
    let mut state = state.borrow_mut();
    let Some(state) = state.as_mut() else {
      return false;
    };
    if !pressed {
      return state.taken.remove(&key);
    }
    if state.taken.contains(&key) {
      return true;
    }
    let Some(system_key) = matched(key, Modifiers::held()) else {
      return false;
    };
    if !(state.on_key)(system_key) {
      return false;
    }
    state.taken.insert(key);
    if system_key != SystemKey::PrintScreen {
      // SAFETY: posts to this thread's own queue, read by its pump below.
      let _ =
        unsafe { PostThreadMessageW(GetCurrentThreadId(), WM_MASK_START, WPARAM(0), LPARAM(0)) };
    }
    true
  })
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
  if code == HC_ACTION as i32 {
    // SAFETY: for HC_ACTION the system passes a KBDLLHOOKSTRUCT it owns for
    // the duration of this call.
    let data = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
    let message = wparam.0 as u32;
    let pressed = matches!(message, WM_KEYDOWN | WM_SYSKEYDOWN);
    let released = matches!(message, WM_KEYUP | WM_SYSKEYUP);
    // Injected input is not passed by as a rule: remote-desktop hosts send
    // the user's own typing that way.
    if data.dwExtraInfo != EVENT_TAG && (pressed || released) && takes(data.vkCode, pressed) {
      return LRESULT(1);
    }
  }
  // SAFETY: passing the callback's own arguments along the hook chain.
  unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

fn send_mask_key() {
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

/// The hook thread: installs the hook and pumps messages so its callback can
/// run, sending the mask key when the callback asks.
fn run(on_key: OnKey, thread_id: &AtomicU32, installed: &mpsc::Sender<Result<(), String>>) {
  // SAFETY: reads this thread's own identifier.
  thread_id.store(unsafe { GetCurrentThreadId() }, Ordering::Release);
  HOOK_STATE.with(|state| {
    *state.borrow_mut() = Some(HookState {
      on_key,
      taken: HashSet::new(),
    });
  });
  // SAFETY: a low-level keyboard hook takes no module handle, and zero asks
  // for every thread in the session.
  let hook = match unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), None, 0) } {
    Ok(hook) => hook,
    Err(error) => {
      HOOK_STATE.with(|state| *state.borrow_mut() = None);
      let _ = installed.send(Err(format!(
        "Could not take the system's screenshot keys: {error}"
      )));
      return;
    }
  };
  let _ = installed.send(Ok(()));

  let mut message = MSG::default();
  // GetMessageW returns zero for the WM_QUIT that dropping the hook posts,
  // and -1 on failure; both end the pump.
  while unsafe { GetMessageW(&mut message, None, 0, 0) }.0 > 0 {
    if message.message == WM_MASK_START {
      send_mask_key();
      continue;
    }
    unsafe {
      let _ = TranslateMessage(&message);
      DispatchMessageW(&message);
    }
  }

  // SAFETY: unhooking from the thread that installed the hook, once.
  let _ = unsafe { UnhookWindowsHookEx(hook) };
  HOOK_STATE.with(|state| *state.borrow_mut() = None);
}

pub(super) struct Hook {
  thread_id: Arc<AtomicU32>,
  worker: Option<JoinHandle<()>>,
}

impl Hook {
  pub(super) fn start(on_key: OnKey) -> Result<Self, String> {
    let thread_id = Arc::new(AtomicU32::new(0));
    let (installed, did_install) = mpsc::channel();
    let worker_thread_id = Arc::clone(&thread_id);
    let worker = std::thread::Builder::new()
      .name("screenwide-system-keys".to_owned())
      .spawn(move || run(on_key, &worker_thread_id, &installed))
      .map_err(|error| format!("Could not take the system's screenshot keys: {error}"))?;
    let mut hook = Self {
      thread_id,
      worker: Some(worker),
    };
    match did_install.recv_timeout(START_TIMEOUT) {
      Ok(Ok(())) => Ok(hook),
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
    let thread_id = self.thread_id.load(Ordering::Acquire);
    if thread_id != 0 {
      // SAFETY: WM_QUIT to the hook thread's own queue ends its pump.
      let _ = unsafe { PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) };
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
    let win_shift = Modifiers {
      shift: true,
      win: true,
      ..Modifiers::default()
    };
    assert_eq!(
      matched(u32::from(VK_S.0), win_shift),
      Some(SystemKey::ScreenClip)
    );
    assert_eq!(
      matched(
        u32::from(VK_S.0),
        Modifiers {
          control: true,
          ..win_shift
        }
      ),
      None
    );
    assert_eq!(
      matched(u32::from(VK_SNAPSHOT.0), Modifiers::default()),
      Some(SystemKey::PrintScreen)
    );
    // Alt+Print Screen still copies the window in front.
    assert_eq!(
      matched(
        u32::from(VK_SNAPSHOT.0),
        Modifiers {
          alt: true,
          ..Modifiers::default()
        }
      ),
      None
    );
  }
}
