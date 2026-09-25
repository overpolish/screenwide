// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { ANNOTATION_DRAW_IN_MS } from "./annotation-kinds";
import { Annotation } from "./annotations";
import { recordingAnnotationRows } from "./components/recording-annotation-layout";
import { moveRecordingAnnotationClip } from "./recording-annotation-geometry";
import {
  mergeRecordingAnnotationClips,
  recordingAnnotationClipAt,
  RecordingAnnotationClip,
  renumberedAnnotationClips,
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
    ).toMatchObject({ endMs: 10_000, startMs: 4_000 - ANNOTATION_DRAW_IN_MS });
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

const counterClip = (
  id: string,
  value: number,
  startMs: number,
): RecordingAnnotationClip => ({
  annotation: {
    aboveCamera: false,
    animated: true,
    id,
    shape: { angle: 0, center: { x: 10, y: 10 }, kind: "counter", value },
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
  endMs: startMs + 3_000,
  startMs,
  trackId: "primary",
});

const values = (clips: RecordingAnnotationClip[]) =>
  clips
    .map((clip) => clip.annotation.shape)
    .filter((shape) => shape.kind === "counter")
    .map((shape) => shape.value);

describe("renumberedAnnotationClips", () => {
  it("numbers counters by where their clips sit on the timeline", () => {
    const dragged = [counterClip("a", 1, 4_000), counterClip("b", 2, 1_000)];
    expect(values(renumberedAnnotationClips(dragged))).toEqual([2, 1]);
  });

  it("gives the lower number to the annotation drawn first on a tie", () => {
    const together = [counterClip("a", 1, 2_000), counterClip("b", 2, 2_000)];
    expect(renumberedAnnotationClips(together)).toBe(together);
  });

  it("counts only counters, and leaves arrows where they are", () => {
    const mixed = [
      counterClip("c", 9, 5_000),
      { annotation, endMs: 3_000, startMs: 0, trackId: "primary" as const },
      counterClip("a", 9, 1_000),
    ];
    const numbered = renumberedAnnotationClips(mixed);
    expect(values(numbered)).toEqual([2, 1]);
    expect(numbered[1]).toBe(mixed[1]);
  });

  it("hands back the very same list when nothing moved", () => {
    const settled = [counterClip("a", 1, 1_000), counterClip("b", 2, 4_000)];
    expect(renumberedAnnotationClips(settled)).toBe(settled);
  });
});
