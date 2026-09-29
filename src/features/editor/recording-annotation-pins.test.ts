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
  withPinKeyframe,
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
    blur: false,
    color: "#ff0000",
    handDrawn: false,
    head: "end",
    manual: false,
    radius: 0,
    redaction: "erase",
    softness: 0,
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

  it("puts a keyframe in time order, over any at the same moment", () => {
    const pin = {
      keyframes: [
        { dx: 0, dy: 0, ms: 1_000 },
        { dx: 0, dy: -30, ms: 5_000 },
      ],
      pinnedMs: 1_000,
    };
    expect(withPinKeyframe(pin, { dx: 2, dy: -8, ms: 3_000 })).toEqual({
      keyframes: [
        { dx: 0, dy: 0, ms: 1_000 },
        { dx: 2, dy: -8, ms: 3_000 },
        { dx: 0, dy: -30, ms: 5_000 },
      ],
      pinnedMs: 1_000,
    });
    // Within a frame's slack of one already there, it takes its place.
    expect(
      withPinKeyframe(pin, { dx: 1, dy: -31, ms: 5_006 }).keyframes,
    ).toEqual([
      { dx: 0, dy: 0, ms: 1_000 },
      { dx: 1, dy: -31, ms: 5_006 },
    ]);
    // A resize made there keeps its size.
    const edges: [number, number, number, number] = [0, 0, 0, -6];
    const resized = {
      ...pin,
      keyframes: [pin.keyframes[0], { dx: 0, dy: -30, edges, ms: 5_000 }],
    };
    expect(
      withPinKeyframe(resized, { dx: 1, dy: -31, ms: 5_006 }).keyframes[1],
    ).toEqual({
      dx: 1,
      dy: -31,
      edges,
      ms: 5_006,
    });
  });
});
