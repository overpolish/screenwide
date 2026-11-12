// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A song split into its singing and the rest, kept beside it and filled in
//! a piece at a time, only where clips play it. The separation model hears
//! 8 s at a time from the song's start without overlapping, so a piece
//! comes out the same whenever it is separated and whatever was separated
//! with it: pieces made at different times sit side by side without a seam.
//! Each part is one FLAC as long as the furthest piece separated, silent
//! where no piece is yet, with a record beside them of the pieces they hold.
//!
//! Both parts are kept, rather than the singing alone with the rest worked
//! out in the mix as the song less it: a compressed song read from partway
//! in, as the mix reads it, does not always start on the exact sample (an
//! MP3 is out by a few milliseconds), and then the two no longer cancel and
//! the singing is heard. Parts made from one reading of the song always
//! line up, and a FLAC is read from any sample exactly.

use std::collections::BTreeSet;
use std::ops::Range;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::write_whole;

/// How long each piece is, in ms: the separation model's chunk, 8 s.
pub const PIECE_MS: u64 = 8_000;
/// The make of the record kept beside the parts. Version 1 kept the singing
/// alone, which the mix could not take out of an MP3 cleanly.
const FORMAT_VERSION: u16 = 2;
/// How loud the parts are kept against the song, which the mix makes up:
/// below full, so a part louder than the song's full scale, as separated
/// parts can be, still fits a FLAC.
pub const PART_LEVEL: f32 = 0.5;

/// Where the singing separated from the sound file `sound` is kept: beside
/// it, named after it.
pub fn singing_path(sound: &Path) -> PathBuf {
  beside(sound, "singing.flac")
}

/// Where the rest of the sound file `sound`, without its singing, is kept.
pub fn rest_path(sound: &Path) -> PathBuf {
  beside(sound, "rest.flac")
}

fn record_path(sound: &Path) -> PathBuf {
  beside(sound, "singing.json")
}

fn beside(sound: &Path, ending: &str) -> PathBuf {
  let stem = sound
    .file_stem()
    .map_or_else(String::new, |stem| stem.to_string_lossy().into_owned());
  sound.with_file_name(format!("{stem}.{ending}"))
}

/// The files the parts separated from `sound` are kept in.
pub(super) fn separated_files(sound: &Path) -> [PathBuf; 3] {
  [singing_path(sound), rest_path(sound), record_path(sound)]
}

#[derive(Deserialize, Serialize)]
struct Record {
  version: u16,
  pieces: BTreeSet<u64>,
}

/// A song's singing and the rest of it, as far as they are separated.
#[derive(Clone, Debug, PartialEq)]
pub struct Separated {
  pub singing: PathBuf,
  pub rest: PathBuf,
  pieces: BTreeSet<u64>,
}

impl Separated {
  /// What is separated of the sound file `sound`, where anything is.
  pub fn read(sound: &Path) -> Option<Self> {
    let (singing, rest) = (singing_path(sound), rest_path(sound));
    let record: Record = serde_json::from_slice(&std::fs::read(record_path(sound)).ok()?).ok()?;
    (record.version == FORMAT_VERSION
      && !record.pieces.is_empty()
      && singing.is_file()
      && rest.is_file())
    .then_some(Self {
      singing,
      rest,
      pieces: record.pieces,
    })
  }

  /// Whether every piece from `from_ms` to `to_ms` of the song is
  /// separated.
  pub fn covers(&self, from_ms: u64, to_ms: u64) -> bool {
    pieces_between(from_ms, to_ms).all(|piece| self.pieces.contains(&piece))
  }

  /// How many pieces are separated.
  pub fn len(&self) -> usize {
    self.pieces.len()
  }

  pub fn is_empty(&self) -> bool {
    self.pieces.is_empty()
  }

  #[cfg(test)]
  pub(crate) fn with_pieces(sound: &Path, pieces: impl IntoIterator<Item = u64>) -> Self {
    Self {
      singing: singing_path(sound),
      rest: rest_path(sound),
      pieces: pieces.into_iter().collect(),
    }
  }
}

/// The pieces the stretch from `from_ms` to `to_ms` of a song falls in.
pub fn pieces_between(from_ms: u64, to_ms: u64) -> Range<u64> {
  if to_ms <= from_ms {
    return 0..0;
  }
  from_ms / PIECE_MS..to_ms.div_ceil(PIECE_MS)
}

/// Of the pieces `wanted`, those `separated` lacks, as runs of pieces that
/// follow each other.
pub fn missing(separated: Option<&Separated>, wanted: &BTreeSet<u64>) -> Vec<Range<u64>> {
  let mut runs: Vec<Range<u64>> = Vec::new();
  for &piece in wanted {
    if separated.is_some_and(|separated| separated.pieces.contains(&piece)) {
      continue;
    }
    match runs.last_mut() {
      Some(run) if run.end == piece => run.end += 1,
      _ => runs.push(piece..piece + 1),
    }
  }
  runs
}

/// Records that `pieces` of the sound file `sound` are now in its parts,
/// beside those they held already.
pub fn record(sound: &Path, pieces: impl IntoIterator<Item = u64>) -> Result<(), String> {
  let mut held = Separated::read(sound)
    .map(|separated| separated.pieces)
    .unwrap_or_default();
  held.extend(pieces);
  let record = Record {
    version: FORMAT_VERSION,
    pieces: held,
  };
  let bytes = serde_json::to_vec(&record).map_err(|error| error.to_string())?;
  write_whole(&record_path(sound), &bytes)
}

/// The parts kept beside `sound` in a copy of the library, or a project's,
/// are replaced by those beside `from` where `from` holds more of them. The
/// parts go first and their record after, so a record never claims a piece
/// they lack.
pub(super) fn copy_if_more(
  from: &Path,
  to: &Path,
  copy: impl Fn(&Path, &Path) -> Result<(), String>,
) -> Result<(), String> {
  let Some(theirs) = Separated::read(from) else {
    return Ok(());
  };
  if Separated::read(to).is_some_and(|ours| ours.len() >= theirs.len()) {
    return Ok(());
  }
  copy(&singing_path(from), &singing_path(to))?;
  copy(&rest_path(from), &rest_path(to))?;
  copy(&record_path(from), &record_path(to))
}

/// Removes parts kept beside `sound` that no record this build reads
/// vouches for, such as singing kept alone by an older make, so pieces
/// separated next are laid over silence rather than over them.
pub fn forget_unrecorded(sound: &Path) {
  if Separated::read(sound).is_some() {
    return;
  }
  for path in separated_files(sound) {
    let _ = std::fs::remove_file(path);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn a_stretch_falls_in_the_pieces_it_touches() {
    assert_eq!(pieces_between(0, 8_000), 0..1);
    assert_eq!(pieces_between(0, 8_001), 0..2);
    assert_eq!(pieces_between(7_999, 8_001), 0..2);
    assert_eq!(pieces_between(8_000, 16_000), 1..2);
    assert_eq!(pieces_between(5_000, 5_000), 0..0);
  }

  #[test]
  fn missing_pieces_come_in_runs_around_those_separated() {
    let separated = Separated::with_pieces(Path::new("s.mp3"), [2, 3, 7]);
    let wanted: BTreeSet<u64> = (0..10).collect();
    assert_eq!(missing(Some(&separated), &wanted), vec![0..2, 4..7, 8..10]);
    assert_eq!(missing(None, &[4, 5, 9].into()), vec![4..6, 9..10]);
  }
}
