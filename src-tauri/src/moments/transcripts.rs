// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use super::format::{read, MomentRecord};

/// Adds `text`, what the `moment`th moment's voice note says, to the moments
/// file at `path`, as one more line. The recording is long finished by now,
/// so the file is appended to rather than rewritten: what is already there
/// is never at risk.
pub(crate) fn add_transcript(path: &Path, moment: usize, text: &str) -> Result<(), String> {
  let recorded = read(path)?;
  let placed = recorded
    .get(moment)
    .ok_or_else(|| "That moment is not in this recording".to_owned())?;
  append(
    path,
    &MomentRecord::Transcript {
      moment,
      name: placed.name.clone(),
      text: text.to_owned(),
      timestamp_us: placed.timestamp_us,
    },
  )
}

fn append(path: &Path, record: &MomentRecord) -> Result<(), String> {
  let failed = |error: std::io::Error| format!("Could not add to the moments file: {error}");
  let mut file = OpenOptions::new()
    .read(true)
    .append(true)
    .open(path)
    .map_err(failed)?;
  let mut line = serde_json::to_vec(record).map_err(|error| error.to_string())?;
  line.push(b'\n');
  // A line a crash cut short is ended first, so the new one stands alone.
  if file.metadata().map_err(failed)?.len() > 0 {
    let mut last = [0];
    file.seek(SeekFrom::End(-1)).map_err(failed)?;
    file.read_exact(&mut last).map_err(failed)?;
    if last[0] != b'\n' {
      line.insert(0, b'\n');
    }
  }
  file.write_all(&line).map_err(failed)?;
  file.sync_data().map_err(failed)
}
