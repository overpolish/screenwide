// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import {
  recordingTimelineSilenceSummary,
  removeRecordingTimelineSilences,
  restoreRecordingTimelineSilences,
} from "../../timeline/editing/recording-timeline-silences";
import { ToolPanelMicrophone } from "../../tool-panels/tool-panel-store";
import { EditorArtifact } from "../../types";
import { useEditorEditGesture } from "../../use-editor-edit-history";

import {
  getRecordingAutoVolume,
  getRecordingNoiseReduction,
  getRecordingVocalCleanup,
  listenToMicrophoneProgress,
  planRecordingSilences,
  setRecordingAutoVolume,
  setRecordingNoiseReduction,
  setRecordingVocalCleanup,
} from "./microphone-api";
import { useMicrophoneSwitch } from "./use-microphone-switch";

export type MicrophoneTools = {
  /** Bring the microphone to a steady loudness with the system audio making
   * way for it, or play both as recorded. */
  changeAutoVolume: (enabled: boolean) => void;
  /** Clean up the microphone's voice, or put it back as it was. */
  cleanUpVoice: (enabled: boolean) => void;
  microphone: ToolPanelMicrophone;
  /** Take the room's steady noise out of the microphone, or put it back. */
  reduceNoise: (enabled: boolean) => void;
  /** Cut the long pauses out, as one step of the edit history. */
  removeSilences: () => void;
  /** Bring back every pause Remove cut, as one step of the edit history. */
  restoreSilences: () => void;
};

/** How far each tool has got, 0 to 1, for the recording it is working on;
 * `null` until the app says the tool has work to do, which it does not when
 * what it makes is kept from before. */
type Progress = {
  artifactId: number;
  autoVolume: number | null;
  noise: number | null;
  silences: number | null;
  voice: number | null;
};
const noProgress = (artifactId: number): Progress => ({
  artifactId,
  autoVolume: null,
  noise: null,
  silences: null,
  voice: null,
});
/** A switch as the panel shows it: `cleaning` while the app works on a turn
 * it has said has work in it, and while the switch is on and the app works
 * for it on its own: rebuilding Vocal cleanup when Reduce noise turns, or
 * measuring for Auto volume when a recording first opens, until it says the
 * work is done. */
const shown = (
  { isPending, state }: { isPending: boolean; state: "off" | "on" },
  progress: number | null,
) =>
  progress !== null && (isPending || (state === "on" && progress < 1))
    ? ("cleaning" as const)
    : state;
type Silences =
  | { artifactId: number; status: "finding" }
  | { artifactId: number; edit: RecordingTimelineEdit; status: "none-found" };

/**
 * The microphone's speech tools for the recording open, where it has a
 * microphone. Removing silences listens in the background and cuts from the
 * edit as it stands when the listen ends, so editing meanwhile is kept.
 */
export function useMicrophoneTools({
  artifact,
  edit,
  onEditChange,
}: {
  artifact: EditorArtifact | null;
  edit: RecordingTimelineEdit | null | undefined;
  onEditChange?: (edit: RecordingTimelineEdit) => void;
}): MicrophoneTools | null {
  const editGesture = useEditorEditGesture();
  const recording = artifact?.kind === "recording" ? artifact : null;
  const artifactId =
    recording?.audioTracks.some((track) => track.kind === "microphone") === true
      ? recording.id
      : null;
  const noise = useMicrophoneSwitch({
    artifactId,
    name: "noise",
    read: getRecordingNoiseReduction,
    write: setRecordingNoiseReduction,
  });
  const voice = useMicrophoneSwitch({
    artifactId,
    name: "vocal cleanup",
    read: getRecordingVocalCleanup,
    write: setRecordingVocalCleanup,
  });
  const autoVolume = useMicrophoneSwitch({
    artifactId,
    name: "auto volume",
    read: getRecordingAutoVolume,
    write: setRecordingAutoVolume,
  });
  const [silences, setSilences] = useState<Silences | null>(null);
  const [progress, setProgress] = useState<Progress | null>(null);
  const editRef = useRef(edit);
  editRef.current = edit;

  useEffect(() => {
    if (artifactId === null) return;
    let stop: (() => void) | undefined;
    let current = true;
    listenToMicrophoneProgress(({ artifactId: id, fraction, tool }) => {
      if (id !== artifactId) return;
      setProgress((previous) => ({
        ...(previous?.artifactId === artifactId
          ? previous
          : noProgress(artifactId)),
        [tool]: fraction,
      }));
    })
      .then((unlisten) => {
        if (current) stop = unlisten;
        else unlisten();
      })
      .catch((cause: unknown) => {
        console.error("Could not follow the microphone tools", cause);
      });
    return () => {
      current = false;
      stop?.();
    };
  }, [artifactId]);

  if (artifactId === null || !recording || !edit) return null;

  const commit = (next: RecordingTimelineEdit) => {
    editGesture.beginGesture();
    onEditChange?.(next);
    editGesture.endGesture();
  };
  const progressNow: Progress =
    progress?.artifactId === artifactId ? progress : noProgress(artifactId);
  const status =
    silences?.artifactId !== artifactId
      ? "idle"
      : silences.status === "finding"
        ? "finding"
        : silences.edit === edit
          ? "none-found"
          : "idle";

  return {
    changeAutoVolume: (enabled) => {
      setProgress({ ...progressNow, autoVolume: null });
      autoVolume.turn(enabled);
    },
    cleanUpVoice: (enabled) => {
      // As with Reduce noise, the bar waits until the app says there is work.
      setProgress({ ...progressNow, voice: null });
      voice.turn(enabled);
    },
    microphone: {
      autoVolume: shown(autoVolume, progressNow.autoVolume),
      autoVolumeProgress: progressNow.autoVolume ?? 0,
      noise: shown(noise, progressNow.noise),
      noiseProgress: progressNow.noise ?? 0,
      silences: {
        ...recordingTimelineSilenceSummary(edit, recording.durationMs),
        progress: progressNow.silences,
        status,
      },
      voice: shown(voice, progressNow.voice),
      voiceProgress: progressNow.voice ?? 0,
    },
    reduceNoise: (enabled) => {
      // The switch turns at once; the bar waits until the app says there is
      // cleaning to do, so turning on a track cleaned before never shows it.
      setProgress({ ...progressNow, noise: null });
      noise.turn(enabled);
    },
    removeSilences: () => {
      if (!onEditChange) return;
      setSilences({ artifactId, status: "finding" });
      setProgress({ ...progressNow, silences: null });
      planRecordingSilences(artifactId)
        .then((cuts) => {
          const latest = editRef.current;
          if (latest?.artifactId !== artifactId) return;
          const next = removeRecordingTimelineSilences(
            latest,
            cuts,
            recording.durationMs,
          );
          if (next === latest) {
            setSilences({ artifactId, edit: latest, status: "none-found" });
            return;
          }
          setSilences(null);
          commit(next);
        })
        .catch((cause: unknown) => {
          console.error("Could not find the pauses to cut", cause);
          setSilences(null);
        });
    },
    restoreSilences: () => {
      const next = restoreRecordingTimelineSilences(edit);
      if (next !== edit) commit(next);
    },
  };
}
