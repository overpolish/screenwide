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

/** `stretches`, in source milliseconds, as recorded seconds kept to `part`,
 * run together where they overlap. */
function joinedIn(stretches: [number, number][], part: Span): Span[] {
  const kept = stretches
    .map(([start, end]) => ({
      end: Math.min(end / 1000, part.end),
      start: Math.max(start / 1000, part.start),
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
 * What one part of a pinned annotation's bar shows of its pin, in recorded
 * seconds, which the pin's source milliseconds are a thousandth of: where
 * it is hidden because its content is out of view, as the tracker found or
 * the hand said (the stretches the menu's Content Back in View answers);
 * where the tracker is unsure of its content, having lost it, found it off
 * the frame or under a cover; and each correction, all kept to `part`.
 */
export function timelineAnnotationPin(
  pin: RecordingAnnotationPin,
  status: RecordingPinStatus | undefined,
  part: Span,
): TimelineAnnotationPin {
  return {
    corrections: pinCorrections(pin)
      .map((ms) => ms / 1000)
      .filter((moment) => moment >= part.start && moment <= part.end),
    outOfView: joinedIn(
      [
        ...(status?.covered ?? []),
        // Said out of view, it stays hidden to the next keyframe or, with
        // none, on past every part of the clip.
        ...outOfViewStretches(pin, Number.POSITIVE_INFINITY),
      ],
      part,
    ),
    unsure: joinedIn(
      [
        ...(status?.weak ?? []),
        ...(status?.hidden ?? []),
        ...(status?.under ?? []),
      ],
      part,
    ),
  };
}

/** The strip's annotation bars, each part of a pinned clip's bar with what
 * it shows of the pin. */
export function withAnnotationPins(
  bars: TimelineAnnotation[],
  clips: readonly {
    annotation: { id: string };
    pin?: RecordingAnnotationPin | null;
  }[],
  statuses: ReadonlyMap<string, RecordingPinStatus>,
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
      ? { ...bar, pin: timelineAnnotationPin(pin, statuses.get(id), bar) }
      : bar;
  });
}
