// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  shownAnnotationClips,
  withHiddenAnnotationClips,
} from "./recording-annotation-visibility";
import { RecordingAnnotationClip } from "./recording-annotations";

const clip = (
  id: string,
  trackId: RecordingAnnotationClip["trackId"],
): RecordingAnnotationClip => ({
  annotation: {
    animated: true,
    id,
    shape: {
      control: { x: 5, y: 5 },
      end: { x: 10, y: 10 },
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
  },
  endMs: 2_000,
  startMs: 0,
  trackId,
});

const ids = (clips: RecordingAnnotationClip[]) =>
  clips.map(({ annotation }) => annotation.id);

describe("annotations on a track turned off", () => {
  const screenOnly = new Set(["primary"] as const);
  const clips = [
    clip("a", "primary"),
    clip("camera", "camera"),
    clip("b", "primary"),
  ];

  it("keep their place through a reorder of the shown ones", () => {
    const shown = shownAnnotationClips(clips, screenOnly);
    expect(ids(shown)).toEqual(["a", "b"]);
    const reordered = [shown[1], shown[0]];
    expect(
      ids(withHiddenAnnotationClips(clips, reordered, screenOnly)),
    ).toEqual(["b", "camera", "a"]);
  });

  it("survive a delete of a shown one", () => {
    expect(
      ids(withHiddenAnnotationClips(clips, [clips[2]], screenOnly)),
    ).toEqual(["camera", "b"]);
  });
});
