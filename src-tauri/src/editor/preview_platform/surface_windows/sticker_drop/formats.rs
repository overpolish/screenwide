// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Reading a picture out of what a drag carries.
//!
//! A drag can offer a picture several ways at once, so it is read in the
//! order that keeps the most of it: an image file of its own, from Explorer;
//! a PNG, which keeps transparency; a file the dragging app offers without
//! one on disk, as browsers do for an image; and last a bitmap.

use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;

use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::HGLOBAL;
use windows::Win32::System::Com::{
  IDataObject, IStream, DVASPECT_CONTENT, FORMATETC, STGMEDIUM, TYMED, TYMED_HGLOBAL, TYMED_ISTREAM,
};
use windows::Win32::System::DataExchange::RegisterClipboardFormatW;
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
use windows::Win32::System::Ole::{ReleaseStgMedium, CF_DIB, CF_DIBV5, CF_HDROP};
use windows::Win32::UI::Shell::{DragQueryFileW, FILEDESCRIPTORW, FILEGROUPDESCRIPTORW, HDROP};

use crate::editor::stickers::dib::bmp_from_dib;
use crate::editor::stickers::dropped::DroppedPicture;
use crate::editor::stickers::import::PICTURE_EXTENSIONS;

/// The most a dropped picture's data is read to, so a stream that never
/// ends cannot take the memory with it.
const MOST_BYTES: usize = 256 << 20;

fn registered(name: PCWSTR) -> u16 {
  unsafe { RegisterClipboardFormatW(name) as u16 }
}

fn format(format: u16, tymed: TYMED, index: i32) -> FORMATETC {
  FORMATETC {
    cfFormat: format,
    ptd: std::ptr::null_mut(),
    dwAspect: DVASPECT_CONTENT.0,
    lindex: index,
    tymed: tymed.0 as u32,
  }
}

fn offers(data: &IDataObject, format: &FORMATETC) -> bool {
  unsafe { data.QueryGetData(format) }.is_ok()
}

fn is_picture_name(name: &str) -> bool {
  std::path::Path::new(name)
    .extension()
    .and_then(|extension| extension.to_str())
    .is_some_and(|extension| {
      PICTURE_EXTENSIONS
        .iter()
        .any(|known| known.eq_ignore_ascii_case(extension))
    })
}

fn global_bytes(global: HGLOBAL) -> Option<Vec<u8>> {
  let size = unsafe { GlobalSize(global) }.min(MOST_BYTES);
  let start = unsafe { GlobalLock(global) };
  if start.is_null() {
    return None;
  }
  let bytes = unsafe { std::slice::from_raw_parts(start.cast::<u8>(), size) }.to_vec();
  let _ = unsafe { GlobalUnlock(global) };
  Some(bytes)
}

fn stream_bytes(stream: &IStream) -> Option<Vec<u8>> {
  let mut bytes = Vec::new();
  let mut chunk = vec![0_u8; 64 << 10];
  while bytes.len() < MOST_BYTES {
    let mut read = 0_u32;
    let result = unsafe {
      stream.Read(
        chunk.as_mut_ptr().cast(),
        chunk.len() as u32,
        Some(&mut read),
      )
    };
    if result.is_err() || read == 0 {
      break;
    }
    bytes.extend_from_slice(&chunk[..read as usize]);
  }
  (!bytes.is_empty()).then_some(bytes)
}

/// What `format` holds, as bytes, from memory or a stream, the medium
/// released after.
fn read(data: &IDataObject, format: &FORMATETC) -> Option<Vec<u8>> {
  let mut medium: STGMEDIUM = unsafe { data.GetData(format) }.ok()?;
  let bytes = match TYMED(medium.tymed as i32) {
    TYMED_HGLOBAL => global_bytes(unsafe { medium.u.hGlobal }),
    TYMED_ISTREAM => unsafe { (*medium.u.pstm).as_ref() }.and_then(stream_bytes),
    _ => None,
  };
  unsafe { ReleaseStgMedium(&mut medium) };
  bytes
}

/// The image files the drag carries, from Explorer.
fn files(data: &IDataObject) -> Vec<PathBuf> {
  let Ok(mut medium) = (unsafe { data.GetData(&format(CF_HDROP.0, TYMED_HGLOBAL, -1)) }) else {
    return Vec::new();
  };
  let drop = HDROP(unsafe { medium.u.hGlobal }.0);
  let count = unsafe { DragQueryFileW(drop, u32::MAX, None) };
  let mut paths = Vec::new();
  for index in 0..count {
    let length = unsafe { DragQueryFileW(drop, index, None) } as usize;
    let mut name = vec![0_u16; length + 1];
    unsafe { DragQueryFileW(drop, index, Some(&mut name)) };
    let path = PathBuf::from(OsString::from_wide(&name[..length]));
    if path.to_str().is_some_and(is_picture_name) {
      paths.push(path);
    }
  }
  unsafe { ReleaseStgMedium(&mut medium) };
  paths
}

/// The first picture among the files the drag offers without them being on
/// disk, by the place it has in the drag's list.
fn virtual_picture(data: &IDataObject) -> Option<i32> {
  let group = registered(w!("FileGroupDescriptorW"));
  let bytes = read(data, &format(group, TYMED_HGLOBAL, -1))?;
  let count = u32::from_le_bytes(bytes.get(..4)?.try_into().ok()?) as usize;
  let first = std::mem::offset_of!(FILEGROUPDESCRIPTORW, fgd);
  let size = std::mem::size_of::<FILEDESCRIPTORW>();
  (0..count).find_map(|index| {
    let at = first + index * size;
    let entry = bytes.get(at..at + size)?;
    // The bytes of a list read out of shared memory carry no alignment, and
    // the descriptor is packed, so its name is copied out before it is read.
    let descriptor = unsafe { std::ptr::read_unaligned(entry.as_ptr().cast::<FILEDESCRIPTORW>()) };
    let file_name = descriptor.cFileName;
    let length = file_name
      .iter()
      .position(|unit| *unit == 0)
      .unwrap_or(file_name.len());
    let name = String::from_utf16_lossy(&file_name[..length]);
    is_picture_name(&name).then_some(index as i32)
  })
}

/// Memory or a stream, whichever the dragging app hands its data over in.
const EITHER: TYMED = TYMED(TYMED_HGLOBAL.0 | TYMED_ISTREAM.0);

/// Whether the drag holds a picture in a form one can be read from.
pub(super) fn holds_picture(data: &IDataObject) -> bool {
  !files(data).is_empty()
    || [
      registered(w!("PNG")),
      registered(w!("FileGroupDescriptorW")),
      CF_DIBV5.0,
      CF_DIB.0,
    ]
    .into_iter()
    .any(|kind| offers(data, &format(kind, EITHER, -1)))
}

/// The dragged picture, in the form that keeps the most of it.
pub(super) fn picture(data: &IDataObject) -> Option<DroppedPicture> {
  if let Some(path) = files(data).into_iter().next() {
    return Some(DroppedPicture::File(path));
  }
  let any = EITHER;
  if let Some(bytes) = read(data, &format(registered(w!("PNG")), any, -1)) {
    return Some(DroppedPicture::Data(bytes));
  }
  if let Some(index) = virtual_picture(data) {
    let contents = registered(w!("FileContents"));
    if let Some(bytes) = read(data, &format(contents, any, index)) {
      return Some(DroppedPicture::Data(bytes));
    }
  }
  [CF_DIBV5.0, CF_DIB.0]
    .into_iter()
    .find_map(|kind| read(data, &format(kind, TYMED_HGLOBAL, -1)))
    .and_then(|dib| bmp_from_dib(&dib))
    .map(DroppedPicture::Data)
}
