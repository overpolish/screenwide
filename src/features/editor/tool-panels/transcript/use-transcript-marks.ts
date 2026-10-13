// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { useEffect, useMemo, useState } from "react";

import type { TranscriptUnit, WordSelection } from "./transcript-units";
import type { TranscriptMarks } from "../../../../bindings/TranscriptMarks";
import type { TranscriptMarkWord } from "../../../../bindings/TranscriptMarkWord";

/** What a unit is marked as needing a look for: a filler to cut, or
 * profanity. */
export type TranscriptMark = "filler" | "profanity";

/** A run of units marked as one, by the folded text that found it, so every
 * one alike can be found together. */
export type TranscriptMarked = { folded: string; span: WordSelection };

const NONE: readonly TranscriptMarked[] = [];

/** The profanity and fillers among `words`, found by the lists the captions
 * hide them by: `src-tauri/src/editor/speech/marks.rs`. */
const findTranscriptMarks = (
  language: string | null,
  words: TranscriptMarkWord[],
  allowed: readonly string[],
) =>
  invoke<TranscriptMarks | null>("find_transcript_marks", {
    allowed,
    language,
    words,
  });

/**
 * The fillers and profanity the transcript still keeps, and each unit's mark.
 * They are found in the background, so they arrive a moment after the words
 * change. Profanity takes the mark where both find a word.
 */
export function useTranscriptMarks(
  units: readonly TranscriptUnit[],
  language: string | null,
  allowedWords: readonly string[],
) {
  const [found, setFound] = useState<{
    fillers: readonly TranscriptMarked[];
    profanity: readonly TranscriptMarked[];
  }>({ fillers: NONE, profanity: NONE });
  useEffect(() => {
    // Pauses are never marked, so only the words are asked about, each
    // answered by its place among them and mapped back to its unit.
    const unitOf: number[] = [];
    const words: TranscriptMarkWord[] = [];
    units.forEach((unit, index) => {
      if (unit.kind !== "word") return;
      unitOf.push(index);
      words.push({
        endMs: unit.endMs,
        removed: unit.removed,
        startMs: unit.startMs,
        text: unit.text,
      });
    });
    const spans = (marks: TranscriptMarks["fillers"] = []) =>
      marks.map(({ first, folded, last }) => ({
        folded,
        span: { first: unitOf[first], last: unitOf[last] },
      }));
    let current = true;
    findTranscriptMarks(language, words, allowedWords)
      .then((marks) => {
        if (current)
          setFound({
            fillers: spans(marks?.fillers),
            profanity: spans(marks?.profanity),
          });
      })
      .catch((cause: unknown) => {
        console.error("Could not find the transcript's marks", cause);
      });
    return () => {
      current = false;
    };
  }, [allowedWords, language, units]);

  const marks = useMemo(() => {
    const marked = new Map<number, TranscriptMark>();
    const add = (list: readonly TranscriptMarked[], mark: TranscriptMark) => {
      for (const { span } of list)
        for (let index = span.first; index <= span.last; index++)
          marked.set(index, mark);
    };
    add(found.fillers, "filler");
    add(found.profanity, "profanity");
    return marked;
  }, [found]);

  return { fillers: found.fillers, marks, profanity: found.profanity };
}
