// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { RecordingAnnotationClip } from "../recording-annotations";
import {
  cutRecordingTimeline,
  createRecordingTimelineEdit,
} from "../recording-timeline-edit";

import {
  nearestTimelineSnapTarget,
  timelineSnapRangeShift,
  timelineSnapTargets,
  TimelineSnapGesture,
} from "./timeline-snap";

const gesture = (targets: number[], snapping = true): TimelineSnapGesture => ({
  isSnapping: () => snapping,
  showGuide: () => undefined,
  targets,
  threshold: 0.05,
});

const clip = (
  id: string,
  startMs: number,
  endMs: number,
): RecordingAnnotationClip => ({
  annotation: {
    aboveCamera: false,
    animated: true,
    id,
    shape: { angle: 0, center: { x: 10, y: 10 }, kind: "counter", value: 1 },
    style: {
      align: "left",
      color: "#ffcc00",
      head: "none",
      radius: 0,
      redaction: "erase",
      strength: 0,
      width: 56,
    },
  },
  endMs,
  startMs,
  trackId: "primary",
});

describe("nearestTimelineSnapTarget", () => {
  it("takes the closest target within reach and none beyond it", () => {
    expect(nearestTimelineSnapTarget(gesture([0.2, 0.3]), 0.27)).toBe(0.3);
    expect(nearestTimelineSnapTarget(gesture([0.2, 0.3]), 0.4)).toBeNull();
  });

  it("holds nothing while snapping is inverted off", () => {
    expect(nearestTimelineSnapTarget(gesture([0.2], false), 0.2)).toBeNull();
  });
});

describe("timelineSnapRangeShift", () => {
  it("shifts by whichever end is nearer a target", () => {
    const shifted = timelineSnapRangeShift(gesture([0.1, 0.52]), 0.13, 0.5);
    expect(shifted?.target).toBe(0.52);
    expect(shifted?.shift).toBeCloseTo(0.02);
  });

  it("leaves a range alone when neither end is within reach", () => {
    expect(timelineSnapRangeShift(gesture([0.8]), 0.1, 0.3)).toBeNull();
  });
});

describe("timelineSnapTargets", () => {
  const edit = cutRecordingTimeline(createRecordingTimelineEdit(1), 0.5);
  const base = {
    annotationClips: [clip("a", 1_000, 2_000), clip("b", 6_000, 7_000)],
    edit,
    hiddenKeyboardFragmentIds: new Set<string>(),
    hiddenKeyboardItemIds: new Set<number>(),
    keyboardItems: [{ endMs: 4_000, id: 7, label: "⌘C", startMs: 3_000 }],
    playheadOutput: 0.25,
    sourceDurationMs: 10_000,
  };

  it("gathers segment edges, other annotations, badges and the playhead", () => {
    const targets = timelineSnapTargets({ ...base, excludeAnnotationId: "a" });
    expect(targets).toEqual(
      expect.arrayContaining([0, 0.5, 1, 0.6, 0.7, 0.3, 0.4, 0.25]),
    );
    expect(targets).not.toContain(0.1);
    expect(targets).not.toContain(0.2);
  });

  it("leaves out badges the lane hides", () => {
    expect(
      timelineSnapTargets({ ...base, hiddenKeyboardItemIds: new Set([7]) }),
    ).not.toContain(0.3);
    expect(
      timelineSnapTargets({
        ...base,
        hiddenKeyboardFragmentIds: new Set(["7:0"]),
      }),
    ).not.toContain(0.4);
  });
});
