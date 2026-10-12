// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo, useState } from "react";

import { findInTranscript, TranscriptMatch } from "./transcript-search";
import { TranscriptUnit, WordSelection } from "./transcript-units";

/** The search over the transcript: what is typed, what it finds, and which
 * match was gone to last, which `onFound` is told of. Changing the search
 * starts again from before the first match. */
export function useTranscriptSearch(
  units: readonly TranscriptUnit[],
  onFound: (match: WordSelection) => void,
) {
  const [query, setQuery] = useState("");
  const [current, setCurrent] = useState(-1);
  const matches = useMemo(
    (): TranscriptMatch[] => findInTranscript(units, query),
    [query, units],
  );

  const step = (by: -1 | 1) => {
    if (matches.length === 0) return;
    const next =
      current < 0 || current >= matches.length
        ? by > 0
          ? 0
          : matches.length - 1
        : (current + by + matches.length) % matches.length;
    setCurrent(next);
    onFound(matches[next]);
  };

  return {
    matches,
    query,
    setQuery: (next: string) => {
      setQuery(next);
      setCurrent(-1);
    },
    step,
  };
}
