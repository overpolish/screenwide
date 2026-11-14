// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type RecordingTimelineEdit } from "../editing/recording-timeline-edit";
import { type Playhead } from "../scrub-playhead";
import { baseOf } from "../strip/strip-part";
import { type StripSelection } from "../strip/strip-selection";
import { type TimeMap } from "../timeline-scale";

import {
  DEFAULT_GAP_MS,
  deleteRecordingTimelineGap,
  gapSegmentIdOf,
  insertRecordingTimelineGap,
} from "./recording-strip-gaps";
import { setRecordingTimelineGapBackground } from "./recording-timeline-gaps";
import { type RecordingStripSteps } from "./use-recording-strip-edits";

import type { Background } from "../../../../bindings/Background";

// The strip's gaps: one inserted at the playhead, and the one chosen given
// a background of its own or taken away, each one step of history; its
// ends are drawn on the strip, as a beat's are (`trimRecordingTimelineGap`).

export function recordingStripGapActions({
  durationMs,
  edit,
  map,
  playhead,
  select,
  selection,
  steps,
}: {
  /** How long the recording was recorded, in ms. */
  durationMs: number;
  edit: RecordingTimelineEdit;
  /** From recorded to played seconds. */
  map: TimeMap;
  playhead: Playhead;
  select: (selection: StripSelection | null) => void;
  selection: StripSelection | null;
  steps: RecordingStripSteps;
}) {
  const chosenId =
    selection?.kind === "beat" ? gapSegmentIdOf(baseOf(selection.id)) : null;
  return {
    /** The gap segment chosen on the strip, or null. */
    chosenId,
    /** Gives the chosen gap `background`, or the recording's again for
     * null. */
    dress: (background: Background | null) => {
      if (chosenId === null) return;
      steps.once(
        setRecordingTimelineGapBackground(edit, { background, id: chosenId }),
      );
    },
    /** Inserts a gap at the playhead. */
    insert: () => {
      steps.once(
        insertRecordingTimelineGap(edit, {
          atMs: playhead.ratio() * map.duration * 1000,
          durationMs,
          lengthMs: DEFAULT_GAP_MS,
        }),
      );
    },
    /** Takes the chosen gap away, closing up its time. */
    remove: () => {
      if (chosenId === null) return;
      steps.once(
        deleteRecordingTimelineGap(edit, { durationMs, id: chosenId }),
      );
      select(null);
    },
  };
}
