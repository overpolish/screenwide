// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use windows::Win32::Media::MediaFoundation::{IMFAttributes, MFCreateAttributes};

pub(crate) fn win<T>(result: windows::core::Result<T>) -> Result<T, String> {
  result.map_err(|error| error.to_string())
}

pub(crate) fn attributes(capacity: u32) -> Result<IMFAttributes, String> {
  let mut value = None;
  win(unsafe { MFCreateAttributes(&mut value, capacity) })?;
  value.ok_or_else(|| "Media Foundation created no attributes".to_owned())
}
