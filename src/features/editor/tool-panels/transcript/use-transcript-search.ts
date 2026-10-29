// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo, useState } from "react";

import { findInTranscript, TranscriptMatch } from "./transcript-search";
import { TranscriptUnit, WordSelection } from "./transcript-units";

/** The search over the transcript for `query`: what it finds, and which
 * match was gone to last, which `onFound` is told of. The query is held by
 * whoever draws the field, which can sit apart from the words; a new query
 * starts again from before the first match. */
export function useTranscriptSearch(
  units: readonly TranscriptUnit[],
  query: string,
  onFound: (match: WordSelection) => void,
) {
  const [last, setLast] = useState({ index: -1, query });
  const current = last.query === query ? last.index : -1;
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
    setLast({ index: next, query });
    onFound(matches[next]);
  };

  return { current, matches, step };
}
