// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { useEditorEditGesture } from "../../use-editor-edit-history";
import { type RecordingTimelineEdit } from "../editing/recording-timeline-edit";
import { keptOf } from "../strip/strip-beat-trim";
import { baseOf, partId } from "../strip/strip-part";
import { chosenItems, type StripSelection } from "../strip/strip-selection";
import { splitAt as splitItems } from "../strip/strip-split";
import { type Span } from "../timeline-scale";

import {
  cutStretch,
  moved,
  putBack,
  restoredCuts,
  splitAt,
  trimmed,
  unsplitAt,
  withListsOf,
} from "./recording-strip-edits";
import {
  lullPlayed,
  type RecordingStripLulls,
  squeezedTo,
} from "./recording-strip-lulls";
import { type RecordingStripTime } from "./recording-strip-time";

import type { LullPlay } from "../../island/lull/lull-island";

// The strip's changes, each made to the timeline edit and handed on as one
// step of the editor's history. A drag tells its change at every step:
// those steps are one gesture, which ends once the drag has rested, so one
// undo takes back the whole drag.

/** How long, in ms, a drag rests before its steps are one undo. */
const REST_MS = 400;

/** Changes made as single steps of history, or as steps of a drag. */
export type RecordingStripSteps = {
  once: (next: RecordingTimelineEdit) => void;
  step: (next: RecordingTimelineEdit | (() => void)) => void;
};

function useEditSteps(
  edit: RecordingTimelineEdit,
  onEdit: (next: RecordingTimelineEdit) => void,
): RecordingStripSteps {
  const gesture = useEditorEditGesture();
  const restRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  // The edit a drag of the timeline began on, whose lists each of its steps
  // carries; none once a step changed the lists themselves.
  const startRef = useRef<RecordingTimelineEdit | null>(null);
  useEffect(
    () => () => {
      if (restRef.current === null) return;
      clearTimeout(restRef.current);
      gesture.endGesture();
    },
    [gesture],
  );
  return {
    /** One change, one step of history. */
    once: (next: RecordingTimelineEdit) => {
      if (next === edit) return;
      gesture.beginGesture();
      onEdit(next);
      gesture.endGesture();
    },
    /** A step of a drag, `next` or what `change` makes. */
    step: (next: RecordingTimelineEdit | (() => void)) => {
      if (restRef.current === null) {
        gesture.beginGesture();
        startRef.current = edit;
      } else clearTimeout(restRef.current);
      restRef.current = setTimeout(() => {
        restRef.current = null;
        gesture.endGesture();
      }, REST_MS);
      if (typeof next === "function") {
        startRef.current = null;
        next();
      } else if (next !== edit) {
        startRef.current ??= edit;
        onEdit(withListsOf(startRef.current, next));
      }
    },
  };
}

/** Where the part `id` of a beat or lull lies, in recorded seconds, as the
 * strip draws it: split at every jump and every split. The first part keeps
 * its item's id, so a whole item is its only part. */
export type PartSpan = (kind: "beat" | "lull", id: string) => Span | undefined;

type Spanned = { id: string; span: Span };

export function partSpans(
  { beats, lulls }: { beats: Spanned[]; lulls: Spanned[] },
  { edges, splits }: Pick<RecordingStripTime, "edges" | "splits">,
): PartSpan {
  const pieces = [...edges, ...splits];
  const parts = (items: Spanned[]) =>
    new Map(
      splitItems(items, pieces, {
        part: (each, span, index) => ({ id: partId(each.id, index), span }),
        spanOf: (each) => each.span,
      }).map(({ id, span }) => [id, span]),
    );
  const beatParts = parts(beats);
  const lullParts = parts(lulls);
  return (kind, id) => (kind === "beat" ? beatParts : lullParts).get(id);
}

/** The stretches `selection` stands for, each beat, lull or part of one
 * chosen, or a drawn range. */
export function chosenSpans(
  selection: StripSelection | null,
  spanOf: PartSpan,
): Span[] {
  if (selection?.kind === "range") return [selection];
  return chosenItems(selection).flatMap(
    ({ id, kind }) => spanOf(kind, id) ?? [],
  );
}

export function useRecordingStripEdits({
  edit,
  lulls,
  onEdit,
  recorded,
  spanOf,
  time,
}: {
  edit: RecordingTimelineEdit;
  lulls: RecordingStripLulls;
  onEdit: (next: RecordingTimelineEdit) => void;
  /** How long the recording was recorded, in seconds. */
  recorded: number;
  spanOf: PartSpan;
  time: RecordingStripTime;
}) {
  const steps = useEditSteps(edit, onEdit);
  return {
    /** Cuts every stretch in `spans`. */
    cut: (spans: Span[]) => {
      steps.once(
        spans.reduce(
          (current, span) => cutStretch(current, span, recorded),
          edit,
        ),
      );
    },
    // A part trims its own stretch, so the parts either side of a split
    // keep theirs.
    onBeatTrim: (id: string, edge: "end" | "start", at: number) => {
      const span = spanOf("beat", id);
      if (!span) return;
      steps.step(
        trimmed(
          edit,
          { at, edge, kept: keptOf(span, time.cuts), span },
          recorded,
        ),
      );
    },
    onLullPlayChange: (id: string, play: LullPlay) => {
      steps.step(lullPlayed(edit, lulls, { id: baseOf(id), play, recorded }));
    },
    // A lull trimmed plays as recorded, as the story's lulls do.
    onLullTrim: (id: string, end: number) => {
      const lull = lulls.lulls.find((each) => each.id === baseOf(id));
      const part = spanOf("lull", id);
      if (!lull || !part) return;
      // The last part can be drawn out again as far as the lull was found.
      const isLast = part.end === lull.span.end;
      const next = trimmed(
        edit,
        {
          at: end,
          edge: "end",
          kept: part,
          span: { end: isLast ? lull.found.end : part.end, start: part.start },
        },
        recorded,
      );
      steps.step(
        lullPlayed(next, lulls, {
          id: lull.id,
          play: { kind: "keep" },
          recorded,
        }),
      );
    },
    onMove: (spans: Span[], before: number | null) => {
      steps.once(moved(edit, { before, spans }, recorded));
    },
    onPutBack: (at: number) => {
      steps.once(putBack(edit, at, recorded));
    },
    onRemoveSplit: (at: number) => {
      steps.once(unsplitAt(edit, at, recorded));
    },
    onRestore: (ids: string[]) => {
      steps.once(restoredCuts(edit, ids));
    },
    onSplit: (at: number) => {
      steps.once(splitAt(edit, at, recorded));
    },
    onSqueezeChange: (squeeze: number) => {
      steps.step(squeezedTo(edit, lulls, { recorded, squeeze }));
    },
    /** Makes a change to the edit that the lanes worked out, as one step,
     * or as a step of a drag. */
    steps,
  };
}
