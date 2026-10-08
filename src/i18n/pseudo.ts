// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * The pseudo-locales' spellings. Rust applies the same ones in
 * `src-tauri/src/i18n/pseudo.rs`; keep the two identical.
 */

const PLAIN = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
// One UTF-16 unit each, so it indexes in step with `PLAIN`.
const ACCENTED = "ȧƀƈḓḗƒɠħīĵķŀḿƞǿƥɋřşŧŭṽẇẋẏẑȦƁƇḒḖƑƓĦĪĴĶĿḾȠǾƤɊŘŞŦŬṼẆẊẎẐ";

/**
 * Every letter accented, every vowel doubled. Text that reads plainly was
 * never translated, and the doubled vowels make it about a third longer, as
 * many translations are.
 */
export const accented = (text: string) =>
  text.replace(/[a-z]/giu, (letter) => {
    const accent = ACCENTED.charAt(PLAIN.indexOf(letter));
    return /[aeiou]/iu.test(letter) ? accent + accent : accent;
  });

/**
 * The text forced right to left, between a right-to-left override and its
 * closing pop, so English reads backwards as a right-to-left language runs.
 * Text that still reads forwards was never translated. Placeholders sit
 * outside the text Fluent hands here, so names and numbers keep their order.
 */
export const bidi = (text: string) => `\u202E${text}\u202C`;
