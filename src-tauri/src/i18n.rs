// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The words the app writes itself: the tray menu, file dialogs, alert titles,
//! window titles and the native overlays. They come from the same Fluent
//! files as the webviews' (`locales/`), so a translation is one set of files.
//! The language is chosen once, at launch, and every window asks for it
//! through `get_app_locale`, so native and web text never disagree.

use std::sync::LazyLock;

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};
use fluent_langneg::{negotiate_languages, NegotiationStrategy};
use serde::Serialize;
use ts_rs::TS;
use unic_langid::LanguageIdentifier;

#[cfg(target_os = "macos")]
mod macos;
mod pseudo;

#[cfg(target_os = "macos")]
pub(crate) use macos::apply_layout_direction;

// `LOCALES`: every folder in `locales/` with its files' text, embedded by
// `build/locales.rs`.
include!(concat!(env!("OUT_DIR"), "/locales.rs"));

/// The language every message is written in first, and the one used when no
/// translation matches the computer's preferences.
const SOURCE: &str = "en-US";
/// Accented, lengthened English, for spotting text that bypasses translation
/// and layouts that break under longer words. Only chosen by the override.
const PSEUDO: &str = "en-XA";
/// English read backwards and laid out right to left, for spotting text that
/// bypasses translation and layouts that do not mirror. Only chosen by the
/// override.
const PSEUDO_BIDI: &str = "en-XB";
/// Chooses the language in place of the computer's preferences, to try a
/// translation or the pseudo-locale without changing system settings.
const OVERRIDE_VARIABLE: &str = "SCREENWIDE_LOCALE";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AppLocale {
  /// The translation in use: a folder in `locales/`, or the pseudo-locale.
  pub language: String,
  /// What dates and numbers are written in: the computer's own locale when it
  /// speaks the translation's language, so British English keeps day-first
  /// dates under the American English translation; the translation's
  /// otherwise, so they never mix two languages.
  pub format_locale: String,
}

struct Catalog {
  locale: AppLocale,
  bundle: FluentBundle<FluentResource>,
  /// The source language, for a message the translation does not have yet.
  /// Absent when the translation is the source.
  source: Option<FluentBundle<FluentResource>>,
}

static CATALOG: LazyLock<Catalog> = LazyLock::new(|| {
  let available: Vec<&str> = LOCALES.iter().map(|(name, _)| *name).collect();
  let locale = resolve(&requested(), &available);
  let pseudo: Option<fn(&str) -> std::borrow::Cow<'_, str>> = match locale.language.as_str() {
    PSEUDO => Some(pseudo::accented),
    PSEUDO_BIDI => Some(pseudo::bidi),
    _ => None,
  };
  let active = if let Some(transform) = pseudo {
    let mut spelled = bundle(SOURCE, SOURCE);
    spelled.set_transform(Some(transform));
    spelled
  } else {
    bundle(&locale.language, &locale.format_locale)
  };
  let source = (locale.language != SOURCE && pseudo.is_none()).then(|| bundle(SOURCE, SOURCE));
  Catalog {
    locale,
    bundle: active,
    source,
  }
});

/// The override when it is set, the computer's preferred languages otherwise,
/// most preferred first.
fn requested() -> Vec<String> {
  match std::env::var(OVERRIDE_VARIABLE) {
    Ok(language) if !language.is_empty() => vec![language],
    _ => sys_locale::get_locales().collect(),
  }
}

fn resolve(requested: &[String], available: &[&str]) -> AppLocale {
  if let Some(pseudo) = requested
    .first()
    .filter(|language| [PSEUDO, PSEUDO_BIDI].contains(&language.as_str()))
  {
    return AppLocale {
      language: pseudo.clone(),
      format_locale: pseudo.clone(),
    };
  }
  let parse = |language: &str| language.parse::<LanguageIdentifier>().ok();
  let requested: Vec<_> = requested
    .iter()
    .filter_map(|language| parse(language))
    .collect();
  let available: Vec<_> = available
    .iter()
    .filter_map(|language| parse(language))
    .collect();
  let source = parse(SOURCE).expect("the source language is a valid identifier");
  let language = negotiate_languages(
    &requested,
    &available,
    Some(&source),
    NegotiationStrategy::Lookup,
  )
  .first()
  .map_or_else(|| source.clone(), |language| (*language).clone());
  let format_locale = requested
    .iter()
    .find(|candidate| candidate.language == language.language)
    .unwrap_or(&language)
    .to_string();
  AppLocale {
    language: language.to_string(),
    format_locale,
  }
}

/// One language's messages, writing numbers and choosing plural forms for
/// `locale`, which `resolve` keeps in the same language. A file that fails
/// to parse still contributes the messages around its error, so one typo in
/// a translation costs one message.
fn bundle(language: &str, locale: &str) -> FluentBundle<FluentResource> {
  let identifier = locale
    .parse()
    .unwrap_or_else(|_| SOURCE.parse().expect("valid"));
  let mut bundle = FluentBundle::new_concurrent(vec![identifier]);
  let sources = LOCALES
    .iter()
    .find(|(name, _)| *name == language)
    .map_or(&[][..], |(_, sources)| *sources);
  for source in sources {
    let resource =
      FluentResource::try_new((*source).to_owned()).unwrap_or_else(|(resource, errors)| {
        eprintln!("Translation {language} has syntax errors: {errors:?}");
        resource
      });
    if let Err(errors) = bundle.add_resource(resource) {
      eprintln!("Translation {language} repeats messages: {errors:?}");
    }
  }
  bundle
}

/// The message `id` in the app's language, with `args` in its placeholders.
/// Falls back to the source language, then to the id itself, which
/// `pnpm i18n:check` keeps from happening by checking every id in `t!`.
pub(crate) fn format(id: &str, args: Option<&FluentArgs>) -> String {
  let catalog = &*CATALOG;
  for bundle in std::iter::once(&catalog.bundle).chain(catalog.source.as_ref()) {
    let Some(pattern) = bundle.get_message(id).and_then(|message| message.value()) else {
      continue;
    };
    let mut errors = Vec::new();
    let text = bundle.format_pattern(pattern, args, &mut errors);
    if !errors.is_empty() {
      eprintln!("Message {id} formatted with errors: {errors:?}");
    }
    return text.into_owned();
  }
  eprintln!("No message {id}");
  id.to_owned()
}

/// A message in the app's language: `t!("id")`, or `t!("id", name = value)`
/// to fill its `{ $name }` placeholders. The id is a literal so
/// `pnpm i18n:check` can find every one in use.
macro_rules! t {
  ($id:literal) => {
    $crate::i18n::format($id, None)
  };
  ($id:literal, $($name:ident = $value:expr),+ $(,)?) => {{
    let mut args = ::fluent_bundle::FluentArgs::new();
    $(args.set(stringify!($name), $value);)+
    $crate::i18n::format($id, Some(&args))
  }};
}
pub(crate) use t;

/// The language the app is in, for a webview to read before its first render.
#[tauri::command]
pub fn get_app_locale() -> AppLocale {
  CATALOG.locale.clone()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn resolved(requested: &[&str], available: &[&str]) -> (String, String) {
    let requested: Vec<String> = requested
      .iter()
      .map(|language| (*language).to_owned())
      .collect();
    let locale = resolve(&requested, available);
    (locale.language, locale.format_locale)
  }

  #[test]
  fn a_regional_preference_keeps_its_formats_under_another_regions_translation() {
    assert_eq!(
      resolved(&["en-GB"], &["en-US"]),
      ("en-US".into(), "en-GB".into())
    );
  }

  #[test]
  fn an_untranslated_language_falls_back_to_the_source_with_its_formats() {
    assert_eq!(
      resolved(&["de-DE"], &["en-US"]),
      ("en-US".into(), "en-US".into())
    );
  }

  #[test]
  fn a_later_preference_with_a_translation_wins_over_an_earlier_one_without() {
    assert_eq!(
      resolved(&["ja-JP", "de-AT", "en-US"], &["de", "en-US"]),
      ("de".into(), "de-AT".into())
    );
  }

  #[test]
  fn the_pseudo_locales_are_taken_as_asked() {
    for pseudo in [PSEUDO, PSEUDO_BIDI] {
      assert_eq!(
        resolved(&[pseudo], &["en-US"]),
        (pseudo.into(), pseudo.into())
      );
    }
  }

  #[test]
  fn unreadable_preferences_fall_back_to_the_source() {
    assert_eq!(
      resolved(&["", "not a locale"], &["en-US"]),
      ("en-US".into(), "en-US".into())
    );
  }
}
