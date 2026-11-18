// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo, useState } from "react";

import {
  findInNotes,
  type NotesMatch,
  replaceMatches,
} from "../document/notes-find";

import type { NotesDocument } from "../document/notes-document";

/** The search over `document`: what it finds, and which match was gone to
 * last. A new search starts again from before the first match. Replacing
 * gives the notes as they would be, for the caller to take as one step of
 * their history. */
export function useNotesFind(document: NotesDocument | null) {
  const [query, setQuery] = useState("");
  const [last, setLast] = useState({ index: -1, query: "" });
  const matches = useMemo(
    (): NotesMatch[] => (document ? findInNotes(document, query) : []),
    [document, query],
  );
  const current =
    last.query === query && last.index < matches.length ? last.index : -1;

  const step = (by: -1 | 1) => {
    if (matches.length === 0) return;
    const next =
      current < 0
        ? by > 0
          ? 0
          : matches.length - 1
        : (current + by + matches.length) % matches.length;
    setLast({ index: next, query });
  };

  /** The notes with the current match replaced, going on to the match after
   * it; null when no match was gone to yet, which goes to the first. */
  const replaceOne = (replacement: string): NotesDocument | null => {
    if (!document || current < 0) {
      step(1);
      return null;
    }
    const replaced = matches[current];
    const next = replaceMatches(document, [replaced], replacement);
    // The match after the replacement, so a replacement that itself
    // matches is not replaced again.
    const after = replaced.from + replacement.length;
    setLast({
      index: findInNotes(next, query).findIndex(
        ({ from, turn }) =>
          turn > replaced.turn || (turn === replaced.turn && from >= after),
      ),
      query,
    });
    return next;
  };

  const replaceAll = (replacement: string): NotesDocument | null => {
    if (!document || matches.length === 0) return null;
    setLast({ index: -1, query });
    return replaceMatches(document, matches, replacement);
  };

  return { current, matches, query, replaceAll, replaceOne, setQuery, step };
}
