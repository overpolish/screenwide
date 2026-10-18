// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where the user's cursor scheme keeps a cursor's file.

use windows::{
  core::{w, PCWSTR},
  Win32::Foundation::ERROR_SUCCESS,
  Win32::System::{
    Environment::ExpandEnvironmentStringsW,
    Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_SZ, RRF_ZEROONFAILURE},
  },
};

/// The scheme's file for `value_name`, or the stock `fallback_name` where the
/// scheme leaves it empty, as a NUL-terminated wide path.
pub(super) fn cursor_path(value_name: &str, fallback_name: &str) -> Option<Vec<u16>> {
  let value_name = value_name.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
  let flags = RRF_RT_REG_SZ | RRF_ZEROONFAILURE;
  let mut byte_length = 0;
  let read = |buffer: Option<*mut std::ffi::c_void>, length: &mut u32| unsafe {
    RegGetValueW(
      HKEY_CURRENT_USER,
      w!("Control Panel\\Cursors"),
      PCWSTR(value_name.as_ptr()),
      flags,
      None,
      buffer,
      Some(length),
    )
  };
  if read(None, &mut byte_length) != ERROR_SUCCESS {
    return None;
  }
  let mut value = vec![0_u16; (byte_length as usize).div_ceil(2).max(1)];
  if read(Some(value.as_mut_ptr().cast()), &mut byte_length) != ERROR_SUCCESS {
    return None;
  }
  let end = value
    .iter()
    .position(|character| *character == 0)
    .unwrap_or(value.len());
  value.truncate(end);
  if value.is_empty() {
    value = std::env::var_os("WINDIR")?
      .to_string_lossy()
      .encode_utf16()
      .chain("\\Cursors\\".encode_utf16())
      .chain(fallback_name.encode_utf16())
      .collect();
  }
  value.push(0);
  let expanded_length = unsafe { ExpandEnvironmentStringsW(PCWSTR(value.as_ptr()), None) };
  if expanded_length == 0 {
    return None;
  }
  let mut expanded = vec![0_u16; expanded_length as usize];
  let written =
    unsafe { ExpandEnvironmentStringsW(PCWSTR(value.as_ptr()), Some(expanded.as_mut_slice())) };
  if written == 0 || written > expanded_length {
    return None;
  }
  expanded.truncate(written as usize);
  Some(expanded)
}
