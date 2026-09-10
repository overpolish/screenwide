// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

mod keys;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binding {
  pub mac_key: u16,
  pub windows_key: u16,
  pub modifiers: u32,
}

pub fn parse(value: &str) -> Result<Binding, String> {
  let mut parts = value.split('+').collect::<Vec<_>>();
  let key = parts.pop().ok_or("Choose a key")?;
  let (mac_key, windows_key) =
    keys::codes(key).ok_or("That key is not supported for this overlay")?;
  let mut modifiers = 0;
  for part in parts {
    let flag = match part {
      "Super" | "Meta" | "Command" => 1,
      "Control" => 2,
      "Alt" => 4,
      "Shift" => 8,
      "CommandOrControl" => {
        if cfg!(target_os = "macos") {
          1
        } else {
          2
        }
      }
      _ => return Err("Unsupported shortcut modifier".to_owned()),
    };
    if modifiers & flag != 0 {
      return Err("Repeated shortcut modifier".to_owned());
    }
    modifiers |= flag;
  }
  Ok(Binding {
    mac_key,
    windows_key,
    modifiers,
  })
}
