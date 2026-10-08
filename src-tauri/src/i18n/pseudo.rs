// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The pseudo-locale's spelling: every letter accented, every vowel doubled.
//! Text that reads plainly was never translated, and the doubled vowels make
//! it about a third longer, as many translations are. The webviews apply the
//! same mapping in `src/i18n/pseudo.ts`; keep the two identical.

use std::borrow::Cow;

const PLAIN: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const ACCENTED: &str = "ȧƀƈḓḗƒɠħīĵķŀḿƞǿƥɋřşŧŭṽẇẋẏẑȦƁƇḒḖƑƓĦĪĴĶĿḾȠǾƤɊŘŞŦŬṼẆẊẎẐ";
const VOWELS: &str = "aeiouAEIOU";

pub(super) fn transform(text: &str) -> Cow<'_, str> {
  let mut result = String::with_capacity(text.len() * 3);
  for letter in text.chars() {
    match PLAIN
      .find(letter)
      .and_then(|index| ACCENTED.chars().nth(index))
    {
      Some(accented) => {
        result.push(accented);
        if VOWELS.contains(letter) {
          result.push(accented);
        }
      }
      None => result.push(letter),
    }
  }
  Cow::Owned(result)
}
