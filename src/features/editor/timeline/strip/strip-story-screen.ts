// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type TimelineBeat } from "../beat-clip/beat-label";
import { type TimelineShortcut } from "../shortcut-lane/shortcut-chip";

import { type StripBeat } from "./strip-beat-row";
import { type LiveLull } from "./strip-story-content";

// A story recording with its screen's track out of the export, as the
// editor makes its strip then: the beats read off the screen (clicks,
// typing, shortcuts, app switches, new pictures) go, with every picture and
// the shortcuts; the voice and the moments stay; the lulls are the gaps
// between them. A lull keeps the id and play of the first one it takes in,
// so it grows from that one.

const FROM_SCREEN: Partial<Record<TimelineBeat["kind"], true>> = {
  click: true,
  keys: true,
  screen: true,
  switch: true,
  type: true,
};

/** The shortest gap between beats, in seconds, that is a lull. */
const LEAST_LULL_S = 0.3;

export function screenlessContent<
  T extends {
    beats: StripBeat[];
    lulls: LiveLull[];
    shortcuts: TimelineShortcut[];
  },
>(content: T, duration: number): T {
  const beats = content.beats
    .filter(({ beat }) => !FROM_SCREEN[beat.kind])
    .map(({ thumbnail: _, ...beat }) => beat);
  const lulls: LiveLull[] = [];
  const addLull = (start: number, end: number) => {
    if (end - start < LEAST_LULL_S) return;
    const span = { end, start };
    const first = content.lulls.find(
      (lull) => lull.span.start >= start - 1e-6 && lull.span.start < end,
    );
    lulls.push({
      id: first?.id ?? `lull-at-${start.toFixed(3)}`,
      play: first?.play ?? { kind: "squeeze" },
      recordedEnd: end,
      span,
    });
  };
  let at = 0;
  for (const { span } of [...beats].sort(
    (one, other) => one.span.start - other.span.start,
  )) {
    addLull(at, span.start);
    at = Math.max(at, span.end);
  }
  addLull(at, duration);
  return { ...content, beats, lulls, shortcuts: [] };
}
