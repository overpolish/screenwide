// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Re-samples the pointer when Option/Alt changes. Unlike a mouse move, a
/// modifier transition carries no window for the display beneath the
/// pointer, so find the owning surface before replaying the move.
pub(super) fn modifier_changed(hwnd: WindowId, alt: bool) -> bool {
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
    modifier_bits(false, false, false, alt),
  );
  true
}

/// The ruler's keyboard phases first, then select-all and copy while a
/// recognition is ready.
pub(super) fn overlay_keyboard_command(
  hwnd: WindowId,
  vk: u16,
  modifiers: u32,
  repeat: bool,
) -> bool {
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
    return false;
  };
  let ready = context
    .surfaces
    .lock()
    .map(|mut set| {
      set
        .find_mut(hwnd)
        .is_some_and(|surface| surface.input_enabled && surface.ocr.phase == ocr::PHASE_READY)
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
  hwnd: WindowId,
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
  crate::text_recognition::settings::key_phase(vk, modifiers, ruler::MAC_KEYS, repeat)
}

/// The key-up half of a latched range, guide or radius key.
pub(super) fn keyboard_release(hwnd: WindowId, vk: u16) -> bool {
  let Some(context) = state::context_for_surface(hwnd) else {
    return false;
  };
  if !context.is_ruler() {
    return false;
  }
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

/// Port of `processKeyboardCommand`.
pub(super) fn ruler_keyboard_phase(context: &Context, hwnd: WindowId, phase: u32) -> bool {
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
