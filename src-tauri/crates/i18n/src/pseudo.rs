// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pseudo-locales' spellings. The webviews apply the same ones in
//! `src/i18n/pseudo.ts`; keep the two identical.

use std::borrow::Cow;

const PLAIN: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const ACCENTED: &str = "ȧƀƈḓḗƒɠħīĵķŀḿƞǿƥɋřşŧŭṽẇẋẏẑȦƁƇḒḖƑƓĦĪĴĶĿḾȠǾƤɊŘŞŦŬṼẆẊẎẐ";
const VOWELS: &str = "aeiouAEIOU";

/// Every letter accented, every vowel doubled. Text that reads plainly was
/// never translated, and the doubled vowels make it about a third longer, as
/// many translations are.
pub(super) fn accented(text: &str) -> Cow<'_, str> {
  let mut result = String::with_capacity(text.len() * 3);
  for letter in text.chars() {
    match PLAIN
      .find(letter)
      .and_then(|index| ACCENTED.chars().nth(index))
    {
      Some(accent) => {
        result.push(accent);
        if VOWELS.contains(letter) {
          result.push(accent);
        }
      }
      None => result.push(letter),
    }
  }
  Cow::Owned(result)
}

/// The text forced right to left, between a right-to-left override and its
/// closing pop, so English reads backwards as a right-to-left language runs.
/// Text that still reads forwards was never translated.
pub(super) fn bidi(text: &str) -> Cow<'_, str> {
  Cow::Owned(format!("\u{202E}{text}\u{202C}"))
}
