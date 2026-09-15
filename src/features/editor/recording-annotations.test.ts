// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { Annotation } from "./annotations";
import { recordingAnnotationRows } from "./components/recording-annotation-layout";
import { moveRecordingAnnotationClip } from "./recording-annotation-geometry";
import {
  mergeRecordingAnnotationClips,
  recordingAnnotationClipAt,
} from "./recording-annotations";
import { createRecordingTimelineEdit } from "./recording-timeline-edit";

const annotation: Annotation = {
  aboveCamera: false,
  id: "arrow-1",
  shape: {
    control: { x: 5, y: 5 },
    end: { x: 10, y: 10 },
    kind: "arrow",
    start: { x: 0, y: 0 },
  },
  style: { color: "#ff0000", head: "end", width: 8 },
};

describe("recording annotation clips", () => {
  it("uses source milliseconds and gives a new mark three seconds", () => {
    expect(
      recordingAnnotationClipAt({
        annotation,
        sourceDurationMs: 20_000,
        sourcePositionMs: 4_000,
      }),
    ).toEqual({
      annotation,
      endMs: 7_000,
      startMs: 4_000,
      trackId: "primary",
    });
  });

  it("maps three output seconds through a two times source segment", () => {
    const edit = {
      ...createRecordingTimelineEdit(1),
      segments: [{ id: 0, playbackRate: 2, sourceEnd: 1, sourceStart: 0 }],
    };
    expect(
      recordingAnnotationClipAt({
        annotation,
        edit,
        sourceDurationMs: 20_000,
        sourcePositionMs: 4_000,
      }),
    ).toMatchObject({ endMs: 10_000, startMs: 4_000 });
  });

  it("clamps a mark started near the end to the source duration", () => {
    expect(
      recordingAnnotationClipAt({
        annotation,
        sourceDurationMs: 20_000,
        sourcePositionMs: 19_500,
      }),
    ).toMatchObject({ endMs: 20_000, startMs: 19_500 });
  });
});

it("merges a visible edit without dropping hidden or other-track arrows", () => {
  const edit = createRecordingTimelineEdit(1);
  const clips = [
    recordingAnnotationClipAt({
      annotation,
      sourceDurationMs: 20_000,
      sourcePositionMs: 4000,
    }),
    recordingAnnotationClipAt({
      annotation: { ...annotation, id: "later" },
      sourceDurationMs: 20_000,
      sourcePositionMs: 12000,
    }),
    recordingAnnotationClipAt({
      annotation: { ...annotation, id: "camera" },
      sourceDurationMs: 20_000,
      sourcePositionMs: 4000,
      trackId: "camera",
    }),
  ];
  const changed = { ...annotation, style: { ...annotation.style, width: 20 } };
  const merged = mergeRecordingAnnotationClips({
    annotations: [changed],
    clips,
    edit,
    positionMs: 4500,
    sourceDurationMs: 20_000,
    trackId: "primary",
  });
  expect(merged).toEqual([
    { ...clips[0], annotation: changed },
    clips[1],
    clips[2],
  ]);
  expect(
    mergeRecordingAnnotationClips({
      annotations: [changed],
      clips: merged,
      edit,
      positionMs: 4500,
      sourceDurationMs: 20_000,
      trackId: "primary",
    }),
  ).toEqual(merged);
});

it("keeps a short clip valid at the final source position", () => {
  expect(
    recordingAnnotationClipAt({
      annotation,
      sourceDurationMs: 20_000,
      sourcePositionMs: 20_000,
    }),
  ).toMatchObject({ endMs: 20000, startMs: 19999 });
});

it("keeps overlapping arrows on one row each across a removed source range", () => {
  const edit = {
    ...createRecordingTimelineEdit(1),
    segments: [
      { id: 0, sourceEnd: 0.2, sourceStart: 0 },
      { id: 1, playbackRate: 2, sourceEnd: 1, sourceStart: 0.3 },
    ],
  };
  const first = {
    annotation,
    endMs: 31000,
    startMs: 8000,
    trackId: "primary" as const,
  };
  const second = {
    ...first,
    annotation: { ...annotation, id: "second" },
    endMs: 48000,
    startMs: 22000,
  };
  const rows = recordingAnnotationRows([first, second], edit, 120_000);
  expect(rows.rowCount).toBe(2);
  expect(rows.fragments).toHaveLength(2);
  expect(rows.fragments[1]).toMatchObject({
    continuedByNext: false,
    continuesPrevious: false,
    row: 1,
  });
});

it("moves a clip by output time while preserving duration through a speed change", () => {
  const edit = {
    ...createRecordingTimelineEdit(1),
    segments: [{ id: 0, playbackRate: 2, sourceEnd: 1, sourceStart: 0 }],
  };
  const clip = {
    ...recordingAnnotationClipAt({
      annotation,
      sourceDurationMs: 20_000,
      sourcePositionMs: 4_000,
    }),
    endMs: 10_000,
  };
  const moved = moveRecordingAnnotationClip({
    clip,
    deltaOutput: 0.1,
    edit,
    sourceDurationMs: 20_000,
  });
  expect(moved.endMs - moved.startMs).toBe(6_000);
  expect(moved.startMs).toBe(6_000);
});
