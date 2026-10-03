// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { RecordingAnnotationClip } from "./recording-annotations";
import { withRecordingSpotlightBlurShared } from "./recording-spotlight-blur";

const spotlight = (
  id: string,
  [startMs, endMs]: [number, number],
  {
    blur = false,
    trackId = "primary",
  }: { blur?: boolean; trackId?: RecordingAnnotationClip["trackId"] } = {},
): RecordingAnnotationClip => ({
  annotation: {
    animated: true,
    id,
    shape: { end: { x: 10, y: 10 }, kind: "spotlight", start: { x: 0, y: 0 } },
    style: {
      align: "left",
      blur,
      color: "#000000",
      handDrawn: false,
      head: "none",
      manual: false,
      radius: 12,
      redaction: "erase",
      shadow: false,
      softness: 10,
      strength: 0,
      tint: false,
      width: 1,
    },
  },
  endMs,
  startMs,
  trackId,
});

const blurs = (clips: RecordingAnnotationClip[]) =>
  Object.fromEntries(
    clips.map((clip) => [clip.annotation.id, clip.annotation.style.blur]),
  );

describe("withRecordingSpotlightBlurShared", () => {
  // The screen's a meets the camera's b, which meets c; d comes after them.
  const before = [
    spotlight("a", [0, 2_000]),
    spotlight("b", [1_500, 4_000], { trackId: "camera" }),
    spotlight("c", [3_000, 5_000]),
    spotlight("d", [6_000, 8_000]),
  ];

  it("spreads a switched setting through every spotlight shown with it", () => {
    const switched = before.map((clip) =>
      clip.annotation.id === "a"
        ? spotlight("a", [0, 2_000], { blur: true })
        : clip,
    );
    expect(blurs(withRecordingSpotlightBlurShared(switched, before))).toEqual({
      a: true,
      b: true,
      c: true,
      d: false,
    });
  });

  it("has a spotlight drawn or moved into a run take the run's setting", () => {
    const blurred = before.map((clip) =>
      spotlight(clip.annotation.id, [clip.startMs, clip.endMs], {
        blur: clip.annotation.id !== "d",
        trackId: clip.trackId,
      }),
    );
    const next = [
      ...blurred.filter((clip) => clip.annotation.id !== "d"),
      spotlight("d", [4_500, 6_500]),
      spotlight("e", [500, 1_000], { trackId: "camera" }),
    ];
    expect(blurs(withRecordingSpotlightBlurShared(next, blurred))).toEqual({
      a: true,
      b: true,
      c: true,
      d: true,
      e: true,
    });
  });
});
