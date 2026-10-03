// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { Annotation } from "../annotations/annotations";

import {
  defaultScreenshotOutput,
  screenshotOutputTemplate,
  ScreenshotWorkspaceOutputSettings,
  withScreenshotWorkspaceItemOutput,
} from "./screenshot-output";

const arrow = (id: string): Annotation => ({
  animated: true,
  id,
  shape: {
    control: { x: 5, y: 0 },
    end: { x: 10, y: 0 },
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
});

const workspace = (): ScreenshotWorkspaceOutputSettings => {
  const canvas = defaultScreenshotOutput(100, 100);
  return { ...canvas, items: [{ id: 1, output: canvas }] };
};

describe("a layer's annotations against the shared canvas", () => {
  it("keeps them on the layer they were drawn on", () => {
    const next = withScreenshotWorkspaceItemOutput(
      workspace(),
      { ...defaultScreenshotOutput(100, 100), annotations: [arrow("a")] },
      1,
    );

    expect(next.items[0].output.annotations).toHaveLength(1);
    // The canvas is what a new layer and the next capture are built from, so an
    // annotation left on it would be inherited by pictures it was never drawn
    // on.
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

  it("switches every layer's spotlights' blur with one layer's", () => {
    const spotlight = (id: string, blur: boolean): Annotation => ({
      ...arrow(id),
      shape: {
        end: { x: 10, y: 10 },
        kind: "spotlight",
        start: { x: 0, y: 0 },
      },
      style: { ...arrow(id).style, blur },
    });
    const canvas = defaultScreenshotOutput(100, 100);
    const current: ScreenshotWorkspaceOutputSettings = {
      ...canvas,
      items: [
        { id: 1, output: { ...canvas, annotations: [spotlight("a", false)] } },
        { id: 2, output: { ...canvas, annotations: [spotlight("b", false)] } },
      ],
    };
    const next = withScreenshotWorkspaceItemOutput(
      current,
      { ...canvas, annotations: [spotlight("a", true)] },
      1,
    );

    expect(
      next.items.map((item) => item.output.annotations[0].style.blur),
    ).toEqual([true, true]);
  });
});

describe("screenshotOutputTemplate", () => {
  it("drops the annotations and keeps everything else", () => {
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
