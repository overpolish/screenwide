// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The camera's frames as FFmpeg decodes them, each with the time it shows
//! from: every frame the camera holds, none dropped or repeated, so the
//! matte has one frame for each camera frame however unsteady its timing.

use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::mpsc::{self, Receiver};

pub(super) struct Decoder {
  child: Child,
  frames: BufReader<ChildStdout>,
  /// Each frame's time in seconds, told by FFmpeg's `showinfo` filter as it
  /// passes the frame on.
  times: Receiver<f64>,
}

impl Decoder {
  /// The camera at `camera` from `from_ms` on, scaled to `size`. Its
  /// timestamps are kept as recorded, so a frame's time is the camera's own.
  pub(super) fn start(
    ffmpeg: &Path,
    camera: &Path,
    from_ms: u64,
    (width, height): (u32, u32),
  ) -> Result<Self, String> {
    let mut command = Command::new(ffmpeg);
    command.args(["-hide_banner", "-nostdin", "-loglevel", "info"]);
    if from_ms > 0 {
      command.args(["-ss", &format!("{:.3}", from_ms as f64 / 1_000.0)]);
    }
    let mut child = command
      .args(["-copyts", "-i"])
      .arg(camera)
      .args(["-map", "0:v:0", "-an", "-vf"])
      .arg(format!("scale={width}:{height}:flags=area,showinfo"))
      .args([
        "-fps_mode",
        "passthrough",
        "-f",
        "rawvideo",
        "-pix_fmt",
        "rgb24",
        "-",
      ])
      .stdin(Stdio::null())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .spawn()
      .map_err(|error| format!("Could not read the camera recording: {error}"))?;
    let frames = BufReader::new(
      child
        .stdout
        .take()
        .ok_or_else(|| "Could not read the camera recording".to_owned())?,
    );
    let log = child
      .stderr
      .take()
      .ok_or_else(|| "Could not read the camera recording".to_owned())?;
    let (told, times) = mpsc::channel();
    std::thread::Builder::new()
      .name("camera-frame-times".to_owned())
      .spawn(move || {
        for line in BufReader::new(log).lines().map_while(Result::ok) {
          if let Some(seconds) = frame_time(&line) {
            if told.send(seconds).is_err() {
              return;
            }
          }
        }
      })
      .map_err(|error| error.to_string())?;
    Ok(Self {
      child,
      frames,
      times,
    })
  }

  /// Fills `rgb` with the next frame, answering when it shows from in
  /// milliseconds, or nothing at the end of the camera.
  pub(super) fn next(&mut self, rgb: &mut [u8]) -> Result<Option<f64>, String> {
    if !read_frame(&mut self.frames, rgb)? {
      return Ok(None);
    }
    let seconds = self
      .times
      .recv()
      .map_err(|_| "FFmpeg did not say when a camera frame shows".to_owned())?;
    Ok(Some(seconds * 1_000.0))
  }
}

impl Drop for Decoder {
  /// A decoder let go before the camera's end is stopped rather than left
  /// decoding frames nothing reads.
  fn drop(&mut self) {
    let _ = self.child.kill();
    let _ = self.child.wait();
  }
}

/// The time in a `showinfo` line about one frame, in seconds.
fn frame_time(line: &str) -> Option<f64> {
  if !line.contains("showinfo") || !line.contains(" n:") {
    return None;
  }
  let after = line.split("pts_time:").nth(1)?;
  after.split_whitespace().next()?.parse().ok()
}

/// Fills `frame` from `input`. False at the end of the stream; a frame cut
/// short there is dropped, as FFmpeg only stops between frames.
fn read_frame(input: &mut impl Read, frame: &mut [u8]) -> Result<bool, String> {
  let mut filled = 0;
  while filled < frame.len() {
    match input.read(&mut frame[filled..]) {
      Ok(0) => return Ok(false),
      Ok(read) => filled += read,
      Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
      Err(error) => return Err(format!("Could not read the camera recording: {error}")),
    }
  }
  Ok(true)
}

#[cfg(test)]
mod tests {
  use super::frame_time;

  #[test]
  fn a_frames_time_is_read_from_its_showinfo_line() {
    let line = "[Parsed_showinfo_1 @ 0x6000] n:   2 pts:   1243 pts_time:2.071667 duration: 20";
    assert_eq!(frame_time(line), Some(2.071667));
    assert_eq!(
      frame_time("[Parsed_showinfo_1 @ 0x6000] config in time_base: 1/600"),
      None
    );
  }
}
