// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { RecordingAnnotationClip } from "./recording-annotations";
import { heldLengthClip } from "./sticker-once";

const sticker = (
  [startMs, endMs]: [number, number],
  once: boolean,
): RecordingAnnotationClip => ({
  annotation: {
    animated: true,
    id: "s",
    shape: {
      angle: 0,
      aspect: 1,
      asset: "image:0123456789abcdef0123456789abcdef",
      center: { x: 0, y: 0 },
      flip: false,
      kind: "sticker",
      play: { cycleMs: 1_200, frame: 0, frames: 12, once },
      size: 96,
    },
    style: {
      align: "left",
      blur: false,
      color: "#000000",
      handDrawn: false,
      head: "none",
      manual: false,
      radius: 0,
      redaction: "erase",
      shadow: false,
      softness: 0,
      strength: 0,
      tint: false,
      width: 1,
    },
  },
  endMs,
  startMs,
  trackId: "primary",
});

const span = (clip: RecordingAnnotationClip) => [clip.startMs, clip.endMs];

describe("a moving sticker played once", () => {
  it("lasts exactly one run of its animation, however it was trimmed", () => {
    expect(span(heldLengthClip(sticker([2_000, 9_000], true), 60_000))).toEqual(
      [2_000, 3_200],
    );
    expect(span(heldLengthClip(sticker([2_000, 2_300], true), 60_000))).toEqual(
      [2_000, 3_200],
    );
  });

  it("starts earlier near the end of the recording so the run is seen whole", () => {
    expect(
      span(heldLengthClip(sticker([59_500, 60_000], true), 60_000)),
    ).toEqual([58_800, 60_000]);
  });

  it("leaves a looping one at the length it was given", () => {
    const looping = sticker([2_000, 9_000], false);
    expect(heldLengthClip(looping, 60_000)).toBe(looping);
  });
});
