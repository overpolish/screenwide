// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_sidecar::ffmpeg_command;
use std::process::Child;

pub(crate) fn run_export(
  args: Vec<OsString>,
  temporary: &Path,
  destination: &Path,
  cancelled: &AtomicBool,
  on_progress: &mut dyn FnMut(u64),
) -> Result<ExportRunResult, String> {
  let mut child = ffmpeg_command()
    .args(args)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|error| {
      let _ = std::fs::remove_file(temporary);
      format!("FFmpeg could not be started: {error}")
    })?;

  let stderr_reader = read_stderr(&mut child)?;
  let stdout = child
    .stdout
    .take()
    .ok_or_else(|| "FFmpeg did not expose its progress output".to_owned())?;
  if cancelled.load(Ordering::Acquire) {
    let _ = child.kill();
  } else {
    for line in BufReader::new(stdout).lines().map_while(Result::ok) {
      if cancelled.load(Ordering::Acquire) {
        let _ = child.kill();
        break;
      }
      if let Some(milliseconds) = progress_milliseconds(&line) {
        on_progress(milliseconds);
      }
    }
  }
  finish(child, stderr_reader, temporary, destination, cancelled)
}

/// Collects FFmpeg's error output on a thread of its own, so a full pipe
/// never stalls it.
fn read_stderr(child: &mut Child) -> Result<std::thread::JoinHandle<Vec<u8>>, String> {
  let stderr = child
    .stderr
    .take()
    .ok_or_else(|| "FFmpeg did not expose its error output".to_owned())?;
  Ok(std::thread::spawn(move || {
    let mut bytes = Vec::new();
    let _ = BufReader::new(stderr).read_to_end(&mut bytes);
    bytes
  }))
}

/// Waits for FFmpeg to end and puts what it wrote at `destination`, unless
/// the export was cancelled or the file is not one a player can open.
fn finish(
  mut child: Child,
  stderr_reader: std::thread::JoinHandle<Vec<u8>>,
  temporary: &Path,
  destination: &Path,
  cancelled: &AtomicBool,
) -> Result<ExportRunResult, String> {
  // Covers a cancellation arriving after FFmpeg's final progress line but
  // before its process has been reaped.
  if cancelled.load(Ordering::Acquire) {
    let _ = child.kill();
  }
  let status = child.wait().map_err(|error| {
    let _ = std::fs::remove_file(temporary);
    format!("FFmpeg could not be completed: {error}")
  })?;
  let stderr = stderr_reader.join().unwrap_or_default();

  if cancelled.load(Ordering::Acquire) {
    let _ = std::fs::remove_file(temporary);
    return Ok(ExportRunResult::Cancelled);
  }

  if !status.success() || !holds_bytes(temporary) || !plays_from_start_to_end(temporary) {
    let _ = std::fs::remove_file(temporary);
    return Err(remux_error(&stderr));
  }

  std::fs::rename(temporary, destination).map_err(|error| {
    let _ = std::fs::remove_file(temporary);
    format!("The recording could not be put in place: {error}")
  })?;

  Ok(ExportRunResult::Completed)
}

/// An FFmpeg export whose video frames are written to its standard input as
/// they are drawn. Dropped before [`Self::finish`], it stops FFmpeg and
/// removes what it wrote.
#[cfg(target_os = "macos")]
pub(crate) struct StreamedExport {
  child: Option<Child>,
  stdin: Option<std::process::ChildStdin>,
  stderr_reader: Option<std::thread::JoinHandle<Vec<u8>>>,
  temporary: PathBuf,
}

#[cfg(target_os = "macos")]
impl StreamedExport {
  /// FFmpeg started with `args`, which read the video from standard input
  /// and write `temporary`.
  pub(crate) fn start(args: Vec<OsString>, temporary: &Path) -> Result<Self, String> {
    let mut child = ffmpeg_command()
      .args(args)
      .stdin(Stdio::piped())
      .stdout(Stdio::null())
      .stderr(Stdio::piped())
      .spawn()
      .map_err(|error| format!("FFmpeg could not be started: {error}"))?;
    let stdin = child.stdin.take();
    let stderr_reader = read_stderr(&mut child)?;
    Ok(Self {
      child: Some(child),
      stdin,
      stderr_reader: Some(stderr_reader),
      temporary: temporary.to_owned(),
    })
  }

  /// The descriptor each frame is written to, open until [`Self::finish`].
  pub(crate) fn input(&self) -> Option<std::os::fd::RawFd> {
    use std::os::fd::AsRawFd;
    self.stdin.as_ref().map(AsRawFd::as_raw_fd)
  }

  /// Ends the video, lets FFmpeg finish the file, and puts it at
  /// `destination`.
  pub(crate) fn finish(
    mut self,
    destination: &Path,
    cancelled: &AtomicBool,
  ) -> Result<ExportRunResult, String> {
    drop(self.stdin.take());
    let (Some(child), Some(stderr_reader)) = (self.child.take(), self.stderr_reader.take()) else {
      return Err("FFmpeg was already stopped".to_owned());
    };
    finish(
      child,
      stderr_reader,
      &self.temporary,
      destination,
      cancelled,
    )
  }

  /// Stops FFmpeg and removes what it wrote, answering why it quit where it
  /// said.
  pub(crate) fn abandon(mut self) -> Option<String> {
    drop(self.stdin.take());
    let mut child = self.child.take()?;
    let _ = child.kill();
    let _ = child.wait();
    let _ = std::fs::remove_file(&self.temporary);
    let stderr = self.stderr_reader.take()?.join().unwrap_or_default();
    (!String::from_utf8_lossy(&stderr).trim().is_empty()).then(|| remux_error(&stderr))
  }
}

#[cfg(target_os = "macos")]
impl Drop for StreamedExport {
  fn drop(&mut self) {
    drop(self.stdin.take());
    if let Some(mut child) = self.child.take() {
      let _ = child.kill();
      let _ = child.wait();
      let _ = std::fs::remove_file(&self.temporary);
    }
  }
}

pub(crate) fn progress_milliseconds(line: &str) -> Option<u64> {
  line
    .strip_prefix("out_time_us=")?
    .parse::<u64>()
    .ok()
    .map(|microseconds| microseconds / 1_000)
}
