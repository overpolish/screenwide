// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";

// The Script island's search sits in its row and finds in the words its
// section holds, which also holds Replace. The two are drawn apart, so what
// is typed, whether Replace is open, and how many the search found meet
// here; the words, which know what was found, say how to go to a match.
// There is one Script island at a time, so there is one search.

/** What the words do for the search while they are shown. */
type ScriptSearchWords = {
  /** Gives the keys back to the words, from a search left empty. */
  focus: () => void;
  step: (by: -1 | 1) => void;
};

type ScriptSearchState = {
  /** How many matches the words found for `query`. */
  found: number;
  isReplacing: boolean;
  query: string;
  /** Null while no words are shown. */
  words: ScriptSearchWords | null;
};

const CLEAR: ScriptSearchState = {
  found: 0,
  isReplacing: false,
  query: "",
  words: null,
};

export const useScriptSearch = create<ScriptSearchState>(() => CLEAR);

/** Puts the search away with the words, so the next script opens clear. */
export const clearScriptSearch = () => {
  useScriptSearch.setState(CLEAR);
};
