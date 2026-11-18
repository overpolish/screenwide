// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useRef, useState } from "react";

import { findSensitive } from "../api";

import type { SensitiveFind } from "../../../bindings/SensitiveFind";
import type { NotesDocument } from "../document/notes-document";

/** How long after the last change the notes are looked over again, in
 * milliseconds, so typing is not looked at a letter at a time. */
const LOOK_AFTER_MS = 800;

/** What may be private in turn `turn`, whose words are `text`. */
export type SensitiveLookup = (
  turn: number,
  text: string,
) => readonly SensitiveFind[];

const NONE: readonly SensitiveFind[] = [];

/** A lookup over finds already known, by paragraph text. */
export const sensitiveByText =
  (found: ReadonlyMap<string, readonly SensitiveFind[]>): SensitiveLookup =>
  (_, text) =>
    found.get(text) ?? NONE;

type Known = {
  /** What was found, by paragraph text. */
  byText: ReadonlyMap<string, readonly SensitiveFind[]>;
  /** Each turn's text when it was last looked at. */
  byTurn: ReadonlyMap<number, string>;
};

/** What may be private in the notes. Each paragraph is looked at once its
 * notes are finished and again a moment after its words change; notes
 * still being taken are not looked at, as they still grow. Until a changed
 * paragraph has been looked at again, what was found in it stays where its
 * words are unchanged, so typing does not take every mark away. */
export function useSensitive(
  document: NotesDocument | null,
  isTaking: boolean,
): SensitiveLookup {
  const [known, setKnown] = useState<Known>(() => ({
    byText: new Map(),
    byTurn: new Map(),
  }));
  // Asked about already, or being: a paragraph is asked about once.
  const askedRef = useRef(new Set<string>());
  useEffect(() => {
    if (!document || isTaking) return;
    const timer = setTimeout(() => {
      const asked = askedRef.current;
      const turns = document.turns.map(({ text }) => text);
      const texts = [...new Set(turns)].filter(
        (text) => text.trim() !== "" && !asked.has(text),
      );
      if (texts.length === 0) return;
      for (const text of texts) asked.add(text);
      findSensitive(texts)
        .then((results) => {
          setKnown((before) => {
            const byText = new Map(before.byText);
            texts.forEach((text, index) => {
              byText.set(text, results[index] ?? NONE);
            });
            return { byText, byTurn: new Map(turns.entries()) };
          });
        })
        .catch((cause: unknown) => {
          for (const text of texts) asked.delete(text);
          console.error("Could not look for what may be private", cause);
        });
    }, LOOK_AFTER_MS);
    return () => {
      clearTimeout(timer);
    };
  }, [document, isTaking]);
  return useCallback(
    (turn, text) => {
      const found = known.byText.get(text);
      if (found) return found;
      const before = known.byTurn.get(turn);
      if (before === undefined) return NONE;
      return (known.byText.get(before) ?? NONE).filter(
        ({ end, start }) => text.slice(start, end) === before.slice(start, end),
      );
    },
    [known],
  );
}
