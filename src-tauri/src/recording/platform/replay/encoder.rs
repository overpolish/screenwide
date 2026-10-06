// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The replay buffer's video encoder: a VideoToolbox session whose output is
//! kept rather than written.
//!
//! AVAssetWriter cannot hand back what it encoded, which is why the buffer
//! drives VideoToolbox itself. The settings match a working recording's, so a
//! saved clip edits and scrubs the same: the same bitrate rule, and a keyframe
//! every half second, which is also how finely a clip's start can be placed.
//! Frames are never reordered, so a clip can be cut and retimed without
//! decode timestamps to keep straight.

use std::ffi::c_void;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use cidre::{arc, cf, cm, cv, os, vt};

use super::super::media::{nanos, time_to_ns, VideoEncoder};
use super::SharedSample;
use crate::recording::encoding::bitrate_bps;
use crate::recording::replay::ring::{EncodedFrame, VideoRing};

/// What the session's callback writes into. Boxed and owned by the encoder
/// so its address stays put for as long as the session can call back.
struct Output {
  dropped: AtomicU64,
  /// Set to each keyframe's timestamp as it lands, for a follower writer to
  /// line its own keyframes up with.
  keyframes: Arc<AtomicI64>,
  ring: Arc<Mutex<VideoRing<SharedSample>>>,
}

pub(super) struct Encoder {
  output: Box<Output>,
  session: arc::R<vt::CompressionSession>,
}

extern "C" fn encoded(
  output: *mut Output,
  _frame: *mut c_void,
  status: os::Status,
  flags: vt::EncodeInfoFlags,
  sample: Option<&cm::SampleBuf>,
) {
  // SAFETY: the session is invalidated before the box it was given is
  // dropped (see `Drop for Encoder`), so the pointer is live for every call.
  let output = unsafe { &*output };
  let Some(sample) =
    sample.filter(|_| status.is_ok() && !flags.contains(vt::EncodeInfoFlags::FRAME_DROPPED))
  else {
    output.dropped.fetch_add(1, Ordering::Relaxed);
    return;
  };
  let Some(pts_ns) = time_to_ns(sample.pts()) else {
    output.dropped.fetch_add(1, Ordering::Relaxed);
    return;
  };
  let keyframe = sample.is_key_frame();
  output
    .ring
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .push(EncodedFrame {
      keyframe,
      pts_ns,
      sample: SharedSample(sample.retained()),
    });
  if keyframe {
    output.keyframes.fetch_max(pts_ns, Ordering::AcqRel);
  }
}

impl Encoder {
  pub(super) fn new(
    width: u32,
    height: u32,
    fps: u32,
    codec: VideoEncoder,
    ring: Arc<Mutex<VideoRing<SharedSample>>>,
    keyframes: Arc<AtomicI64>,
  ) -> Result<Self, String> {
    let mut output = Box::new(Output {
      dropped: AtomicU64::new(0),
      keyframes,
      ring,
    });
    let mut session = vt::CompressionSession::new(
      width,
      height,
      match codec {
        VideoEncoder::H264 => cm::VideoCodec::H264,
        VideoEncoder::Hevc => cm::VideoCodec::HEVC,
      },
      None,
      None,
      None,
      Some(encoded),
      &mut *output as *mut Output,
    )
    .map_err(|error| format!("The replay encoder could not be created: {error:?}"))?;

    let fps = fps.max(1);
    let bitrate = cf::Number::from_i32(bitrate_bps(width, height, fps));
    let frame_rate = cf::Number::from_i32(fps as i32);
    let key_frame_interval = cf::Number::from_i32((fps / 2).max(1) as i32);
    let key_frame_duration = cf::Number::from_f64(0.5);
    let profile = match codec {
      VideoEncoder::H264 => vt::compression_props::profile_level::h264::high_auto_lvl(),
      VideoEncoder::Hevc => vt::compression_props::profile_level::hevc::main_auto_lvl(),
    };
    let properties: [(&cf::String, &cf::Type); 7] = [
      (
        vt::compression_props_keys::real_time(),
        cf::Boolean::value_true().as_type_ref(),
      ),
      (
        vt::compression_props_keys::allow_frame_reordering(),
        cf::Boolean::value_false().as_type_ref(),
      ),
      (
        vt::compression_props_keys::avarage_bit_rate(),
        bitrate.as_type_ref(),
      ),
      (
        vt::compression_props_keys::expected_frame_rate(),
        frame_rate.as_type_ref(),
      ),
      (
        vt::compression_props_keys::max_key_frame_interval(),
        key_frame_interval.as_type_ref(),
      ),
      (
        vt::compression_props_keys::max_key_frame_interval_duration(),
        key_frame_duration.as_type_ref(),
      ),
      (
        vt::compression_props_keys::profile_lvl(),
        profile.as_type_ref(),
      ),
    ];
    for (key, value) in properties {
      session
        .set_prop(key, Some(value))
        .map_err(|error| format!("The replay encoder refused a setting: {error:?}"))?;
    }
    session
      .prepare()
      .map_err(|error| format!("The replay encoder could not start: {error:?}"))?;
    Ok(Self { output, session })
  }

  /// Encodes one frame at `pts_ns`, a keyframe if `keyframe` asks for one.
  pub(super) fn encode(
    &self,
    buf: &cv::PixelBuf,
    pts_ns: i64,
    keyframe: bool,
  ) -> Result<(), String> {
    let force = keyframe.then(|| {
      cf::DictionaryOf::with_keys_values(
        &[vt::compression_props::frame_keys::force_key_frame()],
        &[cf::Boolean::value_true().as_type_ref()],
      )
    });
    self
      .session
      .encode_frame(
        buf,
        nanos(pts_ns),
        cm::Time::invalid(),
        force.as_deref(),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
      )
      .map_err(|error| format!("The replay encoder refused a frame: {error:?}"))
  }

  /// Waits until every frame handed over has reached the ring.
  pub(super) fn flush(&self) -> Result<(), String> {
    self
      .session
      .complete_all()
      .map_err(|error| format!("The replay encoder could not finish its frames: {error:?}"))
  }

  pub(super) fn dropped(&self) -> u64 {
    self.output.dropped.load(Ordering::Relaxed)
  }
}

impl Drop for Encoder {
  fn drop(&mut self) {
    let _ = self.session.complete_all();
    self.session.invalidate();
  }
}
