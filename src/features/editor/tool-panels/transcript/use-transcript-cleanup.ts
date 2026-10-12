// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useMemo, useState } from "react";

import { joinedCutRanges } from "./transcript-cut-join";
import { pausesCutBy } from "./transcript-pauses";
import { TranscriptMarked, useTranscriptMarks } from "./use-transcript-marks";

import type { TranscriptRange } from "../tool-panel-transcript";
import type { TranscriptUnit, WordSelection } from "./transcript-units";
import type { TranscriptMenuMark } from "./use-transcript-menu";

type Removal = "fillers" | "profanity" | "silences";

const NOTHING: ReadonlySet<number> = new Set();

/**
 * The transcript's clean-up tools in one place: fillers and profanity
 * marked, what Remove silences, Remove fillers and Remove profanity would
 * cut while one is hovered, and the cuts and "Not profanity" the rows and
 * the menu ask for. Every cut keeps one pause at its join
 * (`joinedCutRange`).
 */
export function useTranscriptCleanup({
  allowedWords,
  durationMs,
  language,
  onCut,
  onNotProfanity,
  plannedSilences,
  units,
}: {
  allowedWords: readonly string[];
  durationMs: number;
  language: string | null;
  onCut: (ranges: TranscriptRange[]) => void;
  onNotProfanity: (text: string) => void;
  plannedSilences: readonly TranscriptRange[];
  units: readonly TranscriptUnit[];
}) {
  const { fillers, marks, profanity } = useTranscriptMarks(
    units,
    language,
    allowedWords,
  );
  const [previewing, setPreviewing] = useState<Removal | null>(null);
  const preview = useMemo(() => {
    if (!previewing) return { isActive: false, units: NOTHING };
    if (previewing === "silences")
      return { isActive: true, units: pausesCutBy(units, plannedSilences) };
    const reached = new Set<number>();
    for (const { span } of previewing === "fillers" ? fillers : profanity)
      for (let index = span.first; index <= span.last; index++)
        reached.add(index);
    return { isActive: true, units: reached };
  }, [fillers, plannedSilences, previewing, profanity, units]);
  const previewer = useCallback(
    (removal: Removal) => (isPreviewing: boolean) => {
      setPreviewing((current) =>
        isPreviewing ? removal : current === removal ? null : current,
      );
    },
    [],
  );
  const previewers = useMemo(
    () => ({
      fillers: previewer("fillers"),
      profanity: previewer("profanity"),
      silences: previewer("silences"),
    }),
    [previewer],
  );

  const cut = (runs: readonly TranscriptMarked[]) => {
    if (runs.length > 0)
      onCut(
        joinedCutRanges(
          units,
          runs.map((run) => run.span),
          durationMs,
        ),
      );
  };
  /** The marked run the unit at `index` belongs to, and its kind. */
  const markedAt = (index: number) => {
    const mark = marks.get(index);
    if (!mark) return null;
    const runs = mark === "filler" ? fillers : profanity;
    const run = runs.find(
      ({ span }) => span.first <= index && index <= span.last,
    );
    return run ? { mark, run, runs } : null;
  };

  return {
    fillers,
    marks,
    menuActions: {
      "not-profanity": (span: WordSelection) => {
        const marked = markedAt(span.first);
        if (marked?.mark === "profanity") onNotProfanity(marked.run.folded);
      },
      "remove-kind": (span: WordSelection) => {
        const marked = markedAt(span.first);
        if (marked) cut(marked.runs);
      },
      "remove-same": (span: WordSelection) => {
        const marked = markedAt(span.first);
        if (marked)
          cut(marked.runs.filter((run) => run.folded === marked.run.folded));
      },
    },
    /** What a right click on `span` adds to the menu, if its first word is
     * marked. */
    menuMark: (span: WordSelection): TranscriptMenuMark | null => {
      const marked = markedAt(span.first);
      return marked
        ? {
            mark: marked.mark,
            text: units
              .slice(marked.run.span.first, marked.run.span.last + 1)
              .map((unit) => unit.text)
              .join(" "),
          }
        : null;
    },
    preview,
    previewers,
    profanity,
    removeFillers: () => {
      cut(fillers);
    },
    removeProfanity: () => {
      cut(profanity);
    },
  };
}
