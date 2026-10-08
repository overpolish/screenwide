// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::{Path, PathBuf};

use screenwide_transcriber::{Segment, Speech, Transcript, Word};
use whisper_rs::{
  FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState,
  WhisperVadContext, WhisperVadContextParams,
};

/// The voice activity model reads the audio in windows of this many samples:
/// 32 ms at the protocol's rate, which is Silero's own window.
const VAD_WINDOW_SAMPLES: usize = 512;

/// The models last loaded, kept for the requests after them: loading takes
/// far longer than transcribing a short note.
#[derive(Default)]
pub struct Engine {
  loaded: Option<(PathBuf, WhisperContext)>,
  vad: Option<(PathBuf, WhisperVadContext)>,
}

impl Engine {
  pub fn transcribe(
    &mut self,
    model: &Path,
    model_id: &str,
    audio: &Path,
    language: Option<&str>,
  ) -> Result<Transcript, String> {
    let samples = read_samples(audio)?;
    let context = self.context(model)?;
    let mut state = context
      .create_state()
      .map_err(|error| format!("Could not start transcribing: {error}"))?;

    // A code the engine does not know is detected instead of failing.
    let language = language
      .filter(|code| whisper_rs::get_lang_id(code).is_some())
      .unwrap_or("auto");
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some(language));
    params.set_n_threads(threads());
    params.set_token_timestamps(true);
    // Each request is its own recording: nothing said in one should steer
    // the next.
    params.set_no_context(true);
    // Keeps "[Music]" and the like, said by nobody, out of the text.
    params.set_suppress_nst(true);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_special(false);
    params.set_print_timestamps(false);
    state
      .full(params, &samples)
      .map_err(|error| format!("Transcribing failed: {error}"))?;

    let segments = segments(context, &state)?;
    let language = whisper_rs::get_lang_str(state.full_lang_id_from_state())
      .unwrap_or(language)
      .to_owned();
    let text = segments
      .iter()
      .map(|segment| segment.text.as_str())
      .collect::<Vec<_>>()
      .join(" ");
    Ok(Transcript {
      language,
      model: model_id.to_owned(),
      segments,
      text,
    })
  }

  /// How likely each window of the audio is to be speech, telling
  /// `progress` how far it has got. The model runs on the processor: it is
  /// small, and a run over an hour of audio takes seconds there without
  /// waking a GPU.
  ///
  /// The audio is heard a minute at a time, so there is progress to tell.
  /// Each minute is heard from a second before it, so the model has settled
  /// by the time its windows are kept.
  pub fn detect_speech(
    &mut self,
    model: &Path,
    audio: &Path,
    progress: &mut dyn FnMut(f32),
  ) -> Result<Speech, String> {
    const CHUNK_WINDOWS: usize = 1_875;
    const LEAD_IN_WINDOWS: usize = 31;
    let samples = read_samples(audio)?;
    if self.vad.as_ref().is_none_or(|(path, _)| path != model) {
      self.vad = None;
      let path = model
        .to_str()
        .ok_or_else(|| "The voice activity model's path is not valid text".to_owned())?;
      let mut params = WhisperVadContextParams::default();
      params.set_n_threads(threads());
      let context = WhisperVadContext::new(path, params)
        .map_err(|error| format!("Could not load the voice activity model: {error}"))?;
      self.vad = Some((model.to_owned(), context));
    }
    let context = &mut self.vad.as_mut().expect("loaded above").1;
    let windows = samples.len().div_ceil(VAD_WINDOW_SAMPLES);
    let mut probabilities = Vec::with_capacity(windows);
    let mut first = 0;
    while first < windows {
      let lead_in = first.min(LEAD_IN_WINDOWS);
      let from = (first - lead_in) * VAD_WINDOW_SAMPLES;
      let to = ((first + CHUNK_WINDOWS) * VAD_WINDOW_SAMPLES).min(samples.len());
      context
        .detect_speech(&samples[from..to])
        .map_err(|error| format!("Could not listen for speech: {error}"))?;
      let heard = context.probabilities();
      if heard.len() != (to - from).div_ceil(VAD_WINDOW_SAMPLES) {
        return Err("The voice activity model heard in windows of another size".to_owned());
      }
      probabilities.extend_from_slice(&heard[lead_in..]);
      first += CHUNK_WINDOWS;
      progress(first.min(windows) as f32 / windows as f32);
    }
    Ok(Speech {
      probabilities,
      window_samples: VAD_WINDOW_SAMPLES as u32,
    })
  }

  fn context(&mut self, model: &Path) -> Result<&WhisperContext, String> {
    if self.loaded.as_ref().is_none_or(|(path, _)| path != model) {
      self.loaded = None;
      let path = model
        .to_str()
        .ok_or_else(|| "The model's path is not valid text".to_owned())?;
      // The GPU first; a computer without one it can use transcribes on its
      // processor, more slowly.
      let context = WhisperContext::new_with_params(path, WhisperContextParameters::default())
        .or_else(|_| {
          let mut cpu = WhisperContextParameters::default();
          cpu.use_gpu(false);
          WhisperContext::new_with_params(path, cpu)
        })
        .map_err(|error| format!("Could not load the model: {error}"))?;
      self.loaded = Some((model.to_owned(), context));
    }
    Ok(&self.loaded.as_ref().expect("loaded above").1)
  }
}

fn threads() -> i32 {
  std::thread::available_parallelism()
    .map_or(4, |count| count.get().min(8))
    .try_into()
    .unwrap_or(4)
}

fn read_samples(path: &Path) -> Result<Vec<f32>, String> {
  let bytes = std::fs::read(path).map_err(|error| format!("Could not read the audio: {error}"))?;
  let (samples, _) = bytes.as_chunks::<4>();
  Ok(samples.iter().copied().map(f32::from_ne_bytes).collect())
}

/// The engine's times are in hundredths of a second.
fn ms(centiseconds: i64) -> u64 {
  u64::try_from(centiseconds).unwrap_or(0) * 10
}

fn segments(context: &WhisperContext, state: &WhisperState) -> Result<Vec<Segment>, String> {
  let failed = |error: whisper_rs::WhisperError| format!("Could not read the transcript: {error}");
  // Everything from end-of-text up is a control token: timestamps, language
  // tags and the like, which are not speech.
  let first_special = context.token_eot();
  let mut segments = Vec::new();
  for segment in state.as_iter() {
    let mut words: Vec<Word> = Vec::new();
    // A character can be split across tokens, so a word's bytes are joined
    // before they are read as text.
    let mut bytes: Vec<u8> = Vec::new();
    let mut probabilities: Vec<f32> = Vec::new();
    let mut span = (0, 0);
    let mut finish = |bytes: &mut Vec<u8>, probabilities: &mut Vec<f32>, span: (u64, u64)| {
      let text = String::from_utf8_lossy(bytes).trim().to_owned();
      if !text.is_empty() {
        #[expect(clippy::cast_precision_loss, reason = "a word has few tokens")]
        let probability = probabilities.iter().sum::<f32>() / probabilities.len() as f32;
        words.push(Word {
          end_ms: span.1,
          probability,
          start_ms: span.0,
          text,
        });
      }
      bytes.clear();
      probabilities.clear();
    };
    for index in 0..segment.n_tokens() {
      let Some(token) = segment.get_token(index) else {
        continue;
      };
      let data = token.token_data();
      if data.id >= first_special {
        continue;
      }
      let piece = token.to_bytes().map_err(failed)?;
      // A token opening with a space starts a word.
      if piece.first() == Some(&b' ') && !bytes.is_empty() {
        finish(&mut bytes, &mut probabilities, span);
      }
      if bytes.is_empty() {
        span.0 = ms(data.t0);
      }
      span.1 = ms(data.t1);
      bytes.extend_from_slice(piece);
      probabilities.push(data.p);
    }
    finish(&mut bytes, &mut probabilities, span);
    let text = String::from_utf8_lossy(segment.to_bytes().map_err(failed)?)
      .trim()
      .to_owned();
    if text.is_empty() {
      continue;
    }
    segments.push(Segment {
      end_ms: ms(segment.end_timestamp()),
      start_ms: ms(segment.start_timestamp()),
      text,
      words,
    });
  }
  Ok(segments)
}
