// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Handing frames to the encoder, and getting every one of them back.

use super::input::read_back;
use super::*;

impl Encoder {
  /// Hands one NV12 frame to the encoder. Whatever the encoder has finished
  /// by then is appended to `encoded`.
  pub(in super::super) fn encode(
    &mut self,
    texture: &ID3D11Texture2D,
    slot: usize,
    pts_100ns: i64,
    duration_100ns: i64,
    picture: Picture,
    encoded: &mut Vec<EncodedFrame>,
  ) -> Result<(), String> {
    let mut forced = false;
    if picture != Picture::Any && self.forced != ForcedKeyframes::Ignored {
      if let Some(codec) = &self.codec {
        forced = setup::force_keyframe(codec).is_ok();
      }
    }
    if !forced && picture == Picture::Keyframe && self.forced == ForcedKeyframes::Ignored {
      // A new encoder's first frame is always a keyframe.
      self.restart(encoded)?;
    }
    let sample = self.sample(texture, pts_100ns, duration_100ns)?;
    if self.events.is_some() {
      let deadline = Instant::now() + STALL_TIMEOUT;
      if !self.settle(encoded, deadline, |encoder| encoder.need_input > 0)? {
        return Err(format!("{} stopped asking for frames", self.name));
      }
      self.need_input -= 1;
      win(unsafe { self.transform.ProcessInput(self.streams.input, &sample, 0) })?;
    } else {
      self.pending_input(&sample, encoded)?;
    }
    if forced {
      self.forced_pts = Some(pts_100ns);
    }
    if matches!(self.input, Input::Gpu { .. }) {
      self.pending.push_back((pts_100ns, slot));
    }
    self.collect(encoded)
  }

  /// A synchronous encoder takes input once its output has been read.
  fn pending_input(
    &mut self,
    sample: &IMFSample,
    encoded: &mut Vec<EncodedFrame>,
  ) -> Result<(), String> {
    match unsafe { self.transform.ProcessInput(self.streams.input, sample, 0) } {
      Err(error) if error.code() == MF_E_NOTACCEPTING => {
        self.collect(encoded)?;
        win(unsafe { self.transform.ProcessInput(self.streams.input, sample, 0) })
      }
      result => win(result),
    }
  }

  /// Waits until every frame handed over has come back out.
  pub(in super::super) fn flush(&mut self, encoded: &mut Vec<EncodedFrame>) -> Result<(), String> {
    let deadline = Instant::now() + SETTLE_TIMEOUT;
    if self.settle(encoded, deadline, |encoder| encoder.pending.is_empty())? {
      return Ok(());
    }
    // An encoder that holds frames back for its own look-ahead gives them up
    // when drained, and carries on after.
    self.drain(encoded)
  }

  fn drain(&mut self, encoded: &mut Vec<EncodedFrame>) -> Result<(), String> {
    win(unsafe { self.transform.ProcessMessage(MFT_MESSAGE_COMMAND_DRAIN, 0) })?;
    if self.events.is_none() {
      self.collect(encoded)?;
      self.pending.clear();
      return Ok(());
    }
    self.draining = true;
    let deadline = Instant::now() + STALL_TIMEOUT;
    if !self.settle(encoded, deadline, |encoder| !encoder.draining)? {
      return Err(format!("{} did not finish its frames", self.name));
    }
    self.pending.clear();
    Ok(())
  }

  /// Replaces the encoder with a fresh one of the same kind, after taking
  /// every frame the old one still held.
  fn restart(&mut self, encoded: &mut Vec<EncodedFrame>) -> Result<(), String> {
    self.drain(encoded)?;
    self.shut_down();
    let candidate = std::mem::replace(&mut self.candidate, Candidate::Software);
    let mut fresh = Self::start(&self.device, candidate, self.size, self.name.clone())?;
    fresh.forced = self.forced;
    fresh.parameter_sets = self.parameter_sets.take().or(fresh.parameter_sets.take());
    *self = fresh;
    Ok(())
  }

  /// Polls the encoder until `done` holds or `deadline` passes, collecting
  /// what it hands back on the way. Polled, not blocked on, so a stuck
  /// encoder cannot hang the writer.
  fn settle(
    &mut self,
    encoded: &mut Vec<EncodedFrame>,
    deadline: Instant,
    done: impl Fn(&Self) -> bool,
  ) -> Result<bool, String> {
    loop {
      self.collect(encoded)?;
      if done(self) {
        return Ok(true);
      }
      if Instant::now() >= deadline {
        return Ok(false);
      }
      std::thread::sleep(POLL);
    }
  }

  fn sample(
    &self,
    texture: &ID3D11Texture2D,
    pts_100ns: i64,
    duration_100ns: i64,
  ) -> Result<IMFSample, String> {
    let buffer = match &self.input {
      Input::Gpu { .. } => {
        let buffer =
          win(unsafe { MFCreateDXGISurfaceBuffer(&ID3D11Texture2D::IID, texture, 0, false) })?;
        // A surface buffer starts empty; the encoder rejects it unless its
        // length says it holds the whole picture.
        let length = win(unsafe { win(buffer.cast::<IMF2DBuffer>())?.GetContiguousLength() })?;
        win(unsafe { buffer.SetCurrentLength(length) })?;
        buffer
      }
      Input::Cpu { context, staging } => read_back(context, staging, texture, self.size)?,
    };
    let sample = win(unsafe { MFCreateSample() })?;
    win(unsafe { sample.AddBuffer(&buffer) })?;
    win(unsafe { sample.SetSampleTime(pts_100ns) })?;
    win(unsafe { sample.SetSampleDuration(duration_100ns.max(1)) })?;
    Ok(sample)
  }
}
