// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { NotesDocument } from "./notes-document";

// Finding and replacing words in the notes. Only what was said is searched:
// a speaker's name is changed by renaming them, which renames them
// throughout, so a replace never touches it.

/** Where a search found something: in turn `turn`, the letters
 * `[from, to)` of its text. */
export type NotesMatch = { from: number; to: number; turn: number };

/** Every place `query` falls in the notes' words, in reading order, without
 * regard to case. */
export function findInNotes(
  { turns }: NotesDocument,
  query: string,
): NotesMatch[] {
  const needle = query.toLocaleLowerCase();
  if (needle === "") return [];
  const matches: NotesMatch[] = [];
  turns.forEach(({ text }, turn) => {
    const haystack = text.toLocaleLowerCase();
    // Letters whose lower case is longer shift what follows, so a match is
    // kept only while it still falls within the text.
    for (
      let from = haystack.indexOf(needle);
      from !== -1 && from + needle.length <= text.length;
      from = haystack.indexOf(needle, from + needle.length)
    ) {
      matches.push({ from, to: from + needle.length, turn });
    }
  });
  return matches;
}

/** The notes with `replacement` in place of each of `matches`. */
export function replaceMatches(
  document: NotesDocument,
  matches: readonly NotesMatch[],
  replacement: string,
): NotesDocument {
  if (matches.length === 0) return document;
  return {
    ...document,
    turns: document.turns.map((turn, index) => {
      const inTurn = matches.filter((match) => match.turn === index);
      if (inTurn.length === 0) return turn;
      let text = turn.text;
      // From the end, so each replacement leaves the letters before it where
      // the matches say they are.
      for (const { from, to } of [...inTurn].sort((a, b) => b.from - a.from))
        text = text.slice(0, from) + replacement + text.slice(to);
      return { ...turn, text };
    }),
  };
}
