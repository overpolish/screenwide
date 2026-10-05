// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the tray icon and its tooltip show: the recording state, or the
//! seconds left before a Delayed Screenshot.

use std::borrow::Cow;

use tauri::image::Image;

#[cfg(target_os = "windows")]
use super::icons;
use crate::recording::RecordingStatus;

/// One icon per second left, from one up to the longest delay Settings
/// offers. Drawn by `scripts/prepare-tray-countdown-icons.mjs`.
const COUNTDOWN: [&[u8]; 10] = [
  include_bytes!("../../icons/tray-countdown/1.png"),
  include_bytes!("../../icons/tray-countdown/2.png"),
  include_bytes!("../../icons/tray-countdown/3.png"),
  include_bytes!("../../icons/tray-countdown/4.png"),
  include_bytes!("../../icons/tray-countdown/5.png"),
  include_bytes!("../../icons/tray-countdown/6.png"),
  include_bytes!("../../icons/tray-countdown/7.png"),
  include_bytes!("../../icons/tray-countdown/8.png"),
  include_bytes!("../../icons/tray-countdown/9.png"),
  include_bytes!("../../icons/tray-countdown/10.png"),
];

/// A countdown only runs while idle. It is shown only then too, so a
/// recording started a moment before the countdown notices and stops is
/// never hidden behind it.
fn shown_countdown(status: RecordingStatus, countdown: Option<u8>) -> Option<u8> {
  countdown.filter(|_| status == RecordingStatus::Idle)
}

pub(super) fn icon(
  status: RecordingStatus,
  countdown: Option<u8>,
) -> tauri::Result<Image<'static>> {
  match shown_countdown(status, countdown) {
    Some(seconds) => countdown_icon(seconds),
    None => status_icon(status),
  }
}

pub(super) fn tooltip(status: RecordingStatus, countdown: Option<u8>) -> Cow<'static, str> {
  match shown_countdown(status, countdown) {
    Some(seconds) => Cow::Owned(format!("Screenwide - Screenshot in {seconds}s")),
    None => Cow::Borrowed(status_tooltip(status)),
  }
}

fn countdown_icon(seconds: u8) -> tauri::Result<Image<'static>> {
  let index = usize::from(seconds).clamp(1, COUNTDOWN.len()) - 1;
  let image = Image::from_bytes(COUNTDOWN[index])?;
  #[cfg(target_os = "windows")]
  let image = icons::apply_system_foreground(image);
  Ok(image)
}

#[cfg(target_os = "windows")]
fn status_icon(status: RecordingStatus) -> tauri::Result<Image<'static>> {
  let image = Image::from_bytes(match status {
    RecordingStatus::Idle => include_bytes!("../../icons/tray-default.ico").as_slice(),
    RecordingStatus::Starting | RecordingStatus::Stopping => {
      include_bytes!("../../icons/tray-loading.ico").as_slice()
    }
    RecordingStatus::Recording => include_bytes!("../../icons/tray-recording.ico").as_slice(),
    RecordingStatus::Paused => include_bytes!("../../icons/tray-paused.ico").as_slice(),
  })?;
  Ok(icons::apply_system_foreground(image))
}

#[cfg(not(target_os = "windows"))]
fn status_icon(status: RecordingStatus) -> tauri::Result<Image<'static>> {
  Image::from_bytes(match status {
    RecordingStatus::Idle => include_bytes!("../../icons/tray-default.png").as_slice(),
    RecordingStatus::Starting | RecordingStatus::Stopping => {
      include_bytes!("../../icons/tray-loading.png").as_slice()
    }
    RecordingStatus::Recording => include_bytes!("../../icons/tray-recording.png").as_slice(),
    RecordingStatus::Paused => include_bytes!("../../icons/tray-paused.png").as_slice(),
  })
}

const fn status_tooltip(status: RecordingStatus) -> &'static str {
  match status {
    RecordingStatus::Idle => "Screenwide",
    RecordingStatus::Starting => "Screenwide - Starting a recording",
    RecordingStatus::Recording => "Screenwide - Recording",
    RecordingStatus::Paused => "Screenwide - Recording paused",
    RecordingStatus::Stopping => "Screenwide - Finishing the recording",
  }
}
