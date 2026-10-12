// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Finding an H.264 encoder and setting it up the way recordings are encoded.

use super::*;

/// An encoder that could be created: a hardware transform on the capture's
/// own adapter, or Microsoft's software encoder.
pub(super) enum Candidate {
  Hardware(IMFActivate),
  Software,
}

/// The hardware encoders on the adapter `device` lives on, best first, then
/// the software encoder.
pub(super) fn candidates(device: &ID3D11Device) -> Vec<Candidate> {
  let mut found = hardware(device).unwrap_or_else(|error| {
    eprintln!("Replay buffer found no hardware H.264 encoder: {error}");
    Vec::new()
  });
  found.push(Candidate::Software);
  found
}

fn hardware(device: &ID3D11Device) -> Result<Vec<Candidate>, String> {
  let dxgi = win(device.cast::<IDXGIDevice>())?;
  let adapter = win(unsafe { dxgi.GetAdapter() })?;
  let luid = win(unsafe { adapter.GetDesc() })?.AdapterLuid;
  let mut luid_bytes = [0_u8; 8];
  luid_bytes[..4].copy_from_slice(&luid.LowPart.to_le_bytes());
  luid_bytes[4..].copy_from_slice(&luid.HighPart.to_le_bytes());
  let attributes = attributes(1)?;
  win(unsafe { attributes.SetBlob(&MFT_ENUM_ADAPTER_LUID, &luid_bytes) })?;
  let input = MFT_REGISTER_TYPE_INFO {
    guidMajorType: MFMediaType_Video,
    guidSubtype: MFVideoFormat_NV12,
  };
  let output = MFT_REGISTER_TYPE_INFO {
    guidMajorType: MFMediaType_Video,
    guidSubtype: MFVideoFormat_H264,
  };
  let mut list = std::ptr::null_mut();
  let mut count = 0;
  win(unsafe {
    MFTEnum2(
      MFT_CATEGORY_VIDEO_ENCODER,
      MFT_ENUM_FLAG_HARDWARE | MFT_ENUM_FLAG_SORTANDFILTER,
      Some(&input),
      Some(&output),
      &attributes,
      &mut list,
      &mut count,
    )
  })?;
  if list.is_null() {
    return Ok(Vec::new());
  }
  // SAFETY: MFTEnum2 returned `count` owned references in a CoTaskMem array;
  // each is moved out once and the array itself is freed after.
  let found = (0..count as usize)
    .filter_map(|index| unsafe { std::ptr::read(list.add(index)) })
    .map(Candidate::Hardware)
    .collect();
  unsafe { CoTaskMemFree(Some(list.cast_const().cast())) };
  Ok(found)
}

pub(super) fn name(candidate: &Candidate) -> String {
  let Candidate::Hardware(activate) = candidate else {
    return "Microsoft H.264 software encoder".to_owned();
  };
  let mut value = PWSTR::null();
  let mut length = 0;
  if unsafe { activate.GetAllocatedString(&MFT_FRIENDLY_NAME_Attribute, &mut value, &mut length) }
    .is_err()
  {
    return "Hardware H.264 encoder".to_owned();
  }
  let name = unsafe { value.to_string() }.unwrap_or_default();
  unsafe { CoTaskMemFree(Some(value.0.cast_const().cast())) };
  name
}

pub(super) fn transform(candidate: &Candidate) -> Result<IMFTransform, String> {
  match candidate {
    Candidate::Hardware(activate) => win(unsafe { activate.ActivateObject::<IMFTransform>() }),
    Candidate::Software => {
      win(unsafe { CoCreateInstance(&CLSID_MSH264EncoderMFT, None, CLSCTX_INPROC_SERVER) })
    }
  }
}

/// Unlocks an asynchronous transform, returning its event generator.
pub(super) fn unlock(transform: &IMFTransform) -> Result<Option<IMFMediaEventGenerator>, String> {
  let Ok(attributes) = (unsafe { transform.GetAttributes() }) else {
    return Ok(None);
  };
  if unsafe { attributes.GetUINT32(&MF_TRANSFORM_ASYNC) }.unwrap_or(0) == 0 {
    return Ok(None);
  }
  win(unsafe { attributes.SetUINT32(&MF_TRANSFORM_ASYNC_UNLOCK, 1) })?;
  Ok(Some(win(transform.cast::<IMFMediaEventGenerator>())?))
}

/// The bind flags the transform needs its input textures to carry.
pub(super) fn input_bind_flags(transform: &IMFTransform) -> u32 {
  unsafe { transform.GetAttributes() }
    .and_then(|attributes| unsafe { attributes.GetUINT32(&MF_SA_D3D11_BINDFLAGS) })
    .unwrap_or(0)
}

pub(super) fn is_d3d11_aware(transform: &IMFTransform) -> bool {
  unsafe { transform.GetAttributes() }
    .and_then(|attributes| unsafe { attributes.GetUINT32(&MF_SA_D3D11_AWARE) })
    .is_ok_and(|aware| aware != 0)
}

fn set_codec_value(codec: &ICodecAPI, api: &GUID, value: u32) -> Result<(), String> {
  let mut variant = VARIANT::default();
  unsafe {
    let inner = &mut *variant.Anonymous.Anonymous;
    inner.vt = VT_UI4;
    inner.Anonymous.ulVal = value;
  }
  win(unsafe { codec.SetValue(api, &variant) })
}

/// The recording writer's encoder settings, with the two a replay needs on
/// top: frames come out in the order they go in, and promptly.
///
/// Set before the media types, because some encoders only honour codec
/// values given that early.
pub(super) fn configure_codec(codec: &ICodecAPI, fps: u32) {
  for (api, value, what) in [
    (
      CODECAPI_AVEncMPVGOPSize,
      (fps / 2).max(1),
      "keyframe interval",
    ),
    (CODECAPI_AVEncMPVDefaultBPictureCount, 0, "B-frame count"),
    (CODECAPI_AVLowLatencyMode, 1, "low-latency mode"),
  ] {
    // An encoder that does not offer a setting has nothing to take; low
    // latency already rules out B-frames on encoders without the count.
    if unsafe { codec.IsSupported(&api) }.is_err() {
      continue;
    }
    if let Err(error) = set_codec_value(codec, &api, value) {
      eprintln!("Replay buffer encoder did not take its {what}: {error}");
    }
  }
}

pub(super) fn force_keyframe(codec: &ICodecAPI) -> Result<(), String> {
  set_codec_value(codec, &CODECAPI_AVEncVideoForceKeyFrame, 1)
}

/// Picture size and rate, shared by the input and output types. Colour is
/// left untagged, as recordings leave it.
fn picture_type(subtype: GUID, size: Size) -> Result<IMFMediaType, String> {
  let media_type = win(unsafe { MFCreateMediaType() })?;
  let Size { width, height, fps } = size;
  win(unsafe { media_type.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video) })?;
  win(unsafe { media_type.SetGUID(&MF_MT_SUBTYPE, &subtype) })?;
  win(unsafe {
    media_type.SetUINT64(
      &MF_MT_FRAME_SIZE,
      (u64::from(width) << 32) | u64::from(height),
    )
  })?;
  win(unsafe { media_type.SetUINT64(&MF_MT_FRAME_RATE, (u64::from(fps) << 32) | 1) })?;
  win(unsafe { media_type.SetUINT64(&MF_MT_PIXEL_ASPECT_RATIO, (1_u64 << 32) | 1) })?;
  win(unsafe {
    media_type.SetUINT32(&MF_MT_INTERLACE_MODE, MFVideoInterlace_Progressive.0 as u32)
  })?;
  Ok(media_type)
}

/// The output type before the input type, as encoders require.
pub(super) fn set_types(
  transform: &IMFTransform,
  streams: Streams,
  size: Size,
) -> Result<(), String> {
  let output = picture_type(MFVideoFormat_H264, size)?;
  let bitrate = bitrate_bps(size.width, size.height, size.fps);
  win(unsafe {
    output.SetUINT32(
      &MF_MT_AVG_BITRATE,
      u32::try_from(bitrate).unwrap_or(u32::MAX),
    )
  })?;
  win(unsafe { output.SetUINT32(&MF_MT_MPEG2_PROFILE, eAVEncH264VProfile_High.0 as u32) })?;
  win(unsafe { output.SetUINT32(&MF_MT_MAX_KEYFRAME_SPACING, (size.fps / 2).max(1)) })?;
  win(unsafe { transform.SetOutputType(streams.output, &output, 0) })?;
  let input = picture_type(MFVideoFormat_NV12, size)?;
  win(unsafe { transform.SetInputType(streams.input, &input, 0) })
}

pub(super) fn streams(transform: &IMFTransform) -> Streams {
  let (mut input, mut output) = ([0_u32], [0_u32]);
  // E_NOTIMPL means fixed streams numbered from zero.
  if unsafe { transform.GetStreamIDs(&mut input, &mut output) }.is_err() {
    return Streams {
      input: 0,
      output: 0,
    };
  }
  Streams {
    input: input[0],
    output: output[0],
  }
}
