// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { Annotation } from "../annotations/annotations";

import {
  arrangedScreenshotAnnotations,
  screenshotAnnotationArrangements,
} from "./screenshot-annotation-layers";
import {
  defaultScreenshotOutput,
  ScreenshotWorkspaceOutputSettings,
} from "./screenshot-output";

const arrow = (id: string, x = 0): Annotation => ({
  animated: true,
  id,
  shape: {
    control: { x: x + 5, y: 10 },
    end: { x: x + 10, y: 20 },
    kind: "arrow",
    start: { x, y: 0 },
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
});

/** Three 400 pixel wide sources: the first drawn at half size from x 100,
 * the second at full size from x 300, the third at full size from 0. */
const workspace = (
  lists: [Annotation[], Annotation[], Annotation[]],
): ScreenshotWorkspaceOutputSettings => {
  const canvas = defaultScreenshotOutput(1_000, 1_000);
  const placements = [
    [1, 100, 200],
    [2, 300, 400],
    [3, 0, 400],
  ] as const;
  return {
    ...canvas,
    items: placements.map(([id, imageX, imageWidth], at) => ({
      id,
      output: {
        ...canvas,
        annotations: lists[at],
        imageWidth,
        imageX,
        imageY: 10,
      },
    })),
  };
};

const sourceWidths = new Map([
  [1, 400],
  [2, 400],
  [3, 400],
]);

const ids = (output: ScreenshotWorkspaceOutputSettings) =>
  output.items.map((item) => item.output.annotations.map(({ id }) => id));

/** Moves the annotation `id` from whichever layer it is on. */
const move = (
  output: ScreenshotWorkspaceOutputSettings,
  arrangement: "back" | "backward" | "forward" | "front",
  id: string,
) =>
  arrangedScreenshotAnnotations({
    arrangement,
    isMember: (annotation) => annotation.id === id,
    itemId:
      output.items.find((item) =>
        item.output.annotations.some((annotation) => annotation.id === id),
      )?.id ?? -1,
    sourceWidths,
    workspace: output,
  });

describe("annotations stepping between screenshot layers", () => {
  it("steps within the layer before it leaves it", () => {
    const moved = move(
      workspace([[arrow("a"), arrow("b")], [arrow("c")], []]),
      "forward",
      "a",
    );
    expect(moved?.itemId).toBe(1);
    expect(moved && ids(moved.workspace)).toEqual([["b", "a"], ["c"], []]);
  });

  it("carries the top annotation to the bottom of the layer above, in place", () => {
    const moved = move(
      workspace([[arrow("a", 40)], [arrow("c")], []]),
      "forward",
      "a",
    );
    expect(moved?.itemId).toBe(2);
    expect(moved && ids(moved.workspace)).toEqual([[], ["a", "c"], []]);
    // (40, 0) on the first source is canvas (120, 10), which the second
    // source, one to one from (300, 10), holds at (-180, 0).
    expect(moved?.workspace.items[1].output.annotations[0].shape).toEqual({
      control: { x: -177.5, y: 5 },
      end: { x: -175, y: 10 },
      kind: "arrow",
      start: { x: -180, y: 0 },
    });
  });

  it("brings it back to the top of the layer below, where it started", () => {
    const start = workspace([[arrow("b"), arrow("a", 40)], [arrow("c")], []]);
    const up = move(start, "forward", "a");
    const down = up && move(up.workspace, "backward", "a");
    expect(down?.itemId).toBe(1);
    expect(down?.workspace.items[0].output.annotations).toEqual(
      start.items[0].output.annotations,
    );
  });

  it("never leaves the layer on a move to the front or back", () => {
    const output = workspace([[arrow("a")], [arrow("c")], []]);
    expect(move(output, "front", "a")).toBeNull();
    expect(move(output, "back", "c")).toBeNull();
  });

  it("stops at the topmost and bottommost layers", () => {
    const output = workspace([[arrow("a")], [], [arrow("z")]]);
    expect(move(output, "forward", "z")).toBeNull();
    expect(move(output, "backward", "a")).toBeNull();
  });

  it("offers a step past either end, and the ends only within the layer", () => {
    const output = workspace([[arrow("a")], [arrow("b"), arrow("c")], []]);
    expect(
      screenshotAnnotationArrangements(output, 2, ({ id }) => id === "c"),
    ).toEqual({
      canBringForward: true,
      canMoveToBack: true,
      canMoveToFront: false,
      canSendBackward: true,
    });
  });
});
