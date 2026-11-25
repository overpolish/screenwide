// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RefObject, useCallback, useRef } from "react";

import { seekRecordingPreview } from "../api";
import { SeekHandler } from "../timeline/timeline-seek";

/** The player's state a seek reads and moves on. */
export type RecordingPreviewSeekRefs = {
  isPlayingRef: RefObject<boolean>;
  lastSentSeekRef: RefObject<number | null>;
  pendingResumeRequestRef: RefObject<number | null>;
  pendingScrubFrameRef: RefObject<number | null>;
  pendingScrubPositionRef: RefObject<Parameters<SeekHandler> | null>;
  positionRef: RefObject<number>;
  resumeAfterSeekRef: RefObject<boolean>;
  scrubFinishedRef: RefObject<boolean>;
  seekRequestRef: RefObject<number>;
  sessionIdRef: RefObject<number>;
  settleRequestRef: RefObject<number | null>;
  startedRef: RefObject<boolean>;
  wantsPlaybackRef: RefObject<boolean>;
};

type HeldSeek = {
  annotationClips: Parameters<SeekHandler>[2];
  positionMs: number;
};

/**
 * Moves the preview's playhead, a scrub sending the newest position once per
 * display tick. A seek asked for while the session is still starting, as the
 * editor puts its playhead back on opening a recording, is held and sent once
 * the session has started (`sendHeldSeek`): the player refuses a seek for a
 * session it does not hold yet, and the refusal would leave the preview on
 * its first frame, still saying it is starting.
 */
export function useRecordingPreviewSeek({
  isEnabled,
  refs,
  setError,
  updatePlaying,
}: {
  isEnabled: boolean;
  refs: RecordingPreviewSeekRefs;
  setError: (error: string) => void;
  updatePlaying: (isPlaying: boolean) => void;
}) {
  const heldRef = useRef<HeldSeek | null>(null);
  const seek: SeekHandler = (positionMs, phase, annotationClips) => {
    if (!isEnabled) return;
    const normalized = Math.max(0, Math.round(positionMs));
    refs.positionRef.current = normalized;
    if (!refs.startedRef.current) {
      heldRef.current = {
        annotationClips: annotationClips ?? heldRef.current?.annotationClips,
        positionMs: normalized,
      };
      return;
    }
    if (phase === "start") {
      refs.resumeAfterSeekRef.current = refs.isPlayingRef.current;
      refs.wantsPlaybackRef.current = false;
      refs.scrubFinishedRef.current = false;
    }
    // Keep the playing controls visible during a seek that will resume.
    // The backend still pauses and resumes, without flashing paused chrome.
    if (!refs.resumeAfterSeekRef.current) updatePlaying(false);
    const send: SeekHandler = (nextPosition, nextPhase, nextClips) => {
      // The end decodes the exact frame and settles the scrub, so only
      // movement samples at the same playhead position are redundant.
      if (
        nextPosition === refs.lastSentSeekRef.current &&
        nextPhase === "move" &&
        !nextClips
      )
        return;
      refs.lastSentSeekRef.current = nextPosition;
      const requestId = ++refs.seekRequestRef.current;
      if (nextPhase === "end") {
        refs.settleRequestRef.current = requestId;
        if (refs.resumeAfterSeekRef.current)
          refs.pendingResumeRequestRef.current = requestId;
      }
      void seekRecordingPreview({
        annotationClips: nextClips,
        positionMs: nextPosition,
        requestId,
        rough: nextPhase !== "end",
        sessionId: refs.sessionIdRef.current,
      }).catch((cause: unknown) => {
        if (refs.settleRequestRef.current === requestId) {
          refs.settleRequestRef.current = null;
          refs.scrubFinishedRef.current = true;
        }
        setError(String(cause));
      });
    };
    // Send the newest pointer position per display tick to avoid a seek backlog.
    if (phase === "move") {
      refs.pendingScrubPositionRef.current = [
        normalized,
        "move",
        annotationClips,
      ];
      if (refs.pendingScrubFrameRef.current === null) {
        refs.pendingScrubFrameRef.current = requestAnimationFrame(() => {
          refs.pendingScrubFrameRef.current = null;
          const pending = refs.pendingScrubPositionRef.current;
          refs.pendingScrubPositionRef.current = null;
          if (pending !== null) send(...pending);
        });
      }
    } else {
      if (refs.pendingScrubFrameRef.current !== null) {
        cancelAnimationFrame(refs.pendingScrubFrameRef.current);
        refs.pendingScrubFrameRef.current = null;
      }
      refs.pendingScrubPositionRef.current = null;
      send(normalized, phase, annotationClips);
    }
    if (phase === "end") {
      refs.scrubFinishedRef.current = false;
    }
  };
  const seekRef = useRef(seek);
  seekRef.current = seek;
  // Called as a session starts: a seek held for an earlier one is not this
  // session's to make.
  const dropHeldSeek = useCallback(() => {
    heldRef.current = null;
  }, []);
  const sendHeldSeek = useCallback(() => {
    const held = heldRef.current;
    heldRef.current = null;
    if (held) seekRef.current(held.positionMs, "end", held.annotationClips);
  }, []);
  return { dropHeldSeek, seek, sendHeldSeek };
}
