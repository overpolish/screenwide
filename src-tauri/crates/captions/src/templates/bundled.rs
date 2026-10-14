// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The templates the app carries, compiled in from `templates/`.

use serde::Deserialize;

use super::manifest::{versioned, ManifestError};

/// `templates/index.json`: which bundled template is the default, and the
/// order the panel offers them in.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Index {
  pub(super) default: String,
  pub(super) templates: Vec<String>,
}

pub(super) const INDEX: &str = include_str!("../../templates/index.json");

/// Every bundled manifest, one per template folder; the index orders them.
pub(super) const MANIFESTS: [&str; 10] = [
  include_str!("../../templates/box/template.json"),
  include_str!("../../templates/clean/template.json"),
  include_str!("../../templates/pop/template.json"),
  include_str!("../../templates/highlight/template.json"),
  include_str!("../../templates/karaoke/template.json"),
  include_str!("../../templates/punch/template.json"),
  include_str!("../../templates/editorial/template.json"),
  include_str!("../../templates/supersize/template.json"),
  include_str!("../../templates/stack/template.json"),
  include_str!("../../templates/staircase/template.json"),
];

pub(super) fn index() -> Result<Index, ManifestError> {
  serde_json::from_value(versioned(INDEX)?.1).map_err(ManifestError::json)
}
