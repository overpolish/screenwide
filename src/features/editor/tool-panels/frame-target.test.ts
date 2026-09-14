// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it, vi } from "vitest";

import {
  defaultScreenshotOutput,
  ScreenshotOutputSettings,
  ScreenshotWorkspaceOutputSettings,
} from "../screenshot-output";
import { EditorArtifact } from "../types";

import { editorFrameTarget } from "./frame-target";

const screenshotArtifact: EditorArtifact = {
  extension: "png",
  height: 900,
  id: 1,
  items: [{ height: 900, id: 7, width: 1_600 }],
  kind: "screenshot",
  suggestedFileStem: "capture",
  width: 1_600,
};

const recordingArtifact = {
  extension: "mp4",
  height: 900,
  id: 2,
  kind: "recording",
  width: 1_600,
} as EditorArtifact;

describe("frame radius target", () => {
  it("changes the screenshot canvas radius while preserving the layer radius", () => {
    const output = {
      ...defaultScreenshotOutput(1_600, 900, {
        background: 4,
        screenshot: 18,
      }),
      items: [
        {
          id: 7,
          output: defaultScreenshotOutput(800, 450, { screenshot: 27 }),
        },
      ],
    };
    const apply = vi.fn<(next: ScreenshotWorkspaceOutputSettings) => void>();
    const target = editorFrameTarget({
      artifact: screenshotArtifact,
      bakeCamera: false,
      enabledVideoTracks: [],
      onCanvasResize: apply,
      recordingOutput: null,
      screenshotOutput: output,
      selectedScreenshotItemId: 7,
      selectedTrack: null,
    });

    target?.applyRadius(11);

    expect(target?.snapshot.radius).toBe(4);
    expect(apply.mock.calls[0][0].backgroundRadiusPercent).toBe(11);
    expect(apply.mock.calls[0][0].items[0].output.radiusPercent).toBe(27);
  });

  it("changes the selected recording canvas radius", () => {
    const output = defaultScreenshotOutput(1_600, 900, {
      background: 3,
      screenshot: 21,
    });
    const apply = vi.fn<(next: ScreenshotOutputSettings) => void>();
    const target = editorFrameTarget({
      artifact: recordingArtifact,
      bakeCamera: false,
      enabledVideoTracks: ["primary"],
      onRecordingOutputChange: (_track, next) => {
        apply(next);
      },
      recordingOutput: { camera: output, cameraOnTop: false, primary: output },
      screenshotOutput: null,
      selectedScreenshotItemId: null,
      selectedTrack: "primary",
    });

    target?.applyRadius(9);

    expect(target?.snapshot.radius).toBe(3);
    expect(apply.mock.calls[0][0].backgroundRadiusPercent).toBe(9);
    expect(apply.mock.calls[0][0].radiusPercent).toBe(21);
  });
});
