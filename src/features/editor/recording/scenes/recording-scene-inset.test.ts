// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { defaultCameraOverlay } from "../../export/recording-export-settings";
import {
  defaultRecordingOutput,
  screenshotOutputDimensions,
} from "../../screenshot/screenshot-output";
import {
  recordingVideoSelectionOverlay,
  recordingVideoSelectionTargets,
} from "../preview/recording-selection-overlay";

import { arrangedRecordingScene } from "./recording-scene-arrangement";
import { boxEdit } from "./recording-scene-gesture-edits";
import { RecordingSceneClip } from "./recording-scenes";

import type { RecordingSelectionGestureEvent } from "../use-recording-preview-surface";

const source = { height: 900, width: 1600 };
const recording = defaultRecordingOutput({ primary: source });
/** The recording's own crop, padded by its own inset of 40 pixels. */
const output = {
  ...recording.primary,
  cropHeight: recording.primary.cropHeight - 200 + 80,
  cropWidth: recording.primary.cropWidth - 300 + 80,
  cropX: 150 - 40,
  cropY: 100 - 40,
};
const zoom = (inset?: number): RecordingSceneClip => ({
  endMs: 10_000,
  id: "z",
  ...(inset === undefined ? {} : { inset }),
  preset: "full",
  screen: { focusX: 0.5, focusY: 0.5, zoom: 2 },
  startMs: 1_000,
});
const arranged = (clip: RecordingSceneClip, sourceMs = 5_000) =>
  arrangedRecordingScene({
    camera: null,
    clips: [clip],
    output,
    ranges: [{ playbackRate: 1, sourceEndMs: 20_000, sourceStartMs: 0 }],
    sourceMs,
  });

/** The screen's hit target and the selection drawn over it, as the select
 * tool hands them to the native overlay over the scene's `primary`. */
const targetAndSelection = (
  primary: typeof output,
): { selection: unknown; target: unknown } => {
  const shared = {
    cameraOverlay: defaultCameraOverlay(),
    canPreviewBakedCamera: false,
    canvasTool: "select" as const,
    effectiveRecordingOutput: { ...recording, primary },
    isFramed: true,
    previewSourceDimensions: { primary: source },
    selectedVideoTracks: new Set(["primary" as const]),
  };
  return {
    selection: recordingVideoSelectionOverlay({
      ...shared,
      activeVideoTrack: "primary",
    })?.rect,
    target: recordingVideoSelectionTargets(shared)?.[0]?.rect,
  };
};

describe("a scene that pads its screen", () => {
  it("pads the crop outwards round its box, as arrange.rs does", () => {
    const bare = arranged(zoom());
    const padded = arranged(zoom(0.1));
    if (!bare || !padded) throw new Error("the scene places the screen");
    const box = bare.screen.box;
    const pad = 0.1 * Math.min(box.width, box.height);
    expect(padded.screen.box).toEqual(box);
    expect(padded.output.cropX).toBeCloseTo(box.x - pad);
    expect(padded.output.cropY).toBeCloseTo(box.y - pad);
    expect(padded.output.cropWidth).toBeCloseTo(box.width + pad * 2);
    expect(padded.output.cropHeight).toBeCloseTo(box.height + pad * 2);
    // The picture stays where the box puts it.
    expect(padded.output.imageX).toBeCloseTo(bare.output.imageX);
    expect(padded.output.imageWidth).toBeCloseTo(bare.output.imageWidth);
  });

  it("eases its padding in with its arrival", () => {
    const settled = arranged(zoom(0.1));
    const halfway = arranged(zoom(0.1), 1_300);
    if (!settled || !halfway) throw new Error("the scene places the screen");
    expect((halfway.screen.box.x - halfway.output.cropX) * 2).toBeCloseTo(
      settled.screen.box.x - settled.output.cropX,
    );
  });

  it("hands the hit target the padded box the selection is drawn on", () => {
    const scene = arranged(zoom(0.1));
    if (!scene) throw new Error("the scene places the screen");
    const { selection, target } = targetAndSelection(scene.output);
    const canvas = screenshotOutputDimensions(output);
    expect(target).toEqual(selection);
    expect(target).toEqual({
      height: scene.output.cropHeight / canvas.height,
      width: scene.output.cropWidth / canvas.width,
      x: scene.output.cropX / canvas.width,
      y: scene.output.cropY / canvas.height,
    });
  });

  it("keeps the recording's own padding inside a scene that adds none", () => {
    const scene = arranged(zoom());
    if (!scene) throw new Error("the scene places the screen");
    const { selection, target } = targetAndSelection(scene.output);
    expect(target).toEqual(selection);
    // The full scene's box is the recording's padded crop, zoomed in place.
    expect(scene.screen.box.width / scene.screen.box.height).toBeCloseTo(
      output.cropWidth / output.cropHeight,
    );
  });

  it("resizes its box so the padded corner a gesture holds stays put", () => {
    const canvas = { height: 900, width: 1600 };
    const box = { height: 0.4, width: 0.3, x: 0.2, y: 0.3 };
    const clip: RecordingSceneClip = {
      ...zoom(0.1),
      boxes: { screen: box },
    };
    const padded = (screen: typeof box) => {
      const inset =
        0.1 *
        Math.min(screen.width * canvas.width, screen.height * canvas.height);
      return {
        x: screen.x - inset / canvas.width,
        y: screen.y - inset / canvas.height,
      };
    };
    // Dragging the bottom-right handle doubles the padded box about its
    // top-left corner, which the gesture reports as not moving.
    const event: RecordingSelectionGestureEvent = {
      deltaX: 0,
      deltaY: 0,
      edges: 0,
      operation: "resize",
      paneIndex: 0,
      phase: "update",
      scale: 2,
    };
    const edit = boxEdit({ screen: box }, "screen", canvas);
    const screen = edit(clip, event)?.boxes.screen;
    if (!screen) throw new Error("the gesture resizes the box");
    expect(screen.width).toBeCloseTo(0.6);
    expect(padded(screen).x).toBeCloseTo(padded(box).x);
    expect(padded(screen).y).toBeCloseTo(padded(box).y);
    // A move carries the box as far as the padded box travels.
    const moved = edit(clip, {
      ...event,
      deltaX: 0.1,
      operation: "move",
      scale: 1,
    });
    expect(moved?.boxes.screen.x).toBeCloseTo(0.3);
  });
});
