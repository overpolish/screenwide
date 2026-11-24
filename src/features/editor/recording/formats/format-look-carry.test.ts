// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { defaultCameraOverlay } from "../../export/recording-export-settings";
import { defaultRecordingOutput } from "../../screenshot/screenshot-output";

import { withLookCarried } from "./format-look-carry";

import type { FormatDocument } from "./format-document";
import type { FrameFormats } from "../../../../bindings/FrameFormats";

const look = (width: number, height: number): FormatDocument => ({
  cameraOverlay: defaultCameraOverlay(),
  recordingOutput: defaultRecordingOutput({ primary: { height, width } }),
});

const vertical = look(1080, 1920);
const formats: FrameFormats = {
  active: "vertical",
  list: [
    { id: "wide", look: look(1920, 1080), platforms: ["youtube-1080"] },
    { id: "vertical", platforms: ["tiktok", "youtube-shorts"] },
    { id: "square", look: look(1080, 1080), platforms: ["instagram-square"] },
  ],
};

const turned: FormatDocument = {
  ...vertical,
  recordingOutput: {
    ...vertical.recordingOutput,
    primary: {
      ...vertical.recordingOutput.primary,
      imageX: 40,
      rotation: { x: -20, y: -30, z: 0 },
    },
  },
};

const primaryOf = (carried: FrameFormats | null, id: string) =>
  carried?.list.find((format) => format.id === id)?.look?.recordingOutput
    .primary;

describe("a change to the screen on the main canvas", () => {
  // The vertical main canvas laid into the wide one whole: scaled to its
  // height, centred across.
  const scale = 1080 / 1920;
  const across = (main: number) => (1920 - main * scale) / 2;

  it("places the screen on another canvas as it shows on the main one", () => {
    const carried = withLookCarried(
      formats,
      { after: turned, before: vertical },
      "all",
    );
    const wide = primaryOf(carried, "wide");
    expect(wide?.rotation).toEqual({ x: -20, y: -30, z: 0 });
    expect(wide?.imageX).toBeCloseTo(across(1080) + 40 * scale);
    expect(wide?.imageWidth).toBeCloseTo(
      turned.recordingOutput.primary.imageWidth * scale,
    );
    expect(wide?.width).toBe(1920);
  });

  it("follows the main canvas reshaped, its own size kept", () => {
    const narrowed: FormatDocument = {
      ...vertical,
      recordingOutput: {
        ...vertical.recordingOutput,
        primary: { ...vertical.recordingOutput.primary, width: 540 },
      },
    };
    const carried = withLookCarried(
      formats,
      { after: narrowed, before: vertical },
      "all",
    );
    const wide = primaryOf(carried, "wide");
    expect(wide?.imageX).toBeCloseTo(
      across(540) + vertical.recordingOutput.primary.imageX * scale,
    );
    expect(wide?.width).toBe(1920);
  });

  it("leaves a platform set its own way as it is, but for the background", () => {
    const own: FrameFormats = {
      ...formats,
      list: formats.list.map((format) =>
        format.id === "square" ? { ...format, placed: true } : format,
      ),
    };
    const repainted: FormatDocument = {
      ...turned,
      recordingOutput: {
        ...turned.recordingOutput,
        primary: { ...turned.recordingOutput.primary, backgroundColor: "#123" },
      },
    };
    const square = primaryOf(
      withLookCarried(own, { after: repainted, before: vertical }, "all"),
      "square",
    );
    const before = look(1080, 1080).recordingOutput.primary;
    expect(square?.rotation).toEqual(before.rotation);
    expect(square?.imageX).toBe(before.imageX);
    expect(square?.backgroundColor).toBe("#123");
  });

  it("stays in its shape, which its platforms share", () => {
    expect(
      withLookCarried(formats, { after: turned, before: vertical }, "shape"),
    ).toBeNull();
  });
});
