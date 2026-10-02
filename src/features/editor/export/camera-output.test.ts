// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { Annotation } from "../annotations/annotations";
import { recordingAnnotationClipAt } from "../recording/annotations/recording-annotations";
import { hasOutputComposition } from "../screenshot/screenshot-composition";
import { defaultRecordingOutput } from "../screenshot/screenshot-output";
import { createRecordingTimelineEdit } from "../timeline/editing/recording-timeline-edit";
import { EditorArtifact } from "../types";

import {
  cameraOutputChoice,
  composedVideoTracks,
  exportedRecordingEdits,
} from "./camera-output";

const arrow: Annotation = {
  aboveCamera: false,
  animated: true,
  id: "arrow",
  shape: {
    control: { x: 5, y: 0 },
    end: { x: 10, y: 0 },
    kind: "arrow",
    start: { x: 0, y: 0 },
  },
  style: {
    align: "left",
    blur: false,
    color: "#ff383c",
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

const recording = (
  camera: { height: number; width: number } | null,
): EditorArtifact => ({
  audioTracks: [],
  camera: camera && {
    ...camera,
    durationMs: 1_000,
    originalSizeBytes: 1,
    path: "camera.mp4",
  },
  canCompress: true,
  cursorDataVersion: null,
  durationMs: 1_000,
  extension: "mp4",
  hasCursorData: false,
  hasKeyboardData: false,
  height: 1_080,
  id: 2,
  keyboardDataVersion: null,
  kind: "recording",
  originalSizeBytes: 1,
  path: "recording.mp4",
  primaryKind: "screen",
  sourceScalePercent: 100,
  suggestedFileStem: "recording",
  width: 1_920,
});

describe("cameraOutputChoice", () => {
  it("reports the choice while both tracks are kept", () => {
    const artifact = recording({ height: 720, width: 1_280 });
    const enabledVideoTracks = ["primary", "camera"] as const;

    expect(
      cameraOutputChoice({ artifact, bakeCamera: true, enabledVideoTracks }),
    ).toBe("combined");
    expect(
      cameraOutputChoice({ artifact, bakeCamera: false, enabledVideoTracks }),
    ).toBe("separate");
  });

  it("offers no choice without a camera, or with either track left out", () => {
    const withCamera = recording({ height: 720, width: 1_280 });

    for (const [artifact, enabledVideoTracks] of [
      [recording(null), ["primary"]],
      [withCamera, ["camera"]],
      [withCamera, ["primary"]],
    ] as const) {
      expect(
        cameraOutputChoice({ artifact, bakeCamera: true, enabledVideoTracks }),
      ).toBeNull();
    }
  });
});

describe("composedVideoTracks", () => {
  it("leaves a camera saved as its own file out of the picture", () => {
    expect(composedVideoTracks(["primary", "camera"], false)).toEqual([
      "primary",
    ]);
  });

  it("draws the camera where it is combined or the whole export", () => {
    expect(composedVideoTracks(["primary", "camera"], true)).toEqual([
      "primary",
      "camera",
    ]);
    expect(composedVideoTracks(["camera"], false)).toEqual(["camera"]);
  });
});

describe("exportedRecordingEdits", () => {
  const artifact = recording({ height: 720, width: 1_280 });
  const clip = (trackId: "camera" | "primary") =>
    recordingAnnotationClipAt({
      annotation: { ...arrow, id: trackId },
      sourceDurationMs: 1_000,
      sourcePositionMs: 0,
      trackId,
    });
  const timelineEdit = {
    ...createRecordingTimelineEdit(artifact.id),
    annotationClips: [clip("primary"), clip("camera")],
  };
  const recordingOutput = defaultRecordingOutput({
    camera: { height: 720, width: 1_280 },
    primary: { height: 1_080, width: 1_920 },
  });
  const edited = {
    ...recordingOutput,
    camera: { ...recordingOutput.camera, cropX: 40, radiusPercent: 20 },
  };

  it("saves a separate camera unedited, without its annotations", () => {
    const exported = exportedRecordingEdits({
      artifact,
      cameraOutput: "separate",
      recordingOutput: edited,
      timelineEdit,
    });

    const source = { height: 720, width: 1_280 };
    expect(hasOutputComposition(edited.camera, source)).toBe(true);
    expect(hasOutputComposition(exported.recordingOutput.camera, source)).toBe(
      false,
    );
    expect(exported.recordingOutput.primary).toBe(edited.primary);
    expect(
      exported.timelineEdit?.annotationClips?.map((each) => each.trackId),
    ).toEqual(["primary"]);
  });

  it("leaves a combined camera's edits alone", () => {
    const exported = exportedRecordingEdits({
      artifact,
      cameraOutput: "combined",
      recordingOutput: edited,
      timelineEdit,
    });

    expect(exported.recordingOutput).toBe(edited);
    expect(exported.timelineEdit).toBe(timelineEdit);
  });
});
