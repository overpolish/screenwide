// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The fonts a template brings, as its manifest declares them: each file
//! in the package's `fonts/` folder, the family the file names and the
//! weights and slant it covers, which the font list offers it by, and the
//! licence it is shared under, which travels with it. The app's own
//! templates share one `templates/fonts/` folder, compiled in; an export
//! carries the files each one declares.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use super::{bundled, user};

/// The folder a package keeps its fonts and their licences in.
pub(super) const FONTS: &str = "fonts";
/// The kinds of font file a package may bring.
const FONT_EXTENSIONS: [&str; 2] = ["ttf", "otf"];
/// The kind of file a licence is kept in.
const LICENSE_EXTENSION: &str = "txt";
/// The most fonts one template may declare, and how long a family's name.
const MAX_FONTS: usize = 16;
const MAX_FAMILY: usize = 60;

/// A font file a template brings.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TemplateFont {
  /// Its place in the package, in `fonts/`.
  pub file: String,
  /// The family the file names, which a style asks for.
  pub family: String,
  /// The weights it covers on the CSS scale.
  pub weights: Vec<u16>,
  #[serde(default)]
  pub italic: bool,
  /// The licence it is shared under, in `fonts/`, which travels with it.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub license: Option<String>,
}

/// Whether `path` is a file in the package's `fonts/` folder of one of
/// `extensions`.
fn in_fonts(path: &str, extensions: &[&str]) -> bool {
  let Some(name) = path
    .strip_prefix(FONTS)
    .and_then(|rest| rest.strip_prefix('/'))
  else {
    return false;
  };
  !name.is_empty()
    && !name.contains('/')
    && !name.starts_with('.')
    && std::path::Path::new(name)
      .extension()
      .and_then(|extension| extension.to_str())
      .is_some_and(|extension| {
        extensions
          .iter()
          .any(|wanted| extension.eq_ignore_ascii_case(wanted))
      })
}

/// Whether a package may carry a file at `path` for its fonts: a font or
/// a licence in `fonts/`.
pub(super) fn is_font_file(path: &str) -> bool {
  in_fonts(path, &FONT_EXTENSIONS) || in_fonts(path, &[LICENSE_EXTENSION])
}

/// Why `fonts` cannot be declared as they are, if they cannot.
pub(super) fn check(fonts: &[TemplateFont]) -> Result<(), String> {
  if fonts.len() > MAX_FONTS {
    return Err(format!("it declares more than {MAX_FONTS} fonts"));
  }
  for font in fonts {
    if !in_fonts(&font.file, &FONT_EXTENSIONS) {
      return Err(format!(
        "its font \"{}\" is not a .ttf or .otf file in fonts/",
        font.file
      ));
    }
    if let Some(license) = &font.license {
      if !in_fonts(license, &[LICENSE_EXTENSION]) {
        return Err(format!(
          "its licence \"{license}\" is not a .txt file in fonts/"
        ));
      }
    }
    let family = font.family.trim();
    if family.is_empty() || family.chars().count() > MAX_FAMILY {
      return Err(format!(
        "its font \"{}\" names no family, or one longer than {MAX_FAMILY} characters",
        font.file
      ));
    }
    if font.weights.is_empty()
      || font
        .weights
        .iter()
        .any(|weight| !(1..=1000).contains(weight))
    {
      return Err(format!(
        "its font \"{}\" has no weights, or one off the scale of 1 to 1000",
        font.file
      ));
    }
  }
  Ok(())
}

/// Every file `fonts` need in their package: each font and its licence.
pub(super) fn files_of(fonts: &[TemplateFont]) -> impl Iterator<Item = &str> {
  fonts
    .iter()
    .flat_map(|font| std::iter::once(font.file.as_str()).chain(font.license.as_deref()))
}

/// Every font the app's own templates and yours bring, with its file. A
/// font the app's own templates share is given once; the text engines keep
/// any file once whatever they are given.
pub fn template_font_files() -> Vec<(TemplateFont, Cow<'static, [u8]>)> {
  let mut fonts: Vec<(TemplateFont, Cow<'static, [u8]>)> = Vec::new();
  for template in bundled::templates() {
    for font in &template.fonts {
      if fonts.iter().any(|(known, _)| known == font) {
        continue;
      }
      if let Some(bytes) = bundled::font_file(&font.file) {
        fonts.push((font.clone(), Cow::Borrowed(bytes)));
      }
    }
  }
  fonts.extend(
    user::font_files()
      .into_iter()
      .map(|(font, bytes)| (font, Cow::Owned(bytes))),
  );
  fonts
}
