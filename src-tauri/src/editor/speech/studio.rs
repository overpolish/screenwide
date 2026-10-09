// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Studio sound: the microphone rebuilt as a close, dry studio recording by
//! a speech restoration model, run in the transcriber program, and made once
//! into a file of its own in the project. Long quiet waits are left out of
//! the rebuild and come out silent (`waits`); the pauses are turned down as
//! Reduce noise turns them down, and the voice cleaned up as Vocal cleanup
//! cleans it, short of softening plosives. Reduce noise's enhancer is left
//! out: on a rebuilt voice it leaves artefacts and clips laughs. While it is
//! on it is heard in place of Reduce noise and Vocal cleanup; Auto volume
//! levels it as it would any other file. It needs a model downloaded first,
//! from Settings or from the switch.

mod waits;

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use screenwide_transcriber::{CLEAN_SAMPLE_RATE, SAMPLE_RATE};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use ts_rs::TS;

use super::analysis::{self, SpeechMap};
use super::pause_gate::PauseGate;
use super::{choice_file, voice};
use crate::transcription::catalogue::{self, Purpose};
use crate::transcription::models;
use crate::transcription::runner::{decode_audio_stream, Samples, Transcriber};

/// Bumped when a file of an older make should be made again.
const FORMAT_VERSION: u16 = 4;
/// Blocks of 10 ms that the voice's loudness is compared in, at the input's
/// rate; the output has three samples for each.
const BLOCK: usize = 160;
const UPSAMPLE: usize = (CLEAN_SAMPLE_RATE / SAMPLE_RATE) as usize;
/// A block of the restored voice quieter than this, in full-scale RMS
/// (-50 dBFS), holds no voice.
const VOICE_RMS: f64 = 0.003_2;
/// Where the restored voice's loudest sample may reach.
const HEADROOM: f64 = 0.98;
/// How much of the progress bar listening for speech takes, when it has not
/// been listened for already, and cleaning up after; rebuilding is many
/// times slower than either.
const LISTEN_SHARE: f32 = 0.05;
const CLEAN_SHARE: f32 = 0.05;
/// Samples gained and gated at a time.
const GATE_CHUNK: usize = 1 << 16;

/// Whether the microphone is rebuilt as a studio recording.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum StudioSound {
  #[default]
  Off,
  On,
}

fn choice_path(project_folder: &Path, stream: usize) -> PathBuf {
  project_folder.join(format!("studio-{stream}.json"))
}

/// The `stream`th track rebuilt, in the project.
pub(super) fn path(project_folder: &Path, stream: usize) -> PathBuf {
  project_folder.join(format!("studio-{stream}.flac"))
}

/// What the project says about the `stream`th track. A recording starts
/// with Studio sound on, until someone turns it off; it is heard only once
/// its file is made, which needs the model.
pub(super) fn choice(project_folder: &Path, stream: usize) -> StudioSound {
  choice_file::read(&choice_path(project_folder, stream), FORMAT_VERSION).unwrap_or(StudioSound::On)
}

pub(super) fn keep(
  project_folder: &Path,
  stream: usize,
  choice: StudioSound,
) -> Result<(), String> {
  forget_older(project_folder, stream)?;
  choice_file::write(&choice_path(project_folder, stream), FORMAT_VERSION, choice)
    .map_err(|error| format!("Could not keep the Studio sound choice: {error}"))
}

/// Removes a file made by an older make, then marks the choice as kept by
/// this one, so a file is only ever there when it is current. What was
/// chosen for the older make goes with it, and the switch starts again from
/// on, as a new recording does.
fn forget_older(project_folder: &Path, stream: usize) -> Result<(), String> {
  let path = choice_path(project_folder, stream);
  if choice_file::read::<StudioSound>(&path, FORMAT_VERSION).is_some() {
    return Ok(());
  }
  let _ = std::fs::remove_file(self::path(project_folder, stream));
  choice_file::write(&path, FORMAT_VERSION, StudioSound::On)
    .map_err(|error| format!("Could not keep the Studio sound choice: {error}"))
}

/// Whether the `stream`th track has been rebuilt, by the current make.
pub(super) fn is_made(project_folder: &Path, stream: usize) -> bool {
  choice_file::read::<StudioSound>(&choice_path(project_folder, stream), FORMAT_VERSION).is_some()
    && path(project_folder, stream).is_file()
}

/// Whether the `stream`th track is heard rebuilt: the switch is on and the
/// file made, whether or not the model is still on this computer.
pub(super) fn is_heard(project_folder: &Path, stream: usize) -> bool {
  choice(project_folder, stream) == StudioSound::On && is_made(project_folder, stream)
}

/// The model's encoder and vocoder, where it is downloaded.
pub(super) fn model(app: &AppHandle) -> Option<(PathBuf, PathBuf)> {
  let model = catalogue::for_purpose(Purpose::StudioSound)?;
  if !models::is_downloaded(app, model) {
    return None;
  }
  let mut paths = models::paths(app, model).ok()?.into_iter();
  Some((paths.next()?, paths.next()?))
}

/// One rebuild at a time, so a switch turned while a recording is readied
/// on opening waits for the file rather than writing it a second time.
static MAKING: Mutex<()> = Mutex::new(());

/// Rebuilds the `stream`th track of `movie` into the project with the model
/// at `model`, unless a file of this make is there already, telling
/// `progress` how far it has got, and only once there is work to do. With
/// the voice activity model at `vad`, long quiet waits are left out and the
/// pauses are turned down.
pub(super) fn prepare(
  movie: &Path,
  stream: usize,
  project_folder: &Path,
  (model, vad): ((PathBuf, PathBuf), Option<PathBuf>),
  progress: &mut dyn FnMut(f32),
) -> Result<(), String> {
  let _making = MAKING
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  forget_older(project_folder, stream)?;
  if is_made(project_folder, stream) {
    return Ok(());
  }
  progress(0.0);
  let recorded = decode_audio_stream(movie, stream, SAMPLE_RATE)?;
  let speech = vad
    .map(|vad| {
      analysis::speech_map(movie, stream, project_folder, vad, &mut |fraction| {
        progress(fraction * LISTEN_SHARE);
      })
    })
    .transpose()?;
  let stretches = match &speech {
    Some(speech) => waits::stretches(speech, recorded.path())?,
    None => vec![[0, u64::MAX]],
  };
  let restored = Samples::scratch();
  Transcriber::start()?.restore_speech(
    model,
    &recorded,
    stretches,
    &restored,
    &mut |fraction| {
      progress(LISTEN_SHARE + fraction * (1.0 - LISTEN_SHARE - CLEAN_SHARE));
    },
  )?;
  let gain = loudness_gain(&recorded, &restored)?;
  drop(recorded);
  let gated = gain_and_gate(&restored, gain, speech.as_ref())?;
  drop(restored);
  let destination = path(project_folder, stream);
  // Written beside the destination and moved over it whole, so a rebuild
  // cut short never leaves a file that looks finished.
  let partial = destination.with_extension("partial.flac");
  let cleaned = voice::clean_up_rebuilt(&gated, &partial, &mut |fraction| {
    progress(1.0 - CLEAN_SHARE + fraction * CLEAN_SHARE);
  });
  if let Err(error) = cleaned {
    let _ = std::fs::remove_file(&partial);
    return Err(error);
  }
  std::fs::rename(&partial, &destination)
    .map_err(|error| format!("Could not keep the Studio sound: {error}"))
}

/// The gain that puts the restored voice back at the loudness it was
/// recorded at, short of clipping. The model hears the voice brought up to
/// near full scale and speaks it back at that level.
fn loudness_gain(recorded: &Samples, restored: &Samples) -> Result<f32, String> {
  let unreadable = |error: std::io::Error| format!("Could not read the Studio sound: {error}");
  let mut recorded = BufReader::new(File::open(recorded.path()).map_err(unreadable)?);
  let mut restored = BufReader::new(File::open(restored.path()).map_err(unreadable)?);
  let (mut voice_in, mut voice_out, mut peak) = (0.0_f64, 0.0_f64, 0.0_f64);
  let mut input = vec![0_u8; BLOCK * 4];
  let mut output = vec![0_u8; BLOCK * UPSAMPLE * 4];
  let samples = |bytes: &[u8]| -> Vec<f64> {
    bytes
      .as_chunks::<4>()
      .0
      .iter()
      .map(|sample| f64::from(f32::from_le_bytes(*sample)))
      .collect()
  };
  while recorded.read_exact(&mut input).is_ok() && restored.read_exact(&mut output).is_ok() {
    let (block_in, block_out) = (samples(&input), samples(&output));
    let energy_out =
      block_out.iter().map(|sample| sample * sample).sum::<f64>() / block_out.len() as f64;
    peak = block_out
      .iter()
      .fold(peak, |peak, sample| peak.max(sample.abs()));
    if energy_out > VOICE_RMS * VOICE_RMS {
      voice_in +=
        block_in.iter().map(|sample| sample * sample).sum::<f64>() / block_in.len() as f64;
      voice_out += energy_out;
    }
  }
  let gain = if voice_out > 0.0 {
    (voice_in / voice_out).sqrt()
  } else {
    1.0
  };
  Ok(gain.min(HEADROOM / peak.max(f64::MIN_POSITIVE)) as f32)
}

/// The restored samples in `restored` with `gain` applied and, where
/// `speech` hears none, turned down as Reduce noise turns its pauses down:
/// the model rebuilds a click or a breath between words as cleanly as the
/// words themselves.
fn gain_and_gate(
  restored: &Samples,
  gain: f32,
  speech: Option<&SpeechMap>,
) -> Result<Samples, String> {
  let unreadable = |error: std::io::Error| format!("Could not read the Studio sound: {error}");
  let unwritable = |error: std::io::Error| format!("Could not write the Studio sound: {error}");
  let length = std::fs::metadata(restored.path())
    .map_err(unreadable)?
    .len()
    / 4;
  let mut gate = speech.map(|speech| PauseGate::new(speech, CLEAN_SAMPLE_RATE, length));
  let mut reader = BufReader::new(File::open(restored.path()).map_err(unreadable)?);
  let gated = Samples::scratch();
  let mut writer = BufWriter::new(File::create(gated.path()).map_err(unwritable)?);
  let mut bytes = vec![0_u8; GATE_CHUNK * 4];
  let mut samples = Vec::with_capacity(GATE_CHUNK);
  let mut offset = 0;
  while offset < length {
    let count = GATE_CHUNK.min((length - offset) as usize);
    reader
      .read_exact(&mut bytes[..count * 4])
      .map_err(unreadable)?;
    samples.clear();
    samples.extend(
      bytes[..count * 4]
        .as_chunks::<4>()
        .0
        .iter()
        .map(|sample| f32::from_le_bytes(*sample) * gain),
    );
    if let Some(gate) = &mut gate {
      gate.apply(offset, &mut samples);
    }
    for (sample, out) in samples.iter().zip(bytes.as_chunks_mut::<4>().0) {
      *out = sample.to_le_bytes();
    }
    writer.write_all(&bytes[..count * 4]).map_err(unwritable)?;
    offset += count as u64;
  }
  writer.flush().map_err(unwritable)?;
  Ok(gated)
}
