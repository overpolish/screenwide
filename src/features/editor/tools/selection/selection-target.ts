// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { MicrophoneTools } from "../../recording/microphone/use-microphone-tools";
import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
  screenshotWorkspaceItemOutput,
  ScreenshotWorkspaceOutputSettings,
} from "../../screenshot/screenshot-output";
import {
  AudioTrackVolume,
  CameraOverlaySettings,
  EditorArtifact,
  recordingAudioStreamIndex,
  recordingAudioTrackId,
  RecordingTrackId,
  RecordingVideoTrackId,
} from "../../types";
import { bakedCameraSelectionTarget } from "../camera/baked-camera-target";
import {
  ToolPanelAudioSelection,
  ToolPanelSoundTrack,
} from "../tool-panel-store";

import {
  EditorSelectionTarget,
  placedSelectionTarget,
} from "./placed-selection-target";

export type { EditorSelectionTarget } from "./placed-selection-target";

export type SelectionTargetInputs = {
  artifact: EditorArtifact | null;
  bakeCamera: boolean;
  cameraOverlay: CameraOverlaySettings;
  /** The video tracks the workspace draws, which leave out a camera saved as
   * a separate file. */
  composedVideoTracks: RecordingVideoTrackId[];
  recordingOutput: RecordingOutputSettings | null | undefined;
  screenshotOutput: ScreenshotWorkspaceOutputSettings | null | undefined;
  selectedScreenshotItemId: number | null;
  selectedTrack: string | null;
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
  if (!track || !output || !inputs.composedVideoTracks.includes(track))
    return null;
  // A baked camera is placed by its overlay rather than by an output of its
  // own, so it is read and written there instead.
  if (track === "camera" && inputs.bakeCamera)
    return bakedCameraSelectionTarget({
      artifact,
      cameraOutput: output.camera,
      cameraOverlay: inputs.cameraOverlay,
      canvas: { height: output.primary.height, width: output.primary.width },
      onCameraOverlayChange: inputs.onCameraOverlayChange,
      onRecordingOutputChange: inputs.onRecordingOutputChange,
    });
  const source =
    track === "primary"
      ? { height: artifact.height, width: artifact.width }
      : artifact.camera;
  if (!source) return null;
  return placedSelectionTarget({
    apply: (next) => {
      inputs.onRecordingOutputChange?.(track, next);
    },
    kind: track,
    label:
      track === "primary"
        ? t("editor-island-screen")
        : t("editor-island-camera"),
    settings: output[track],
    source,
    turns: track === "primary",
    // Only the screen track carries a pad, so only it has a colour to fill
    // one with.
    workspace: track === "primary" ? "recording" : null,
  });
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
      artifact.items.length > 1
        ? t("editor-island-layer", { number: index + 1 })
        : t("editor-island-screenshot"),
    settings: screenshotWorkspaceItemOutput(output, item.id),
    source: { height: item.height, width: item.width },
    turns: true,
    workspace: "screenshot",
  });
};

type AudioInputs = {
  artifact: EditorArtifact | null;
  audioTrackVolumes: AudioTrackVolume[];
  microphone?: MicrophoneTools | null;
};

/**
 * The selected audio track, where an audio track is what is selected.
 *
 * An audio track is heard rather than placed, so it is kept apart from the
 * placed layers rather than given no-op placements of its own.
 */
export const editorAudioSelection = ({
  artifact,
  microphone,
  selectedTrack,
}: Pick<AudioInputs, "artifact" | "microphone"> & {
  selectedTrack: string | null;
}): ToolPanelAudioSelection | null => {
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
    kind: "audio",
    label: track.label || t("editor-island-audio"),
    ...(track.kind === "microphone" && microphone
      ? { microphone: microphone.microphone }
      : {}),
  };
};

/** Every audio track of the recording, as the Sound island holds them at
 * once: each at the level the editor holds for its stream, and the
 * microphone with its speech tools. */
export const editorSoundTracks = ({
  artifact,
  audioTrackVolumes,
  microphone,
}: AudioInputs): ToolPanelSoundTrack[] =>
  artifact?.kind === "recording"
    ? artifact.audioTracks.map((track) => ({
        decibels:
          audioTrackVolumes.find(
            (volume) => volume.streamIndex === track.streamIndex,
          )?.decibels ?? 0,
        id: recordingAudioTrackId(track.streamIndex),
        label: track.label || t("editor-island-audio"),
        ...(track.kind === "microphone" && microphone
          ? { microphone: microphone.microphone }
          : {}),
      }))
    : [];

export const editorSelectionTarget = (
  inputs: SelectionTargetInputs,
): EditorSelectionTarget | null => {
  const { artifact } = inputs;
  if (!artifact) return null;
  return artifact.kind === "recording"
    ? recordingSelectionTarget(inputs, artifact)
    : screenshotSelectionTarget(inputs, artifact);
};
