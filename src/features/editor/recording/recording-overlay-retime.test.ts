// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  deleteRecordingTimelineRange,
  RecordingTimelineEdit,
  trimRecordingTimelineSegment,
} from "../timeline/editing/recording-timeline-edit";
import { moveRecordingTimelineSegment } from "../timeline/editing/recording-timeline-move";

import { recordingAnnotationRows } from "./annotations/recording-annotation-layout";
import { RecordingAnnotationClip } from "./annotations/recording-annotations";
import {
  RecordingOverlayTimelines,
  retimeRecordingOverlays,
} from "./recording-overlay-retime";
import { RecordingSceneClip } from "./scenes/recording-scenes";

const DURATION_MS = 10_000;

/** 0-4 s, 4-7 s and 7-10 s of a 10 s recording, uncut. */
const edit = (
  overrides: Partial<RecordingTimelineEdit> = {},
): RecordingTimelineEdit => ({
  artifactId: 1,
  nextSegmentId: 3,
  segments: [
    { id: 0, sourceEnd: 0.4, sourceStart: 0 },
    { id: 1, sourceEnd: 0.7, sourceStart: 0.4 },
    { id: 2, sourceEnd: 1, sourceStart: 0.7 },
  ],
  ...overrides,
});

const annotation = (startMs: number, endMs: number) =>
  ({
    annotation: { id: "arrow", shape: { kind: "arrow" } },
    endMs,
    startMs,
    trackId: "primary",
  }) as unknown as RecordingAnnotationClip;

const scene = (startMs: number, endMs: number): RecordingSceneClip => ({
  endMs,
  id: "scene",
  preset: "camera-only",
  startMs,
});

const spans = (clips: { endMs: number; startMs: number }[] = []) =>
  clips.map(({ endMs, startMs }) => [startMs, endMs]);

/** `next` committed after `previous`, as the editor commits it. */
const commit = (
  previous: RecordingTimelineEdit,
  next: RecordingTimelineEdit,
  {
    sourceDurationMs = DURATION_MS,
    timelines = new WeakMap(),
  }: { sourceDurationMs?: number; timelines?: RecordingOverlayTimelines } = {},
) =>
  retimeRecordingOverlays({ next, previous }, { sourceDurationMs, timelines });

describe("retimeRecordingOverlays", () => {
  it("moves clips with the footage a reorder moves, one per place it plays", () => {
    const before = edit({
      annotationClips: [annotation(3_000, 5_000)],
      sceneClips: [scene(3_000, 5_000)],
    });
    // 4-7 s now plays first, then 0-4 s, then 7-10 s.
    const after = commit(
      before,
      moveRecordingTimelineSegment(before, { boundary: 0, segmentId: 1 }),
    );
    expect(spans(after.annotationClips)).toEqual([
      [0, 1_000],
      [6_000, 7_000],
    ]);
    expect(after.annotationClips?.[0].annotation.id).toBe("arrow");
    expect(after.annotationClips?.[1].annotation.id).not.toBe("arrow");
    expect(spans(after.sceneClips)).toEqual([
      [0, 1_000],
      [6_000, 7_000],
    ]);
    expect(after.sceneClips?.[0].id).toBe("scene");
    expect(after.sceneClips?.[1].id).not.toBe("scene");
  });

  it("leaves a clip whose footage still plays where it did", () => {
    const before = edit({ annotationClips: [annotation(1_000, 3_000)] });
    const after = commit(
      before,
      moveRecordingTimelineSegment(before, { boundary: 3, segmentId: 1 }),
    );
    expect(after.annotationClips).toBe(before.annotationClips);
  });

  // The edit from the report: a 14 038 ms recording cut 5 803.5 ms in, a
  // whole millisecond away from any clip time.
  it("keeps both parts clear of a cut that falls between milliseconds", () => {
    const before = edit({
      annotationClips: [annotation(2_956, 7_447)],
      nextSegmentId: 2,
      segments: [
        { id: 0, sourceEnd: 0.4134144240702561, sourceStart: 0 },
        { id: 1, sourceEnd: 1, sourceStart: 0.4134144240702561 },
      ],
    });
    const after = commit(
      before,
      moveRecordingTimelineSegment(before, { boundary: 0, segmentId: 1 }),
      { sourceDurationMs: 14_038 },
    );
    expect(spans(after.annotationClips)).toEqual([
      [0, 1_643],
      [11_190, 14_038],
    ]);
    const { fragments } = recordingAnnotationRows(
      after.annotationClips ?? [],
      after,
      14_038,
    );
    expect(fragments.every((fragment) => fragment.row === 0)).toBe(true);
  });

  it("shortens a clip for good where a cut takes its footage", () => {
    const before = edit({ annotationClips: [annotation(3_000, 6_000)] });
    // Cut 4-5 s out: the clip loses that second.
    const cut = commit(before, deleteRecordingTimelineRange(before, 0.4, 0.5));
    expect(spans(cut.annotationClips)).toEqual([[3_000, 5_000]]);
    // Giving the footage back afterwards does not bring it back into the
    // clip: the footage on either side of it stays, apart.
    const restored = commit(cut, { ...cut, segments: before.segments });
    expect(spans(restored.annotationClips)).toEqual([
      [3_000, 4_000],
      [5_000, 6_000],
    ]);
  });

  it("drops a clip whose footage is all cut away", () => {
    const before = edit({ sceneClips: [scene(4_200, 4_800)] });
    const after = commit(
      before,
      deleteRecordingTimelineRange(before, 0.4, 0.5),
    );
    expect(after.sceneClips).toEqual([]);
  });

  it("re-times a gesture's every edit from the one it started on", () => {
    const start = edit({ annotationClips: [annotation(5_000, 8_000)] });
    const timelines: RecordingOverlayTimelines = new WeakMap();
    const trim = (sourcePosition: number) =>
      trimRecordingTimelineSegment(start, {
        edge: "end",
        minimumDuration: 0,
        segmentId: 1,
        sourcePosition,
      });
    // The trim pulls 4-7 s in to 4-6 s, then lets it back out to 7 s.
    const pulled = commit(start, trim(0.6), { timelines });
    expect(spans(pulled.annotationClips)).toEqual([[5_000, 7_000]]);
    const released = commit(pulled, trim(0.7), { timelines });
    expect(spans(released.annotationClips)).toEqual([[5_000, 8_000]]);
  });

  it("leaves a list the edit made itself as it is", () => {
    const before = edit({ annotationClips: [annotation(3_000, 6_000)] });
    const cut = deleteRecordingTimelineRange(before, 0.4, 0.5);
    const own = [annotation(1_000, 2_000)];
    const after = commit(before, { ...cut, annotationClips: own });
    expect(after.annotationClips).toBe(own);
  });
});
