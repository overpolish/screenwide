// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The clipboard's formats as bytes: read to be put back, and written.

use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
  EmptyClipboard, EnumClipboardFormats, GetClipboardData, SetClipboardData,
};
use windows::Win32::System::Memory::{
  GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
};

/// Formats whose handles are not memory that can be copied: bitmaps,
/// metafiles, palettes and the owner's own drawing. Apps put bitmaps on the
/// clipboard as device-independent memory too, which is copied.
const UNCOPYABLE: &[u32] = &[2, 3, 9, 14, 0x80, 0x82, 0x83, 0x8E];
/// The range of formats whose handles are GDI objects.
const GDI_FORMATS: std::ops::RangeInclusive<u32> = 0x300..=0x3FF;
/// Larger formats are not kept: copying them would hold the paste up.
const LARGEST_KEPT: usize = 64 * 1024 * 1024;

/// Every format on the clipboard that is plain memory, as its bytes. The
/// clipboard must be open.
pub(super) fn held() -> Vec<(u32, Vec<u8>)> {
  let mut formats = Vec::new();
  let mut format = 0;
  loop {
    // Safety: the clipboard is open.
    format = unsafe { EnumClipboardFormats(format) };
    if format == 0 {
      break;
    }
    if UNCOPYABLE.contains(&format) || GDI_FORMATS.contains(&format) {
      continue;
    }
    // Safety: the clipboard is open, and the memory is only read while
    // locked.
    unsafe {
      let Ok(handle) = GetClipboardData(format) else {
        continue;
      };
      let memory = HGLOBAL(handle.0);
      let size = GlobalSize(memory);
      if size == 0 || size > LARGEST_KEPT {
        continue;
      }
      let data = GlobalLock(memory);
      if data.is_null() {
        continue;
      }
      formats.push((
        format,
        std::slice::from_raw_parts(data.cast::<u8>(), size).to_vec(),
      ));
      let _ = GlobalUnlock(memory);
    }
  }
  formats
}

/// Puts `bytes` on the open clipboard as `format`.
pub(super) fn set(format: u32, bytes: &[u8]) {
  // Safety: the memory is written while locked, and either handed to the
  // clipboard, which then owns it, or freed.
  unsafe {
    let Ok(memory) = GlobalAlloc(GMEM_MOVEABLE, bytes.len().max(1)) else {
      return;
    };
    let data = GlobalLock(memory);
    if data.is_null() {
      let _ = GlobalFree(Some(memory));
      return;
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), data.cast::<u8>(), bytes.len());
    let _ = GlobalUnlock(memory);
    if SetClipboardData(format, Some(HANDLE(memory.0))).is_err() {
      let _ = GlobalFree(Some(memory));
    }
  }
}

/// Puts back what the clipboard held. The clipboard must be open.
pub(super) fn put_back(held: &[(u32, Vec<u8>)]) {
  // Safety: the clipboard is open.
  if unsafe { EmptyClipboard() }.is_err() {
    return;
  }
  for (format, bytes) in held {
    set(*format, bytes);
  }
}
