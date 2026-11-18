// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Finding what in a text may be private with GLiNER-PII base (Knowledgator
//! and Wordcab, Apache-2.0), in its authors' 8-bit ONNX export: names of
//! people and companies, addresses, emails, phone numbers, account and card
//! numbers, and links. GLiNER is asked for each of those by name, alongside
//! the text, and scores every run of up to twelve words against each.
//!
//! What the model is given follows its Python package and gline-rs (Apache-2.0),
//! which this mirrors: the text cut into words, each word tokenized apart, and
//! the names asked for put before them as `<<ENT>> name <<ENT>> ... <<SEP>>`.

use std::path::{Path, PathBuf};

use ort::session::Session;
use ort::value::Tensor;
use screenwide_transcriber::{SensitiveKind, SensitiveSpan};
use tokenizers::Tokenizer;

/// The feature's name in what the helper reports.
const WHAT: &str = "Sensitive information";
/// What the model is asked for, by the names it was trained on, what each
/// is, and how sure the model must be of it. At 0.5 it found 18 of 21
/// things in meeting-notes paragraphs with one false find, where 0.3 found
/// one more with five. A company takes 0.6: everyday words such as "store"
/// and "Marketing" scored 0.5 to 0.56 as one, real companies mostly 0.75
/// and up.
const LABELS: [(&str, SensitiveKind, f32); 8] = [
  ("name", SensitiveKind::Name, 0.5),
  ("organization", SensitiveKind::Company, 0.6),
  ("location address", SensitiveKind::Address, 0.5),
  ("email address", SensitiveKind::Email, 0.5),
  ("phone number", SensitiveKind::Phone, 0.5),
  ("account number", SensitiveKind::Account, 0.5),
  ("credit card", SensitiveKind::Card, 0.5),
  ("url", SensitiveKind::Link, 0.5),
];
/// The longest run of words the model scores, from its configuration.
const MAX_WIDTH: usize = 12;
/// How many words go to the model at once: a long turn is read in pieces,
/// which keeps the run of spans it scores, words times twelve, small.
const CHUNK_WORDS: usize = 256;
/// DeBERTa-v3's tokens opening and closing a sequence.
const OPENING: i64 = 1;
const CLOSING: i64 = 2;

pub struct SensitiveFinder {
  files: (PathBuf, PathBuf),
  session: Session,
  tokenizer: Tokenizer,
  /// The names asked for, as the tokens of each prompt word.
  prompt: Vec<Vec<u32>>,
}

impl SensitiveFinder {
  pub fn load(model: &Path, tokenizer_file: &Path) -> Result<Self, String> {
    let session = crate::session::session(model, WHAT)?;
    let tokenizer = Tokenizer::from_file(tokenizer_file)
      .map_err(|error| format!("Could not load the {WHAT} tokenizer: {error}"))?;
    let mut words = Vec::with_capacity(LABELS.len() * 2 + 1);
    for (label, _, _) in LABELS {
      words.push("<<ENT>>");
      words.push(label);
    }
    words.push("<<SEP>>");
    let prompt = words
      .into_iter()
      .map(|word| encode(&tokenizer, word))
      .collect::<Result<_, _>>()?;
    Ok(Self {
      files: (model.to_owned(), tokenizer_file.to_owned()),
      session,
      tokenizer,
      prompt,
    })
  }

  /// Whether this finder was loaded from `model` and `tokenizer`.
  pub fn is_for(&self, model: &Path, tokenizer: &Path) -> bool {
    self.files.0 == model && self.files.1 == tokenizer
  }

  /// What in `text` may be private, in order.
  pub fn find(&mut self, text: &str) -> Result<Vec<SensitiveSpan>, String> {
    let words = words(text);
    let mut found = Vec::new();
    for chunk in words.chunks(CHUNK_WORDS) {
      found.extend(self.find_in(text, chunk)?);
    }
    Ok(found)
  }

  fn find_in(
    &mut self,
    text: &str,
    words: &[(usize, usize)],
  ) -> Result<Vec<SensitiveSpan>, String> {
    let (ids, words_mask) = self.encoded(text, words)?;
    let length = ids.len();
    let count = words.len();
    let mut span_index = vec![0_i64; count * MAX_WIDTH * 2];
    let mut span_mask = vec![false; count * MAX_WIDTH];
    for start in 0..count {
      for width in 0..MAX_WIDTH.min(count - start) {
        let at = start * MAX_WIDTH + width;
        span_index[at * 2] = start as i64;
        span_index[at * 2 + 1] = (start + width) as i64;
        span_mask[at] = true;
      }
    }
    let tensor = |values: Vec<i64>, shape: Vec<usize>| Tensor::from_array((shape, values));
    let outputs = self
      .session
      .run(ort::inputs![
        "input_ids" => tensor(ids, vec![1, length]).map_err(failed)?,
        "attention_mask" => tensor(vec![1; length], vec![1, length]).map_err(failed)?,
        "words_mask" => tensor(words_mask, vec![1, length]).map_err(failed)?,
        "text_lengths" => tensor(vec![count as i64], vec![1, 1]).map_err(failed)?,
        "span_idx" => tensor(span_index, vec![1, count * MAX_WIDTH, 2]).map_err(failed)?,
        "span_mask" => Tensor::from_array((vec![1, count * MAX_WIDTH], span_mask)).map_err(failed)?,
      ])
      .map_err(failed)?;
    let (shape, logits) = outputs["logits"]
      .try_extract_tensor::<f32>()
      .map_err(failed)?;
    if shape.len() != 4 || shape[1] as usize != count || shape[3] as usize != LABELS.len() {
      return Err(format!("The {WHAT} model answered in an unexpected shape"));
    }
    let width = shape[2] as usize;
    let mut scored = Vec::new();
    for start in 0..count {
      for span in 0..width.min(count - start) {
        for (class, &(_, _, threshold)) in LABELS.iter().enumerate() {
          let logit = logits[((start * width) + span) * LABELS.len() + class];
          let score = 1.0 / (1.0 + (-logit).exp());
          if score >= threshold {
            scored.push((start, start + span, class, score));
          }
        }
      }
    }
    Ok(
      flat(scored)
        .into_iter()
        .map(|(first, last, class)| SensitiveSpan {
          start: utf16_offset(text, words[first].0),
          end: utf16_offset(text, words[last].1),
          kind: LABELS[class].1,
        })
        .collect(),
    )
  }

  /// The tokens of the prompt and the words, opened and closed, and which
  /// token begins each word, numbered from 1.
  fn encoded(&self, text: &str, words: &[(usize, usize)]) -> Result<(Vec<i64>, Vec<i64>), String> {
    let mut ids = vec![OPENING];
    let mut words_mask = vec![0];
    for piece in &self.prompt {
      ids.extend(piece.iter().map(|&id| i64::from(id)));
      words_mask.extend(std::iter::repeat_n(0, piece.len()));
    }
    for (number, &(start, end)) in words.iter().enumerate() {
      let pieces = encode(&self.tokenizer, &text[start..end])?;
      for (index, &id) in pieces.iter().enumerate() {
        ids.push(i64::from(id));
        words_mask.push(if index == 0 { number as i64 + 1 } else { 0 });
      }
    }
    ids.push(CLOSING);
    words_mask.push(0);
    Ok((ids, words_mask))
  }
}

fn encode(tokenizer: &Tokenizer, word: &str) -> Result<Vec<u32>, String> {
  tokenizer
    .encode(word, false)
    .map(|encoding| encoding.get_ids().to_vec())
    .map_err(|error| format!("Could not tokenize for {WHAT}: {error}"))
}

/// The text's words as GLiNER cuts it: runs of letters, digits and
/// underscores, joined across a hyphen, and each other character that is
/// not a space on its own, as `[start, end)` in bytes.
fn words(text: &str) -> Vec<(usize, usize)> {
  let is_word = |c: char| c.is_alphanumeric() || c == '_';
  let mut words = Vec::new();
  let mut chars = text.char_indices().peekable();
  while let Some((start, c)) = chars.next() {
    if c.is_whitespace() {
      continue;
    }
    if !is_word(c) {
      words.push((start, start + c.len_utf8()));
      continue;
    }
    let mut end = start + c.len_utf8();
    loop {
      match chars.peek().copied() {
        Some((at, next)) if is_word(next) => {
          end = at + next.len_utf8();
          chars.next();
        }
        Some((_, '-')) => {
          let mut ahead = chars.clone();
          ahead.next();
          match ahead.peek().copied() {
            Some((at, next)) if is_word(next) => {
              chars.next();
              chars.next();
              end = at + next.len_utf8();
            }
            _ => break,
          }
        }
        _ => break,
      }
    }
    words.push((start, end));
  }
  words
}

/// The spans kept from those scored, each as `(first word, last word,
/// class)`: the surest first, then each that overlaps none kept already, in
/// reading order.
fn flat(mut scored: Vec<(usize, usize, usize, f32)>) -> Vec<(usize, usize, usize)> {
  scored.sort_by(|a, b| b.3.total_cmp(&a.3));
  let mut kept: Vec<(usize, usize, usize)> = Vec::new();
  for (first, last, class, _) in scored {
    if kept.iter().all(|&(a, b, _)| last < a || first > b) {
      kept.push((first, last, class));
    }
  }
  kept.sort_unstable();
  kept
}

/// The offset `byte` into `text` in UTF-16 code units.
fn utf16_offset(text: &str, byte: usize) -> usize {
  text[..byte].encode_utf16().count()
}

fn failed(error: ort::Error) -> String {
  format!("{WHAT} could not run: {error}")
}

#[cfg(test)]
mod tests;
