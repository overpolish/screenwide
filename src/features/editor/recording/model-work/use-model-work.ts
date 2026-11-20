// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { TranscriptRequest } from "../../tools/tool-panel-transcript";
import { CameraOverlaySettings, EditorArtifact } from "../../types";
import { useEditorEditGesture } from "../../use-editor-edit-history";
import { CameraEffectsTool } from "../camera/use-camera-effects";
import { MicrophoneTools } from "../microphone/use-microphone-tools";

import {
  cancelModelWork,
  getModelWork,
  listenToModelWork,
} from "./model-work-api";

import type { ModelJob } from "../../../../bindings/ModelJob";
import type { ModelWork } from "../../../../bindings/ModelWork";

type Held = { artifactId: number; jobs: ModelJob[] };

/**
 * The models' work under way on the recording open, as the app lists it,
 * and its Cancel: the work stops, and its tool is turned off the way its
 * switch or the picture would turn it off, so the work is not asked for
 * again. Where the tool lives in the edit (Follow face, Remove background,
 * the camera's objects) that is one step of the edit history, made before
 * the work stops so nothing starts it again meanwhile; a microphone tool's
 * switch is turned once its work has stopped, as turning it waits on the
 * work. Restore voice and Smart ducking keep what they had done, and carry
 * on from there when turned on again.
 */
export function useModelWork({
  artifact,
  cameraEffects,
  cameraOverlay,
  edit,
  microphone,
  onCameraOverlayChange,
  onEditChange,
  onTranscriptRequest,
}: {
  artifact: EditorArtifact | null;
  cameraEffects: CameraEffectsTool | null;
  cameraOverlay: CameraOverlaySettings;
  edit: RecordingTimelineEdit | null | undefined;
  microphone: MicrophoneTools | null;
  onTranscriptRequest: (request: TranscriptRequest) => void;
  onCameraOverlayChange?: (settings: CameraOverlaySettings) => void;
  onEditChange?: (edit: RecordingTimelineEdit) => void;
}) {
  const artifactId = artifact?.kind === "recording" ? artifact.id : null;
  const gesture = useEditorEditGesture();
  const [held, setHeld] = useState<Held | null>(null);
  const editRef = useRef(edit);
  editRef.current = edit;

  useEffect(() => {
    if (artifactId === null) return;
    let stop: (() => void) | undefined;
    let current = true;
    const hold = ({ artifactId: id, jobs }: Held) => {
      if (current && id === artifactId) setHeld({ artifactId, jobs });
    };
    listenToModelWork(hold)
      .then((unlisten) => {
        if (current) stop = unlisten;
        else unlisten();
      })
      .catch((cause: unknown) => {
        console.error("Could not follow the models' work", cause);
      });
    getModelWork(artifactId)
      .then(hold)
      .catch((cause: unknown) => {
        console.error("Could not read the models' work", cause);
      });
    return () => {
      current = false;
      stop?.();
    };
  }, [artifactId]);

  const jobs =
    artifactId !== null && held?.artifactId === artifactId ? held.jobs : [];

  /** Turns off a tool that lives in the edit, as one step of its history. */
  const changeEdit = (change: () => void) => {
    gesture.beginGesture();
    change();
    gesture.endGesture();
  };
  const stopFollowingFace = () => {
    changeEdit(() => {
      if (cameraOverlay.followFace) {
        const { followFace: _followFace, ...still } = cameraOverlay;
        onCameraOverlayChange?.(still);
      }
      const latest = editRef.current;
      if (latest?.sceneClips?.some((clip) => clip.followFace))
        onEditChange?.({
          ...latest,
          sceneClips: latest.sceneClips.map((clip) =>
            clip.followFace ? { ...clip, followFace: false } : clip,
          ),
        });
    });
  };
  const keepBackground = () => {
    const latest = editRef.current;
    if (!latest?.cameraBackgroundRanges) return;
    changeEdit(() => {
      onEditChange?.({ ...latest, cameraBackgroundRanges: undefined });
    });
  };
  // Objects still being followed are the ones the work is for; those
  // followed already are laid into the matte and stay.
  const dropObjectsBeingFollowed = () => {
    const objects = cameraEffects?.objects;
    for (const pick of objects?.shown.picks ?? [])
      if (!pick.followed) objects?.remove(pick.id);
  };
  const turnedOffBefore: Partial<Record<ModelWork, () => void>> = {
    cameraObjects: dropObjectsBeingFollowed,
    enhance: () => {
      cameraEffects?.change({ enhance: false });
    },
    followFace: stopFollowingFace,
    removeBackground: keepBackground,
    transcript: () => {
      onTranscriptRequest({ type: "cancel" });
    },
  };
  const turnedOffAfter: Partial<Record<ModelWork, () => void>> = {
    autoVolume: () => microphone?.changeAutoVolume(false),
    reduceNoise: () => microphone?.reduceNoise(false),
    restoreVoice: () => microphone?.changeRestoreVoice(false),
    smartDucking: () => microphone?.changeSmartDucking(false),
    vocalCleanup: () => microphone?.cleanUpVoice(false),
  };

  return {
    cancel: (work: ModelWork) => {
      if (artifactId === null) return;
      turnedOffBefore[work]?.();
      cancelModelWork(artifactId, work)
        .catch((cause: unknown) => {
          console.error("Could not stop the models' work", cause);
        })
        .finally(() => {
          turnedOffAfter[work]?.();
        });
    },
    jobs,
  };
}
