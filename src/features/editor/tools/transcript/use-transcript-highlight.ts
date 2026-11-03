// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { TranscriptUnit, WordSelection } from "./transcript-units";

/**
 * Tells the editor which stretch the chosen or edited words take, so its
 * timeline can show it, and that none do once the panel goes away. Sent only
 * when the stretch changes, not on every render.
 */
export function useTranscriptHighlight(
  units: readonly TranscriptUnit[],
  span: WordSelection | null,
  onHighlight: (range: { endMs: number; startMs: number } | null) => void,
) {
  const onHighlightRef = useRef(onHighlight);
  onHighlightRef.current = onHighlight;
  const startMs = span ? units[span.first]?.startMs : undefined;
  const endMs = span ? units[span.last]?.endMs : undefined;
  useEffect(() => {
    onHighlightRef.current(
      startMs === undefined || endMs === undefined ? null : { endMs, startMs },
    );
  }, [endMs, startMs]);
  useEffect(
    () => () => {
      onHighlightRef.current(null);
    },
    [],
  );
}
