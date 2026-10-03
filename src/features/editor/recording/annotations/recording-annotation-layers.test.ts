// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { Annotation, AnnotationShape } from "../../annotations/annotations";

import {
  arrangedRecordingAnnotations,
  RecordingCameraPlacement,
  recordingAnnotationArrangements,
} from "./recording-annotation-layers";
import { RecordingAnnotationClip } from "./recording-annotations";

const annotation = (id: string, shape: AnnotationShape): Annotation => ({
  animated: true,
  id,
  shape,
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
});

const arrow = (
  id: string,
  trackId: RecordingAnnotationClip["trackId"],
): RecordingAnnotationClip => ({
  annotation: annotation(id, {
    control: { x: 5, y: 5 },
    end: { x: 10, y: 10 },
    kind: "arrow",
    start: { x: 0, y: 0 },
  }),
  endMs: 2_000,
  startMs: 0,
  trackId,
});

/** The screen's pixels drawn one to one from (100, 50); a 400 pixel camera
 * drawn 100 canvas pixels wide from (700, 400): a quarter of a pixel each,
 * which draws its annotations at half their weight. The screen was captured
 * at two pixels to the point. */
const placement: RecordingCameraPlacement = {
  camera: { scale: 0.25, x: 700, y: 400 },
  screen: { scale: 1, x: 100, y: 50 },
  screenCaptureScale: 2,
};

const move = (
  clips: RecordingAnnotationClip[],
  arrangement: "back" | "backward" | "forward" | "front",
  id: string,
) =>
  arrangedRecordingAnnotations({
    arrangement,
    clips,
    isMember: (clip) => clip.annotation.id === id,
    placement,
  });

describe("annotations moving between the screen and the camera", () => {
  it("hands the top screen annotation to the camera, in place and in look", () => {
    const next = move(
      [arrow("screen", "primary"), arrow("a", "primary")],
      "forward",
      "a",
    );
    // Under the camera's own annotations; the screen's keep their place.
    expect(next.map((clip) => [clip.annotation.id, clip.trackId])).toEqual([
      ["a", "camera"],
      ["screen", "primary"],
    ]);
    const moved = next[0];
    // Screen (0, 0) is canvas (100, 50), which the camera, a quarter of a
    // pixel to its own from (700, 400), holds at (-2400, -1400).
    expect(moved.annotation.shape).toMatchObject({
      end: { x: -2_360, y: -1_360 },
      start: { x: -2_400, y: -1_400 },
    });
    // Eight points at two pixels each is 16 canvas pixels; the camera draws
    // its own at half weight, so it carries 32.
    expect(moved.annotation.style.width).toBe(32);
  });

  it("steps within the screen before it hands over", () => {
    const moved = move(
      [arrow("a", "primary"), arrow("b", "primary")],
      "forward",
      "a",
    );
    expect(moved.map((clip) => [clip.annotation.id, clip.trackId])).toEqual([
      ["b", "primary"],
      ["a", "primary"],
    ]);
  });

  it("hands it back to the top of the screen, where it was", () => {
    const start = [arrow("b", "primary"), arrow("a", "primary")];
    const there = move(start, "forward", "a");
    expect(move(there, "backward", "a")).toEqual(start);
  });

  it("hands a redaction to the camera under the camera's own marks", () => {
    const redaction: RecordingAnnotationClip = {
      ...arrow("r", "primary"),
      annotation: annotation("r", {
        end: { x: 10, y: 10 },
        kind: "redact",
        seed: 1,
        start: { x: 0, y: 0 },
      }),
    };
    const moved = move(
      [redaction, arrow("a", "primary"), arrow("c", "camera")],
      "forward",
      "r",
    );
    expect(moved.map((clip) => [clip.annotation.id, clip.trackId])).toEqual([
      ["r", "camera"],
      ["a", "primary"],
      ["c", "camera"],
    ]);
  });

  it("keeps a pinned clip, and moves to the front or back, on its own picture", () => {
    const pinned: RecordingAnnotationClip = {
      ...arrow("p", "primary"),
      pin: { keyframes: [], pinnedMs: 0 },
    };
    expect(move([arrow("a", "primary")], "front", "a")).toHaveLength(1);
    expect(move([arrow("a", "primary")], "front", "a")[0].trackId).toBe(
      "primary",
    );
    expect(
      recordingAnnotationArrangements(
        [pinned],
        (clip) => clip.annotation.id === "p",
        placement,
      ).canBringForward,
    ).toBe(false);
  });

  it("offers no handover where the camera is not drawn into the video", () => {
    expect(
      recordingAnnotationArrangements(
        [arrow("a", "primary")],
        (clip) => clip.annotation.id === "a",
        null,
      ),
    ).toEqual({
      canBringForward: false,
      canMoveToBack: false,
      canMoveToFront: false,
      canSendBackward: false,
    });
  });
});
