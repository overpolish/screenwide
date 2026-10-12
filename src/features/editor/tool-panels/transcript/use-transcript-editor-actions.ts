// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo } from "react";

import { joinedCutRanges } from "./transcript-cut-join";
import { TranscriptUnit, WordSelection } from "./transcript-units";
import { wordRuns, wordRunSpans } from "./transcript-words";

import type {
  TranscriptCorrection,
  TranscriptRange,
} from "../tool-panel-transcript";

/**
 * Delete, edit, restore and restore-spelling for the chosen units. Delete
 * takes the silence around what it cuts and keeps one pause at the join
 * (`joinedCutRange`); edit does nothing for a span that holds a pause, since
 * a pause is never corrected.
 */
export function useTranscriptEditorActions({
  durationMs,
  onCorrect,
  onCut,
  onRestore,
  pendingAdd,
  setEditing,
  units,
}: {
  /** How long the recording is, which stands in for the word after the
   * last. */
  durationMs: number;
  onCorrect: (correction: TranscriptCorrection) => void;
  onCut: (ranges: TranscriptRange[]) => void;
  onRestore: (ranges: TranscriptRange[]) => void;
  pendingAdd: (correction: TranscriptCorrection) => void;
  setEditing: (span: WordSelection) => void;
  units: readonly TranscriptUnit[];
}) {
  const removedFlags = useMemo(
    () => units.map((unit) => unit.removed),
    [units],
  );
  const rangeOf = ({ first, last }: WordSelection) => ({
    endMs: units[last].endMs,
    startMs: units[first].startMs,
  });
  const correct = (span: WordSelection, text: string) => {
    const correction = { ...rangeOf(span), text };
    pendingAdd(correction);
    onCorrect(correction);
  };
  const heardOf = (span: WordSelection) =>
    units
      .slice(span.first, span.last + 1)
      .map((unit) => unit.heard)
      .join(" ");
  const actions = {
    delete: (span: WordSelection) => {
      const spans = wordRunSpans(removedFlags, { ...span, removed: false });
      if (spans.length > 0) onCut(joinedCutRanges(units, spans, durationMs));
    },
    edit: (span: WordSelection) => {
      const hasPause = units
        .slice(span.first, span.last + 1)
        .some((unit) => unit.kind === "pause");
      if (!hasPause) setEditing(span);
    },
    restore: (span: WordSelection) => {
      const struck = wordRuns(units, removedFlags, {
        ...span,
        removed: true,
      });
      if (struck.length > 0) onRestore(struck);
    },
    spelling: (span: WordSelection) => {
      correct(span, heardOf(span));
    },
  };
  return { actions, correct };
}
