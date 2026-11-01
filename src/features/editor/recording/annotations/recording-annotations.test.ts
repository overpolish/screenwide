// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { ANNOTATION_DRAW_IN_MS } from "../../annotations/annotation-draw-in";
import { Annotation } from "../../annotations/annotations";
import { createRecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";

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

  it("times the clip on the edited video, under a two times source segment", () => {
    const edit = {
      ...createRecordingTimelineEdit(1),
      segments: [{ id: 0, playbackRate: 2, sourceEnd: 1, sourceStart: 0 }],
    };
    // The recording's 4 s plays 2 s into the video.
    expect(
      recordingAnnotationClipAt({
        annotation,
        edit,
        sourceDurationMs: 20_000,
        sourcePositionMs: 4_000,
      }),
    ).toMatchObject({
      endMs: 5_000,
      startMs: 2_000 - ANNOTATION_DRAW_IN_MS,
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
