// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub(crate) const FORMAT_VERSION: u16 = 1;

/// How far before the press a moment is placed. The press comes after the
/// thing worth keeping has happened, so the moment is put back to where it
/// began. Fixed rather than a setting: one sensible reach is enough.
pub(super) const LOOK_BACK_US: u64 = 2_000_000;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(
  rename_all = "camelCase",
  rename_all_fields = "camelCase",
  tag = "type"
)]
pub(super) enum MomentRecord {
  Header {
    timebase: String,
    version: u16,
  },
  /// The kind is written out in full with every moment rather than once in
  /// the header: a kind added or renamed mid-recording still lands as it was
  /// when its shortcut was pressed.
  Moment {
    color: String,
    kind_id: String,
    name: String,
    timestamp_us: u64,
  },
  /// The voice note recorded with the `moment`th moment, counting from
  /// zero, at `notes/<moment>.wav` beside this file. Written once the note
  /// is, so a record always has its file.
  Note {
    duration_ms: u64,
    moment: usize,
  },
  /// What was said in the `moment`th moment's voice note, added once it is
  /// transcribed, after the recording has stopped. The moment's name and
  /// time are repeated so the line reads on its own. A note transcribed
  /// again gets another line, and the last one counts. Only the text is
  /// kept: a note is read, never edited word by word.
  Transcript {
    moment: usize,
    name: String,
    timestamp_us: u64,
    text: String,
  },
}

/// Where the voice note of the `moment`th moment is, beside the moments file
/// in `folder`.
pub(crate) fn note_path(folder: &Path, moment: usize) -> PathBuf {
  folder.join("notes").join(format!("{moment}.wav"))
}

/// The moments file of the recording project at `project`, if it has one.
pub(crate) fn for_project(project: &Path) -> Option<PathBuf> {
  let name = crate::project::read(project)
    .ok()?
    .recorded()
    .ok()?
    .media
    .moments
    .clone()?;
  crate::project::resolve(project, &name).ok()
}

pub(super) fn header() -> MomentRecord {
  MomentRecord::Header {
    timebase: "recording-microseconds".to_owned(),
    version: FORMAT_VERSION,
  }
}

/// Where a press at recording time `pressed_us` puts its moment.
pub(super) const fn placed_at_us(pressed_us: u64) -> u64 {
  pressed_us.saturating_sub(LOOK_BACK_US)
}

/// One moment as a recording keeps it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RecordedMoment {
  pub color: String,
  pub kind_id: String,
  pub name: String,
  pub timestamp_us: u64,
  /// How long the moment's voice note is, if it has one.
  pub note_duration_ms: Option<u64>,
  /// What its voice note says, once transcribed.
  pub transcript: Option<String>,
}

/// Every complete moment in the sidecar at `path`, in the order they were
/// pressed. A crash can cut a line short, and transcripts may be added after
/// it, so an unreadable line is passed over rather than ending the file.
pub(crate) fn read(path: &Path) -> Result<Vec<RecordedMoment>, String> {
  let reader = BufReader::new(File::open(path).map_err(|error| error.to_string())?);
  let mut lines = reader.lines();
  let header = lines
    .next()
    .transpose()
    .map_err(|error| error.to_string())?
    .and_then(|line| serde_json::from_str::<MomentRecord>(&line).ok());
  match header {
    Some(MomentRecord::Header { version, .. }) if version == FORMAT_VERSION => {}
    Some(MomentRecord::Header { version, .. }) => {
      return Err(format!("Moments version {version} is not supported"));
    }
    _ => return Err("The moments file has no valid header".to_owned()),
  }
  let mut moments = Vec::new();
  for line in lines {
    let line = line.map_err(|error| error.to_string())?;
    if line.trim().is_empty() {
      continue;
    }
    let Ok(record) = serde_json::from_str::<MomentRecord>(&line) else {
      continue;
    };
    match record {
      MomentRecord::Moment {
        color,
        kind_id,
        name,
        timestamp_us,
      } => moments.push(RecordedMoment {
        color,
        kind_id,
        name,
        note_duration_ms: None,
        timestamp_us,
        transcript: None,
      }),
      MomentRecord::Note {
        duration_ms,
        moment,
      } => {
        if let Some(recorded) = moments.get_mut(moment) {
          recorded.note_duration_ms = Some(duration_ms);
        }
      }
      MomentRecord::Transcript { moment, text, .. } => {
        if let Some(recorded) = moments.get_mut(moment) {
          recorded.transcript = Some(text);
        }
      }
      MomentRecord::Header { .. } => {}
    }
  }
  Ok(moments)
}
