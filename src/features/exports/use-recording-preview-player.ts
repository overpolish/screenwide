// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Channel } from "@tauri-apps/api/core";
import { RefObject, useCallback, useEffect, useRef, useState } from "react";

import {
  pauseRecordingPreview,
  seekRecordingPreview,
  selectRecordingPreviewAudio,
  setRecordingPreviewAudioVolumes,
  setRecordingPreviewComposition,
  setRecordingPreviewCursorEffects,
  setRecordingPreviewKeyboardEffects,
  setRecordingPreviewZoom,
  startRecordingPreviewPlayer,
  stopRecordingPreviewPlayer,
} from "./api";
import { ScrubPhase } from "./components/scrub-timeline";
import { playRecordingPreview } from "./recording-preview-playback-api";
import { RecordingPreviewPlayerEvent } from "./recording-preview-player-contract";
import { recordingPreviewSettingsKey } from "./recording-preview-settings-key";
import { RecordingTimelineEdit } from "./recording-timeline-edit";
import {
  recordingTimelinePlaybackRangeFrom,
  recordingTimelinePlaybackRanges,
  recordingTimelinePlaybackRangesFrom,
} from "./recording-timeline-playback";
import {
  AudioTrackVolume,
  CursorEffectSettings,
  KeyboardEffectSettings,
  RecordingPreviewLayout,
} from "./types";
import { useRecordingPreviewSettings } from "./use-recording-preview-settings";
import {
  type RecordingSelectionGestureEvent,
  useRecordingPreviewSurface,
} from "./use-recording-preview-surface";

let sessionSequence = 0;
type PreviewTiming = [durationMs: number, framesPerSecond: number | null];

export function useRecordingPreviewPlayer({
  artifactId,
  audioTrackVolumes,
  bakeCamera,
  cameraCanvasRef,
  cameraOverlay,
  cursorEffects,
  enabledStreamIndices,
  isEditorSuspended,
  isEnabled,
  keyboardEffects,
  nativeEditorOwnsLayout,
  nativeLayoutHasPanes,
  nativeLayoutKey,
  onPosition,
  onSelectionChange,
  onSelectionGesture,
  onZoomChange,
  recordingOutput,
  screenCanvasRef,
  selection,
  selectionTargets,
  timelineEdit,
  zoomPercent,
}: {
  artifactId: number;
  audioTrackVolumes: AudioTrackVolume[];
  bakeCamera: boolean;
  cameraCanvasRef: RefObject<HTMLCanvasElement | null>;
  cameraOverlay: import("./types").CameraOverlaySettings;
  cursorEffects: CursorEffectSettings;
  enabledStreamIndices: number[];
  isEditorSuspended: boolean;
  isEnabled: boolean;
  keyboardEffects: KeyboardEffectSettings;
  nativeEditorOwnsLayout: boolean;
  nativeLayoutHasPanes: boolean;
  nativeLayoutKey: string;
  onPosition: (positionMs: number) => void;
  recordingOutput: import("./screenshot-output").RecordingOutputSettings;
  screenCanvasRef: RefObject<HTMLCanvasElement | null>;
  onSelectionChange?: (paneIndex: number | null) => void;
  onSelectionGesture?: (event: RecordingSelectionGestureEvent) => void;
  onZoomChange?: (zoomPercent: number) => void;
  selection?: {
    paneIndex: number;
    radiusPercent: number;
    rect: { height: number; width: number; x: number; y: number };
    layerId?: number;
  } | null;
  selectionTargets?:
    | {
        paneIndex: number;
        radiusPercent: number;
        rect: { height: number; width: number; x: number; y: number };
        layerId?: number;
      }[]
    | null;
  timelineEdit?: RecordingTimelineEdit | null;
  zoomPercent?: number;
}) {
  const isPlayingRef = useRef(false);
  const wantsPlaybackRef = useRef(false);
  const resumeAfterSeekRef = useRef(false);
  const scrubFinishedRef = useRef(true);
  const onPositionRef = useRef(onPosition);
  const audioTrackVolumesRef = useRef(audioTrackVolumes);
  const cursorEffectsRef = useRef(cursorEffects);
  const keyboardEffectsRef = useRef(keyboardEffects);
  const compositionRef = useRef({ bakeCamera, cameraOverlay, recordingOutput });
  const enabledStreamIndicesRef = useRef(enabledStreamIndices);
  const timingRef = useRef<PreviewTiming>([0, null]);
  const timelineEditRef = useRef(timelineEdit);
  const positionRef = useRef(0);
  const seekRequestRef = useRef(0);
  const lastSentSeekRef = useRef<number | null>(null);
  const pendingScrubFrameRef = useRef<number | null>(null);
  const pendingScrubPositionRef = useRef<number | null>(null);
  const pendingResumeRequestRef = useRef<number | null>(null);
  const settleRequestRef = useRef<number | null>(null);
  const sessionIdRef = useRef(0);
  const startedRef = useRef(false);
  const [durationMs, setDurationMs] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [layout, setLayout] = useState<RecordingPreviewLayout | null>(null);
  const [isPreparing, setIsPreparing] = useState(true);
  const beginPreparing = useCallback(() => {
    // eslint-disable-next-line @eslint-react/set-state-in-effect
    setIsPreparing(true);
  }, []);
  const finishPreparing = useCallback(() => {
    setIsPreparing(false);
  }, []);
  const applyLayout = useCallback((next: RecordingPreviewLayout) => {
    setLayout(next);
    if (next.panes.length === 0) setIsPreparing(false);
  }, []);
  onPositionRef.current = onPosition;
  audioTrackVolumesRef.current = audioTrackVolumes;
  cursorEffectsRef.current = cursorEffects;
  keyboardEffectsRef.current = keyboardEffects;
  compositionRef.current = { bakeCamera, cameraOverlay, recordingOutput };
  enabledStreamIndicesRef.current = enabledStreamIndices;
  timelineEditRef.current = timelineEdit;

  const playbackRanges = useCallback(
    () =>
      recordingTimelinePlaybackRanges(
        timelineEditRef.current,
        timingRef.current[0],
      ),
    [],
  );
  const playbackRangeFrom = useCallback(
    (sourcePositionMs: number) =>
      recordingTimelinePlaybackRangeFrom(playbackRanges(), sourcePositionMs),
    [playbackRanges],
  );
  useRecordingPreviewSettings({
    audioTrackVolumes,
    cursorEffects,
    isEnabled,
    keyboardEffects,
    sessionIdRef,
    setError,
    startedRef,
  });

  useRecordingPreviewSurface({
    bakeCamera,
    cameraCanvasRef,
    cameraOverlay,
    isEditorSuspended,
    isEnabled,
    isPlaying,
    nativeEditorOwnsLayout,
    nativeLayoutHasPanes,
    nativeLayoutKey,
    onError: setError,
    onSelectionChange,
    onSelectionGesture,
    onZoomChange,
    recordingOutput,
    screenCanvasRef,
    selection,
    selectionTargets,
    sessionIdRef,
    startedRef,
    zoomPercent,
  });

  const updatePlaying = (playing: boolean) => {
    isPlayingRef.current = playing;
    setIsPlaying(playing);
  };

  useEffect(() => {
    if (!isEnabled) return;
    let disposed = false;
    const initialSettingsKey = recordingPreviewSettingsKey({
      audioTrackVolumes,
      bakeCamera,
      cameraOverlay,
      cursorEffects,
      enabledStreamIndices,
      keyboardEffects,
      recordingOutput,
    });
    const sessionId = Date.now() * 1_000 + (++sessionSequence % 1_000);
    sessionIdRef.current = sessionId;
    seekRequestRef.current = 0;
    lastSentSeekRef.current = null;
    beginPreparing();
    const eventChannel = new Channel<RecordingPreviewPlayerEvent>();
    eventChannel.onmessage = (event) => {
      if (disposed) return;
      if (event.event === "error") {
        setError(event.data.message);
        finishPreparing();
        return;
      }
      if (event.event === "ended") {
        wantsPlaybackRef.current = false;
        updatePlaying(false);
        positionRef.current = timingRef.current[0];
        onPositionRef.current(timingRef.current[0]);
        void pauseRecordingPreview(sessionIdRef.current).catch(() => undefined);
        return;
      }
      if (event.event === "rangeEnded") {
        positionRef.current = event.data.positionMs;
        onPositionRef.current(event.data.positionMs);
        const ranges = playbackRanges();
        const endedIndex = ranges.findIndex(
          (range) => Math.abs(range.sourceEndMs - event.data.positionMs) < 1,
        );
        if (endedIndex < 0 || endedIndex + 1 >= ranges.length) {
          wantsPlaybackRef.current = false;
          updatePlaying(false);
          void pauseRecordingPreview(sessionIdRef.current).catch(
            () => undefined,
          );
          return;
        }
        const next = ranges[endedIndex + 1];
        positionRef.current = next.sourceStartMs;
        onPositionRef.current(next.sourceStartMs);
        void playRecordingPreview(
          sessionIdRef.current,
          Math.round(next.sourceEndMs),
          { startPositionMs: Math.round(next.sourceStartMs) },
        ).catch((cause: unknown) => {
          wantsPlaybackRef.current = false;
          updatePlaying(false);
          setError(String(cause));
        });
        return;
      }
      if (event.event === "ready") {
        if (event.data.requestId < seekRequestRef.current) return;
        finishPreparing();
        positionRef.current = event.data.positionMs;
        onPositionRef.current(event.data.positionMs);
        if (pendingResumeRequestRef.current === event.data.requestId) {
          pendingResumeRequestRef.current = null;
          settleRequestRef.current = null;
          resumeAfterSeekRef.current = false;
          scrubFinishedRef.current = true;
          wantsPlaybackRef.current = true;
          const { index, ranges } = playbackRangeFrom(event.data.positionMs);
          void playRecordingPreview(
            sessionIdRef.current,
            Math.round(ranges[index]?.sourceEndMs ?? timingRef.current[0]),
            {
              playbackRanges: recordingTimelinePlaybackRangesFrom(
                playbackRanges(),
                event.data.positionMs,
              ),
            },
          ).catch((cause: unknown) => {
            wantsPlaybackRef.current = false;
            updatePlaying(false);
            setError(String(cause));
          });
        } else if (settleRequestRef.current === event.data.requestId) {
          settleRequestRef.current = null;
          scrubFinishedRef.current = true;
        }
        return;
      }
      // The playhead is frontend-driven while a scrub is in progress; stale
      // worker positions must not yank it backwards.
      if (!scrubFinishedRef.current) return;
      if (event.event === "position" && !isPlayingRef.current) return;
      positionRef.current = event.data.positionMs;
      onPositionRef.current(event.data.positionMs);
      if (event.event === "playing") {
        // Starting audio has a short native prebuffer. If Pause won during
        // that interval, its cancelled worker can still report that startup
        // reached Playing; the latest UI intent remains authoritative.
        if (wantsPlaybackRef.current) updatePlaying(true);
      }
      if (event.event === "paused" && !wantsPlaybackRef.current)
        updatePlaying(false);
    };
    void startRecordingPreviewPlayer({
      artifactId,
      audioTrackVolumes,
      bakeCamera,
      cameraOverlay,
      cursorEffects,
      enabledStreamIndices,
      eventChannel,
      keyboardEffects,
      recordingOutput,
      sessionId,
    })
      .then((info) => {
        if (disposed) return;
        applyLayout(info.layout);
        timingRef.current = [info.durationMs, info.framesPerSecond];
        setDurationMs(info.durationMs);
        startedRef.current = true;
        if (
          nativeEditorOwnsLayout &&
          !isEditorSuspended &&
          zoomPercent !== undefined
        ) {
          void setRecordingPreviewZoom(sessionId, zoomPercent).catch(
            (cause: unknown) => {
              if (!disposed) setError(String(cause));
            },
          );
        }
        const latestSettingsKey = recordingPreviewSettingsKey({
          audioTrackVolumes: audioTrackVolumesRef.current,
          bakeCamera: compositionRef.current.bakeCamera,
          cameraOverlay: compositionRef.current.cameraOverlay,
          cursorEffects: cursorEffectsRef.current,
          enabledStreamIndices: enabledStreamIndicesRef.current,
          keyboardEffects: keyboardEffectsRef.current,
          recordingOutput: compositionRef.current.recordingOutput,
        });
        // Startup already installed the exact settings passed above. Repeating
        // them restarts a paused native decoder for cursor and composition,
        // making first-open review needlessly decode its first frame three
        // times. Only catch up when controls genuinely changed while startup
        // was in flight.
        if (latestSettingsKey === initialSettingsKey) return;
        void Promise.all([
          selectRecordingPreviewAudio(
            enabledStreamIndicesRef.current,
            sessionId,
          ),
          setRecordingPreviewAudioVolumes(
            audioTrackVolumesRef.current,
            sessionId,
          ),
          setRecordingPreviewCursorEffects(cursorEffectsRef.current, sessionId),
          setRecordingPreviewKeyboardEffects(
            keyboardEffectsRef.current,
            sessionId,
          ),
          setRecordingPreviewComposition({
            bakeCamera: compositionRef.current.bakeCamera,
            cameraOverlay: compositionRef.current.cameraOverlay,
            recordingOutput: compositionRef.current.recordingOutput,
            sessionId,
          }),
        ]).catch((cause: unknown) => {
          if (!disposed) setError(String(cause));
        });
      })
      .catch((cause: unknown) => {
        if (!disposed) {
          setError(String(cause));
          finishPreparing();
        }
      });
    return () => {
      disposed = true;
      startedRef.current = false;
      lastSentSeekRef.current = null;
      pendingScrubPositionRef.current = null;
      if (pendingScrubFrameRef.current !== null) {
        cancelAnimationFrame(pendingScrubFrameRef.current);
        pendingScrubFrameRef.current = null;
      }
      pendingResumeRequestRef.current = null;
      settleRequestRef.current = null;
      resumeAfterSeekRef.current = false;
      wantsPlaybackRef.current = false;
      scrubFinishedRef.current = true;
      void stopRecordingPreviewPlayer(sessionId).catch(() => undefined);
    };
    // eslint-disable-next-line @eslint-react/exhaustive-deps
  }, [artifactId, isEnabled]);

  const play = useCallback(() => {
    if (!isEnabled) return;
    resumeAfterSeekRef.current = false;
    wantsPlaybackRef.current = true;
    scrubFinishedRef.current = true;
    lastSentSeekRef.current = null;
    setError(null);
    updatePlaying(true);
    void (async () => {
      try {
        const { index, ranges } = playbackRangeFrom(positionRef.current);
        const range = ranges[index];
        if (
          positionRef.current < range.sourceStartMs ||
          positionRef.current >= range.sourceEndMs
        ) {
          positionRef.current = range.sourceStartMs;
          await seekRecordingPreview({
            positionMs: Math.round(range.sourceStartMs),
            requestId: ++seekRequestRef.current,
            sessionId: sessionIdRef.current,
          });
        }
        await playRecordingPreview(
          sessionIdRef.current,
          Math.round(range.sourceEndMs),
          {
            playbackRanges: recordingTimelinePlaybackRangesFrom(
              playbackRanges(),
              positionRef.current,
            ),
          },
        );
      } catch (cause) {
        wantsPlaybackRef.current = false;
        updatePlaying(false);
        setError(String(cause));
      }
    })();
  }, [isEnabled, playbackRangeFrom, playbackRanges]);
  const pause = useCallback(() => {
    if (!isEnabled) return;
    resumeAfterSeekRef.current = false;
    wantsPlaybackRef.current = false;
    pendingResumeRequestRef.current = null;
    settleRequestRef.current = null;
    scrubFinishedRef.current = true;
    lastSentSeekRef.current = null;
    updatePlaying(false);
    void pauseRecordingPreview(sessionIdRef.current).catch((cause: unknown) => {
      setError(String(cause));
    });
  }, [isEnabled]);
  const seek = (positionMs: number, phase: ScrubPhase) => {
    if (!isEnabled) return;
    const normalized = Math.max(0, Math.round(positionMs));
    if (phase === "start") {
      resumeAfterSeekRef.current = isPlayingRef.current;
      wantsPlaybackRef.current = false;
      scrubFinishedRef.current = false;
    }
    positionRef.current = normalized;
    // A seek that will resume playback keeps the UI in its playing state:
    // flipping the button to "play" and revealing the paused chrome for the
    // split second between click, still, and resumed playback reads as a
    // stutter. Internally the backend still pauses and resumes; only the
    // presented state holds steady. A seek from a genuine pause (or a resume
    // that fails - its catch below drops the state) behaves as before.
    if (!resumeAfterSeekRef.current) updatePlaying(false);
    const send = (nextPosition: number, nextPhase: ScrubPhase) => {
      // Start/end also carry native OSC visibility, so only movement samples
      // at the same playhead position are redundant.
      if (nextPosition === lastSentSeekRef.current && nextPhase === "move")
        return;
      lastSentSeekRef.current = nextPosition;
      const requestId = ++seekRequestRef.current;
      if (nextPhase === "end") {
        settleRequestRef.current = requestId;
        if (resumeAfterSeekRef.current)
          pendingResumeRequestRef.current = requestId;
      }
      void seekRecordingPreview({
        positionMs: nextPosition,
        requestId,
        rough: nextPhase !== "end",
        selectionVisible:
          nextPhase === "start"
            ? false
            : nextPhase === "end"
              ? true
              : undefined,
        sessionId: sessionIdRef.current,
      }).catch((cause: unknown) => {
        if (settleRequestRef.current === requestId) {
          settleRequestRef.current = null;
          scrubFinishedRef.current = true;
        }
        setError(String(cause));
      });
    };
    // Raw pointer events can arrive substantially faster than either the
    // display or decoder. Send only the newest position once per display tick
    // so the Tauri command queue cannot build a stale seek backlog.
    if (phase === "move") {
      pendingScrubPositionRef.current = normalized;
      if (pendingScrubFrameRef.current === null) {
        pendingScrubFrameRef.current = requestAnimationFrame(() => {
          pendingScrubFrameRef.current = null;
          const pending = pendingScrubPositionRef.current;
          pendingScrubPositionRef.current = null;
          if (pending !== null) send(pending, "move");
        });
      }
    } else {
      if (pendingScrubFrameRef.current !== null) {
        cancelAnimationFrame(pendingScrubFrameRef.current);
        pendingScrubFrameRef.current = null;
      }
      pendingScrubPositionRef.current = null;
      send(normalized, phase);
    }
    if (phase === "end") {
      scrubFinishedRef.current = false;
    }
  };

  const getPositionMs = useCallback(() => positionRef.current, []);

  return {
    durationMs,
    error,
    framesPerSecond: timingRef.current[1],
    getPositionMs,
    isPlaying,
    isPreparing,
    layout,
    pause,
    play,
    seek,
  };
}
