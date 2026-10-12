// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useMemo, useState } from "react";

import { findFillerWords } from "./transcript-fillers";
import { findProfanity } from "./transcript-profanity";
import { loadProfanity, ProfanityLexicon } from "./transcript-profanity-lists";

import type { TranscriptUnit, WordSelection } from "./transcript-units";

/** What a unit is marked as needing a look for: a filler to cut, or
 * profanity. */
export type TranscriptMark = "filler" | "profanity";

/** A run of units marked as one, by the folded text that found it, so every
 * one alike can be found together. */
export type TranscriptMarked = { folded: string; span: WordSelection };

/**
 * The fillers and profanity the transcript still keeps, and each unit's mark.
 * The profanity lists load in the background, so profanity is marked once
 * its language's list has arrived; fillers at once. Profanity takes the mark
 * where both find a word.
 */
export function useTranscriptMarks(
  units: readonly TranscriptUnit[],
  language: string | null,
  allowedWords: readonly string[],
) {
  const [lexicon, setLexicon] = useState<{
    language: string | null;
    lists: ProfanityLexicon;
  } | null>(null);
  useEffect(() => {
    let current = true;
    loadProfanity(language)
      .then((lists) => {
        if (current) setLexicon({ language, lists });
      })
      .catch((cause: unknown) => {
        console.error("Could not read the profanity lists", cause);
      });
    return () => {
      current = false;
    };
  }, [language]);

  const fillers = useMemo(
    () => findFillerWords(units, language),
    [language, units],
  );
  const profanity = useMemo(
    () =>
      lexicon?.language === language
        ? findProfanity(units, lexicon.lists, new Set(allowedWords))
        : [],
    [allowedWords, language, lexicon, units],
  );
  const marks = useMemo(() => {
    const marked = new Map<number, TranscriptMark>();
    const add = (found: readonly TranscriptMarked[], mark: TranscriptMark) => {
      for (const { span } of found)
        for (let index = span.first; index <= span.last; index++)
          marked.set(index, mark);
    };
    add(fillers, "filler");
    add(profanity, "profanity");
    return marked;
  }, [fillers, profanity]);

  return { fillers, marks, profanity };
}
