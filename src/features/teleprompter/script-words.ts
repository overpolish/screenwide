// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

// A script as the teleprompter shows and reads it. It is shown exactly as
// written, every character in place, so the field it is edited in can lie
// over it letter for letter; the Markdown that only styles it (heading and
// list marks, emphasis) is shown dimmed and never read. Each word keeps
// where it starts in the text, so a place in the script survives an edit.

export type ScriptPiece =
  /** A word read aloud, and its place among them. */
  | { index: number; kind: "word"; text: string }
  /** Markdown that only styles the script, dimmed. */
  | { kind: "mark"; text: string }
  /** Spaces, line breaks and punctuation that stands on its own. */
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

/** A run of marks on its own at a line's start: a heading's hashes, a
 * quote, a list's bullet or number. */
const LINE_MARK = /^(?:#{1,6}|>|[-+*]|\d+[.)])$/;
/** Emphasis round a word, and what stands after it. */
const AROUND = /^([#>*_`~]*)(.*?)([*_`~]*)([^\p{L}\p{N}*_`~]*)$/su;

export function scriptText(markdown: string): ScriptText {
  const pieces: ScriptPiece[] = [];
  const spoken: string[] = [];
  const starts: number[] = [];
  const add = (kind: "mark" | "text", text: string) => {
    if (text !== "") pieces.push({ kind, text });
  };
  let atLineStart = true;
  for (const match of markdown.matchAll(/\s+|\S+/g)) {
    const run = match[0];
    if (/^\s/.test(run)) {
      add("text", run);
      if (run.includes("\n")) atLineStart = true;
      continue;
    }
    const isLineMark = atLineStart && LINE_MARK.test(run);
    atLineStart = false;
    const [, lead = "", core = "", trail = "", after = ""] =
      AROUND.exec(run) ?? [];
    const form = spokenForm(core);
    if (isLineMark || form === "") {
      add(isLineMark || /^[#>*_`~]+$/.test(run) ? "mark" : "text", run);
      continue;
    }
    add("mark", lead);
    pieces.push({ index: spoken.length, kind: "word", text: core });
    spoken.push(form);
    starts.push(match.index + lead.length);
    add("mark", trail);
    add("text", after);
  }
  return { pieces, spoken, starts };
}

/**
 * The place, the index of the next word to read, after `before` was edited
 * into `after`: still at the same word, which moves with whatever was
 * typed or taken out ahead of it.
 */
export function placeAfterEdit(
  before: { markdown: string; text: ScriptText },
  after: { markdown: string; text: ScriptText },
  place: number,
) {
  const word = Math.floor(place);
  let at = before.text.starts[word] ?? before.markdown.length;
  let common = 0;
  while (
    common < at &&
    common < after.markdown.length &&
    before.markdown[common] === after.markdown[common]
  )
    common += 1;
  if (common < at)
    at = Math.max(common, at + after.markdown.length - before.markdown.length);
  const found = after.text.starts.findIndex((start) => start >= at);
  return found < 0 ? after.text.spoken.length : found;
}
