// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn attributes(capacity: u32) -> Result<IMFAttributes, String> {
  let mut value = None;
  unsafe { MFCreateAttributes(&mut value, capacity) }.map_err(|error| error.to_string())?;
  value.ok_or_else(|| "Media Foundation created no attributes".to_owned())
}

pub(super) fn encoder_config(gop_frames: u32) -> Result<IPropertyStore, String> {
  let mut raw = std::ptr::null_mut();
  win(unsafe { PSCreateMemoryPropertyStore(&IPropertyStore::IID, &mut raw) })?;
  let store = unsafe { IPropertyStore::from_raw(raw) };
  let mut value = PROPVARIANT::default();
  unsafe {
    (*value.Anonymous.Anonymous).vt = VT_UI4;
    (*value.Anonymous.Anonymous).Anonymous.ulVal = gop_frames;
  }
  let key = PROPERTYKEY {
    fmtid: CODECAPI_AVEncMPVGOPSize,
    pid: 0,
  };
  win(unsafe { store.SetValue(&key, &value) })?;
  Ok(store)
}

/// Best-effort identification of the H.264 transform the sink writer chose,
/// so GOP behaviour can be tied to a vendor when a recording misbehaves.
pub(super) fn video_type(
  subtype: windows::core::GUID,
  width: u32,
  height: u32,
  fps: u32,
) -> Result<IMFMediaType, String> {
  let media_type = unsafe { MFCreateMediaType() }.map_err(|error| error.to_string())?;
  let packed_size = (u64::from(width) << 32) | u64::from(height);
  let packed_rate = (u64::from(fps) << 32) | 1;
  let square_pixels = (1_u64 << 32) | 1;
  win(unsafe { media_type.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video) })?;
  win(unsafe { media_type.SetGUID(&MF_MT_SUBTYPE, &subtype) })?;
  win(unsafe { media_type.SetUINT64(&MF_MT_FRAME_SIZE, packed_size) })?;
  win(unsafe { media_type.SetUINT64(&MF_MT_FRAME_RATE, packed_rate) })?;
  win(unsafe { media_type.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, square_pixels) })?;
  win(unsafe {
    media_type.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)
  })?;
  Ok(media_type)
}
