// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the tray icon and its tooltip show: the recording state, the
//! seconds left before a Delayed Screenshot, that the replay buffer is on, or
//! that it just saved a clip.

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

/// Drawn by `scripts/prepare-tray-countdown-icons.mjs`.
const REPLAY: &[u8] = include_bytes!("../../icons/tray-replay.png");
const REPLAY_SAVED: &[u8] = include_bytes!("../../icons/tray-replay-saved.png");

/// What the tray shows, in order of precedence: a replay clip just saved,
/// then a recording's own state, then a Delayed Screenshot's countdown, then
/// a running replay buffer, then the plain mark. A save is confirmed over
/// everything, for the moment it shows: it is the only answer a save gets. A
/// countdown only runs while idle and is shown only then too, so a recording
/// started a moment before the countdown notices and stops is never hidden
/// behind it. The replay buffer runs beside everything, so it only ever shows
/// when nothing else has anything to say.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Shown {
  Countdown(u8),
  Replay,
  ReplaySaved,
  Status(RecordingStatus),
}

impl Shown {
  pub(super) fn of(
    status: RecordingStatus,
    countdown: Option<u8>,
    replay_on: bool,
    replay_saved: bool,
  ) -> Self {
    match (status, countdown) {
      _ if replay_saved => Self::ReplaySaved,
      (RecordingStatus::Idle, Some(seconds)) => Self::Countdown(seconds),
      (RecordingStatus::Idle, None) if replay_on => Self::Replay,
      _ => Self::Status(status),
    }
  }
}

/// The countdown and replay state are read here, on the main thread, rather
/// than passed in: a tick queued behind a cancellation then shows what is
/// current, not what was.
pub(super) fn apply(tray: &tauri::tray::TrayIcon, status: RecordingStatus) {
  let shown = Shown::of(
    status,
    crate::screenshots::delayed::remaining(),
    crate::recording::replay::is_on(tray.app_handle()),
    super::replay_saved::showing(),
  );
  if let Ok(icon) = icon(shown) {
    let _ = tray.set_icon(Some(icon));
  }
  #[cfg(target_os = "macos")]
  let _ = tray.set_icon_as_template(true);
  let _ = tray.set_tooltip(Some(tooltip(shown)));
}

pub(super) fn icon(shown: Shown) -> tauri::Result<Image<'static>> {
  match shown {
    Shown::Countdown(seconds) => countdown_icon(seconds),
    Shown::Replay => mask_icon(REPLAY),
    Shown::ReplaySaved => mask_icon(REPLAY_SAVED),
    Shown::Status(status) => status_icon(status),
  }
}

pub(super) fn tooltip(shown: Shown) -> Cow<'static, str> {
  match shown {
    Shown::Countdown(seconds) => Cow::Owned(format!("Screenwide - Screenshot in {seconds}s")),
    Shown::Replay => Cow::Borrowed("Screenwide - Replay buffer on"),
    Shown::ReplaySaved => Cow::Borrowed("Screenwide - Replay saved"),
    Shown::Status(status) => Cow::Borrowed(status_tooltip(status)),
  }
}

fn countdown_icon(seconds: u8) -> tauri::Result<Image<'static>> {
  let index = usize::from(seconds).clamp(1, COUNTDOWN.len()) - 1;
  mask_icon(COUNTDOWN[index])
}

/// A drawn state icon: a template image on macOS, recoloured to the taskbar's
/// foreground on Windows.
fn mask_icon(bytes: &'static [u8]) -> tauri::Result<Image<'static>> {
  let image = Image::from_bytes(bytes)?;
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

#[cfg(test)]
mod tests;
