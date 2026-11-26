// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Which microphone a session opens for the one chosen: that one while it is
//! connected, otherwise the system's default. A headset switched off then
//! leaves the computer's own microphone listening rather than nothing.

use cpal::traits::{DeviceTrait, HostTrait};

/// The id of the choice that follows the system: whichever microphone macOS
/// or Windows has as its default when a session opens, rather than one
/// microphone. The island offers it first in its list.
pub const SYSTEM_DEFAULT_MICROPHONE: &str = "default";

/// The id of the microphone `device_id` names while it is connected,
/// otherwise the system's default. Fails only when neither is there.
pub fn available_microphone(device_id: &str) -> Result<String, String> {
  let host = cpal::default_host();
  let listed = host
    .input_devices()
    .map_err(|error| error.to_string())?
    .filter_map(|device| device.id().ok().map(|id| id.to_string()))
    .collect::<Vec<_>>();
  let default = host
    .default_input_device()
    .and_then(|device| device.id().ok())
    .map(|id| id.to_string());
  choose(device_id, &listed, default)
}

/// [`available_microphone`] over the ids the system lists and its default.
fn choose(device_id: &str, listed: &[String], default: Option<String>) -> Result<String, String> {
  if device_id == SYSTEM_DEFAULT_MICROPHONE || listed.iter().any(|id| id == device_id) {
    return Ok(device_id.to_owned());
  }
  default.ok_or_else(|| {
    "The selected microphone is no longer available, and there is no default microphone".to_owned()
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  fn ids(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|&id| id.to_owned()).collect()
  }

  #[test]
  fn a_connected_choice_is_opened_as_it_is() {
    let chosen = choose(
      "headset",
      &ids(&["headset", "built-in"]),
      Some("built-in".to_owned()),
    );
    assert_eq!(chosen, Ok("headset".to_owned()));
  }

  #[test]
  fn a_choice_switched_off_gives_way_to_the_default() {
    let chosen = choose("headset", &ids(&["built-in"]), Some("built-in".to_owned()));
    assert_eq!(chosen, Ok("built-in".to_owned()));
  }

  #[test]
  fn following_the_system_is_kept_to_open_whichever_is_default() {
    let chosen = choose(SYSTEM_DEFAULT_MICROPHONE, &[], Some("built-in".to_owned()));
    assert_eq!(chosen, Ok(SYSTEM_DEFAULT_MICROPHONE.to_owned()));
  }

  #[test]
  fn with_no_microphone_at_all_it_fails() {
    assert!(choose("headset", &[], None).is_err());
  }
}
