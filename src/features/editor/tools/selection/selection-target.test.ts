// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it, vi } from "vitest";

import { defaultCameraOverlay } from "../../export/recording-export-settings";
import { sourceRect } from "../../screenshot/screenshot-geometry";
import { screenshotLayout } from "../../screenshot/screenshot-layout";
import {
  defaultScreenshotOutput,
  ScreenshotOutputSettings,
  ScreenshotWorkspaceOutputSettings,
} from "../../screenshot/screenshot-output";
import { withScreenshotSourceCrop } from "../../screenshot/screenshot-output-settings";
import { EditorArtifact } from "../../types";

import {
  editorAudioSelection,
  editorSelectionTarget,
} from "./selection-target";

const source = { height: 1_000, scaleFactor: 2, width: 1_000 };

const artifact: EditorArtifact = {
  deletedItemIds: [],
  extension: "png",
  height: source.height,
  id: 1,
  items: [
    {
      annotations: [],
      height: source.height,
      id: 7,
      isBlank: false,
      scaleFactor: 2,
      width: source.width,
    },
  ],
  kind: "screenshot",
  suggestedFileStem: "capture",
  width: source.width,
};

const workspaceOutput = (
  output: ScreenshotOutputSettings,
): ScreenshotWorkspaceOutputSettings => ({
  ...defaultScreenshotOutput(source.width, source.height),
  items: [{ id: 7, output }],
});

const targetFor = (output: ScreenshotOutputSettings) => {
  const apply =
    vi.fn<(next: ScreenshotOutputSettings, itemId: number) => void>();
  const target = editorSelectionTarget({
    artifact,
    bakeCamera: false,
    cameraOverlay: defaultCameraOverlay(),
    composedVideoTracks: [],
    onScreenshotOutputChange: apply,
    recordingOutput: null,
    screenshotOutput: workspaceOutput(output),
    selectedScreenshotItemId: 7,
    selectedTrack: null,
  });
  if (!target) throw new Error("Expected a screenshot selection target");
  return { apply, target };
};

/** The layer's own picture, cut to three fifths of the source and placed at
 * the middle of the canvas. */
const croppedLayer = (settings: Partial<ScreenshotOutputSettings> = {}) =>
  withScreenshotSourceCrop(
    {
      ...defaultScreenshotOutput(source.width, source.height),
      cropHeight: 600,
      cropWidth: 600,
      cropX: 200,
      cropY: 200,
      ...settings,
    },
    sourceRect({ height: 0.6, width: 0.6, x: 0.2, y: 0.2 }),
  );

describe("selection padding", () => {
  it("grows the frame past the picture by the same share on every side", () => {
    const settings = croppedLayer();
    const { apply, target } = targetFor(settings);

    target.applyInset(3);

    const next = apply.mock.calls[0][0];
    expect(apply).toHaveBeenCalledOnce();
    expect(apply.mock.calls[0][1]).toBe(7);
    // 3% of the 1000 pixel canvas on every side.
    expect(next.cropHeight).toBeCloseTo(660);
    expect(next.cropWidth).toBeCloseTo(660);
    expect(next.cropX).toBeCloseTo(170);
    expect(next.cropY).toBeCloseTo(170);
    expect(next.inset).toEqual({ canvas: 3 });
    // The picture itself does not move: only the frame around it changes.
    expect(next.imageX).toBe(settings.imageX);
    expect(next.imageY).toBe(settings.imageY);
    expect(next.imageWidth).toBe(settings.imageWidth);
    expect(next.sourceCrop).toEqual(settings.sourceCrop);
  });

  it("lets a padded layer run past the canvas edges", () => {
    const { apply, target } = targetFor(
      withScreenshotSourceCrop(
        defaultScreenshotOutput(source.width, source.height),
        sourceRect({ height: 1, width: 1, x: 0, y: 0 }),
      ),
    );

    target.applyInset(6);

    const next = apply.mock.calls[0][0];
    expect(next.cropX).toBeCloseTo(-60);
    expect(next.cropY).toBeCloseTo(-60);
    expect(next.cropWidth).toBeCloseTo(1_120);
    expect(next.cropHeight).toBeCloseTo(1_120);
  });

  it("measures each inset from the picture rather than from the last frame", () => {
    const first = targetFor(croppedLayer());
    first.target.applyInset(1.5);
    const once = first.apply.mock.calls[0][0];

    const again = targetFor(once);
    again.target.applyInset(1.5);

    expect(again.apply.mock.calls[0][0].cropWidth).toBe(once.cropWidth);
    expect(again.apply.mock.calls[0][0].cropX).toBe(once.cropX);
  });

  it("keeps the pad's colour at no padding, so taking it back up is instant", () => {
    const padded = croppedLayer({
      cropHeight: 700,
      cropWidth: 700,
      cropX: 150,
      cropY: 150,
      recenterInsetColor: "#ffffff",
    });
    const { apply, target } = targetFor(padded);

    target.applyInset(0);

    const next = apply.mock.calls[0][0];
    const layout = screenshotLayout(source, next);
    expect(layout.crop).toEqual(layout.sourceCrop);
    expect(next.recenterInsetColor).toBe("#ffffff");
  });

  it("shows a padding saved in another unit as the look it was nearest", () => {
    // 50 pixels round a 600 pixel picture, a share of a twelfth: Large.
    const framed = targetFor(
      croppedLayer({ cropHeight: 700, cropWidth: 700, cropX: 150, cropY: 150 }),
    ).target;
    const shared = targetFor(croppedLayer({ inset: { percent: 6 } })).target;
    // 16 points, read as pixels of a 1080 high canvas: Tight.
    const pointed = targetFor(
      croppedLayer({ inset: { canvas: (16 / 1_080) * 100 } }),
    ).target;

    expect(framed.selection.inset).toBe(6);
    expect(shared.selection.inset).toBe(3);
    expect(pointed.selection.inset).toBe(1.5);
  });
});

describe("selection corners", () => {
  it("rounds a screenshot layer by a look", () => {
    const { apply, target } = targetFor(croppedLayer());

    target.applyRadius({ canvas: 3 });

    expect(apply.mock.calls[0][0].radius).toEqual({ canvas: 3 });
    expect(
      targetFor(croppedLayer({ radius: { canvas: 1 } })).target.selection
        .radius,
    ).toEqual({ canvas: 1 });
  });
});

/** A recording of a screen with two audio tracks, the way the editor hands
 * one to the panels. */
const recording: EditorArtifact = {
  audioTracks: [
    { kind: "system-audio", label: "System audio", streamIndex: 0 },
    { kind: "microphone", label: "Microphone", streamIndex: 1 },
  ],
  camera: null,
  canCompress: true,
  cursorDataVersion: null,
  durationMs: 5_000,
  extension: "mp4",
  hasCursorData: false,
  hasKeyboardData: false,
  height: 2338,
  id: 2,
  keyboardDataVersion: null,
  kind: "recording",
  originalSizeBytes: 1_000,
  path: "/tmp/recording.mp4",
  primaryKind: "screen",
  sourceScalePercent: 200,
  suggestedFileStem: "recording",
  width: 3600,
};

describe("audio selection", () => {
  it("names the track the recording named", () => {
    for (const [selectedTrack, label] of [
      ["audio:0", "System audio"],
      ["audio:1", "Microphone"],
    ])
      expect(
        editorAudioSelection({ artifact: recording, selectedTrack }),
      ).toEqual({ kind: "audio", label });
  });

  it("is nothing at all where a video track or no track is selected", () => {
    for (const selectedTrack of ["primary", "camera", "audio:7", null]) {
      expect(
        editorAudioSelection({ artifact: recording, selectedTrack }),
      ).toBeNull();
    }
    expect(
      editorAudioSelection({ artifact, selectedTrack: "audio:0" }),
    ).toBeNull();
  });
});
