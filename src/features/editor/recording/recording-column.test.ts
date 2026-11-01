// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { columnMode as screenshotColumnMode } from "../screenshot/screenshot-column";

import { columnMode, modeTool } from "./recording-column";

const SCREEN = { picture: "screen" } as const;

describe("the recording's mode follows the layer chosen with Select", () => {
  it("opens Screen or Camera for the track chosen, whatever chose it", () => {
    expect(
      columnMode({ ...SCREEN, panel: null, tool: "select", track: "primary" }),
    ).toBe("screen");
    expect(
      columnMode({ ...SCREEN, panel: null, tool: "select", track: "camera" }),
    ).toBe("camera");
  });

  it("opens Camera for a camera-only recording's own picture", () => {
    expect(
      columnMode({
        panel: null,
        picture: "camera",
        tool: "select",
        track: "primary",
      }),
    ).toBe("camera");
  });

  it("puts Screen and Camera away with nothing chosen", () => {
    expect(
      columnMode({ ...SCREEN, panel: null, tool: "select", track: null }),
    ).toBe(null);
  });

  it("takes the chosen layer's mode over an island-only one", () => {
    expect(
      columnMode({
        ...SCREEN,
        panel: "cursor",
        tool: "select",
        track: "primary",
      }),
    ).toBe("screen");
    expect(
      columnMode({ ...SCREEN, panel: "cursor", tool: "select", track: null }),
    ).toBe("cursor");
  });

  it("leaves the picture's own tools their modes, and pens none", () => {
    expect(
      columnMode({ ...SCREEN, panel: null, tool: "canvas", track: "primary" }),
    ).toBe("frame");
    expect(
      columnMode({ ...SCREEN, panel: null, tool: "scene", track: "camera" }),
    ).toBe("scene");
    expect(
      columnMode({ ...SCREEN, panel: null, tool: "arrow", track: "primary" }),
    ).toBe(null);
    expect(
      columnMode({ ...SCREEN, panel: null, tool: "crop", track: "primary" }),
    ).toBe(null);
  });

  it("opens Screen and Camera by choosing their track, the others by letting it go", () => {
    expect(modeTool("screen", "screen")).toEqual({
      panel: null,
      tool: "select",
      track: "primary",
    });
    expect(modeTool("camera", "screen")).toEqual({
      panel: null,
      tool: "select",
      track: "camera",
    });
    expect(modeTool("cursor", "screen")).toEqual({
      panel: "cursor",
      tool: "select",
      track: null,
    });
    expect(modeTool("scene", "screen")).toEqual({
      panel: null,
      tool: "scene",
    });
  });

  it("opens Camera on a camera-only recording's own track", () => {
    expect(modeTool("camera", "camera")).toEqual({
      panel: null,
      tool: "select",
      track: "primary",
    });
  });
});

describe("the screenshot's mode follows the layer chosen with Select", () => {
  it("lights Screenshot for a layer chosen with Select alone", () => {
    expect(screenshotColumnMode("select", true)).toBe("screen");
    expect(screenshotColumnMode("select", false)).toBe(null);
    expect(screenshotColumnMode("crop", true)).toBe(null);
    expect(screenshotColumnMode("canvas", true)).toBe("frame");
  });
});
