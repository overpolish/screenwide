// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Failures a debug build can be told to fake, so every error path the user
//! can meet is reachable on any machine without the hardware that causes it.
//!
//! `SCREENWIDE_FAULT` holds a comma-separated list of the names below, for
//! example `SCREENWIDE_FAULT=camera-mode,stop pnpm tauri dev`. A release
//! build never reads it: there `active` is a constant `false`.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Fault {
  /// The selected camera mode reads as no longer offered, to the bar's
  /// warning and to the start alike.
  CameraMode,
  /// The selected microphone reads as disconnected at the start.
  Microphone,
  /// A recording's capture fails to open.
  RecordingStart,
  /// A running recording reports a capture failure once it is under way.
  Capture,
  /// A recording fails to finish when stopped.
  Stop,
  /// The replay buffer's capture fails to open.
  ReplayStart,
}

#[cfg(debug_assertions)]
impl Fault {
  const ALL: [(Self, &'static str); 6] = [
    (Self::CameraMode, "camera-mode"),
    (Self::Microphone, "microphone"),
    (Self::RecordingStart, "recording-start"),
    (Self::Capture, "capture"),
    (Self::Stop, "stop"),
    (Self::ReplayStart, "replay-start"),
  ];
}

#[cfg(debug_assertions)]
pub(crate) fn active(fault: Fault) -> bool {
  use std::sync::OnceLock;

  static FAULTS: OnceLock<Vec<Fault>> = OnceLock::new();
  FAULTS
    .get_or_init(|| {
      let requested = std::env::var("SCREENWIDE_FAULT").unwrap_or_default();
      let faults: Vec<Fault> = requested
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .filter_map(|name| {
          let found = Fault::ALL.iter().find(|(_, known)| *known == name);
          if found.is_none() {
            eprintln!("SCREENWIDE_FAULT: unknown fault {name:?}");
          }
          found.map(|(fault, _)| *fault)
        })
        .collect();
      if !faults.is_empty() {
        eprintln!("SCREENWIDE_FAULT: faking {faults:?}");
      }
      faults
    })
    .contains(&fault)
}

#[cfg(not(debug_assertions))]
pub(crate) const fn active(_fault: Fault) -> bool {
  false
}

/// The error a faked failure reports, in place of the real one.
pub(crate) fn message(fault: Fault) -> String {
  format!("{fault:?} failed (faked by SCREENWIDE_FAULT)")
}
