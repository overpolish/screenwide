// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Dictation's cleanup helper: a small language model, run with llama.cpp,
//! that applies the corrections a speaker makes as they speak. It reads
//! requests from standard input until it closes and answers each on
//! standard output, as the other helpers do, keeping the model loaded
//! between requests, since loading takes far longer than a correction.

use std::io::{BufRead, Write};
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{LlamaChatMessage, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use screenwide_transcriber::{Job, Reply, Request};

/// Room for the instructions, a dictation's text and the corrected text.
const CONTEXT_TOKENS: u32 = 4_096;
/// How many prompt tokens are read at once.
const BATCH_TOKENS: usize = 512;

fn main() {
  // llama.cpp and ggml would otherwise write their every step to standard
  // error.
  llama_cpp_2::send_logs_to_tracing(llama_cpp_2::LogOptions::default().with_logs_enabled(false));
  let Ok(backend) = LlamaBackend::init() else {
    eprintln!("Could not start llama.cpp");
    std::process::exit(2);
  };
  let mut loaded: Option<(PathBuf, LlamaModel)> = None;
  let mut stdout = std::io::stdout().lock();
  for line in std::io::stdin().lock().lines() {
    let Ok(line) = line else { break };
    if line.trim().is_empty() {
      continue;
    }
    let Ok(Request { id, job }) = serde_json::from_str::<Request>(&line) else {
      eprintln!("Could not read a request");
      break;
    };
    let reply = match job {
      // Empty text loads the model without asking it anything, so the
      // first correction is not kept waiting for it.
      Job::CorrectText {
        model,
        instructions,
        text,
      } => model_at(&backend, &mut loaded, &model)
        .and_then(|model| {
          if text.trim().is_empty() {
            Ok(String::new())
          } else {
            correct(&backend, model, &instructions, &text)
          }
        })
        .map(|text| Reply::Corrected { id, text }),
      _ => Err("This helper only corrects dictated text".to_owned()),
    }
    .unwrap_or_else(|message| Reply::Failed { id, message });
    let Ok(json) = serde_json::to_string(&reply) else {
      std::process::exit(3);
    };
    if writeln!(stdout, "{json}")
      .and_then(|()| stdout.flush())
      .is_err()
    {
      break;
    }
  }
}

/// The model at `path`, loaded unless it already is.
fn model_at<'a>(
  backend: &LlamaBackend,
  loaded: &'a mut Option<(PathBuf, LlamaModel)>,
  path: &Path,
) -> Result<&'a LlamaModel, String> {
  if !matches!(loaded, Some((at, _)) if at == path) {
    *loaded = None;
    let params = LlamaModelParams::default().with_n_gpu_layers(u32::MAX);
    let model = LlamaModel::load_from_file(backend, path, &params)
      .map_err(|error| format!("Could not load the cleanup model: {error}"))?;
    *loaded = Some((path.to_owned(), model));
  }
  loaded
    .as_ref()
    .map(|(_, model)| model)
    .ok_or_else(|| "The cleanup model is not loaded".to_owned())
}

/// `text` as the model corrects it, told what to do by `instructions`,
/// picking the likeliest token each time so the same text always comes out
/// the same.
fn correct(
  backend: &LlamaBackend,
  model: &LlamaModel,
  instructions: &str,
  text: &str,
) -> Result<String, String> {
  let prompt = prompt(model, instructions, text)?;
  let vocab = model.vocab();
  let tokens = vocab.tokenize(prompt.as_bytes(), true, true);
  let text_tokens = vocab.tokenize(text.as_bytes(), false, false).len();
  // A correction is never much longer than what was said; a model that
  // runs on is stopped rather than waited for.
  let most = text_tokens * 2 + 32;
  if tokens.len() + most > CONTEXT_TOKENS as usize {
    return Err("The text is too long to correct".to_owned());
  }
  let params = LlamaContextParams::default()
    .with_n_ctx(NonZeroU32::new(CONTEXT_TOKENS))
    .with_n_batch(BATCH_TOKENS as u32);
  let mut context = model
    .new_context(backend, params)
    .map_err(|error| format!("Could not start the cleanup model: {error}"))?;
  read_prompt(&mut context, &tokens)?;

  let mut sampler = LlamaSampler::greedy();
  let mut batch = LlamaBatch::new(1, 1);
  let mut written = Vec::new();
  let first = i32::try_from(tokens.len()).unwrap_or(i32::MAX);
  for position in (first..).take(most) {
    let token = sampler.sample(&context, -1);
    if vocab.is_eog(token) {
      break;
    }
    written.extend(vocab.token_to_piece(token, false, None));
    batch.clear();
    batch
      .add(token, position, &[0], true)
      .map_err(|error| format!("Could not correct the text: {error}"))?;
    context
      .decode(&mut batch)
      .map_err(|error| format!("Could not correct the text: {error}"))?;
  }
  Ok(without_thinking(&String::from_utf8_lossy(&written)))
}

/// Reads `tokens` into `context` a batch at a time, asking for the next
/// token's likelihoods after the last.
fn read_prompt(
  context: &mut LlamaContext<'_>,
  tokens: &[llama_cpp_2::token::LlamaToken],
) -> Result<(), String> {
  let mut batch = LlamaBatch::new(BATCH_TOKENS, 1);
  for (chunk_at, chunk) in tokens.chunks(BATCH_TOKENS).enumerate() {
    batch.clear();
    for (offset, &token) in chunk.iter().enumerate() {
      let at = chunk_at * BATCH_TOKENS + offset;
      let position = i32::try_from(at).unwrap_or(i32::MAX);
      batch
        .add(token, position, &[0], at + 1 == tokens.len())
        .map_err(|error| format!("Could not read the text: {error}"))?;
    }
    context
      .decode(&mut batch)
      .map_err(|error| format!("Could not read the text: {error}"))?;
  }
  Ok(())
}

/// `written` without any reasoning a model put before its answer: Qwen's
/// thinking block, or Gemma's thought channel.
fn without_thinking(written: &str) -> String {
  let answer = written
    .rsplit_once("</think>")
    .map_or(written, |(_, answer)| answer);
  let answer = answer
    .rsplit_once("<channel|>")
    .map_or(answer, |(_, answer)| answer);
  answer.trim_start_matches("<think>").trim().to_owned()
}

/// The chat prompt for `instructions` and then `text`, in the model's own
/// format, its thinking turned off: these models are trained to answer at
/// once, and a reasoning model's thinking would only cost time.
fn prompt(model: &LlamaModel, instructions: &str, text: &str) -> Result<String, String> {
  let template = model
    .chat_template(None)
    .map_err(|error| format!("The cleanup model has no chat template: {error}"))?;
  let source = template.to_string().unwrap_or_default();
  // Gemma 4's turns, which llama.cpp's own templates do not know yet.
  if source.contains("<|turn>") {
    return Ok(format!(
      "<|turn>system\n{instructions}<turn|>\n<|turn>user\n{text}<turn|>\n<|turn>model\n"
    ));
  }
  let message = |role: &str, content: &str| {
    LlamaChatMessage::new(role.to_owned(), content.to_owned())
      .map_err(|error| format!("Could not write the prompt: {error}"))
  };
  let chat = [message("system", instructions)?, message("user", text)?];
  let prompt = model
    .apply_chat_template(&template, &chat, true)
    .map_err(|error| format!("Could not write the prompt: {error}"))?;
  // Qwen's way of answering without thinking: an empty thinking block.
  if source.contains("enable_thinking") && source.contains("<think>") {
    return Ok(format!("{prompt}<think>\n\n</think>\n\n"));
  }
  Ok(prompt)
}
