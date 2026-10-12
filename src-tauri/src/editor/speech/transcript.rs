// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Transcribing the recording's microphone, to edit and caption by: the
//! stretches of speech transcribed one at a time so the work can be cancelled
//! between them, and the words kept beside the project. The words and how
//! they are planned and settled are `screenwide_speech::transcript`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

use screenwide_transcriber::SAMPLE_RATE;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use ts_rs::TS;

use super::{vad_model, SpeechSource};
use crate::transcription::{language, models};
use screenwide_speech::catalogue::Purpose;
use screenwide_speech::runner::samples::decode_audio_stream;
use screenwide_speech::runner::Transcriber;
use screenwide_speech::transcript::{
  plan, word_edges, word_ends, word_starts, RecordingTranscript, RecordingTranscriptWord,
};
use screenwide_speech::{analysis, silences};

/// The event that tells the editor how far transcribing a recording has got.
const PROGRESS_EVENT: &str = "editor://transcript-progress";

/// Bumped when what is kept stops matching what transcription gives, so an
/// old file is transcribed again rather than read wrongly.
const FORMAT_VERSION: u16 = 1;

/// Listening for speech takes this share of the bar; transcribing it the rest.
const LISTEN_SHARE: f32 = 0.5;

/// What is kept beside the project: the words, and the model and language
/// that made them, so a different model later reads as a re-transcribe.
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Kept {
  version: u16,
  model: String,
  language: String,
  words: Vec<RecordingTranscriptWord>,
}

/// How far transcribing a recording has got, 0 to 1.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TranscriptProgress {
  pub artifact_id: u64,
  pub fraction: f32,
}

/// A transcription under way per recording, so a second ask for the same one
/// is refused and cancellation knows what to stop.
static RUNNING: LazyLock<Mutex<HashMap<u64, Arc<AtomicBool>>>> =
  LazyLock::new(|| Mutex::new(HashMap::new()));

/// A progress callback that tells the editor how far `artifact_id` has got.
fn reporter(app: &AppHandle, artifact_id: u64) -> impl FnMut(f32) {
  let app = app.clone();
  move |fraction| {
    let _ = app.emit(
      PROGRESS_EVENT,
      TranscriptProgress {
        artifact_id,
        fraction,
      },
    );
  }
}

/// Transcribes the recording open in the editor afresh, answering with its
/// words. A recording already being transcribed is refused rather than done
/// twice; [`cancel_recording_transcription`] stops the work under way.
#[tauri::command]
pub async fn transcribe_recording(
  app: AppHandle,
  artifact_id: u64,
) -> Result<RecordingTranscript, String> {
  // Everything that can refuse is settled before the work is registered, so
  // a refusal never leaves a stale "transcribing" entry behind.
  let source = SpeechSource::open(&app, artifact_id)?;
  let model = models::usable(&app, Purpose::Transcripts)
    .ok_or_else(|| "Download the Transcripts model first".to_owned())?;
  let model_file = models::paths(&app, model)?
    .into_iter()
    .next()
    .ok_or_else(|| "The transcription model has no file".to_owned())?;
  let model_id = model.id.to_owned();
  let language = language::resolved(&language::current());
  let vad = vad_model(&app)?;

  let stop = Arc::new(AtomicBool::new(false));
  {
    let mut running = RUNNING
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    if running.contains_key(&artifact_id) {
      return Err("This recording is already being transcribed".to_owned());
    }
    running.insert(artifact_id, stop.clone());
  }
  let mut progress = reporter(&app, artifact_id);
  let result = tauri::async_runtime::spawn_blocking(move || {
    transcribe(
      &source,
      model_file,
      &model_id,
      language,
      vad,
      stop,
      &mut progress,
    )
  })
  .await
  .map_err(|error| error.to_string())
  .and_then(|result| result);
  // Whatever happened, this recording is no longer being transcribed.
  RUNNING
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .remove(&artifact_id);
  result
}

/// Asks to stop the transcription of `artifact_id` under way, if there is
/// one. It stops between stretches of speech, so a cancellation never leaves
/// a half-written transcript.
#[tauri::command]
pub fn cancel_recording_transcription(artifact_id: u64) -> Result<(), String> {
  if let Some(stop) = RUNNING
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .get(&artifact_id)
  {
    stop.store(true, Ordering::Release);
  }
  Ok(())
}

/// The transcript kept for the recording open in the editor, if one was made.
#[tauri::command]
pub fn get_recording_transcript(
  app: AppHandle,
  artifact_id: u64,
) -> Result<Option<RecordingTranscript>, String> {
  let source = SpeechSource::open(&app, artifact_id)?;
  Ok(read_cache(&source.project_folder, source.microphone))
}

/// Plans the stretches, transcribes each, and keeps the words beside the
/// project. Nothing said is nothing to transcribe.
fn transcribe(
  source: &SpeechSource,
  model: PathBuf,
  model_id: &str,
  language: Option<String>,
  vad: PathBuf,
  stop: Arc<AtomicBool>,
  progress: &mut dyn FnMut(f32),
) -> Result<RecordingTranscript, String> {
  let speech = analysis::speech_map(
    &source.movie,
    source.microphone,
    &source.project_folder,
    vad,
    &mut |fraction| progress(fraction * LISTEN_SHARE),
  )?;
  let stretches = plan::plan(&speech, source.duration_ms as f64);
  let transcript = if stretches.is_empty() {
    RecordingTranscript {
      language: language.clone().unwrap_or_default(),
      model: model_id.to_owned(),
      words: Vec::new(),
    }
  } else {
    let samples = decode_audio_stream(&source.movie, source.microphone, SAMPLE_RATE)?;
    let mut running = Transcriber::start()?.stopped_by(stop);
    let done = running.transcribe(
      model,
      model_id,
      &samples,
      language.clone(),
      stretches,
      &mut |fraction| progress(LISTEN_SHARE + fraction * (1.0 - LISTEN_SHARE)),
    )?;
    let mut transcript = flatten(&done);
    let speech = silences::speech_ranges(&speech, source.duration_ms as f64);
    word_starts::settle_word_starts(&mut transcript.words, &speech);
    word_ends::settle_word_ends(&mut transcript.words, &speech);
    word_edges::settle_word_edges_from(&mut transcript.words, samples.path())?;
    transcript
  };
  let kept = Kept {
    language: transcript.language.clone(),
    model: transcript.model.clone(),
    version: FORMAT_VERSION,
    words: transcript.words.clone(),
  };
  write_cache(&source.project_folder, source.microphone, &kept)?;
  Ok(transcript)
}

/// The transcript's words, flattened from its segments in the order said.
fn flatten(transcript: &screenwide_transcriber::Transcript) -> RecordingTranscript {
  RecordingTranscript {
    language: transcript.language.clone(),
    model: transcript.model.clone(),
    words: transcript
      .segments
      .iter()
      .flat_map(|segment| {
        segment.words.iter().map(|word| RecordingTranscriptWord {
          end_ms: word.end_ms,
          probability: word.probability,
          start_ms: word.start_ms,
          text: word.text.clone(),
        })
      })
      .collect(),
  }
}

fn transcript_path(project_folder: &Path, stream: usize) -> PathBuf {
  project_folder.join(format!("transcript-{stream}.json"))
}

fn read_cache(project_folder: &Path, stream: usize) -> Option<RecordingTranscript> {
  let kept: Kept =
    serde_json::from_slice(&std::fs::read(transcript_path(project_folder, stream)).ok()?).ok()?;
  (kept.version == FORMAT_VERSION).then_some(RecordingTranscript {
    language: kept.language,
    model: kept.model,
    words: kept.words,
  })
}

/// Keeps the transcript beside the project, written to a side file and moved
/// over it whole, so a transcription cut short never leaves a file that
/// looks finished.
fn write_cache(project_folder: &Path, stream: usize, kept: &Kept) -> Result<(), String> {
  let path = transcript_path(project_folder, stream);
  let partial = path.with_extension("partial.json");
  let bytes = serde_json::to_vec(kept).map_err(|error| error.to_string())?;
  if let Err(error) = std::fs::write(&partial, &bytes) {
    let _ = std::fs::remove_file(&partial);
    return Err(format!("Could not keep the transcript: {error}"));
  }
  std::fs::rename(&partial, &path)
    .map_err(|error| format!("Could not keep the transcript: {error}"))
}
