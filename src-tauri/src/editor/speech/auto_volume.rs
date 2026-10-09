// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Auto volume: the microphone brought to a steady, standard loudness.
//! Phrases louder than the speaker's usual level come down by two thirds of
//! what they rise above it, the voice is then lifted or lowered to -16 LUFS,
//! a loudness voices are commonly published at, and its peaks are held below
//! -1 dB. It makes no file: it is a chain of FFmpeg filters played over
//! whichever file the track is heard from, in the preview and the export
//! alike, and the volume slider works on top of it. Each file is measured
//! once, for where leveling starts and how far to lift, and the measure is
//! kept beside the project.

mod loudness;
mod measures;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use self::measures::measured;
use super::choice_file;
use super::heard::{self, Processing};

/// The make of the choice file.
const FORMAT_VERSION: u16 = 1;
/// The loudness the voice is brought to, in LUFS.
const TARGET_LUFS: f64 = -16.0;
/// How far above the voice's loudness leveling starts, in dB. Against the
/// voices measured, this takes 4 dB to 5 dB off the loudest phrases.
const ABOVE_LOUDNESS_DB: f64 = 3.0;
/// Every 3 dB above where leveling starts comes out as 1 dB.
const RATIO: f64 = 3.0;
/// The most a quiet microphone is lifted by, in dB.
const MOST_LIFT_DB: f64 = 30.0;
/// A track this quiet or quieter, in LUFS, holds nothing to level: the
/// absolute gate of EBU R128.
const SILENT_LUFS: f64 = -70.0;
/// Peaks held below -1 dB. The lookahead is made up for, so the voice stays
/// lined up with the picture.
const LIMITER: &str = "alimiter=limit=-1dB:attack=5:release=50:level=0:latency=1";

/// Whether the microphone is brought to a steady loudness.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum AutoVolume {
  #[default]
  Off,
  On,
}

/// How one file is leveled, from its measure.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Leveling {
  pub(crate) threshold_db: f64,
  pub(crate) makeup_db: f64,
}

impl Leveling {
  /// The FFmpeg filters that level the file, to be put in its chain.
  pub(crate) fn filters(&self) -> String {
    format!(
      "{},volume={:.2}dB,{LIMITER}",
      compressor(self.threshold_db),
      self.makeup_db
    )
  }
}

fn compressor(threshold_db: f64) -> String {
  // FFmpeg takes a threshold between -60 dB and 0 dB. A 5 ms attack catches
  // most of a word's first peak, which a slower one leaves for the limiter
  // to flatten after the lift.
  let threshold_db = threshold_db.clamp(-60.0, 0.0);
  format!("acompressor=threshold={threshold_db:.2}dB:ratio={RATIO}:attack=5:release=250")
}

fn choice_path(project_folder: &Path, stream: usize) -> PathBuf {
  project_folder.join(format!("auto-volume-{stream}.json"))
}

/// What the project says about the `stream`th track's volume. A recording
/// starts with Auto volume on, until someone turns it off.
pub(super) fn choice(project_folder: &Path, stream: usize) -> AutoVolume {
  choice_file::read(&choice_path(project_folder, stream), FORMAT_VERSION).unwrap_or(AutoVolume::On)
}

pub(super) fn keep(project_folder: &Path, stream: usize, choice: AutoVolume) -> Result<(), String> {
  choice_file::write(&choice_path(project_folder, stream), FORMAT_VERSION, choice)
    .map_err(|error| format!("Could not keep the auto volume choice: {error}"))
}

/// Whether the `stream`th track as it is heard now still has to be measured.
pub(super) fn is_unmeasured(project_folder: &Path, stream: usize) -> bool {
  let heard = heard::heard_file(project_folder, stream);
  measured(project_folder, stream, heard.as_deref()).is_none()
}

/// Measures each of the microphone `streams` of `movie` with Auto volume on
/// that is not yet measured as it is heard, waiting for any measure under
/// way, so an export started while the editor measures is still leveled.
pub(crate) fn measure_unmeasured(
  project_folder: &Path,
  streams: &[usize],
  movie: &Path,
  duration_ms: u64,
) -> Result<(), String> {
  for &stream in streams {
    if choice(project_folder, stream) == AutoVolume::On {
      measure(project_folder, stream, movie, duration_ms, &mut |_| {})?;
    }
  }
  Ok(())
}

/// One measure at a time, so two switches turned together neither measure
/// the same file twice nor write over each other's measures.
static MEASURING: Mutex<()> = Mutex::new(());

/// Measures the `stream`th track of `movie` as it is heard now, unless it has
/// been already, telling `progress` how far it has got, from the start of the
/// work to its end, and only once there is work to do.
pub(super) fn measure(
  project_folder: &Path,
  stream: usize,
  movie: &Path,
  duration_ms: u64,
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  let _measuring = MEASURING
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  let heard = heard::heard_file(project_folder, stream);
  if measured(project_folder, stream, heard.as_deref()).is_some() {
    return Ok(());
  }
  progress(0.0);
  let (source, source_stream) = heard.as_deref().map_or((movie, stream), |file| (file, 0));
  // Three reads: as it is, for where leveling starts; leveled, for how far to
  // lift it; and lifted through the limiter, which takes some loudness off
  // the peaks it holds, for how much more to lift it to make up for that.
  let recorded = loudness::integrated(source, source_stream, None, duration_ms, &mut |fraction| {
    progress(fraction / 3.0);
  })?;
  let leveling = if recorded > SILENT_LUFS {
    let threshold_db = recorded + ABOVE_LOUDNESS_DB;
    let leveled = loudness::integrated(
      source,
      source_stream,
      Some(&compressor(threshold_db)),
      duration_ms,
      &mut |fraction| progress((1.0 + fraction) / 3.0),
    )?;
    let first = Leveling {
      threshold_db,
      makeup_db: (TARGET_LUFS - leveled).min(MOST_LIFT_DB),
    };
    let limited = loudness::integrated(
      source,
      source_stream,
      Some(&first.filters()),
      duration_ms,
      &mut |fraction| progress((2.0 + fraction) / 3.0),
    )?;
    Some(Leveling {
      threshold_db,
      makeup_db: (first.makeup_db + TARGET_LUFS - limited).min(MOST_LIFT_DB),
    })
  } else {
    None
  };
  measures::keep(project_folder, stream, heard.as_deref(), leveling)?;
  progress(1.0);
  Ok(())
}

/// The Auto volume filters of each of the `streams` of the project in
/// `project_folder`, for those with the switch on whose heard file has been
/// measured and has a voice.
pub(crate) fn heard_filters(project_folder: &Path, streams: &[usize]) -> Vec<(usize, String)> {
  streams
    .iter()
    .filter(|&&stream| choice(project_folder, stream) == AutoVolume::On)
    .filter_map(|&stream| {
      let heard = heard::heard_file(project_folder, stream);
      measured(project_folder, stream, heard.as_deref())
        .flatten()
        .map(|leveling| (stream, leveling.filters()))
    })
    .collect()
}

/// The leveling of each file the `stream`th track can be heard from that has
/// been measured, the recording among them.
pub(crate) fn levelings(project_folder: &Path, stream: usize) -> Vec<(Processing, Leveling)> {
  let recording = (Processing::default(), None);
  let files = heard::files(project_folder, &[stream])
    .into_iter()
    .map(|(_, processing, path)| (processing, Some(path)));
  std::iter::once(recording)
    .chain(files)
    .filter_map(|(processing, path)| {
      measured(project_folder, stream, path.as_deref())
        .flatten()
        .map(|leveling| (processing, leveling))
    })
    .collect()
}
