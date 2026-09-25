// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// An opaque colour from `#rgb`, `#rrggbb`, or `#gg` grey, with or without
/// the leading `#`.
pub(crate) fn parse_hex_colour(value: &str) -> Result<[u8; 4], String> {
  let value = value.strip_prefix('#').unwrap_or(value);
  if !matches!(value.len(), 2 | 3 | 6) || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
    return Err("The screenshot background colour is not valid".to_owned());
  }
  let expanded = match value.len() {
    2 => value.repeat(3),
    3 => value.chars().flat_map(|character| [character; 2]).collect(),
    _ => value.to_owned(),
  };
  let channel =
    |start| u8::from_str_radix(&expanded[start..start + 2], 16).map_err(|e| e.to_string());
  Ok([channel(0)?, channel(2)?, channel(4)?, u8::MAX])
}
