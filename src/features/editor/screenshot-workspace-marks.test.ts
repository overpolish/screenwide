// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { Annotation } from "./annotations";
import {
  defaultScreenshotOutput,
  screenshotOutputTemplate,
  ScreenshotWorkspaceOutputSettings,
  withScreenshotWorkspaceItemOutput,
} from "./screenshot-output";

const arrow = (id: string): Annotation => ({
  aboveCamera: false,
  id,
  shape: {
    control: { x: 5, y: 0 },
    end: { x: 10, y: 0 },
    kind: "arrow",
    start: { x: 0, y: 0 },
  },
  style: { color: "#ff383c", head: "end", width: 8 },
});

const workspace = (): ScreenshotWorkspaceOutputSettings => {
  const canvas = defaultScreenshotOutput(100, 100);
  return { ...canvas, items: [{ id: 1, output: canvas }] };
};

describe("a layer's marks against the shared canvas", () => {
  it("keeps them on the layer they were drawn on", () => {
    const next = withScreenshotWorkspaceItemOutput(
      workspace(),
      { ...defaultScreenshotOutput(100, 100), annotations: [arrow("a")] },
      1,
    );

    expect(next.items[0].output.annotations).toHaveLength(1);
    // The canvas is what a new layer and the next capture are built from, so
    // a mark left on it would be inherited by pictures it was never drawn on.
    expect(next.annotations).toEqual([]);
  });

  it("leaves the layer's own inset colour off the canvas too", () => {
    const current = { ...workspace(), recenterInsetColor: "#101010" };
    const next = withScreenshotWorkspaceItemOutput(
      current,
      { ...defaultScreenshotOutput(100, 100), recenterInsetColor: "#ffffff" },
      1,
    );

    expect(next.recenterInsetColor).toBe("#101010");
    expect(next.items[0].output.recenterInsetColor).toBe("#ffffff");
  });

  it("carries the shared canvas fields the layer changed", () => {
    const next = withScreenshotWorkspaceItemOutput(
      workspace(),
      { ...defaultScreenshotOutput(100, 100), backgroundColor: "#123456" },
      1,
    );

    expect(next.backgroundColor).toBe("#123456");
  });
});

describe("screenshotOutputTemplate", () => {
  it("drops the marks and keeps everything else", () => {
    const settings = {
      ...defaultScreenshotOutput(100, 100),
      annotations: [arrow("a")],
      backgroundColor: "#123456",
    };

    const template = screenshotOutputTemplate(settings);

    expect(template.annotations).toEqual([]);
    expect(template.backgroundColor).toBe("#123456");
    expect(settings.annotations).toHaveLength(1);
  });
});
