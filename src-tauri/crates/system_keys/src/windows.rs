// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use screenwide_keyboard_hook::{Modifiers, Phase, Priority, Registration, Route};
use windows::Win32::UI::Input::KeyboardAndMouse::{VIRTUAL_KEY, VK_R, VK_S, VK_SNAPSHOT, VK_T};

use super::{OnKey, SystemKey};

pub(super) const SUPPORTED: bool = true;

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

/// The system keys' place on the process's keyboard hook. The hook keeps a
/// taken key's repeats and release from the system too, and sends the key
/// that stops releasing Windows from opening Start.
pub(super) struct Hook {
  _registration: Registration,
}

impl Hook {
  pub(super) fn start(on_key: OnKey) -> Result<Self, String> {
    let registration = screenwide_keyboard_hook::take(Priority::SystemKeys, move |event| {
      let taken = event.phase == Phase::Press
        && event.may_take
        && matched(event.key, event.modifiers).is_some_and(&on_key);
      if taken {
        Route::Take
      } else {
        Route::Pass
      }
    })
    .map_err(|error| format!("Could not take the system's screenshot keys: {error}"))?;
    Ok(Self {
      _registration: registration,
    })
  }
}

#[cfg(test)]
mod tests;
