// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
  screenshotWorkspaceItemOutput,
  ScreenshotWorkspaceOutputSettings,
} from "../screenshot-output";
import {
  AudioTrackVolume,
  CameraOverlaySettings,
  EditorArtifact,
  recordingAudioStreamIndex,
  RecordingTrackId,
  RecordingVideoTrackId,
} from "../types";

import { bakedCameraSelectionTarget } from "./baked-camera-target";
import {
  EditorSelectionTarget,
  placedSelectionTarget,
} from "./placed-selection-target";
import { ToolPanelHandlers } from "./tool-panel-bridge";
import { ToolPanelAudioSelection } from "./tool-panel-store";

export type { EditorSelectionTarget } from "./placed-selection-target";

type SelectionTargetInputs = {
  artifact: EditorArtifact | null;
  bakeCamera: boolean;
  cameraOverlay: CameraOverlaySettings;
  enabledVideoTracks: RecordingVideoTrackId[];
  recordingOutput: RecordingOutputSettings | null | undefined;
  screenshotOutput: ScreenshotWorkspaceOutputSettings | null | undefined;
  selectedScreenshotItemId: number | null;
  selectedTrack: string | null;
  onBakeCameraChange?: (bake: boolean) => void;
  onCameraOverlayChange?: (settings: CameraOverlaySettings) => void;
  onRecordingOutputChange?: (
    track: RecordingVideoTrackId,
    next: ScreenshotOutputSettings,
  ) => void;
  onScreenshotOutputChange?: (
    next: ScreenshotOutputSettings,
    itemId: number,
  ) => void;
};

const recordingSelectionTarget = (
  inputs: SelectionTargetInputs,
  artifact: Extract<EditorArtifact, { kind: "recording" }>,
): EditorSelectionTarget | null => {
  const track =
    inputs.selectedTrack === "primary" || inputs.selectedTrack === "camera"
      ? inputs.selectedTrack
      : null;
  const output = inputs.recordingOutput;
  if (!track || !output || !inputs.enabledVideoTracks.includes(track))
    return null;
  // Baking draws the camera into the screen's picture, so it takes both
  // tracks; with either one left out the switch has nothing to offer.
  const canBake =
    inputs.enabledVideoTracks.includes("primary") &&
    inputs.enabledVideoTracks.includes("camera") &&
    artifact.camera !== null;
  // A baked camera is placed by its overlay rather than by an output of its
  // own, so it is read and written there instead.
  if (track === "camera" && inputs.bakeCamera)
    return bakedCameraSelectionTarget({
      artifact,
      cameraOutput: output.camera,
      cameraOverlay: inputs.cameraOverlay,
      onBakeCameraChange: inputs.onBakeCameraChange,
      onCameraOverlayChange: inputs.onCameraOverlayChange,
      onRecordingOutputChange: inputs.onRecordingOutputChange,
    });
  const source =
    track === "primary"
      ? { height: artifact.height, width: artifact.width }
      : artifact.camera;
  if (!source) return null;
  const target = placedSelectionTarget({
    apply: (next) => {
      inputs.onRecordingOutputChange?.(track, next);
    },
    kind: track,
    label: track === "primary" ? "Screen" : "Camera",
    settings: output[track],
    source,
    // Only the screen track carries a pad, so only it has a colour to fill
    // one with.
    workspace: track === "primary" ? "recording" : null,
  });
  if (track !== "camera") return target;
  return {
    ...target,
    applyBake: (bake) => {
      inputs.onBakeCameraChange?.(bake);
    },
    selection: { ...target.selection, canBake, isBaked: false },
  };
};

const screenshotSelectionTarget = (
  inputs: SelectionTargetInputs,
  artifact: Extract<EditorArtifact, { kind: "screenshot" }>,
): EditorSelectionTarget | null => {
  const output = inputs.screenshotOutput;
  const index = artifact.items.findIndex(
    (item) => item.id === inputs.selectedScreenshotItemId,
  );
  if (!output || index < 0) return null;
  const item = artifact.items[index];
  // A composite carries no layer names, so a layer is known by where it sits
  // in the stack, counted from the bottom the way the layer actions count it.
  return placedSelectionTarget({
    apply: (next) => {
      inputs.onScreenshotOutputChange?.(next, item.id);
    },
    kind: "layer",
    label:
      artifact.items.length > 1 ? `Layer ${String(index + 1)}` : "Screenshot",
    settings: screenshotWorkspaceItemOutput(output, item.id),
    source: { height: item.height, width: item.width },
    workspace: "screenshot",
  });
};

/**
 * The one audio track the selection panel acts on: what to show for it, and
 * the editor's own way of changing how loud it is played.
 *
 * An audio track is heard rather than placed, so it is kept apart from the
 * placed layers rather than given no-op placements of its own.
 */
export type EditorAudioSelectionTarget = {
  /** Play the track this much louder or quieter than it was recorded. */
  applyVolume: (decibels: number) => void;
  selection: ToolPanelAudioSelection;
};

type AudioSelectionTargetInputs = {
  artifact: EditorArtifact | null;
  audioTrackVolumes: AudioTrackVolume[];
  selectedTrack: string | null;
  onSelectedTrackVolumeChange?: (decibels: number) => void;
};

/**
 * The selected audio track, where an audio track is what is selected.
 *
 * The level is the one the editor holds for this stream, and no entry means
 * the track is played at the level it was recorded at.
 */
export const editorAudioSelectionTarget = ({
  artifact,
  audioTrackVolumes,
  onSelectedTrackVolumeChange,
  selectedTrack,
}: AudioSelectionTargetInputs): EditorAudioSelectionTarget | null => {
  if (artifact?.kind !== "recording") return null;
  // The selection travels as a plain string across the bridge, so the track
  // id is read back the way the editor writes it.
  const streamIndex = recordingAudioStreamIndex(
    selectedTrack as RecordingTrackId | null,
  );
  if (streamIndex === null) return null;
  const track = artifact.audioTracks.find(
    (audio) => audio.streamIndex === streamIndex,
  );
  if (!track) return null;
  return {
    applyVolume: (decibels) => {
      onSelectedTrackVolumeChange?.(decibels);
    },
    selection: {
      decibels:
        audioTrackVolumes.find((volume) => volume.streamIndex === streamIndex)
          ?.decibels ?? 0,
      kind: "audio",
      label: track.label || "Audio",
    },
  };
};

export const editorSelectionTarget = (
  inputs: SelectionTargetInputs,
): EditorSelectionTarget | null => {
  const { artifact } = inputs;
  if (!artifact) return null;
  return artifact.kind === "recording"
    ? recordingSelectionTarget(inputs, artifact)
    : screenshotSelectionTarget(inputs, artifact);
};

/**
 * The selection panel's asks, answered against whatever is selected now.
 *
 * Each goes back out through the editor's own handler for it, so a number
 * typed in the panel takes the identical path through the editor's state as
 * the drag that would have produced it.
 */
export const selectionPanelHandlers = (
  target: EditorSelectionTarget | null,
): Pick<
  ToolPanelHandlers,
  | "onBakeCameraChange"
  | "onSelectionDropShadowChange"
  | "onSelectionInsetChange"
  | "onSelectionPlacementChange"
  | "onSelectionRadiusChange"
  | "onSelectionRecenter"
  | "onSelectionReset"
> => ({
  onBakeCameraChange: (bake) => {
    target?.applyBake(bake);
  },
  onSelectionDropShadowChange: (dropShadow) => {
    target?.applyDropShadow(dropShadow);
  },
  // The pad is the layer's own frame grown past its picture, so it travels
  // with the layer's output the way its size and position do.
  onSelectionInsetChange: (inset) => {
    target?.applyInset(inset);
  },
  onSelectionPlacementChange: (placement) => {
    target?.applyPlacement(placement);
  },
  onSelectionRadiusChange: (radius) => {
    target?.applyRadius(Math.min(50, Math.max(0, radius)));
  },
  onSelectionRecenter: () => {
    target?.recenter();
  },
  onSelectionReset: () => {
    target?.reset();
  },
});

/** The volume the audio selection offers, answered against the selected
 * track. */
export const audioPanelHandlers = (
  target: EditorAudioSelectionTarget | null,
): Pick<ToolPanelHandlers, "onAudioVolumeChange"> => ({
  onAudioVolumeChange: (decibels) => {
    target?.applyVolume(decibels);
  },
});
