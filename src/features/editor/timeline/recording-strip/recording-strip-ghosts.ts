// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type TimelineAnnotation } from "../annotation-lane/annotation-lane";
import { type TimelineAudioClip } from "../audio-lane/audio-stack";

import type { RecordingAnnotationClip } from "../../../../bindings/RecordingAnnotationClip";

// What the format in hand leaves out stays on its strip, drawn faintly
// where it plays in the other formats, so formats that do not match are
// seen at a glance. Ghosts stack after the format's own bars, which keep
// their lanes; packed, a ghost rises into any room left above.

/** The lane after every one of `bars`. */
const laneAfter = (bars: readonly { lane: number }[]) =>
  Math.max(-1, ...bars.map(({ lane }) => lane)) + 1;

/** The annotations the format in hand leaves out, as ghost bars under
 * `own`. */
export const withAnnotationGhosts = (
  own: TimelineAnnotation[],
  ghosts: readonly RecordingAnnotationClip[],
): TimelineAnnotation[] => {
  if (ghosts.length === 0) return own;
  const first = laneAfter(own);
  return [
    ...own,
    ...ghosts.map((clip, index) => ({
      end: clip.endMs / 1000,
      id: clip.annotation.id,
      isGhost: true,
      kind: clip.annotation.shape.kind,
      lane: first + index,
      start: clip.startMs / 1000,
    })),
  ];
};

/** The sound effects and songs the format in hand leaves out, as ghosts
 * under `own`; the songs' rail draws them behind its own. */
export const withAudioGhosts = (
  own: TimelineAudioClip[],
  ghosts: TimelineAudioClip[],
): TimelineAudioClip[] => {
  if (ghosts.length === 0) return own;
  const first = laneAfter(own);
  return [
    ...own,
    ...ghosts.map((clip, index) => ({
      ...clip,
      isGhost: true,
      lane: first + index,
    })),
  ];
};
