// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  ownEditedAutoZooms,
  visibleScreenArea,
  withAutoZooms,
} from "./recording-scene-auto-zoom";
import { RecordingSceneClip } from "./recording-scenes";

import type { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";

const zoom = (id: string, startMs: number, endMs: number) =>
  ({
    auto: true,
    endMs,
    id,
    preset: "full",
    screen: { focusX: 0.3, focusY: 0.4, zoom: 2 },
    startMs,
  }) as RecordingSceneClip;
const own = (id: string, startMs: number, endMs: number) =>
  ({ endMs, id, preset: "split-two-thirds", startMs }) as RecordingSceneClip;

describe("withAutoZooms", () => {
  it("replaces the auto zooms and keeps your own scenes", () => {
    const next = withAutoZooms(
      [zoom("old", 0, 4_000), own("mine", 10_000, 20_000)],
      [zoom("new", 30_000, 35_000)],
    );
    expect(next.map((clip) => clip.id)).toEqual(["mine", "new"]);
  });

  it("fits a zoom beside a scene of your own it runs into", () => {
    const [mine, fitted] = withAutoZooms(
      [own("mine", 10_000, 20_000)],
      [zoom("new", 18_000, 26_000)],
    );
    expect(mine).toMatchObject({ endMs: 20_000, startMs: 10_000 });
    expect(fitted).toMatchObject({ endMs: 26_000, id: "new", startMs: 20_000 });
  });

  it("keeps the longer side of a scene of yours inside a zoom", () => {
    const next = withAutoZooms(
      [own("mine", 12_000, 14_000)],
      [zoom("new", 10_000, 20_000)],
    );
    expect(next[1]).toMatchObject({ endMs: 20_000, startMs: 14_000 });
  });

  it("leaves out a zoom with too little room left to play", () => {
    const next = withAutoZooms(
      [own("a", 0, 10_000), own("b", 11_000, 20_000)],
      [zoom("new", 9_000, 12_000)],
    );
    expect(next.map((clip) => clip.id)).toEqual(["a", "b"]);
  });
});

describe("ownEditedAutoZooms", () => {
  it("makes an edited auto zoom yours and leaves the rest made", () => {
    const before = [zoom("a", 0, 4_000), zoom("b", 10_000, 14_000)];
    const [kept, edited] = ownEditedAutoZooms(before, [
      before[0],
      { ...before[1], endMs: 16_000 },
    ]);
    expect(kept.auto).toBe(true);
    expect(edited.auto).toBeUndefined();
    expect(edited.endMs).toBe(16_000);
  });

  it("makes a piece split off an auto zoom yours", () => {
    const before = [zoom("a", 0, 8_000)];
    const next = ownEditedAutoZooms(before, [
      { ...before[0], endMs: 4_000 },
      { ...before[0], id: "a-2", startMs: 4_000 },
    ]);
    expect(next.every((clip) => clip.auto === undefined)).toBe(true);
  });
});

describe("visibleScreenArea", () => {
  it("is the part of the picture the screen's box shows", () => {
    // A 2000 by 1000 picture drawn 1000 wide, its right half in the box.
    const output = {
      cropHeight: 500,
      cropWidth: 500,
      cropX: 100,
      cropY: 50,
      imageWidth: 1_000,
      imageX: -400,
      imageY: 50,
    } as ScreenshotOutputSettings;
    expect(visibleScreenArea(output, { height: 1_000, width: 2_000 })).toEqual({
      height: 1,
      width: 0.5,
      x: 0.5,
      y: 0,
    });
  });
});
