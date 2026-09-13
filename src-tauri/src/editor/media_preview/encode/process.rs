// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(in crate::editor) fn run_export(
  args: Vec<OsString>,
  temporary: &Path,
  destination: &Path,
  cancelled: &AtomicBool,
  on_progress: &mut dyn FnMut(u64),
) -> Result<ExportRunResult, String> {
  let mut child = Command::new(ffmpeg_path())
    .args(args)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .map_err(|error| {
      let _ = std::fs::remove_file(temporary);
      format!("FFmpeg could not be started: {error}")
    })?;

  let stderr = child
    .stderr
    .take()
    .ok_or_else(|| "FFmpeg did not expose its error output".to_owned())?;
  let stderr_reader = std::thread::spawn(move || {
    let mut bytes = Vec::new();
    let _ = BufReader::new(stderr).read_to_end(&mut bytes);
    bytes
  });
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

pub(in crate::editor::media_preview) fn progress_milliseconds(line: &str) -> Option<u64> {
  line
    .strip_prefix("out_time_us=")?
    .parse::<u64>()
    .ok()
    .map(|microseconds| microseconds / 1_000)
}
