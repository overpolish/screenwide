// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { withCropShape } from "../../recording/preview/recording-selection";

import {
  keepsShape,
  recordingCropAspectKey,
  sizeOfAspect,
} from "./crop-aspect-store";

const SOURCE = { height: 1080, width: 1920 };

describe("the size a shape keeps of a crop", () => {
  it("is the largest of a ratio inside the crop", () => {
    expect(sizeOfAspect("1:1", { height: 600, width: 1000 }, SOURCE)).toEqual({
      height: 600,
      width: 600,
    });
    expect(sizeOfAspect("9:16", { height: 600, width: 1000 }, SOURCE)).toEqual({
      height: 600,
      width: 338,
    });
    expect(sizeOfAspect("16:9", { height: 900, width: 800 }, SOURCE)).toEqual({
      height: 450,
      width: 800,
    });
  });

  it("is the capture's own shape for the original", () => {
    expect(
      sizeOfAspect("original", { height: 1000, width: 1000 }, SOURCE),
    ).toEqual({ height: 563, width: 1000 });
  });

  it("is none for a free crop, nor a locked one, which has its shape", () => {
    expect(sizeOfAspect("free", { height: 10, width: 30 }, SOURCE)).toBeNull();
    expect(
      sizeOfAspect("locked", { height: 10, width: 30 }, SOURCE),
    ).toBeNull();
  });
});

describe("whether a crop keeps its shape", () => {
  it("keeps it for every shape but free", () => {
    expect(keepsShape(undefined)).toBe(false);
    expect(keepsShape("free")).toBe(false);
    expect(keepsShape("locked")).toBe(true);
    expect(keepsShape("original")).toBe(true);
    expect(keepsShape("4:3")).toBe(true);
  });

  it("reads the screen's over a scene from the scene, the camera's not", () => {
    expect(recordingCropAspectKey("primary", "a")).toBe("recording:scene:a");
    expect(recordingCropAspectKey("primary", null)).toBe("recording:primary");
    expect(recordingCropAspectKey("camera", "a")).toBe("recording:camera");
  });

  it("tells a recording's crop window, and only a crop window", () => {
    const pane = {
      paneIndex: 0,
      radiusPercent: 0,
      rect: { height: 1, width: 1, x: 0, y: 0 },
    };
    const aspects = {
      "recording:camera": "locked",
      "recording:scene:a": "1:1",
    } as const;
    const crop = { ...pane, cropMode: true };
    expect(withCropShape(crop, aspects, "a").cropKeepsShape).toBe(true);
    expect(withCropShape(crop, aspects, null).cropKeepsShape).toBe(false);
    expect(
      withCropShape({ ...crop, layerId: 1, paneIndex: 0 }, aspects, null)
        .cropKeepsShape,
    ).toBe(true);
    expect(withCropShape(pane, aspects, "a").cropKeepsShape).toBeUndefined();
  });
});
