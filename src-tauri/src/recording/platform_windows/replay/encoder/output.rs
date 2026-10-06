// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the encoder hands back, as frames the replay ring can keep.

use std::sync::Arc;

use super::super::annexb;
use super::*;

/// An attribute blob, such as the sequence header an encoder publishes on
/// its output type.
pub(super) fn blob(attributes: &IMFMediaType, key: &GUID) -> Option<Vec<u8>> {
  let length = unsafe { attributes.GetBlobSize(key) }.ok()?;
  let mut data = vec![0_u8; length as usize];
  unsafe { attributes.GetBlob(key, &mut data, None) }.ok()?;
  Some(data)
}

impl Encoder {
  /// Takes every event the encoder has raised so far, and every frame it has
  /// ready. Never waits.
  pub(in super::super) fn collect(
    &mut self,
    encoded: &mut Vec<EncodedFrame>,
  ) -> Result<(), String> {
    let Some(events) = self.events.clone() else {
      while self.output(encoded)? {}
      return Ok(());
    };
    loop {
      let event = match unsafe { events.GetEvent(MF_EVENT_FLAG_NO_WAIT) } {
        Ok(event) => event,
        Err(error) if error.code() == MF_E_NO_EVENTS_AVAILABLE => return Ok(()),
        Err(error) => return Err(format!("{} stopped: {error}", self.name)),
      };
      let kind = MF_EVENT_TYPE(win(unsafe { event.GetType() })? as i32);
      if kind == METransformNeedInput {
        self.need_input += 1;
      } else if kind == METransformHaveOutput {
        self.output(encoded)?;
      } else if kind == METransformDrainComplete {
        self.draining = false;
      } else if kind == MEError {
        let status = unsafe { event.GetStatus() }.unwrap_or_default();
        return Err(format!(
          "{} failed: {}",
          self.name,
          windows::core::Error::from_hresult(status)
        ));
      }
    }
  }

  /// Reads one frame from the encoder, reporting whether there was one.
  fn output(&mut self, encoded: &mut Vec<EncodedFrame>) -> Result<bool, String> {
    let sample = if self.provides_samples {
      None
    } else {
      let sample = win(unsafe { MFCreateSample() })?;
      let buffer = win(unsafe { MFCreateMemoryBuffer(self.output_size) })?;
      win(unsafe { sample.AddBuffer(&buffer) })?;
      Some(sample)
    };
    let mut buffers = [MFT_OUTPUT_DATA_BUFFER {
      dwStreamID: self.streams.output,
      pSample: std::mem::ManuallyDrop::new(sample),
      dwStatus: 0,
      pEvents: std::mem::ManuallyDrop::new(None),
    }];
    let mut status = 0;
    let result = unsafe { self.transform.ProcessOutput(0, &mut buffers, &mut status) };
    // SAFETY: each field is taken exactly once, so the references the
    // encoder handed back are released here and nowhere else.
    let (sample, _events) = unsafe {
      (
        std::mem::ManuallyDrop::take(&mut buffers[0].pSample),
        std::mem::ManuallyDrop::take(&mut buffers[0].pEvents),
      )
    };
    match result {
      Ok(()) => {}
      Err(error) if error.code() == MF_E_TRANSFORM_NEED_MORE_INPUT => return Ok(false),
      Err(error) if error.code() == MF_E_TRANSFORM_STREAM_CHANGE => {
        let changed = win(unsafe {
          self
            .transform
            .GetOutputAvailableType(self.streams.output, 0)
        })?;
        win(unsafe {
          self
            .transform
            .SetOutputType(self.streams.output, &changed, 0)
        })?;
        if let Some(header) =
          blob(&changed, &MF_MT_MPEG_SEQUENCE_HEADER).filter(|header| !header.is_empty())
        {
          self.parameter_sets = Some(header);
        }
        return Ok(true);
      }
      Err(error) => {
        return Err(format!(
          "{} could not hand back a frame: {error}",
          self.name
        ))
      }
    }
    let Some(sample) = sample else {
      return Ok(true);
    };
    self.keep(&sample, encoded)?;
    Ok(true)
  }

  fn keep(&mut self, sample: &IMFSample, encoded: &mut Vec<EncodedFrame>) -> Result<(), String> {
    let pts_100ns = win(unsafe { sample.GetSampleTime() })?;
    while self
      .pending
      .front()
      .is_some_and(|&(pts, _)| pts <= pts_100ns)
    {
      self.pending.pop_front();
    }
    let buffer = win(unsafe { sample.ConvertToContiguousBuffer() })?;
    let mut data = std::ptr::null_mut();
    let mut length = 0;
    win(unsafe { buffer.Lock(&mut data, None, Some(&mut length)) })?;
    // SAFETY: the locked buffer holds `length` valid bytes until Unlock.
    let bytes = unsafe { std::slice::from_raw_parts(data, length as usize) };
    let kept = self.frame(sample, pts_100ns, bytes);
    let _ = unsafe { buffer.Unlock() };
    encoded.extend(kept);
    Ok(())
  }

  fn frame(&mut self, sample: &IMFSample, pts_100ns: i64, bytes: &[u8]) -> Option<EncodedFrame> {
    if bytes.is_empty() {
      return None;
    }
    let keyframe = match unsafe { sample.GetUINT32(&MFSampleExtension_CleanPoint) } {
      Ok(clean) => clean != 0,
      Err(_) => annexb::is_idr(bytes),
    };
    if self.forced_pts == Some(pts_100ns) {
      self.forced_pts = None;
      let forced = if keyframe {
        ForcedKeyframes::Honoured
      } else {
        ForcedKeyframes::Ignored
      };
      if forced != self.forced {
        eprintln!(
          "Replay buffer: {} {} keyframe requests",
          self.name,
          if keyframe { "honours" } else { "ignores" }
        );
        self.forced = forced;
      }
    }
    // A clip starts on a keyframe, so each one has to carry the parameter
    // sets; an encoder that only wrote them once has them added here.
    let prefix = match keyframe.then(|| annexb::parameter_sets(bytes)) {
      Some(Some(sets)) => {
        self.parameter_sets = Some(sets);
        None
      }
      Some(None) => self.parameter_sets.as_deref(),
      None => None,
    };
    let sample: Arc<[u8]> = match prefix {
      Some(prefix) => [prefix, bytes].concat().into(),
      None => Arc::from(bytes),
    };
    Some(EncodedFrame {
      keyframe,
      pts_ns: pts_100ns.saturating_mul(100),
      sample,
    })
  }
}
