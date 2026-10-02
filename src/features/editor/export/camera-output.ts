// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  defaultRecordingOutput,
  RecordingOutputSettings,
} from "../screenshot/screenshot-output";
import { RecordingTimelineEdit } from "../timeline/recording-timeline-types";
import { EditorArtifact, RecordingVideoTrackId } from "../types";

/** How a recording with a screen and a camera is delivered: the camera drawn
 * into one video, or screen and camera saved as separate files. */
export type CameraOutput = "combined" | "separate";

/**
 * The output choice the editor offers, or null where there is none to make.
 * Drawing the camera into the screen's video takes both tracks, so the choice
 * is only offered while neither is left out of the export.
 */
export const cameraOutputChoice = ({
  artifact,
  bakeCamera,
  enabledVideoTracks,
}: {
  artifact: EditorArtifact | null;
  bakeCamera: boolean;
  enabledVideoTracks: readonly RecordingVideoTrackId[];
}): CameraOutput | null =>
  artifact?.kind === "recording" &&
  artifact.camera &&
  enabledVideoTracks.includes("primary") &&
  enabledVideoTracks.includes("camera")
    ? bakeCamera
      ? "combined"
      : "separate"
    : null;

/**
 * The video tracks the workspace draws and edits, out of those kept in the
 * export. With separate files the workspace shows the screen file as it will
 * be saved: the camera is saved beside it untouched, so it is left out of the
 * picture. The camera is its own pane only where it is the whole export.
 */
export const composedVideoTracks = (
  enabledVideoTracks: RecordingVideoTrackId[],
  bakeCamera: boolean,
): RecordingVideoTrackId[] =>
  !bakeCamera &&
  enabledVideoTracks.includes("primary") &&
  enabledVideoTracks.includes("camera")
    ? ["primary"]
    : enabledVideoTracks;

/**
 * What the export is made from. A camera saved as a separate file is the
 * recording's own camera with only the timeline's cuts and speed: its output
 * goes back to the source framing, and annotations drawn on it stay behind,
 * so nothing the workspace does not show is written into it.
 */
export const exportedRecordingEdits = ({
  artifact,
  cameraOutput,
  recordingOutput,
  timelineEdit,
}: {
  artifact: EditorArtifact | null;
  cameraOutput: CameraOutput | null;
  recordingOutput: RecordingOutputSettings;
  timelineEdit: RecordingTimelineEdit | null;
}) => {
  if (
    cameraOutput !== "separate" ||
    artifact?.kind !== "recording" ||
    !artifact.camera
  )
    return { recordingOutput, timelineEdit };
  return {
    recordingOutput: {
      ...recordingOutput,
      camera: defaultRecordingOutput({
        camera: artifact.camera,
        primary: artifact,
      }).camera,
    },
    timelineEdit: timelineEdit && {
      ...timelineEdit,
      annotationClips: timelineEdit.annotationClips?.filter(
        (clip) => clip.trackId !== "camera",
      ),
    },
  };
};
