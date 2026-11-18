// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What in the notes may be private, for the notes page to point out: found
//! by form, and by the meeting notes download's model once it is on this
//! computer (`screenwide_notes::sensitive`). Each paragraph is looked at once
//! for as long as the app runs, so after an edit only the paragraphs changed
//! are looked at again.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, PoisonError};
use std::time::{Duration, Instant};

use screenwide_notes::sensitive;
use screenwide_speech::runner::sensitive::{SensitiveKind, SensitiveSpan};
use screenwide_speech::runner::Transcriber;
use serde::Serialize;
use tauri::AppHandle;
use ts_rs::TS;

use crate::transcription::models;

/// How many paragraphs' finds are kept before they are all let go.
const KEPT: usize = 4_096;

/// What was found in each paragraph looked at, by its text.
static FOUND: LazyLock<Mutex<HashMap<String, Vec<SensitiveSpan>>>> = LazyLock::new(Mutex::default);

/// How long the helper stays loaded after it last looked, so the pauses in
/// a run of edits do not each load the model again.
const KEEP_HELPER: Duration = Duration::from_secs(60);

/// The helper with the model loaded, and when it last looked.
static HELPER: Mutex<Option<(Transcriber, Instant)>> = Mutex::new(None);

/// Something in a paragraph that may be private, as `[start, end)` in
/// UTF-16 code units, as the page counts them.
#[derive(Clone, Copy, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SensitiveFind {
  pub start: u32,
  pub end: u32,
  pub kind: SensitiveFindKind,
}

/// What a [`SensitiveFind`] is.
#[derive(Clone, Copy, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SensitiveFindKind {
  Name,
  Company,
  Address,
  Email,
  Phone,
  Account,
  Card,
  Link,
  Health,
  Religion,
  Sexuality,
  Ethnicity,
  Politics,
  Union,
}

impl From<SensitiveSpan> for SensitiveFind {
  fn from(span: SensitiveSpan) -> Self {
    Self {
      start: u32::try_from(span.start).unwrap_or(u32::MAX),
      end: u32::try_from(span.end).unwrap_or(u32::MAX),
      kind: match span.kind {
        SensitiveKind::Name => SensitiveFindKind::Name,
        SensitiveKind::Company => SensitiveFindKind::Company,
        SensitiveKind::Address => SensitiveFindKind::Address,
        SensitiveKind::Email => SensitiveFindKind::Email,
        SensitiveKind::Phone => SensitiveFindKind::Phone,
        SensitiveKind::Account => SensitiveFindKind::Account,
        SensitiveKind::Card => SensitiveFindKind::Card,
        SensitiveKind::Link => SensitiveFindKind::Link,
        SensitiveKind::Health => SensitiveFindKind::Health,
        SensitiveKind::Religion => SensitiveFindKind::Religion,
        SensitiveKind::Sexuality => SensitiveFindKind::Sexuality,
        SensitiveKind::Ethnicity => SensitiveFindKind::Ethnicity,
        SensitiveKind::Politics => SensitiveFindKind::Politics,
        SensitiveKind::Union => SensitiveFindKind::Union,
      },
    }
  }
}

/// What may be private in each of `texts`, the notes' paragraphs, in the
/// same order.
#[tauri::command]
pub async fn find_sensitive(
  app: AppHandle,
  texts: Vec<String>,
) -> Result<Vec<Vec<SensitiveFind>>, String> {
  tauri::async_runtime::spawn_blocking(move || find(&app, &texts))
    .await
    .map_err(|error| error.to_string())
}

fn found() -> std::sync::MutexGuard<'static, HashMap<String, Vec<SensitiveSpan>>> {
  FOUND.lock().unwrap_or_else(PoisonError::into_inner)
}

fn find(app: &AppHandle, texts: &[String]) -> Vec<Vec<SensitiveFind>> {
  let mut unseen: Vec<String> = {
    let found = found();
    texts
      .iter()
      .filter(|text| !found.contains_key(*text))
      .cloned()
      .collect()
  };
  unseen.sort_unstable();
  unseen.dedup();
  if !unseen.is_empty() {
    let by_model = by_model(app, &unseen);
    let mut found = found();
    if found.len() + unseen.len() > KEPT {
      found.clear();
    }
    for (text, model) in unseen.into_iter().zip(by_model) {
      let forms = sensitive::patterns(&text);
      let special = sensitive::special_words(&text);
      found.insert(text, sensitive::merged(model, forms, special));
    }
  }
  let found = found();
  texts
    .iter()
    .map(|text| {
      found
        .get(text)
        .map(|spans| spans.iter().copied().map(SensitiveFind::from).collect())
        .unwrap_or_default()
    })
    .collect()
}

/// What the model finds in each of `texts`; nothing while it is not on this
/// computer, or when it cannot run, which is said and leaves the forms.
fn by_model(app: &AppHandle, texts: &[String]) -> Vec<Vec<SensitiveSpan>> {
  let nothing = || vec![Vec::new(); texts.len()];
  let Some((model, tokenizer)) = models::sensitive_files(app) else {
    return nothing();
  };
  let mut helper = HELPER.lock().unwrap_or_else(PoisonError::into_inner);
  let found = match helper.take() {
    Some((running, _)) => Ok(running),
    None => Transcriber::start(),
  }
  .and_then(|mut running| {
    let found = running.find_sensitive(model, tokenizer, texts.to_vec())?;
    *helper = Some((running, Instant::now()));
    Ok(found)
  });
  drop(helper);
  std::thread::spawn(|| {
    std::thread::sleep(KEEP_HELPER);
    let mut helper = HELPER.lock().unwrap_or_else(PoisonError::into_inner);
    if helper
      .as_ref()
      .is_some_and(|(_, used)| used.elapsed() >= KEEP_HELPER)
    {
      *helper = None;
    }
  });
  match found {
    Ok(found) if found.len() == texts.len() => found,
    Ok(_) => {
      eprintln!("The sensitive information model answered for other texts");
      nothing()
    }
    Err(error) => {
      eprintln!("Could not look for sensitive information: {error}");
      nothing()
    }
  }
}
