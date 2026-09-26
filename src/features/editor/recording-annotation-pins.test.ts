// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { ANNOTATION_DRAW_IN_MS } from "./annotation-kinds";
import { Annotation } from "./annotations";
import { moveRecordingAnnotationClip } from "./recording-annotation-geometry";
import {
  clearedPinCorrections,
  pinnedClip,
  withoutPinKeyframe,
} from "./recording-annotation-pins";
import {
  mergeRecordingAnnotationClips,
  RecordingAnnotationClip,
} from "./recording-annotations";
import { createRecordingTimelineEdit } from "./recording-timeline-edit";

const annotation: Annotation = {
  aboveCamera: false,
  animated: true,
  id: "arrow-1",
  shape: {
    control: { x: 5, y: 5 },
    end: { x: 10, y: 10 },
    kind: "arrow",
    start: { x: 0, y: 0 },
  },
  style: {
    align: "left",
    color: "#ff0000",
    head: "end",
    radius: 0,
    redaction: "erase",
    strength: 0,
    width: 8,
  },
};

describe("pinned recording annotations", () => {
  const clip: RecordingAnnotationClip = {
    annotation,
    endMs: 7_000,
    startMs: 3_000,
    trackId: "primary",
  };

  it("pins at the playhead inside the clip, and where it was placed outside it", () => {
    expect(pinnedClip(clip, 5_000).pin).toEqual({
      keyframes: [{ dx: 0, dy: 0, ms: 5_000 }],
      pinnedMs: 5_000,
    });
    expect(pinnedClip(clip, 9_000).pin?.pinnedMs).toBe(
      3_000 + ANNOTATION_DRAW_IN_MS,
    );
  });

  it("takes the keyframe a native drag left without moving what was drawn", () => {
    const pinned = pinnedClip(clip, 5_000);
    const pin = {
      keyframes: [
        ...(pinned.pin?.keyframes ?? []),
        { dx: 12, dy: -4, ms: 6_000 },
      ],
      pinnedMs: 5_000,
    };
    const merged = mergeRecordingAnnotationClips({
      annotations: [annotation],
      clips: [pinned],
      edit: createRecordingTimelineEdit(1),
      pins: [{ annotationId: annotation.id, pin }],
      positionMs: 6_000,
      sourceDurationMs: 20_000,
      trackId: "primary",
    });
    expect(merged).toEqual([{ ...pinned, pin }]);
  });

  it("moves its keyframes with a moved clip", () => {
    const moved = moveRecordingAnnotationClip({
      clip: pinnedClip(clip, 5_000),
      deltaOutput: 0.05,
      edit: createRecordingTimelineEdit(1),
      sourceDurationMs: 20_000,
    });
    expect(moved.pin).toEqual({
      keyframes: [{ dx: 0, dy: 0, ms: 6_000 }],
      pinnedMs: 6_000,
    });
  });

  it("clears corrections down to the frame it was pinned on", () => {
    const pin = {
      keyframes: [
        { dx: 0, dy: 0, ms: 5_000 },
        { dx: 3, dy: 1, ms: 6_000 },
      ],
      pinnedMs: 5_000,
    };
    expect(clearedPinCorrections(pin).keyframes).toEqual([
      { dx: 0, dy: 0, ms: 5_000 },
    ]);
    // Removing the pinned frame hands its role to the keyframe left.
    expect(withoutPinKeyframe(pin, 5_000)).toEqual({
      keyframes: [{ dx: 3, dy: 1, ms: 6_000 }],
      pinnedMs: 6_000,
    });
    expect(withoutPinKeyframe(withoutPinKeyframe(pin, 5_000), 6_000)).toEqual({
      keyframes: [{ dx: 3, dy: 1, ms: 6_000 }],
      pinnedMs: 6_000,
    });
  });
});
