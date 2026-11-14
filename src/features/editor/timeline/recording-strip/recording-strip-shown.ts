// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type StripBeat } from "../strip/strip-beat-row";
import { SLIVER_S, splitParts } from "../strip/strip-split";
import { type TimelineStripProps } from "../strip/timeline-strip-props";
import { type Span } from "../timeline-scale";

import { type RecordingBeat } from "./recording-beats";
import { gapBeatId, gapTrimReach } from "./recording-strip-gaps";
import { isPlayed, type RecordingStripTime } from "./recording-strip-time";

import type { RecordingTimelineThumbnail } from "../../types";
import type { RecordingTimelineSegment } from "../recording-timeline-types";

// What the strip draws of the recording's readings and its edit: the beats
// found in what it holds, each with its picture, and the gaps among them;
// and every part cut where play jumps or the blade split it.

/** The beats the strip draws: those `found` in the recording, a speech
 * beat saying its line as `said` corrects it, a shortcut's hidden where its
 * keys are not drawn or were deleted, each with the frame `thumbnails` hold
 * at its middle; then each of `gaps`, past the recording in recorded seconds
 * as its segment lies past its end, where the map places it. */
export function shownStripBeats({
  deletedShortcuts,
  found,
  gaps,
  isKeyboardShown,
  isScreenShown,
  recorded,
  said,
  thumbnails,
}: {
  deletedShortcuts: ReadonlySet<string>;
  found: RecordingBeat[];
  gaps: readonly RecordingTimelineSegment[];
  isKeyboardShown: boolean;
  isScreenShown: boolean;
  /** How long the recording was recorded, in seconds. */
  recorded: number;
  said: ReadonlyMap<string, { text: string }>;
  thumbnails: RecordingTimelineThumbnail[];
}): StripBeat[] {
  const recordedBeats = found.map((beat) => {
    const middle = (beat.span.start + beat.span.end) / 2;
    const frame =
      thumbnails.length > 0
        ? thumbnails[
            Math.min(
              thumbnails.length - 1,
              Math.floor(
                (middle / Math.max(recorded, 1e-6)) * thumbnails.length,
              ),
            )
          ]
        : undefined;
    const line = beat.beat.kind === "speech" ? said.get(beat.id) : undefined;
    return {
      ...beat,
      ...(line ? { beat: { kind: "speech", line: line.text } as const } : {}),
      isHidden:
        beat.beat.kind === "keys" &&
        (!isKeyboardShown ||
          deletedShortcuts.has(beat.id.slice("keys-".length))),
      // With the screen out there is no picture to show.
      thumbnail: isScreenShown ? (frame?.url ?? null) : undefined,
    };
  });
  const gapBeats = gaps.map((segment) => ({
    beat: { kind: "gap" } as const,
    id: gapBeatId(segment.id),
    reach: gapTrimReach(segment, recorded * 1000),
    span: {
      end: segment.sourceEnd * recorded,
      start: segment.sourceStart * recorded,
    },
  }));
  return [...recordedBeats, ...gapBeats];
}

/**
 * `props` drawn as `time` cuts them. Captions and shortcuts are read as
 * recorded, so one a cut took out whole would be drawn at the seam as a
 * sliver, or as keys never pressed in what plays (a beat leaves itself out
 * under a px): those are left out. What runs across a jump is drawn as a
 * part each side of it, and a beat or lull the blade split as a part each
 * side of the split; the lanes over the timeline are laid out where they
 * play instead (`strip-lane-time.ts`). Where an item's edge and a jump's
 * are one moment apart only by rounding, a part would be a sliver: those
 * are left out too.
 */
export function shownStripParts(
  props: TimelineStripProps,
  time: RecordingStripTime,
): TimelineStripProps {
  const made = splitParts(props, time.edges, time.splits);
  const captions =
    made.captions?.filter((span) => isPlayed(time.map, span)) ?? [];
  const shown = {
    ...made,
    captions: captions.length > 0 ? captions : null,
    shortcuts: made.shortcuts.filter((span) => isPlayed(time.map, span)),
  };
  if (time.edges.length === 0 && time.splits.length === 0) return shown;
  const isWhole = ({ end, start }: Span) => end - start >= SLIVER_S;
  return {
    ...shown,
    beats: shown.beats.filter(({ span }) => isWhole(span)),
    captions: shown.captions?.filter(isWhole) ?? null,
    cuts: shown.cuts.filter(isWhole),
    lulls: shown.lulls.filter(({ span }) => isWhole(span)),
    shortcuts: shown.shortcuts.filter(isWhole),
  };
}
