// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn active_overlay() -> Option<Overlay> {
  match OVERLAY.load(Ordering::Acquire) {
    1 => Some(Overlay::Ruler),
    2 => Some(Overlay::TextRecognition),
    3 => Some(Overlay::Annotate),
    _ => None,
  }
}

pub(super) fn post_alt_transition(down: bool) -> bool {
  let previous = ALT_DOWN.swap(down, Ordering::AcqRel);
  if previous == down {
    return false;
  }
  let target = TARGET.load(Ordering::Acquire);
  let mut flags = FLAG_MODIFIER;
  if down {
    flags |= FLAG_ALT_DOWN;
  } else {
    flags |= FLAG_RELEASE;
  }
  if target != 0 {
    let _ = unsafe {
      PostMessageW(
        Some(HWND(target as *mut _)),
        OVERLAY_KEY_EVENT,
        WPARAM(VK_MENU.0 as usize),
        LPARAM(flags),
      )
    };
  }
  true
}

/// A bare Alt press enters Win32's menu-activation mode even when the app has
/// no visible menu. An unassigned key while Alt remains held cancels that mode
/// without releasing Alt or stealing its Ruler meaning. This is only needed
/// for virtual-input paths that update async state without reaching our hook.
pub(super) fn cancel_alt_menu_activation() {
  let key = KEYBDINPUT {
    wVk: VIRTUAL_KEY(0xe8),
    ..Default::default()
  };
  let inputs = [
    INPUT {
      r#type: INPUT_KEYBOARD,
      Anonymous: INPUT_0 { ki: key },
    },
    INPUT {
      r#type: INPUT_KEYBOARD,
      Anonymous: INPUT_0 {
        ki: KEYBDINPUT {
          dwFlags: KEYEVENTF_KEYUP,
          ..key
        },
      },
    },
  ];
  let _ = unsafe { SendInput(&inputs, size_of::<INPUT>() as i32) };
}

pub(super) unsafe extern "system" fn hook_proc(
  code: i32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  if code == HC_ACTION as i32 {
    let data = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
    let down = matches!(wparam.0 as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
    let up = matches!(wparam.0 as u32, WM_KEYUP | WM_SYSKEYUP);
    if down || up {
      let (modifiers, repeat) = update_pressed(data.vkCode, down);
      let overlay = active_overlay();
      let alt = overlay == Some(Overlay::Ruler) && matches!(data.vkCode, 0x12 | 0xa4 | 0xa5);
      if alt {
        LAST_ALT_HOOK.with(|at| *at.borrow_mut() = Some(Instant::now()));
        let _ = post_alt_transition(down);
        let target = TARGET.load(Ordering::Acquire);
        if target != 0 {
          return LRESULT(1);
        }
      }
      if overlay
        .is_some_and(|overlay| routes_to_overlay(overlay, data.vkCode, modifiers, repeat, down, up))
      {
        let target = TARGET.load(Ordering::Acquire);
        if target != 0 {
          let mut flags = 0;
          if modifiers & 2 != 0 {
            flags |= FLAG_COMMAND;
          }
          if modifiers & 8 != 0 {
            flags |= FLAG_SHIFT;
          }
          if modifiers & 4 != 0 {
            flags |= FLAG_ALT_DOWN;
          }
          if modifiers & 1 != 0 {
            flags |= FLAG_SUPER_DOWN;
          }
          if modifiers & 2 != 0 {
            flags |= FLAG_CONTROL_DOWN;
          }
          if repeat {
            flags |= FLAG_REPEAT;
          }
          if up {
            flags |= FLAG_RELEASE;
          }
          let posted = unsafe {
            PostMessageW(
              Some(HWND(target as *mut _)),
              OVERLAY_KEY_EVENT,
              WPARAM(data.vkCode as usize),
              LPARAM(flags),
            )
          }
          .is_ok();
          if posted {
            // Match the macOS event monitor: an overlay command is consumed
            // and does not type into whichever app was behind it.
            return LRESULT(1);
          }
        }
      }
    }
  }
  unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

pub(crate) fn alt_pressed() -> bool {
  ALT_DOWN.load(Ordering::Acquire)
}

pub(super) fn update_pressed(vk: u32, down: bool) -> (u32, bool) {
  PRESSED.with(|pressed| {
    let mut pressed = pressed.borrow_mut();
    let index = (vk as usize).min(pressed.len() - 1);
    let repeat = down && pressed[index];
    pressed[index] = down;
    let control = pressed[0x11] || pressed[0xa2] || pressed[0xa3];
    let shift = pressed[0x10] || pressed[0xa0] || pressed[0xa1];
    let alt = pressed[0x12] || pressed[0xa4] || pressed[0xa5];
    let super_key = pressed[0x5b] || pressed[0x5c];
    let modifiers = (u32::from(control) * 2)
      | (u32::from(shift) * 8)
      | (u32::from(alt) * 4)
      | u32::from(super_key);
    (modifiers, repeat)
  })
}

pub(super) fn routes_to_overlay(
  overlay: Overlay,
  vk: u32,
  modifiers: u32,
  repeat: bool,
  down: bool,
  up: bool,
) -> bool {
  if overlay == Overlay::Annotate {
    return down && annotate_consumes(vk, modifiers);
  }
  if overlay == Overlay::TextRecognition {
    return down
      && crate::text_recognition::settings::key_phase(vk as u16, modifiers, false, false)
        .is_some();
  }
  if matches!(vk, 0x12 | 0xa4 | 0xa5) {
    return down || up;
  }
  let index = (vk as usize).min(255);
  if up {
    return CONSUMED.with(|consumed| {
      let mut consumed = consumed.borrow_mut();
      let was_consumed = consumed[index];
      consumed[index] = false;
      was_consumed
    });
  }
  if !down {
    return false;
  }
  if CONSUMED.with(|consumed| consumed.borrow()[index]) {
    return true;
  }
  let matched =
    crate::ruler::settings::key_command(vk as u16, modifiers, false, repeat, false).is_some();
  if matched {
    CONSUMED.with(|consumed| consumed.borrow_mut()[index] = true);
  }
  matched
}

/// The only keys live annotation takes off the desktop: undo, the two clears
/// and the arrow tool. Escape and the activation shortcut are deliberately
/// absent - they belong to their own global registrations, and consuming them
/// here would leave no way out of the overlay.
fn annotate_consumes(vk: u32, modifiers: u32) -> bool {
  /// `update_pressed`'s bits, which are not the overlay protocol's.
  const CONTROL: u32 = 2;
  match vk {
    // Ctrl+Z alone. Ctrl+Shift+Z is redo, which the overlay has no notion of,
    // so it is left for whatever is behind it.
    0x5a => modifiers == CONTROL,
    // Backspace, Delete and A, each unmodified.
    0x08 | 0x2e | 0x41 => modifiers == 0,
    _ => false,
  }
}
