// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The rate control the export's H.264 encoder is held to.

use ::windows::Win32::{
  Foundation::PROPERTYKEY,
  System::{Com::StructuredStorage::PROPVARIANT, Variant::VT_UI4},
  UI::Shell::PropertiesSystem::{IPropertyStore, PSCreateMemoryPropertyStore},
};

use super::*;

/// Constant bitrate at `bitrate`, for `MF_SINK_WRITER_ENCODER_CONFIG`.
///
/// Given only an average bitrate, each vendor's encoder picks its own rate
/// control: most hold it constant, some let it float with the picture. Held
/// constant on every GPU, an export comes out at the size its estimate
/// promised. The values must reach the encoder this way, before its media
/// types are negotiated: the Microsoft AVC DX12 encoder ignores codec API
/// values set later.
pub(super) fn constant_bitrate(bitrate: u32) -> Result<IPropertyStore, String> {
  let mut raw = std::ptr::null_mut();
  win(unsafe { PSCreateMemoryPropertyStore(&IPropertyStore::IID, &mut raw) })?;
  let store = unsafe { IPropertyStore::from_raw(raw) };
  for (api, value) in [
    (
      CODECAPI_AVEncCommonRateControlMode,
      eAVEncCommonRateControlMode_CBR.0 as u32,
    ),
    (CODECAPI_AVEncCommonMeanBitRate, bitrate),
  ] {
    let mut variant = PROPVARIANT::default();
    unsafe {
      (*variant.Anonymous.Anonymous).vt = VT_UI4;
      (*variant.Anonymous.Anonymous).Anonymous.ulVal = value;
    }
    let key = PROPERTYKEY { fmtid: api, pid: 0 };
    win(unsafe { store.SetValue(&key, &variant) })?;
  }
  Ok(store)
}
