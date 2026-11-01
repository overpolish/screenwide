// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** A word's text as the word and the punctuation either side of it:
 * "(yes)," is "(", "yes" and "),". */
export type CaptionWordParts = { after: string; before: string; word: string };

const LETTER = /[\p{L}\p{N}]/u;
const EDGES = /^([^\p{L}\p{N}]*)(.*?)([^\p{L}\p{N}]*)$/su;

const hasLetters = (text: string) => LETTER.test(text);

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

/** What an edit that left `left`, normalized, in place of the word or
 * words of `text` makes of them: `text` with its punctuation either side
 * kept (`withCaptionPunctuation`), or null where it left no letters of
 * words that had them, which takes them out. Text that was only a mark is
 * taken out only once emptied. */
export function editedWordText(text: string, left: string): string | null {
  if (left === "" || (hasLetters(text) && !hasLetters(left))) return null;
  return withCaptionPunctuation(captionWordParts(text), left);
}
