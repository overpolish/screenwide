// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type CropAspect } from "../../island/crop-aspect-choice";
import { cropWithInsetColor } from "../../recenter-inset-channel";
import {
  commitScreenshotCrop,
  resetCommittedScreenshotCrop,
} from "../../screenshot/screenshot-crop";
import {
  fullSourceRect,
  sourceRect,
  SourceRect,
} from "../../screenshot/screenshot-geometry";
import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
  screenshotWorkspaceItemOutput,
  ScreenshotWorkspaceOutputSettings,
} from "../../screenshot/screenshot-output";
import {
  screenshotSourceCrop,
  withScreenshotSourceCrop,
} from "../../screenshot/screenshot-output-settings";
import { EditorArtifact, EditorKind, RecordingVideoTrackId } from "../../types";
import { ToolPanelHandlers } from "../tool-panel-handlers";
import { ToolPanelCrop } from "../tool-panel-store";

import {
  cropAspectKey,
  recordingCropAspectKey,
  sizeOfAspect,
  useCropAspectStore,
} from "./crop-aspect-store";

import type { RecordingSceneControls } from "../../recording/scenes/recording-scene-channel";
import type { ToolPanelScene } from "../tool-panel-scene";

/**
 * The one source rectangle the crop panel acts on: the shape it keeps, and
 * the editor paths that change it.
 *
 * Both are the paths a crop drag already takes - the same
 * `commitScreenshotCrop` a released handle goes through - so a shape chosen
 * in the panel rebases the outer inset frame exactly as the gesture's would,
 * and the panel never has to know whether it is looking at a recording track
 * or a screenshot layer.
 */
export type EditorCropTarget = {
  /** Give the crop a shape to keep, cutting it to that shape about its
   * middle, or let it go free. */
  chooseAspect: (aspect: CropAspect) => void;
  /** Back to the whole source, the committed crop taken away, free of any
   * shape. */
  reset: () => void;
  snapshot: ToolPanelCrop;
};

type CropTargetInputs = {
  artifact: EditorArtifact | null;
  /** The shape each crop keeps, by `cropAspectKey`. */
  aspects: Partial<Record<string, CropAspect>>;
  bakeCamera: boolean;
  /** The video tracks the workspace draws, which leave out a camera saved as
   * a separate file. */
  composedVideoTracks: RecordingVideoTrackId[];
  recordingOutput: RecordingOutputSettings | null | undefined;
  /** The scene under the playhead, where the screen's crop is the scene's
   * own while it shows the screen, and the controls that crop it. */
  scene: ToolPanelScene | null;
  sceneControls:
    Pick<RecordingSceneControls, "cropScreen" | "resetScreenCrop"> | undefined;
  screenshotOutput: ScreenshotWorkspaceOutputSettings | null | undefined;
  selectedScreenshotItemId: number | null;
  selectedTrack: string | null;
  onRecordingOutputChange?: (
    track: RecordingVideoTrackId,
    next: ScreenshotOutputSettings,
  ) => void;
  onScreenshotOutputChange?: (
    next: ScreenshotOutputSettings,
    itemId: number,
  ) => void;
};

const clamp = (value: number, minimum: number, maximum: number) =>
  Math.min(maximum, Math.max(minimum, value));

/**
 * A crop of the asked-for size, around the middle of the one in hand.
 *
 * A size says how much of the source to keep, not which part of it: the
 * rectangle grows and shrinks about its own centre, and slides back inside the
 * source rather than shrinking when that centre is near an edge. A size past
 * the whole source is the whole source; a size below one source pixel is one.
 */
export const centredSourceCrop = (
  current: SourceRect,
  size: { height: number; width: number },
  source: { height: number; width: number },
): SourceRect => {
  const width = clamp(size.width / source.width, 1 / source.width, 1);
  const height = clamp(size.height / source.height, 1 / source.height, 1);
  return sourceRect({
    height,
    width,
    x: clamp(current.x + current.width / 2 - width / 2, 0, 1 - width),
    y: clamp(current.y + current.height / 2 - height / 2, 0, 1 - height),
  });
};

const target = ({
  applyOutput,
  aspect,
  aspectKey,
  settings,
  source,
  workspace,
}: {
  /** Commit a whole output, the path the workspace's own gesture takes. */
  applyOutput: (next: ScreenshotOutputSettings) => void;
  aspect: CropAspect;
  /** Which crop the shape is kept for (`cropAspectKey`). */
  aspectKey: string;
  settings: ScreenshotOutputSettings;
  source: { height: number; width: number };
  /** The workspace whose recenter inset to re-read, or null where a crop
   * there never carries one. */
  workspace: EditorKind | null;
}): EditorCropTarget => {
  const crop = screenshotSourceCrop(settings);
  // A crop changes the colour its padding is filled with, which is read
  // again behind it and kept in the crop's undo step.
  const recrop = (next: SourceRect, output: ScreenshotOutputSettings) => {
    const commit = () => {
      applyOutput(output);
    };
    if (workspace) cropWithInsetColor(workspace, next, commit);
    else commit();
  };
  const apply = (size: { height: number; width: number }) => {
    const next = centredSourceCrop(crop, size, source);
    recrop(
      next,
      commitScreenshotCrop(
        settings,
        withScreenshotSourceCrop(settings, next),
        source,
      ),
    );
  };
  const size = {
    height: Math.round(crop.height * source.height),
    width: Math.round(crop.width * source.width),
  };
  return {
    chooseAspect: (next) => {
      useCropAspectStore.getState().choose(aspectKey, next);
      const shaped = sizeOfAspect(next, size, source);
      if (shaped) apply(shaped);
    },
    reset: () => {
      useCropAspectStore.getState().choose(aspectKey, "free");
      recrop(fullSourceRect(), resetCommittedScreenshotCrop(settings, source));
    },
    // A crop still the whole capture has no drawn shape to lock.
    snapshot: {
      aspect,
      canLock: Math.min(crop.width, crop.height) < 1 - 1e-6,
    },
  };
};

/** The screen's crop in the scene under the playhead: each scene keeps its
 * own, and its own shape. A preset scene has no crop of its own to lock;
 * the first crop makes it custom. */
const sceneCropTarget = (
  aspects: CropTargetInputs["aspects"],
  controls: NonNullable<CropTargetInputs["sceneControls"]>,
  scene: ToolPanelScene & { sceneId: string },
): EditorCropTarget => {
  const aspectKey = recordingCropAspectKey("primary", scene.sceneId);
  return {
    chooseAspect: (next) => {
      useCropAspectStore.getState().choose(aspectKey, next);
      // Locked, the crop keeps the shape it has.
      if (next !== "locked") controls.cropScreen(next);
    },
    reset: () => {
      useCropAspectStore.getState().choose(aspectKey, "free");
      controls.resetScreenCrop();
    },
    snapshot: {
      aspect: aspects[aspectKey] ?? "free",
      canLock: scene.boxes !== null,
    },
  };
};

const recordingCropTarget = (
  inputs: CropTargetInputs,
  artifact: Extract<EditorArtifact, { kind: "recording" }>,
): EditorCropTarget | null => {
  const output = inputs.recordingOutput;
  if (!output) return null;
  const bakedCamera =
    inputs.bakeCamera &&
    inputs.composedVideoTracks.includes("primary") &&
    inputs.composedVideoTracks.includes("camera");
  // The same track a frame edit lands on: a camera drawn as its own pane is
  // cropped in its own output, and a baked one has none of its own to crop.
  const track: RecordingVideoTrackId =
    inputs.selectedTrack === "camera" &&
    inputs.composedVideoTracks.includes("camera") &&
    !bakedCamera &&
    artifact.camera
      ? "camera"
      : "primary";
  const { scene, sceneControls } = inputs;
  if (
    track === "primary" &&
    scene?.hasSceneAtPlayhead &&
    scene.panes.screen &&
    scene.sceneId &&
    sceneControls
  )
    return sceneCropTarget(inputs.aspects, sceneControls, {
      ...scene,
      sceneId: scene.sceneId,
    });
  const source =
    track === "camera" && artifact.camera
      ? { height: artifact.camera.height, width: artifact.camera.width }
      : { height: artifact.height, width: artifact.width };
  const aspectKey = recordingCropAspectKey(track, null);
  return target({
    applyOutput: (next) => {
      inputs.onRecordingOutputChange?.(track, next);
    },
    aspect: inputs.aspects[aspectKey] ?? "free",
    aspectKey,
    settings: output[track],
    source,
    // Only the screen track carries a recenter inset, so only its crop has
    // one to look at again.
    workspace: track === "primary" ? "recording" : null,
  });
};

const screenshotCropTarget = (
  inputs: CropTargetInputs,
  artifact: Extract<EditorArtifact, { kind: "screenshot" }>,
): EditorCropTarget | null => {
  const output = inputs.screenshotOutput;
  if (!output || artifact.items.length === 0) return null;
  // The layer in hand, and the newest one when nothing is selected: the very
  // layer the Crop toolbar button selects before it takes the tool up.
  const item =
    artifact.items.find(
      (candidate) => candidate.id === inputs.selectedScreenshotItemId,
    ) ?? artifact.items[artifact.items.length - 1];
  const aspectKey = cropAspectKey("screenshot", item.id);
  return target({
    applyOutput: (next) => {
      inputs.onScreenshotOutputChange?.(next, item.id);
    },
    aspect: inputs.aspects[aspectKey] ?? "free",
    aspectKey,
    settings: screenshotWorkspaceItemOutput(output, item.id),
    source: { height: item.height, width: item.width },
    workspace: "screenshot",
  });
};

export const editorCropTarget = (
  inputs: CropTargetInputs,
): EditorCropTarget | null => {
  const { artifact } = inputs;
  if (!artifact) return null;
  return artifact.kind === "recording"
    ? recordingCropTarget(inputs, artifact)
    : screenshotCropTarget(inputs, artifact);
};

/** The crop panel's asks, answered against the workspace's own source. */
export const cropPanelHandlers = (
  target: EditorCropTarget | null,
): Pick<ToolPanelHandlers, "onCropAspectChange" | "onCropReset"> => ({
  onCropAspectChange: (aspect) => {
    target?.chooseAspect(aspect);
  },
  onCropReset: () => {
    target?.reset();
  },
});
