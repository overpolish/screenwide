// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use windows::Win32::UI::Input::KeyboardAndMouse::{
  SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
  VIRTUAL_KEY, VK_CONTROL, VK_V,
};

use crate::Refusal;

/// Control-V. Its virtual keys follow the keyboard's layout, as Windows'
/// own paste does.
pub(super) fn press_paste() -> Result<(), Refusal> {
  let key = |key: VIRTUAL_KEY, flags: KEYBD_EVENT_FLAGS| INPUT {
    r#type: INPUT_KEYBOARD,
    Anonymous: INPUT_0 {
      ki: KEYBDINPUT {
        wVk: key,
        dwFlags: flags,
        ..Default::default()
      },
    },
  };
  let none = KEYBD_EVENT_FLAGS::default();
  let inputs = [
    key(VK_CONTROL, none),
    key(VK_V, none),
    key(VK_V, KEYEVENTF_KEYUP),
    key(VK_CONTROL, KEYEVENTF_KEYUP),
  ];
  #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
  let size = size_of::<INPUT>() as i32;
  // Safety: the inputs are whole keyboard events.
  let sent = unsafe { SendInput(&inputs, size) };
  if sent as usize == inputs.len() {
    Ok(())
  } else {
    Err(Refusal::Failed("Could not press the paste keys".to_owned()))
  }
}
