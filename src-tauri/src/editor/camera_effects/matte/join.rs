// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The chunks of the matte joined into one file, as the transcriber made
//! them alike: same size, rate and encoding, so no frame is encoded again.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The mattes at `parts`, one after the other, into `output`.
pub(super) fn join(parts: &[PathBuf], output: &Path) -> Result<(), String> {
  let failed =
    |error: &dyn std::fmt::Display| format!("Could not keep the camera's matte: {error}");
  if let [only] = parts {
    return std::fs::copy(only, output)
      .map(|_| ())
      .map_err(|error| failed(&error));
  }
  let list = output.with_extension("parts.txt");
  let listed = parts
    .iter()
    .map(|part| {
      format!(
        "file '{}'\n",
        part.display().to_string().replace('\'', "'\\''")
      )
    })
    .collect::<String>();
  std::fs::write(&list, listed).map_err(|error| failed(&error))?;
  let status = Command::new(screenwide_sidecar::ffmpeg_path())
    .args(["-hide_banner", "-nostdin", "-loglevel", "error"])
    .args(["-f", "concat", "-safe", "0", "-i"])
    .arg(&list)
    .args(["-c", "copy", "-movflags", "+faststart", "-y"])
    .arg(output)
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status();
  let _ = std::fs::remove_file(&list);
  match status {
    Ok(status) if status.success() => Ok(()),
    Ok(_) => Err("FFmpeg could not join the camera's matte".to_owned()),
    Err(error) => Err(failed(&error)),
  }
}
