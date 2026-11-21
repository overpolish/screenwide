// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { snappedCorners, snappedInset } from "../layer-measure";
import {
  snappedTemplate,
  withSnappedMeasures,
} from "../screenshot/screenshot-measure-snap";
import {
  RecordingOutputSettings,
  reopenedRecordingOutput,
  restoredRecordingOutput,
} from "../screenshot/screenshot-output";
import { RecordingVideoTrackId } from "../types";

import type { RecordingTimelineEdit } from "../../../bindings/RecordingTimelineEdit";
import type { CaptureSource } from "../layer-measure";
import type { EditorArtifact } from "../types";

type RecordingArtifact = Extract<EditorArtifact, { kind: "recording" }>;

/** Each of a recording's pictures as captured: the screen at the display's
 * scale, the camera a point to its pixel. An audio recording has none. */
export const recordingCaptures = (
  artifact: RecordingArtifact,
): Partial<Record<RecordingVideoTrackId, CaptureSource>> => ({
  ...(artifact.primaryKind === "audio"
    ? {}
    : {
        primary: {
          height: artifact.height,
          scaleFactor: Math.max(1, artifact.sourceScalePercent) / 100,
          width: artifact.width,
        },
      }),
  ...(artifact.camera
    ? {
        camera: {
          height: artifact.camera.height,
          scaleFactor: 1,
          width: artifact.camera.width,
        },
      }
    : {}),
});

/** Which looks `track`'s corners are offered: a camera's picture, a camera
 * recorded alone included, may be a circle. */
const looksOf = (
  artifact: RecordingArtifact,
  track: RecordingVideoTrackId,
) => ({
  circle: track === "camera" || artifact.primaryKind !== "screen",
});

/** The look a recording's project saved, its padding and corners snapped
 * to the looks they were nearest (`withSnappedMeasures`). */
const snappedRecordingOutput = (
  saved: RecordingOutputSettings,
  artifact: RecordingArtifact,
): RecordingOutputSettings => {
  const captures = recordingCaptures(artifact);
  const snapped = (track: RecordingVideoTrackId) => {
    const source = captures[track];
    return source
      ? withSnappedMeasures(saved[track], source, looksOf(artifact, track))
      : snappedTemplate(saved[track], looksOf(artifact, track));
  };
  return { camera: snapped("camera"), primary: snapped("primary") };
};

/** The remembered look, for this recording: its padding and corners
 * snapped to the looks this recording's pictures are offered. */
const snappedRecordingTemplate = (
  persisted: Partial<RecordingOutputSettings>,
  artifact: RecordingArtifact,
): Partial<RecordingOutputSettings> => ({
  ...(persisted.camera
    ? { camera: snappedTemplate(persisted.camera, { circle: true }) }
    : {}),
  ...(persisted.primary
    ? {
        primary: snappedTemplate(
          persisted.primary,
          looksOf(artifact, "primary"),
        ),
      }
    : {}),
});

/** A recording's output with each picture's corners a look it is offered,
 * whatever look it was seeded from: a screen seeded from a camera's circle
 * takes the roundest of its own. */
const withRecordingCornerLooks = (
  output: RecordingOutputSettings,
  artifact: RecordingArtifact,
): RecordingOutputSettings => ({
  camera: {
    ...output.camera,
    radius: snappedCorners(output.camera.radius, { circle: true }),
  },
  primary: {
    ...output.primary,
    radius: snappedCorners(output.primary.radius, looksOf(artifact, "primary")),
  },
});

/**
 * The output a recording opens with: a project's own look exactly as it was
 * left, or else the remembered look fitted to this recording. Padding and
 * corners saved in another unit become the looks they were nearest.
 */
export const openedRecordingOutput = (
  artifact: EditorArtifact | null,
  persisted: RecordingOutputSettings | null,
): RecordingOutputSettings => {
  const recording = artifact?.kind === "recording" ? artifact : null;
  const sources = {
    camera: recording?.camera,
    primary: { height: artifact?.height ?? 1, width: artifact?.width ?? 1 },
  };
  if (!recording) return restoredRecordingOutput({ ...sources, persisted });
  const look = recording.projectLook;
  return withRecordingCornerLooks(
    look
      ? reopenedRecordingOutput({
          ...sources,
          saved: snappedRecordingOutput(look.recordingOutput, recording),
        })
      : restoredRecordingOutput({
          ...sources,
          persisted:
            persisted && snappedRecordingTemplate(persisted, recording),
        }),
    recording,
  );
};

/** A project's scenes as it opens: paddings and corners saved in another
 * unit snapped to the looks they were nearest. */
export const withSnappedScenes = (
  edit: RecordingTimelineEdit,
  artifact: RecordingArtifact,
): RecordingTimelineEdit =>
  edit.sceneClips
    ? {
        ...edit,
        sceneClips: edit.sceneClips.map((clip) => ({
          ...clip,
          ...(clip.inset ? { inset: snappedInset(clip.inset) } : {}),
          ...(clip.radius
            ? {
                radius: {
                  ...(clip.radius.screen
                    ? {
                        screen: snappedCorners(
                          clip.radius.screen,
                          looksOf(artifact, "primary"),
                        ),
                      }
                    : {}),
                  ...(clip.radius.camera
                    ? {
                        camera: snappedCorners(clip.radius.camera, {
                          circle: true,
                        }),
                      }
                    : {}),
                },
              }
            : {}),
        })),
      }
    : edit;
