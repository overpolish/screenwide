// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Re-samples the pointer when Option/Alt changes. Unlike a mouse move, a
/// modifier transition carries no HWND for the display beneath the pointer,
/// so find the owning compositor surface before replaying the move.
pub(super) fn modifier_changed(hwnd: HWND, alt: bool) -> bool {
  let Some(context) = state::context_for_surface(hwnd) else {
    return false;
  };
  let Some(pointer) = super::super::surface::cursor_position() else {
    return false;
  };
  let target = context.surfaces.lock().ok().and_then(|mut set| {
    set
      .all_mut()
      .find(|surface| {
        surface.input_enabled && surface.visible && surface.contains_screen_point(pointer)
      })
      .map(|surface| {
        let (x, y) = surface.screen_to_client(pointer.x, pointer.y);
        (surface.hwnd(), x, y)
      })
  });
  let Some((target, x, y)) = target else {
    return false;
  };
  process(
    &context,
    target,
    PHASE_MOVE,
    x,
    y,
    modifier_bits(0, false, alt),
  );
  true
}

/// The ruler's keyboard phases first, then Ctrl+A / Ctrl+C while a recognition
/// is ready - the Windows spelling of the macOS key-down monitor
/// (`+input.m:472-581`).
pub(super) fn keyboard_command(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> bool {
  let control = (unsafe { GetKeyState(VK_CONTROL.0 as i32) }) < 0;
  let shift = (unsafe { GetKeyState(VK_SHIFT.0 as i32) }) < 0;
  let alt = (unsafe { GetKeyState(VK_MENU.0 as i32) }) < 0;
  let super_key = (unsafe { GetKeyState(VK_LWIN.0 as i32) }) < 0
    || (unsafe { GetKeyState(VK_RWIN.0 as i32) }) < 0;
  let modifiers =
    (u32::from(control) * 2) | (u32::from(shift) * 8) | (u32::from(alt) * 4) | u32::from(super_key);
  // Bit 30 of `lparam` is set when this key-down is an auto-repeat.
  let repeat = lparam.0 & (1 << 30) != 0;
  overlay_keyboard_command(hwnd, wparam.0 as u16, modifiers, repeat)
}

pub(super) fn overlay_keyboard_command(hwnd: HWND, vk: u16, modifiers: u32, repeat: bool) -> bool {
  let Some(context) = state::context_for_surface(hwnd) else {
    return false;
  };
  if context.is_ruler() {
    return ruler_keyboard_command(&context, hwnd, vk, modifiers, repeat);
  }
  if !context.is_text_recognition() {
    return false;
  }
  let Some(phase) = ocr_keyboard_phase(vk, modifiers, repeat) else {
    eprintln!("OCR key: vk 0x{vk:x} modifiers {modifiers} repeat {repeat} matches no shortcut");
    return false;
  };
  let ready = context
    .surfaces
    .lock()
    .map(|mut set| {
      let surface = set.find_mut(hwnd);
      let state = surface
        .as_ref()
        .map(|surface| (surface.input_enabled, surface.ocr.phase));
      eprintln!("OCR key: phase {phase} for surface {hwnd:?}, input/ocr phase {state:?}");
      surface.is_some_and(|surface| surface.input_enabled && surface.ocr.phase == ocr::PHASE_READY)
    })
    .unwrap_or(false);
  if !ready {
    return false;
  }
  state::dispatch_input(&context, phase, Point::default(), 0);
  true
}

pub(super) fn ruler_keyboard_command(
  context: &Context,
  hwnd: HWND,
  vk: u16,
  modifiers: u32,
  repeat: bool,
) -> bool {
  let latched = context
    .ruler
    .lock()
    .map(|session| session.latched())
    .unwrap_or(false);
  let Some(key) = ruler::key_command(vk, modifiers, repeat, latched) else {
    return false;
  };
  if !ruler_keyboard_phase(context, hwnd, key.phase) {
    return false;
  }
  // A held key latches so the same physical press cannot re-fire and so its
  // key-up knows which release phase to send.
  if key.release.is_some() {
    if let Ok(mut session) = context.ruler.lock() {
      match key.phase {
        20 | 21 => session.range_key = vk,
        26 | 27 => session.guide_key = vk,
        31 => session.radius_key = vk,
        _ => {}
      }
    }
  }
  true
}

pub(super) fn ocr_keyboard_phase(vk: u16, modifiers: u32, repeat: bool) -> Option<u32> {
  crate::text_recognition::settings::key_phase(vk, modifiers, false, repeat)
}

/// The key-up half of a latched range, guide or radius key (`+input.m:440-471`).
pub(super) fn keyboard_release(hwnd: HWND, wparam: WPARAM) -> bool {
  let Some(context) = state::context_for_surface(hwnd) else {
    return false;
  };
  if !context.is_ruler() {
    return false;
  }
  let vk = wparam.0 as u16;
  let phase = {
    let Ok(mut session) = context.ruler.lock() else {
      return false;
    };
    if session.range_key == vk {
      session.range_key = 0;
      22
    } else if session.guide_key == vk {
      session.guide_key = 0;
      28
    } else if session.radius_key == vk {
      session.radius_key = 0;
      32
    } else {
      return false;
    }
  };
  ruler_keyboard_phase(&context, hwnd, phase);
  true
}

/// Port of `processKeyboardCommand` (`+input.m:299-319`).
pub(super) fn ruler_keyboard_phase(context: &Context, hwnd: HWND, phase: u32) -> bool {
  let live = context
    .surfaces
    .lock()
    .map(|mut set| {
      set
        .find_mut(hwnd)
        .is_some_and(|surface| surface.input_enabled && surface.visible)
    })
    .unwrap_or(false);
  if !live {
    return false;
  }
  let result = state::dispatch_input(context, phase, Point::default(), 0);
  if result.status == STATUS_INVALID {
    return false;
  }
  state::apply_ruler_result(context, &result);
  apply_result_cursor(context, hwnd, &result);
  true
}
