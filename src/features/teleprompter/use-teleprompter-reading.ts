// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";

import {
  startTeleprompterVoice,
  stopTeleprompterVoice,
  TELEPROMPTER_VOICE_EVENT,
  TeleprompterVoice,
  TeleprompterVoiceProblem,
} from "./api";
import { TeleprompterMode } from "./teleprompter-preferences";
import { followedPlace } from "./voice-follow";

/** Where the voice stands: not asked for, loading its model, heard, or why
 * it cannot be. */
export type VoiceState =
  | { kind: "idle" | "listening" | "loading" }
  | { kind: "problem"; problem: TeleprompterVoiceProblem };

const isVoiceProblem = (value: unknown): value is TeleprompterVoiceProblem =>
  value === "noModel" || value === "noMicrophone" || value === "failed";

/**
 * The place being read, as the index of the next word, fractional between
 * words: moved on at `wordsPerMinute` while playing steadily, or to the
 * word after the last ones heard while following the voice. Where the
 * voice cannot be followed it says why, and the place moves steadily
 * meanwhile. Pausing stops listening; the place stays. Read to its end,
 * the script says so (`onEnd`), for the reading to stop there.
 */
export function useTeleprompterReading({
  hasModel,
  isPlaying,
  mode,
  onEnd,
  spoken,
  wordsPerMinute,
}: {
  /** Whether dictation's model is downloaded. */
  hasModel: boolean;
  isPlaying: boolean;
  mode: TeleprompterMode;
  /** The last word has been read. */
  onEnd: () => void;
  /** The script's words, in their spoken form. */
  spoken: readonly string[];
  wordsPerMinute: number;
}) {
  const [place, setPlace] = useState(0);
  // The place as it moves, read by the loops that move it on.
  const placeRef = useRef(0);
  // What the voice last answered while listening; until it does, its model
  // is loading.
  const [answer, setAnswer] = useState<VoiceState | null>(null);
  const spokenRef = useRef(spoken);
  spokenRef.current = spoken;
  const onEndRef = useRef(onEnd);
  onEndRef.current = onEnd;
  const listens = isPlaying && mode === "voice";
  const voice: VoiceState = listens
    ? (answer ?? { kind: "loading" })
    : { kind: "idle" };
  const isSteady = isPlaying && (mode === "steady" || voice.kind === "problem");

  /** Moves the place on as it is read, telling `onEnd` once the last word
   * has been. */
  const moveTo = (next: number) => {
    placeRef.current = next;
    setPlace(next);
    if (next >= spokenRef.current.length) onEndRef.current();
  };

  useEffect(() => {
    if (!listens) return;
    let disposed = false;
    const unlistening = listen<TeleprompterVoice>(
      TELEPROMPTER_VOICE_EVENT,
      ({ payload }) => {
        if (disposed) return;
        if (payload.kind === "ready") setAnswer({ kind: "listening" });
        else if (payload.kind === "failed")
          setAnswer({ kind: "problem", problem: "failed" });
        else {
          setAnswer({ kind: "listening" });
          moveTo(
            followedPlace(
              spokenRef.current,
              payload.words,
              Math.floor(placeRef.current),
            ),
          );
        }
      },
    );
    startTeleprompterVoice().catch((problem: unknown) => {
      if (!disposed)
        setAnswer({
          kind: "problem",
          problem: isVoiceProblem(problem) ? problem : "failed",
        });
    });
    return () => {
      disposed = true;
      setAnswer(null);
      void unlistening.then((unlisten) => {
        unlisten();
      });
      void stopTeleprompterVoice();
    };
    // A model that has just arrived lets it listen where it could not.
  }, [hasModel, listens]);

  useEffect(() => {
    if (!isSteady) return;
    let frame = 0;
    let last = performance.now();
    const tick = (now: number) => {
      const seconds = (now - last) / 1_000;
      last = now;
      moveTo(
        Math.min(
          spokenRef.current.length,
          placeRef.current + (wordsPerMinute / 60) * seconds,
        ),
      );
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [isSteady, wordsPerMinute]);

  return {
    place,
    /** Puts the place at word `index`, the start being 0. */
    setPlace: (index: number) => {
      const next = Math.max(0, Math.min(spoken.length, index));
      placeRef.current = next;
      setPlace(next);
    },
    voice,
  };
}
