// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** A word's text as the word and the punctuation either side of it:
 * "(yes)," is "(", "yes" and "),". */
export type CaptionWordParts = { after: string; before: string; word: string };

const LETTER = /[\p{L}\p{N}]/u;
const EDGES = /^([^\p{L}\p{N}]*)(.*?)([^\p{L}\p{N}]*)$/su;

export const hasLetters = (text: string) => LETTER.test(text);

/** `text` split at its first and last letter or digit, so marks inside a
 * word stay part of it ("don't", "e-mail"). Text with no letters or digits
 * is all word, so a lone mark can still be edited. */
export function captionWordParts(text: string): CaptionWordParts {
  if (!hasLetters(text)) return { after: "", before: "", word: text };
  const [, before = "", word = "", after = ""] = EDGES.exec(text) ?? [];
  return { after, before, word };
}

/** What `typed` in place of the word in `parts` leaves the text as: the
 * punctuation either side is kept, unless what was typed brings its own on
 * that side ("captions" keeps "captions,"'s comma, "captions." swaps it). */
export function withCaptionPunctuation(
  parts: CaptionWordParts,
  typed: string,
): string {
  const own = captionWordParts(typed);
  return `${own.before || parts.before}${own.word}${own.after || parts.after}`;
}
