// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Finding what in some texts may be private, with the model loaded once for
//! a run of them.

use std::path::PathBuf;

use screenwide_transcriber::{Job, Reply};
pub use screenwide_transcriber::{SensitiveKind, SensitiveSpan};

use super::Transcriber;

impl Transcriber {
  /// What may be private in each of `texts`, in the same order, found by
  /// the GLiNER model at `model` with its tokenizer at `tokenizer`.
  pub fn find_sensitive(
    &mut self,
    model: PathBuf,
    tokenizer: PathBuf,
    texts: Vec<String>,
  ) -> Result<Vec<Vec<SensitiveSpan>>, String> {
    let job = Job::FindSensitive {
      model,
      tokenizer,
      texts,
    };
    match self.ask(job, &mut |_| {})? {
      Reply::SensitiveFound { found, .. } => Ok(found),
      _ => Err("The transcriber answered with something else".to_owned()),
    }
  }
}
