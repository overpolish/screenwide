// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it, vi } from "vitest";

import { ANNOTATE_SCENE, annotateLayout } from "./annotate-scene";
import { windowParts } from "./sample-window-layout";
import { TOOL_PICTURE, TOOL_SCALE } from "./scene-kit";

// The type is measured on a canvas, which the tests have none of: the
// Publish label comes out 47 points wide, as Inter sets it, the width the
// app's offscreen test (`annotate/preview_tests.rs`) lays its button out at.
vi.stubGlobal("document", {
  createElement: () => ({
    getContext: () => ({ font: "", measureText: () => ({ width: 47 }) }),
  }),
});

const look = (counterSize: number, counterAngle: number) =>
  ({
    color: "#ff3b30",
    counterAngle,
    counterSize,
    keep: true,
    kind: "annotate",
  }) as const;

/** Where the scene presses to drop its counter. */
function counterDrop(counterSize: number, counterAngle: number) {
  const pointer = ANNOTATE_SCENE.pointer(
    1.65,
    TOOL_PICTURE,
    look(counterSize, counterAngle),
  );
  if (!pointer?.pressed) throw new Error("the counter is not being dropped");
  return pointer;
}

describe("the Annotate scene", () => {
  const { button, heading } = windowParts(annotateLayout(TOOL_PICTURE.width));
  if (!button || !heading) throw new Error("the window has no button");

  it("lays the button out where the app's test has it", () => {
    expect(button).toEqual({ height: 24, width: 75, x: 907, y: 104 });
  });

  it.each([
    [28, 0],
    [80, 0],
    [56, Math.PI / 2],
    [56, 2.4],
  ])(
    "drops a %d point counter aimed at %d so its tail touches the button",
    (size, angle) => {
      const { x, y } = counterDrop(size, angle);
      const reach = (size / 2) * TOOL_SCALE * 1.5;
      const tip = [x + Math.cos(angle) * reach, y + Math.sin(angle) * reach];
      const onEdge =
        Math.abs(tip[0] - button.x) < 1e-6 ||
        Math.abs(tip[0] - (button.x + button.width)) < 1e-6 ||
        Math.abs(tip[1] - button.y) < 1e-6 ||
        Math.abs(tip[1] - (button.y + button.height)) < 1e-6;
      expect(onEdge).toBe(true);
      expect(tip[0]).toBeGreaterThanOrEqual(button.x - 1e-6);
      expect(tip[0]).toBeLessThanOrEqual(button.x + button.width + 1e-6);
      expect(tip[1]).toBeGreaterThanOrEqual(button.y - 1e-6);
      expect(tip[1]).toBeLessThanOrEqual(button.y + button.height + 1e-6);
    },
  );

  it("draws its arrow to the window's heading", () => {
    const end = ANNOTATE_SCENE.pointer(1.15, TOOL_PICTURE, look(28, 0));
    expect(end).toEqual({ pressed: true, x: 718, y: 66 });
    expect(end && end.x < heading[0]).toBe(true);
  });
});
