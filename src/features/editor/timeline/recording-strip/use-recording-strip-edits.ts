// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { useEditorEditGesture } from "../../use-editor-edit-history";
import { type RecordingTimelineEdit } from "../editing/recording-timeline-edit";
import { keptOf } from "../strip/strip-beat-trim";
import { baseOf } from "../strip/strip-part";
import { type StripSelection } from "../strip/strip-selection";
import { type Span } from "../timeline-scale";

import { type RecordingBeat } from "./recording-beats";
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

/** The stretches `selection` stands for, each beat, lull or drawn range. */
export function chosenSpans(
  selection: StripSelection | null,
  { beats, lulls }: { beats: RecordingBeat[]; lulls: RecordingStripLulls },
): Span[] {
  if (!selection) return [];
  if (selection.kind === "range") return [selection];
  const items =
    selection.kind === "items"
      ? selection.items
      : selection.kind === "beat" || selection.kind === "lull"
        ? [selection]
        : [];
  return items.flatMap(({ id, kind }) =>
    kind === "beat"
      ? (beats.find((beat) => beat.id === baseOf(id))?.span ?? [])
      : (lulls.lulls.find((lull) => lull.id === baseOf(id))?.found ?? []),
  );
}

export function useRecordingStripEdits({
  beats,
  edit,
  lulls,
  onEdit,
  recorded,
  time,
}: {
  beats: RecordingBeat[];
  edit: RecordingTimelineEdit;
  lulls: RecordingStripLulls;
  onEdit: (next: RecordingTimelineEdit) => void;
  /** How long the recording was recorded, in seconds. */
  recorded: number;
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
    onBeatTrim: (id: string, edge: "end" | "start", at: number) => {
      const beat = beats.find((each) => each.id === baseOf(id));
      if (!beat) return;
      steps.step(
        trimmed(
          edit,
          { at, edge, kept: keptOf(beat.span, time.cuts), span: beat.span },
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
      if (!lull) return;
      const next = trimmed(
        edit,
        { at: end, edge: "end", kept: lull.span, span: lull.found },
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
