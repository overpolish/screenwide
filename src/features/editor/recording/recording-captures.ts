// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { snappedInset } from "../layer-measure";
import {
  cornersIn,
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

/** Whether `track`'s corners are points of its capture, as a screen's are,
 * or a share of its frame, as a camera's are, a camera recorded alone
 * included. */
export const cornersInPoints = (
  artifact: RecordingArtifact,
  track: RecordingVideoTrackId,
) => track === "primary" && artifact.primaryKind === "screen";

const cornerUnit = (
  artifact: RecordingArtifact,
  track: RecordingVideoTrackId,
) => (cornersInPoints(artifact, track) ? "points" : "percent");

/** The look a recording's project saved, its padding and corners kept in
 * the units each picture takes them in (`withSnappedMeasures`). */
const snappedRecordingOutput = (
  saved: RecordingOutputSettings,
  artifact: RecordingArtifact,
): RecordingOutputSettings => {
  const captures = recordingCaptures(artifact);
  const snapped = (track: RecordingVideoTrackId) => {
    const source = captures[track];
    return source
      ? withSnappedMeasures(saved[track], source, cornerUnit(artifact, track))
      : snappedTemplate(saved[track], cornerUnit(artifact, track));
  };
  return { camera: snapped("camera"), primary: snapped("primary") };
};

/** The remembered look, for this recording: its padding and corners in the
 * units this recording's pictures take them in. */
const snappedRecordingTemplate = (
  persisted: Partial<RecordingOutputSettings>,
  artifact: RecordingArtifact,
): Partial<RecordingOutputSettings> => ({
  ...(persisted.camera
    ? { camera: snappedTemplate(persisted.camera, "percent") }
    : {}),
  ...(persisted.primary
    ? {
        primary: snappedTemplate(
          persisted.primary,
          cornerUnit(artifact, "primary"),
        ),
      }
    : {}),
});

/** A recording's output with each picture's corners in the unit it takes
 * them in, whatever look it was seeded from, so the unit a radius is kept
 * in says what it measures: the screen's in points, a camera's, a camera
 * recorded alone included, in percent. */
const withRecordingCornerUnits = (
  output: RecordingOutputSettings,
  artifact: RecordingArtifact,
): RecordingOutputSettings => ({
  camera: {
    ...output.camera,
    radius: cornersIn(output.camera.radius, "percent"),
  },
  primary: {
    ...output.primary,
    radius: cornersIn(output.primary.radius, cornerUnit(artifact, "primary")),
  },
});

/**
 * The output a recording opens with: a project's own look exactly as it was
 * left, or else the remembered look fitted to this recording. Padding and
 * corners saved as percents become the looks in points they were nearest,
 * and each picture's corners are kept in the unit it takes them in.
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
  return withRecordingCornerUnits(
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

/** A project's scenes as it opens: paddings and the screen's corners saved
 * as percents snapped to the looks in points they were nearest, the screen's
 * corners in the unit the recording's own are in. */
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
          ...(clip.radius?.screen
            ? {
                radius: {
                  ...clip.radius,
                  screen: cornersIn(
                    clip.radius.screen,
                    cornerUnit(artifact, "primary"),
                  ),
                },
              }
            : {}),
        })),
      }
    : edit;
