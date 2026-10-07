// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::{Path, PathBuf};

use screenwide_transcriber::{Request, Segment, Transcript, Word};
use whisper_rs::{
  FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState,
};

/// The model last loaded, kept for the requests after it: loading takes far
/// longer than transcribing a short note.
#[derive(Default)]
pub struct Engine {
  loaded: Option<(PathBuf, WhisperContext)>,
}

impl Engine {
  pub fn transcribe(&mut self, request: &Request) -> Result<Transcript, String> {
    let samples = read_samples(&request.audio)?;
    let context = self.context(&request.model)?;
    let mut state = context
      .create_state()
      .map_err(|error| format!("Could not start transcribing: {error}"))?;

    // A code the engine does not know is detected instead of failing.
    let language = request
      .language
      .as_deref()
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
    let language = state
      .full_lang_id_from_state()
      .ok()
      .and_then(whisper_rs::get_lang_str)
      .unwrap_or(language)
      .to_owned();
    let text = segments
      .iter()
      .map(|segment| segment.text.as_str())
      .collect::<Vec<_>>()
      .join(" ");
    Ok(Transcript {
      language,
      model: request.model_id.clone(),
      segments,
      text,
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
  for segment in 0..state.full_n_segments().map_err(failed)? {
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
    for token in 0..state.full_n_tokens(segment).map_err(failed)? {
      let data = state.full_get_token_data(segment, token).map_err(failed)?;
      if data.id >= first_special {
        continue;
      }
      let piece = state.full_get_token_bytes(segment, token).map_err(failed)?;
      // A token opening with a space starts a word.
      if piece.first() == Some(&b' ') && !bytes.is_empty() {
        finish(&mut bytes, &mut probabilities, span);
      }
      if bytes.is_empty() {
        span.0 = ms(data.t0);
      }
      span.1 = ms(data.t1);
      bytes.extend_from_slice(&piece);
      probabilities.push(data.p);
    }
    finish(&mut bytes, &mut probabilities, span);
    let text = String::from_utf8_lossy(&state.full_get_segment_bytes(segment).map_err(failed)?)
      .trim()
      .to_owned();
    if text.is_empty() {
      continue;
    }
    segments.push(Segment {
      end_ms: ms(state.full_get_segment_t1(segment).map_err(failed)?),
      start_ms: ms(state.full_get_segment_t0(segment).map_err(failed)?),
      text,
      words,
    });
  }
  Ok(segments)
}
