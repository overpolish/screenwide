// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { ANNOTATION_DRAW_IN_MS } from "../../annotations/annotation-kinds";
import { Annotation } from "../../annotations/annotations";
import { createRecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";

import { moveRecordingAnnotationClip } from "./recording-annotation-geometry";
import { recordingAnnotationRows } from "./recording-annotation-layout";
import {
  mergeRecordingAnnotationClips,
  recordingAnnotationClipAt,
} from "./recording-annotations";

const annotation: Annotation = {
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
    shadow: false,
    softness: 0,
    strength: 0,
    tint: false,
    width: 8,
  },
};

describe("recording annotation clips", () => {
  it("starts a draw-in before the playhead and ends three seconds after it", () => {
    expect(
      recordingAnnotationClipAt({
        annotation,
        sourceDurationMs: 20_000,
        sourcePositionMs: 4_000,
      }),
    ).toEqual({
      annotation,
      endMs: 7_000,
      startMs: 4_000 - ANNOTATION_DRAW_IN_MS,
      trackId: "primary",
    });
  });

  it("has finished drawing the annotation in by the playhead it was placed at", () => {
    const clip = recordingAnnotationClipAt({
      annotation,
      sourceDurationMs: 20_000,
      sourcePositionMs: 4_000,
    });
    // The reveal caps its phase at a third of the clip and never lengthens
    // it, so an annotation placed a whole phase back is whole at the playhead.
    const phaseMs = Math.min(
      ANNOTATION_DRAW_IN_MS,
      (clip.endMs - clip.startMs) / 3,
    );
    expect(4_000 - clip.startMs).toBeGreaterThanOrEqual(phaseMs);
  });

  it("maps three output seconds and the arrival through a two times source segment", () => {
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
    ).toMatchObject({
      endMs: 10_000,
      startMs: 4_000 - 2 * ANNOTATION_DRAW_IN_MS,
    });
  });

  it("takes what room there is at the start of the recording", () => {
    expect(
      recordingAnnotationClipAt({
        annotation,
        sourceDurationMs: 20_000,
        sourcePositionMs: 0,
      }),
    ).toMatchObject({ endMs: 3_000, startMs: 0 });
    expect(
      recordingAnnotationClipAt({
        annotation,
        sourceDurationMs: 20_000,
        sourcePositionMs: 200,
      }),
    ).toMatchObject({ endMs: 3_200, startMs: 0 });
  });

  it("clamps an annotation started near the end to the source duration", () => {
    expect(
      recordingAnnotationClipAt({
        annotation,
        sourceDurationMs: 20_000,
        sourcePositionMs: 19_500,
      }),
    ).toMatchObject({ endMs: 20_000, startMs: 19_500 - ANNOTATION_DRAW_IN_MS });
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
  ).toMatchObject({ endMs: 20000, startMs: 19_999 - ANNOTATION_DRAW_IN_MS });
});

it("puts the clip drawn in front on the row above one it overlaps, across a removed source range", () => {
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
  expect(rows.fragments.map((fragment) => fragment.row)).toEqual([1, 0]);
  expect(
    recordingAnnotationRows([second, first], edit, 120_000).fragments.map(
      (fragment) => fragment.row,
    ),
  ).toEqual([1, 0]);
  expect(rows.fragments[1]).toMatchObject({
    continuedByNext: false,
    continuesPrevious: false,
  });
});

it("stacks every camera clip over the screen clips it overlaps, whatever the list order", () => {
  const edit = createRecordingTimelineEdit(1);
  const camera = {
    annotation: { ...annotation, id: "camera" },
    endMs: 30_000,
    startMs: 0,
    trackId: "camera" as const,
  };
  const screen = [
    { ...camera, annotation, trackId: "primary" as const },
    {
      ...camera,
      annotation: { ...annotation, id: "screen" },
      trackId: "primary" as const,
    },
  ];
  const rows = recordingAnnotationRows(
    [camera, ...screen],
    edit,
    120_000,
  ).fragments.map((fragment) => [fragment.item.annotation.id, fragment.row]);
  expect(rows).toEqual([
    [annotation.id, 2],
    ["screen", 1],
    ["camera", 0],
  ]);
});

it("moves a clip by output time while preserving duration through a speed change", () => {
  const edit = {
    ...createRecordingTimelineEdit(1),
    segments: [{ id: 0, playbackRate: 2, sourceEnd: 1, sourceStart: 0 }],
  };
  const clip = {
    annotation,
    endMs: 10_000,
    startMs: 4_000,
    trackId: "primary" as const,
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
