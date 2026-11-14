// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { EditorArtifact } from "../types";

import { newestScreenshotLayer } from "./screenshot-blank";
import { defaultScreenshotOutput } from "./screenshot-output";
import {
  reopenedScreenshotWorkspace,
  seedScreenshotItemOutput,
} from "./screenshot-seed";

const layer = (id: number, isBlank = false) => ({
  annotations: [],
  height: 1_080,
  id,
  isBlank,
  width: 1_920,
});

describe("a blank canvas", () => {
  it("draws its blank layer with no shadow or corners", () => {
    const template = {
      ...defaultScreenshotOutput(1_920, 1_080, { screenshot: 12 }),
      dropShadow: true,
    };
    const output = seedScreenshotItemOutput(template, layer(1, true));
    expect(output.dropShadow).toBe(false);
    expect(output.radiusPercent).toBe(0);
    // A picture takes the look.
    expect(seedScreenshotItemOutput(template, layer(2)).dropShadow).toBe(true);
  });

  it("hands tools the newest picture, the blank layer only when alone", () => {
    expect(newestScreenshotLayer([layer(1, true), layer(2)])?.id).toBe(2);
    expect(newestScreenshotLayer([layer(2), layer(3, true)])?.id).toBe(2);
    expect(newestScreenshotLayer([layer(3, true)])?.id).toBe(3);
  });

  it("reopens without the picture deleted to leave it blank", () => {
    const artifact: EditorArtifact = {
      deletedItemIds: [1],
      extension: "png",
      height: 1_080,
      id: 9,
      items: [layer(1), layer(2, true)],
      kind: "screenshot",
      suggestedFileStem: "canvas",
      width: 1_920,
    };
    const canvas = defaultScreenshotOutput(1_920, 1_080);
    const reopened = reopenedScreenshotWorkspace(artifact, {
      ...canvas,
      items: [{ id: 2, output: canvas }],
    });
    expect(reopened.items.map((item) => item.id)).toEqual([2]);
  });
});
