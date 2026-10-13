// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { emitTo } from "@tauri-apps/api/event";
import { useEffect, useMemo, useRef, useState } from "react";

import { t } from "../../../../i18n/i18n";
import { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";
import { toolPanelLabel } from "../../tool-panels/tool-panel-window";
import { EditorArtifact, EditorKind } from "../../types";
import { useEditorEditGesture } from "../../use-editor-edit-history";
import { MicrophoneTools } from "../microphone/use-microphone-tools";
import { useModel } from "../microphone/use-model";
import { recordingSceneControls } from "../scenes/recording-scene-channel";
import { sceneNeedsCamera } from "../scenes/recording-scenes";

import {
  recordingTranscriptAllowedWords,
  recordingTranscriptCanRestore,
  recordingTranscriptCorrectionsMs,
  recordingTranscriptRemovedMs,
} from "./recording-timeline-transcript";
import { useRecordingTranscriptPreviewControls } from "./recording-transcript-channel";
import {
  cancelRecordingTranscription,
  getRecordingTranscript,
  listenToTranscriptProgress,
  TRANSCRIPT_PLAYHEAD_EVENT,
  transcribeRecording,
} from "./transcript-api";
import {
  editForTranscriptRequest,
  isTranscriptEditRequest,
  playedRecordingStretches,
} from "./transcript-edit-requests";
import { useTranscriptRevision } from "./transcript-revision";

import type {
  ToolPanelTranscript,
  TranscriptRequest,
} from "../../tool-panels/tool-panel-transcript";
import type { TranscriptWord } from "../../tool-panels/transcript/transcript-words";

/**
 * The Transcript tool for the recording open, where it has a microphone: the
 * model, whether there are words, and what the edit leaves out. It applies
 * every ask the panel sends, each edit one step of the edit history. Remove
 * silences is the microphone's own, shared through `microphone`, and Restore
 * all brings its cuts back with the words'. The playhead travels to the panel
 * window separately, throttled to the display, off the settings snapshot.
 */
export function useTranscriptTools({
  artifact,
  edit,
  microphone,
  onEditChange,
  workspace,
}: {
  artifact: EditorArtifact | null;
  edit: RecordingTimelineEdit | null | undefined;
  microphone: MicrophoneTools | null;
  workspace: EditorKind;
  onEditChange?: (edit: RecordingTimelineEdit) => void;
}): {
  handle: (request: TranscriptRequest) => void;
  snapshot: ToolPanelTranscript | null;
} {
  const editGesture = useEditorEditGesture();
  const recording = artifact?.kind === "recording" ? artifact : null;
  const hasMicrophone =
    recording?.audioTracks.some((track) => track.kind === "microphone") ===
    true;
  const artifactId = hasMicrophone ? recording.id : null;
  const sourceDurationMs = recording?.durationMs ?? 0;
  const model = useModel("transcripts");
  const preview = useRecordingTranscriptPreviewControls(workspace);

  const [status, setStatus] = useState<ToolPanelTranscript["status"]>("none");
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [revision, setRevision] = useState(0);
  const [words, setWords] = useState<readonly TranscriptWord[]>([]);
  const wordsRef = useRef(words);
  wordsRef.current = words;
  const editRef = useRef(edit);
  editRef.current = edit;
  const cancelledRef = useRef(false);
  const removed = useMemo(
    () => (edit ? recordingTranscriptRemovedMs(edit, sourceDurationMs) : []),
    [edit, sourceDurationMs],
  );
  const corrections = useMemo(
    () =>
      edit ? recordingTranscriptCorrectionsMs(edit, sourceDurationMs) : [],
    [edit, sourceDurationMs],
  );
  const played = useMemo(
    () => (edit ? playedRecordingStretches(edit, sourceDurationMs) : []),
    [edit, sourceDurationMs],
  );
  const sceneClips = edit?.sceneClips;
  const scenes = useMemo(
    () =>
      (sceneClips ?? []).map((clip) => ({
        endMs: clip.endMs,
        id: clip.id,
        needsCamera: sceneNeedsCamera(clip),
        startMs: clip.startMs,
      })),
    [sceneClips],
  );

  // The transcript already made, if any, read once the recording opens. The
  // state is reset here so a new recording never shows the last one's words.
  useEffect(() => {
    if (artifactId === null) return;
    let current = true;
    /* eslint-disable @eslint-react/set-state-in-effect */
    setWords([]);
    setStatus("none");
    setError(null);
    setProgress(0);
    setRevision(0);
    /* eslint-enable @eslint-react/set-state-in-effect */
    getRecordingTranscript(artifactId)
      .then((transcript) => {
        if (!current) return;
        if (transcript) {
          setWords(transcript.words);
          setStatus("ready");
        } else {
          setStatus("none");
        }
      })
      .catch((cause: unknown) => {
        if (!current) return;
        console.error("Could not read the transcript", cause);
        setStatus("none");
      });
    return () => {
      current = false;
    };
  }, [artifactId]);

  // How far transcribing has got, while it works.
  useEffect(() => {
    if (artifactId === null) return;
    let current = true;
    let stop: (() => void) | undefined;
    listenToTranscriptProgress(({ artifactId: id, fraction }) => {
      if (id !== artifactId) return;
      setProgress(fraction);
    })
      .then((unlisten) => {
        if (current) stop = unlisten;
        else unlisten();
      })
      .catch((cause: unknown) => {
        console.error("Could not follow the transcript's progress", cause);
      });
    return () => {
      current = false;
      stop?.();
    };
  }, [artifactId]);

  // The playhead, in source milliseconds, sent to the panel window on its
  // own: once a frame, and only when it moved.
  useEffect(() => {
    if (!preview) return;
    let lastMs = -1;
    let frame: number | null = null;
    const unsubscribe = preview.subscribe((sourceMs) => {
      if (sourceMs === lastMs) return;
      lastMs = sourceMs;
      if (frame !== null) return;
      frame = requestAnimationFrame(() => {
        frame = null;
        void emitTo(toolPanelLabel(workspace), TRANSCRIPT_PLAYHEAD_EVENT, {
          playheadMs: lastMs,
        }).catch(() => undefined);
      });
    });
    return () => {
      unsubscribe();
      if (frame !== null) cancelAnimationFrame(frame);
    };
  }, [preview, workspace]);

  // What is made of the words elsewhere in the window follows them.
  const isReady = status === "ready";
  useEffect(() => {
    useTranscriptRevision.setState({ artifactId, isReady, revision });
  }, [artifactId, isReady, revision]);

  /** One edit to the timeline, committed as one step of the edit history. */
  const commit = (
    change: (edit: RecordingTimelineEdit) => RecordingTimelineEdit,
  ) => {
    const current = editRef.current;
    if (!onEditChange || !current) return;
    const next = change(current);
    if (next === current) return;
    editGesture.beginGesture();
    onEditChange(next);
    editGesture.endGesture();
  };

  const handle = (request: TranscriptRequest) => {
    if (artifactId === null) return;
    if (isTranscriptEditRequest(request)) {
      commit((current) =>
        editForTranscriptRequest(current, request, {
          sourceDurationMs,
          words: wordsRef.current,
        }),
      );
      return;
    }
    switch (request.type) {
      case "download":
        model.download();
        break;
      case "transcribe":
        cancelledRef.current = false;
        setError(null);
        setStatus("transcribing");
        setProgress(0);
        transcribeRecording(artifactId)
          .then((transcript) => {
            setWords(transcript.words);
            setStatus("ready");
            setRevision((current) => current + 1);
          })
          .catch((cause: unknown) => {
            setStatus(wordsRef.current.length > 0 ? "ready" : "none");
            // Stopping on purpose is not a failure to tell anyone about.
            if (cancelledRef.current) return;
            console.error("Could not transcribe the recording", cause);
            setError(t("editor-panels-transcript-failed"));
          });
        break;
      case "cancel":
        cancelledRef.current = true;
        setStatus(wordsRef.current.length > 0 ? "ready" : "none");
        void cancelRecordingTranscription(artifactId).catch(() => undefined);
        break;
      case "remove-silences":
        microphone?.removeSilences();
        break;
      case "seek":
        preview?.seek(request.ms);
        break;
      case "highlight":
        preview?.highlight(request.range);
        break;
      case "scene":
        recordingSceneControls(workspace)?.chooseForSpan(
          request.target,
          request.choice,
        );
        break;
    }
  };

  if (artifactId === null || !recording || !edit) {
    return { handle, snapshot: null };
  }

  return {
    handle,
    snapshot: {
      allowedWords: recordingTranscriptAllowedWords(edit),
      artifactId,
      canRestore:
        recordingTranscriptCanRestore(edit) ||
        (microphone?.microphone.silences.count ?? 0) > 0,
      corrections,
      durationMs: sourceDurationMs,
      error,
      model: model.shown,
      played,
      progress,
      removed,
      revision,
      scenes,
      silences: microphone?.microphone.silences ?? null,
      status,
    },
  };
}
