// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { TextSpan } from "../../../../../bindings/TextSpan";
import type { TextStyle } from "../../../../../bindings/TextStyle";

/** Whether `look` draws the words chosen in a text any differently: in its
 * accent, or highlighted. The twin of `screenwide_text::emphasis::sets_apart`. */
export const setsWordsApart = (look: TextStyle) =>
  look.accent === "word" || look.highlightWords;

/** One word of a text: what it says, and the bytes it takes in the text's
 * UTF-8, which is how the stored text and Rust name it. */
export type TextWord = { span: TextSpan; text: string };

/** White space as Rust's `char::is_whitespace` reads it: the Unicode
 * White_Space characters, so a word here is the word Rust sets apart. */
const WHITE_SPACE =
  /[\t\n\v\f\r \u0085\u00a0\u1680\u2000-\u200a\u2028\u2029\u202f\u205f\u3000]/u;

/** How many bytes `character`, one code point, takes in UTF-8. */
const utf8Length = (character: string) => {
  const code = character.codePointAt(0) ?? 0;
  return code < 0x80 ? 1 : code < 0x800 ? 2 : code < 0x10000 ? 3 : 4;
};

/** `text`'s lines that hold words, each as its words in order: runs of
 * characters between white space, as `screenwide_text::emphasis::words`
 * reads them. */
export const textWordLines = (text: string): TextWord[][] => {
  const lines: TextWord[][] = [];
  let line: TextWord[] = [];
  let word: TextWord | null = null;
  let at = 0;
  for (const character of text) {
    const length = utf8Length(character);
    if (WHITE_SPACE.test(character)) {
      if (word) line.push(word);
      word = null;
      if (character === "\n") {
        if (line.length > 0) lines.push(line);
        line = [];
      }
    } else if (word) {
      word.text += character;
      word.span.end += length;
    } else {
      word = { span: { end: at + length, start: at }, text: character };
    }
    at += length;
  }
  if (word) line.push(word);
  if (line.length > 0) lines.push(line);
  return lines;
};

/** Whether `word` is among the words `emphasis` chose: any stored range
 * that reaches into it, as Rust reads the choice. */
export const isChosenWord = (emphasis: TextSpan[], word: TextWord) =>
  emphasis.some(
    (span) => span.start < word.span.end && word.span.start < span.end,
  );
