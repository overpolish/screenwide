// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::{Cell, RefCell};

use screenwide_keyboard_hook::{KeyEvent, Modifiers, Phase, Route};

use super::*;

thread_local! {
  /// Keys the Ruler took, so their repeats and release are posted to it too.
  /// Only the hook's thread routes keys, and tests their own.
  static CONSUMED: RefCell<[bool; 256]> = const { RefCell::new([false; 256]) };
  /// The overlay `CONSUMED` belongs to.
  static CONSUMED_FOR: Cell<u32> = const { Cell::new(0) };
}

pub(super) fn active_overlay() -> Option<Overlay> {
  match OVERLAY.load(Ordering::Acquire) {
    1 => Some(Overlay::Ruler),
    2 => Some(Overlay::TextRecognition),
    _ => None,
  }
}

pub(super) fn post_alt_transition(down: bool) -> bool {
  let previous = ALT_DOWN.swap(down, Ordering::AcqRel);
  if previous == down {
    return false;
  }
  let flags = FLAG_MODIFIER | if down { FLAG_ALT_DOWN } else { FLAG_RELEASE };
  post(u32::from(VK_MENU.0), flags);
  true
}

/// A bare Alt press enters Win32's menu-activation mode even when the app has
/// no visible menu. An unassigned key while Alt remains held cancels that mode
/// without releasing Alt or stealing its Ruler meaning. This is only needed
/// for virtual-input paths that update async state without reaching our hook.
pub(super) fn cancel_alt_menu_activation() {
  screenwide_keyboard_hook::send_mask_key();
}

/// The overlay's handler on the process's keyboard hook.
pub(super) fn route(event: &KeyEvent) -> Route {
  if !event.may_take && !event.owned {
    return Route::Pass;
  }
  let generation = GENERATION.load(Ordering::Acquire);
  if CONSUMED_FOR.replace(generation) != generation {
    CONSUMED.with(|consumed| consumed.borrow_mut().fill(false));
  }
  let vk = event.key;
  let up = event.phase == Phase::Release;
  let down = !up;
  let repeat = event.phase == Phase::Repeat;
  let modifiers = modifier_bits(with_own_key(event));
  let overlay = active_overlay();
  if overlay == Some(Overlay::Ruler) && matches!(vk, 0x12 | 0xa4 | 0xa5) {
    if let Ok(mut at) = LAST_ALT_HOOK.lock() {
      *at = Some(Instant::now());
    }
    let _ = post_alt_transition(down);
    if TARGET.load(Ordering::Acquire) != 0 {
      return Route::Take;
    }
  }
  if !overlay.is_some_and(|overlay| routes_to_overlay(overlay, vk, modifiers, repeat, down, up)) {
    return Route::Pass;
  }
  let mut flags = 0;
  if modifiers & 2 != 0 {
    flags |= FLAG_COMMAND | FLAG_CONTROL_DOWN;
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
  if repeat {
    flags |= FLAG_REPEAT;
  }
  if up {
    flags |= FLAG_RELEASE;
  }
  // Match the macOS event monitor: an overlay command is consumed and does
  // not type into whichever app was behind it.
  if post(vk, flags) {
    Route::Take
  } else {
    Route::Pass
  }
}

/// The modifiers with the event's own key applied, as the overlays' key
/// tables read them: a modifier counts from its own press.
fn with_own_key(event: &KeyEvent) -> Modifiers {
  let mut modifiers = event.modifiers;
  let down = event.phase != Phase::Release;
  match event.key {
    0x10 | 0xa0 | 0xa1 => modifiers.shift = down,
    0x11 | 0xa2 | 0xa3 => modifiers.control = down,
    0x12 | 0xa4 | 0xa5 => modifiers.alt = down,
    0x5b | 0x5c => modifiers.win = down,
    _ => {}
  }
  modifiers
}

fn modifier_bits(modifiers: Modifiers) -> u32 {
  (u32::from(modifiers.control) * 2)
    | (u32::from(modifiers.shift) * 8)
    | (u32::from(modifiers.alt) * 4)
    | u32::from(modifiers.win)
}

pub(crate) fn alt_pressed() -> bool {
  ALT_DOWN.load(Ordering::Acquire)
}

pub(super) fn routes_to_overlay(
  overlay: Overlay,
  vk: u32,
  modifiers: u32,
  repeat: bool,
  down: bool,
  up: bool,
) -> bool {
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
