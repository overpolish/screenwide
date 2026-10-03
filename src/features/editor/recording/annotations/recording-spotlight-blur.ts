// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  sharedSpotlightBlurs,
  withSharedBlur,
} from "../../annotations/spotlight-blur";

import { RecordingAnnotationClip } from "./recording-annotations";

/**
 * The clips with every run of spotlights shown together, on the screen and
 * the camera alike, carrying one Blur setting. A run is joined through any
 * overlap, so a spotlight meeting one that meets a third shares the setting
 * with both, and none is left disagreeing with one shown beside it. A clip
 * moved in time joins its new run as a newcomer would. Clips that need no
 * change are handed back as they were.
 */
export const withRecordingSpotlightBlurShared = (
  next: RecordingAnnotationClip[],
  previous: readonly RecordingAnnotationClip[],
): RecordingAnnotationClip[] => {
  const spotlights = next
    .filter((clip) => clip.annotation.shape.kind === "spotlight")
    .sort((a, b) => a.startMs - b.startMs);
  const groups: RecordingAnnotationClip[][] = [];
  let group: RecordingAnnotationClip[] = [];
  let reach = -Infinity;
  for (const clip of spotlights) {
    if (clip.startMs >= reach) {
      group = [];
      groups.push(group);
    }
    group.push(clip);
    reach = Math.max(reach, clip.endMs);
  }
  const before = new Map(
    previous.map((clip) => [clip.annotation.id, clip] as const),
  );
  const settled = new Map(
    spotlights.flatMap((clip) => {
      const was = before.get(clip.annotation.id);
      return was && was.startMs === clip.startMs && was.endMs === clip.endMs
        ? [[clip.annotation.id, was.annotation] as const]
        : [];
    }),
  );
  const changes = sharedSpotlightBlurs(
    groups.map((group) => group.map((clip) => clip.annotation)),
    settled,
  );
  if (changes.size === 0) return next;
  return next.map((clip) =>
    changes.has(clip.annotation.id)
      ? { ...clip, annotation: withSharedBlur(clip.annotation, changes) }
      : clip,
  );
};
