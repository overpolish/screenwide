// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

// A script as the teleprompter shows and reads it: plain text, shown exactly
// as written, every character in place, so the field it is edited in can lie
// over it letter for letter. Each word keeps where it starts in the text, so
// a place in the script survives an edit.

export type ScriptPiece =
  /** A word read aloud, and its place among them. */
  | { index: number; kind: "word"; text: string }
  /** Spaces, line breaks and marks that stand on their own, such as a
   * dash, which are not read. */
  | { kind: "text"; text: string };

export type ScriptText = {
  pieces: ScriptPiece[];
  /** Every word, in reading order, as it is matched: see `spokenForm`. */
  spoken: string[];
  /** Where each word starts in the text, in characters. */
  starts: number[];
};

/** A word as it is matched against what is heard: lower case, without
 * accents or punctuation, so "Don't," and "dont" are one word. */
export const spokenForm = (text: string) =>
  text
    .toLocaleLowerCase()
    .normalize("NFKD")
    .replace(/\p{M}/gu, "")
    .replace(/[^\p{L}\p{N}]/gu, "");

export function scriptText(text: string): ScriptText {
  const pieces: ScriptPiece[] = [];
  const spoken: string[] = [];
  const starts: number[] = [];
  for (const match of text.matchAll(/\s+|\S+/g)) {
    const run = match[0];
    const form = spokenForm(run);
    if (form === "") {
      pieces.push({ kind: "text", text: run });
      continue;
    }
    pieces.push({ index: spoken.length, kind: "word", text: run });
    spoken.push(form);
    starts.push(match.index);
  }
  return { pieces, spoken, starts };
}

/** The script as written, and as it is read. */
type Written = { script: ScriptText; text: string };

/**
 * The place, the index of the next word to read, after `before` was edited
 * into `after`: still at the same word, which moves with whatever was
 * typed or taken out ahead of it.
 */
export function placeAfterEdit(before: Written, after: Written, place: number) {
  const word = Math.floor(place);
  let at = before.script.starts[word] ?? before.text.length;
  let common = 0;
  while (
    common < at &&
    common < after.text.length &&
    before.text[common] === after.text[common]
  )
    common += 1;
  if (common < at)
    at = Math.max(common, at + after.text.length - before.text.length);
  const found = after.script.starts.findIndex((start) => start >= at);
  return found < 0 ? after.script.spoken.length : found;
}
