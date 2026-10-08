// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * The pseudo-locale's spelling: every letter accented, every vowel doubled.
 * Text that reads plainly was never translated, and the doubled vowels make
 * it about a third longer, as many translations are. Rust applies the same
 * mapping in `src-tauri/src/i18n/pseudo.rs`; keep the two identical.
 */
const PLAIN = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
// One UTF-16 unit each, so it indexes in step with `PLAIN`.
const ACCENTED = "ȧƀƈḓḗƒɠħīĵķŀḿƞǿƥɋřşŧŭṽẇẋẏẑȦƁƇḒḖƑƓĦĪĴĶĿḾȠǾƤɊŘŞŦŬṼẆẊẎẐ";

export const pseudo = (text: string) =>
  text.replace(/[a-z]/giu, (letter) => {
    const accented = ACCENTED.charAt(PLAIN.indexOf(letter));
    return /[aeiou]/iu.test(letter) ? accented + accented : accented;
  });
