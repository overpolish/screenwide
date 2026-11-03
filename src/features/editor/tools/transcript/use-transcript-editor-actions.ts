// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo } from "react";

import { joinedCutRanges } from "./transcript-cut-join";
import {
  heardRuns,
  heardSpanOf,
  isHeardInOrder,
  TranscriptOrder,
} from "./transcript-playback-order";
import { WordSelection } from "./transcript-units";
import { wordRuns, wordRunSpans } from "./transcript-words";

import type {
  TranscriptCorrection,
  TranscriptRange,
} from "../tool-panel-transcript";

/**
 * Delete, edit, restore and restore-spelling for the chosen units, which
 * are spans of the units as they play. What they cut or restore is worked
 * out among the units as they were heard, one run of the recording at a
 * time, so words a move set apart are never joined into one stretch. Delete
 * takes the silence around what it cuts and keeps one pause at the join
 * (`joinedCutRange`); edit does nothing for a span that holds a pause, since
 * a pause is never corrected, or for one that does not read as it was
 * heard, which no one correction can cover.
 */
export function useTranscriptEditorActions({
  durationMs,
  onCorrect,
  onCut,
  onRestore,
  order,
  pendingAdd,
  setEditing,
}: {
  /** How long the recording is, which stands in for the word after the
   * last. */
  durationMs: number;
  onCorrect: (correction: TranscriptCorrection) => void;
  onCut: (ranges: TranscriptRange[]) => void;
  onRestore: (ranges: TranscriptRange[]) => void;
  order: TranscriptOrder;
  pendingAdd: (correction: TranscriptCorrection) => void;
  setEditing: (span: WordSelection) => void;
}) {
  const { heard, units } = order;
  const removedFlags = useMemo(
    () => heard.map((unit) => unit.removed),
    [heard],
  );
  const correct = (span: WordSelection, text: string) => {
    if (!isHeardInOrder(order, span)) return;
    const { first, last } = heardSpanOf(order, span);
    const correction = {
      endMs: heard[last].endMs,
      startMs: heard[first].startMs,
      text,
    };
    pendingAdd(correction);
    onCorrect(correction);
  };
  const actions = {
    delete: (span: WordSelection) => {
      const spans = heardRuns(order, span).flatMap((run) =>
        wordRunSpans(removedFlags, { ...run, removed: false }),
      );
      if (spans.length > 0) onCut(joinedCutRanges(heard, spans, durationMs));
    },
    edit: (span: WordSelection) => {
      const hasPause = units
        .slice(span.first, span.last + 1)
        .some((unit) => unit.kind === "pause");
      if (!hasPause && isHeardInOrder(order, span)) setEditing(span);
    },
    restore: (span: WordSelection) => {
      const struck = heardRuns(order, span).flatMap((run) =>
        wordRuns(heard, removedFlags, { ...run, removed: true }),
      );
      if (struck.length > 0) onRestore(struck);
    },
    spelling: (span: WordSelection) => {
      correct(
        span,
        units
          .slice(span.first, span.last + 1)
          .map((unit) => unit.heard)
          .join(" "),
      );
    },
  };
  return { actions, correct };
}
