// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it, vi } from "vitest";

import { defaultCameraOverlay } from "../recording-export-settings";
import { sourceRect } from "../screenshot-geometry";
import { screenshotLayout } from "../screenshot-layout";
import {
  defaultScreenshotOutput,
  ScreenshotOutputSettings,
  ScreenshotWorkspaceOutputSettings,
} from "../screenshot-output";
import { withScreenshotSourceCrop } from "../screenshot-output-settings";
import { EditorArtifact } from "../types";

import {
  audioPanelHandlers,
  editorAudioSelectionTarget,
  editorSelectionTarget,
} from "./selection-target";

const source = { height: 1_000, width: 1_000 };

const artifact: EditorArtifact = {
  extension: "png",
  height: source.height,
  id: 1,
  items: [{ height: source.height, id: 7, width: source.width }],
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
    enabledVideoTracks: [],
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
  it("grows the frame past the picture by the same amount on every side", () => {
    const settings = croppedLayer();
    const { apply, target } = targetFor(settings);

    target.applyInset(40);

    const next = apply.mock.calls[0][0];
    expect(apply).toHaveBeenCalledOnce();
    expect(apply.mock.calls[0][1]).toBe(7);
    expect(next.cropHeight).toBeCloseTo(680);
    expect(next.cropWidth).toBeCloseTo(680);
    expect(next.cropX).toBeCloseTo(160);
    expect(next.cropY).toBeCloseTo(160);
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

    target.applyInset(120);

    const next = apply.mock.calls[0][0];
    expect(next.cropX).toBe(-120);
    expect(next.cropY).toBe(-120);
    expect(next.cropWidth).toBe(1_240);
    expect(next.cropHeight).toBe(1_240);
  });

  it("measures each inset from the picture rather than from the last frame", () => {
    const first = targetFor(croppedLayer());
    first.target.applyInset(30);
    const once = first.apply.mock.calls[0][0];

    const again = targetFor(once);
    again.target.applyInset(30);

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

  it("shows the padding it has and the layer's shorter side as its limit", () => {
    const { target } = targetFor(
      croppedLayer({ cropHeight: 700, cropWidth: 700, cropX: 150, cropY: 150 }),
    );

    expect(target.selection.inset).toBe(50);
    expect(target.selection.insetMaximum).toBe(600);
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
  it("names the track the recording named and starts at the recorded level", () => {
    const target = editorAudioSelectionTarget({
      artifact: recording,
      audioTrackVolumes: [],
      selectedTrack: "audio:1",
    });

    expect(target?.selection).toEqual({
      decibels: 0,
      kind: "audio",
      label: "Microphone",
    });
  });

  it("shows the level the editor holds for that stream", () => {
    const target = editorAudioSelectionTarget({
      artifact: recording,
      audioTrackVolumes: [
        { decibels: -12, streamIndex: 0 },
        { decibels: 6, streamIndex: 1 },
      ],
      selectedTrack: "audio:0",
    });

    expect(target?.selection).toMatchObject({
      decibels: -12,
      label: "System audio",
    });
  });

  it("is nothing at all where a video track or no track is selected", () => {
    for (const selectedTrack of ["primary", "camera", "audio:7", null]) {
      expect(
        editorAudioSelectionTarget({
          artifact: recording,
          audioTrackVolumes: [],
          selectedTrack,
        }),
      ).toBeNull();
    }
    expect(
      editorAudioSelectionTarget({
        artifact,
        audioTrackVolumes: [],
        selectedTrack: "audio:0",
      }),
    ).toBeNull();
  });

  it("sends a volume back out through the editor's own handler", () => {
    const onSelectedTrackVolumeChange = vi.fn<(decibels: number) => void>();
    const target = editorAudioSelectionTarget({
      artifact: recording,
      audioTrackVolumes: [{ decibels: 6, streamIndex: 1 }],
      onSelectedTrackVolumeChange,
      selectedTrack: "audio:1",
    });

    audioPanelHandlers(target).onAudioVolumeChange?.(-3);
    // The reset is the same request at the recorded level.
    audioPanelHandlers(target).onAudioVolumeChange?.(0);

    expect(onSelectedTrackVolumeChange.mock.calls).toEqual([[-3], [0]]);
  });
});
