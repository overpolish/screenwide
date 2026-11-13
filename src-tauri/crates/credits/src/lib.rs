// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Who made a song, sound effect or picture found on Openverse, under which
//! licence, and where it is published, kept with it in the library and in
//! each project that uses it. Only what may be used with no more than a
//! credit is ever taken: public domain, CC0 and CC BY. A CC BY work must be
//! credited wherever it is used, so an export that uses one is written with
//! a credits file beside it (`credits_text`); the others need none.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Who made a work and under which licence it may be used.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Credit {
  /// The work's own title.
  pub title: String,
  pub creator: Option<String>,
  pub creator_url: Option<String>,
  /// The licence's short name, as Openverse writes it: `by`, `cc0` or `pdm`.
  pub license: String,
  pub license_version: Option<String>,
  pub license_url: Option<String>,
  /// Where the work is published.
  pub source_url: Option<String>,
  /// The credit line its licence asks for, as Openverse writes it.
  pub attribution: String,
}

impl Credit {
  /// Whether its licence asks for it to be credited where it is used.
  pub fn is_owed(&self) -> bool {
    self.license == "by"
  }
}

/// The credits file an export carries for the works it uses, each owed one
/// once, in the order given; none where none is owed. Written in English, as
/// the credit lines themselves are.
pub fn credits_text<'a>(credits: impl IntoIterator<Item = &'a Credit>) -> Option<String> {
  let mut written: Vec<&Credit> = Vec::new();
  for credit in credits.into_iter().filter(|credit| credit.is_owed()) {
    if !written.contains(&credit) {
      written.push(credit);
    }
  }
  if written.is_empty() {
    return None;
  }
  let mut text = String::from("Credits\n");
  for credit in written {
    text.push('\n');
    text.push_str(&credit.attribution);
    text.push('\n');
    if let Some(source) = &credit.source_url {
      text.push_str("Source: ");
      text.push_str(source);
      text.push('\n');
    }
  }
  Some(text)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn credit(title: &str, license: &str) -> Credit {
    Credit {
      title: title.to_owned(),
      creator: Some("Ana".to_owned()),
      creator_url: None,
      license: license.to_owned(),
      license_version: Some("4.0".to_owned()),
      license_url: None,
      source_url: Some(format!("https://freesound.org/s/{title}")),
      attribution: format!("\"{title}\" by Ana is licensed under CC BY 4.0."),
    }
  }

  #[test]
  fn only_works_owed_a_credit_are_credited_each_once() {
    let rain = credit("Rain", "by");
    let wind = credit("Wind", "cc0");
    let text = credits_text([&rain, &wind, &rain]).unwrap();
    assert_eq!(
      text,
      "Credits\n\n\"Rain\" by Ana is licensed under CC BY 4.0.\nSource: https://freesound.org/s/Rain\n"
    );
  }

  #[test]
  fn no_file_where_nothing_is_owed() {
    assert_eq!(credits_text([&credit("Wind", "cc0")]), None);
  }
}
