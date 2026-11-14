// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  outOfViewStretches,
  pinCorrections,
} from "../../recording/annotations/recording-annotation-pins";
import { type TimelineAnnotationPin } from "../annotation-lane/annotation-bar-pin";
import { type TimelineAnnotation } from "../annotation-lane/annotation-lane";
import { baseOf } from "../strip/strip-part";
import { type Span } from "../timeline-scale";

import type { RecordingAnnotationPin } from "../../recording/annotations/recording-annotations";
import type { RecordingPinStatus } from "../../recording/annotations/use-recording-pin-status";

/** From recorded seconds to where they play, `isEnd` placing a moment as
 * the end of what stops there (`TimeMap.toPlayed`). */
type ToPlayed = (recorded: number, isEnd?: boolean) => number;

/** `stretches`, in source milliseconds, where they play kept to `part`,
 * run together where they overlap. One on past every part (an infinite
 * end) stays so. */
function joinedIn(
  stretches: [number, number][],
  part: Span,
  toPlayed: ToPlayed,
): Span[] {
  const played = (ms: number, isEnd?: boolean) =>
    Number.isFinite(ms) ? toPlayed(ms / 1000, isEnd) : ms;
  const kept = stretches
    .map(([start, end]) => ({
      end: Math.min(played(end, true), part.end),
      start: Math.max(played(start), part.start),
    }))
    .filter(({ end, start }) => end > start)
    .sort((one, other) => one.start - other.start);
  const joined: Span[] = [];
  for (const stretch of kept) {
    const last = joined[joined.length - 1] as Span | undefined;
    if (last && stretch.start <= last.end)
      last.end = Math.max(last.end, stretch.end);
    else joined.push({ ...stretch });
  }
  return joined;
}

/**
 * What one bar of a pinned annotation shows of its pin, where it plays,
 * from the pin's source milliseconds (`toPlayed` lays recorded seconds out
 * as the bar is): where it is hidden because its content is out of view,
 * as the tracker found or the hand said (the stretches the menu's Content
 * Back in View answers); where the tracker is unsure of its content,
 * having lost it, found it off the frame or under a cover; and each
 * correction, all kept to `part`.
 */
export function timelineAnnotationPin(
  pin: RecordingAnnotationPin,
  status: RecordingPinStatus | undefined,
  { part, toPlayed }: { part: Span; toPlayed: ToPlayed },
): TimelineAnnotationPin {
  return {
    corrections: pinCorrections(pin)
      .map((ms) => toPlayed(ms / 1000))
      .filter((moment) => moment >= part.start && moment <= part.end),
    outOfView: joinedIn(
      [
        ...(status?.covered ?? []),
        // Said out of view, it stays hidden to the next keyframe or, with
        // none, on past every part of the clip.
        ...outOfViewStretches(pin, Number.POSITIVE_INFINITY),
      ],
      part,
      toPlayed,
    ),
    unsure: joinedIn(
      [
        ...(status?.weak ?? []),
        ...(status?.hidden ?? []),
        ...(status?.under ?? []),
      ],
      part,
      toPlayed,
    ),
  };
}

/** The strip's annotation bars, each of a pinned clip with what it shows
 * of the pin, laid out where it plays by `toPlayed`. */
export function withAnnotationPins(
  bars: TimelineAnnotation[],
  {
    clips,
    statuses,
  }: {
    clips: readonly {
      annotation: { id: string };
      pin?: RecordingAnnotationPin | null;
    }[];
    statuses: ReadonlyMap<string, RecordingPinStatus>;
  },
  toPlayed: ToPlayed,
): TimelineAnnotation[] {
  const pins = new Map(
    clips.flatMap((clip) =>
      clip.pin ? [[clip.annotation.id, clip.pin] as const] : [],
    ),
  );
  return bars.map((bar) => {
    const id = baseOf(bar.id);
    const pin = pins.get(id);
    return pin
      ? {
          ...bar,
          pin: timelineAnnotationPin(pin, statuses.get(id), {
            part: bar,
            toPlayed,
          }),
        }
      : bar;
  });
}
