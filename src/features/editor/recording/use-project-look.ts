// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { useEffect, useMemo, useRef } from "react";

import {
  normalizedCameraOverlay,
  normalizedCursorEffects,
  normalizedKeyboardEffects,
} from "../preview/preview-settings-normalization";
import { normalizedScreenshotOutput } from "../screenshot/screenshot-output";
import {
  AudioTrackVolume,
  RecordingProjectLook,
  RecordingVideoTrackId,
} from "../types";

/** A choice the window made for one recording; stale once another opens. */
type ForArtifact<T> = { artifactId: number } & T;

const saveRecordingProjectLook = (
  artifactId: number,
  look: RecordingProjectLook,
) =>
  invoke<null>("set_recording_project_look", {
    artifactId,
    look: {
      ...look,
      cameraOverlay: normalizedCameraOverlay(
        look.cameraOverlay,
        look.recordingOutput.primary,
      ),
      cursorEffects: normalizedCursorEffects(look.cursorEffects),
      keyboardEffects: normalizedKeyboardEffects(look.keyboardEffects),
      recordingOutput: {
        camera: normalizedScreenshotOutput(look.recordingOutput.camera),
        primary: normalizedScreenshotOutput(look.recordingOutput.primary),
      },
    },
    // Time orders the saves: Rust keeps whichever is newest, and a project
    // reopened later always saves past what it was left with.
    revision: Date.now(),
  });

/** The track choices a project's look restores for `artifactId`: each null
 * where the look keeps none, which leaves the recording's own default. */
export const seededTrackChoices = (
  artifactId: number | undefined,
  look: RecordingProjectLook | null | undefined,
) => {
  if (artifactId === undefined || !look) {
    return {
      audioTrackVolumes: null,
      trackSelection: null,
      videoTrackSelection: null,
    };
  }
  const {
    audioTrackVolumes: values,
    enabledStreamIndices: streamIndices,
    enabledVideoTracks: tracks,
  } = look;
  return {
    audioTrackVolumes: values ? { artifactId, values } : null,
    trackSelection: streamIndices ? { artifactId, streamIndices } : null,
    videoTrackSelection: tracks ? { artifactId, tracks } : null,
  };
};

/**
 * Saves the open recording's look in its project as it changes, so it
 * reopens as it was left. Saves go one at a time, each the newest look, so a
 * drag writes as fast as the disk takes it rather than once per step, and
 * the last change is written before the window can close. Nothing is saved
 * until the window has seeded the recording, so the placeholder it starts
 * with never lands in a project.
 */
export function useProjectLook({
  audioTrackVolumes,
  bakeCamera,
  cameraOverlay,
  cursorEffects,
  keyboardEffects,
  recordingId,
  recordingOutput,
  trackSelection,
  videoTrackSelection,
}: Omit<
  RecordingProjectLook,
  "audioTrackVolumes" | "enabledStreamIndices" | "enabledVideoTracks"
> & {
  audioTrackVolumes: ForArtifact<{ values: AudioTrackVolume[] }> | null;
  /** The seeded recording, or undefined while there is none. */
  recordingId: number | undefined;
  trackSelection: ForArtifact<{ streamIndices: number[] }> | null;
  videoTrackSelection: ForArtifact<{
    tracks: RecordingVideoTrackId[];
  }> | null;
}) {
  const look = useMemo<RecordingProjectLook | null>(() => {
    if (recordingId === undefined) return null;
    const own = <T extends { artifactId: number }>(choice: T | null) =>
      choice?.artifactId === recordingId ? choice : null;
    return {
      audioTrackVolumes: own(audioTrackVolumes)?.values ?? null,
      bakeCamera,
      cameraOverlay,
      cursorEffects,
      enabledStreamIndices: own(trackSelection)?.streamIndices ?? null,
      enabledVideoTracks: own(videoTrackSelection)?.tracks ?? null,
      keyboardEffects,
      recordingOutput,
    };
  }, [
    audioTrackVolumes,
    bakeCamera,
    cameraOverlay,
    cursorEffects,
    keyboardEffects,
    recordingId,
    recordingOutput,
    trackSelection,
    videoTrackSelection,
  ]);

  const queueRef = useRef<{
    next: { artifactId: number; look: RecordingProjectLook } | null;
    saving: boolean;
  }>({ next: null, saving: false });
  useEffect(() => {
    if (recordingId === undefined || !look) return;
    const queue = queueRef.current;
    queue.next = { artifactId: recordingId, look };
    const pump = () => {
      const next = queue.next;
      if (queue.saving || !next) return;
      queue.next = null;
      queue.saving = true;
      saveRecordingProjectLook(next.artifactId, next.look)
        .catch((cause: unknown) => {
          console.error("Could not save the project's look", cause);
        })
        .finally(() => {
          queue.saving = false;
          pump();
        });
    };
    pump();
  }, [look, recordingId]);
}
