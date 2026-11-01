// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { RecordingSceneClip } from "./recording-scenes";
import { sceneClipsBetween } from "./scene-clips-between";

const scene: RecordingSceneClip = {
  endMs: 2000,
  id: "a",
  preset: "full",
  startMs: 0,
};

/** Scenes between two lists, the recording's own corners 12 for the screen
 * and 30 for the camera. */
const clipsBetween = sceneClipsBetween(
  (clip, pane) => clip.radius?.[pane] ?? (pane === "camera" ? 30 : 12),
);

describe("sceneClipsBetween", () => {
  it("pads, rounds and turns a scene part of the way", () => {
    const to = {
      ...scene,
      inset: 0.1,
      radius: { screen: 40 },
      rotation: { x: 0, y: 20, z: 0 },
    };
    const [half] = clipsBetween([scene], [to], 0.5);
    expect(half.inset).toBeCloseTo(0.05);
    // From the recording's own corners, which the scene took until now.
    expect(half.radius).toEqual({ screen: 26 });
    expect(half.rotation).toEqual({ x: 0, y: 10, z: 0 });
  });

  it("starts on the old scene and ends on the new one", () => {
    const to = { ...scene, inset: 0.2 };
    expect(clipsBetween([scene], [to], 0)[0].inset).toBe(0);
    expect(clipsBetween([scene], [to], 1)[0].inset).toBe(0.2);
  });

  it("draws a scene only the new clips have as it is", () => {
    const added = { ...scene, id: "b", inset: 0.2 };
    expect(clipsBetween([scene], [added], 0.5)).toEqual([added]);
  });

  it("leaves what it does not move as the new scene has it", () => {
    const [half] = clipsBetween([scene], [scene], 0.5);
    expect(half).toEqual(scene);
  });
});
