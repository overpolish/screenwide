// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! An FFmpeg taking the matte's raw frames on its input and writing them as
//! H.264, the one codec every platform decodes; at this quality the edge
//! keeps its soft steps. A keyframe each second lets a preview moved to any
//! point decode at most a second of it to get there. There are no B-frames:
//! with them the file starts through an edit list, which Media Foundation
//! ignores, so Windows would read every matte frame two frames late.

use std::io::{BufWriter, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};

pub(super) struct Encoder {
  child: Child,
  input: Option<BufWriter<ChildStdin>>,
}

impl Encoder {
  pub(super) fn start(
    ffmpeg: &Path,
    (width, height): (u32, u32),
    frames_per_second: u32,
    output: &Path,
  ) -> Result<Self, String> {
    let rate = frames_per_second.max(1).to_string();
    let mut child = Command::new(ffmpeg)
      .args(["-hide_banner", "-nostdin", "-loglevel", "error"])
      .args(["-f", "rawvideo", "-pix_fmt", "gray"])
      .args(["-s", &format!("{width}x{height}"), "-r", &rate, "-i", "-"])
      .args(["-c:v", "libx264", "-preset", "veryfast", "-crf", "12"])
      .args(["-g", &rate, "-bf", "0"])
      .args(["-pix_fmt", "yuv420p", "-movflags", "+faststart", "-y"])
      .arg(output)
      .stdin(Stdio::piped())
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .spawn()
      .map_err(|error| format!("Could not write the camera's matte: {error}"))?;
    let input = child.stdin.take().map(BufWriter::new);
    Ok(Self { child, input })
  }

  pub(super) fn write(&mut self, frame: &[u8]) -> Result<(), String> {
    self
      .input
      .as_mut()
      .ok_or_else(|| "The encoder has closed".to_owned())?
      .write_all(frame)
      .map_err(|error| format!("Could not write the camera's matte: {error}"))
  }

  /// Closes the encoder's input, which ends the file, and waits for it.
  pub(super) fn finish(mut self) -> Result<(), String> {
    if let Some(mut input) = self.input.take() {
      input
        .flush()
        .map_err(|error| format!("Could not write the camera's matte: {error}"))?;
    }
    let status = self
      .child
      .wait()
      .map_err(|error| format!("Could not write the camera's matte: {error}"))?;
    if status.success() {
      Ok(())
    } else {
      Err("FFmpeg could not write the camera's matte".to_owned())
    }
  }
}

impl Drop for Encoder {
  /// An encoder dropped unfinished, on a failure, is stopped rather than
  /// left writing half a file.
  fn drop(&mut self) {
    if self.input.take().is_some() {
      let _ = self.child.kill();
      let _ = self.child.wait();
    }
  }
}
