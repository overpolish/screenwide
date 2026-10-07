// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { emit } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";

import { getRecordingMomentNote } from "./recording-moments";

/** Told to the editor when a note starts, so the timeline stops playing
 * under it. */
export const MOMENT_NOTE_PLAYING_EVENT = "moments://note-playing";

type Playing = {
  source: AudioBufferSourceNode;
  /** The audio context time the note's start was, or would have been, heard
   * at, so a resumed note keeps counting from where it stopped. */
  startedAt: number;
};

/**
 * When the audio context's output is actually being heard: an `<audio>`
 * element's own clock starts counting the moment it is asked to play and
 * then waits for the speakers, which runs a progress bar ahead of the sound
 * and stalls it. The output timestamp is the sound itself.
 */
const heardAt = (context: AudioContext) => {
  const { contextTime } = context.getOutputTimestamp();
  // Not every engine reports `outputLatency`; without it, none is assumed.
  return contextTime ?? context.currentTime - (context.outputLatency || 0);
};

/**
 * Plays the `moment`th moment's voice note and reports how far it is, from
 * 0 to 1, by what has reached the speakers rather than what has been sent.
 */
export function useNotePlayback(artifactId: number, moment: number) {
  const contextRef = useRef<AudioContext | null>(null);
  const bufferRef = useRef<AudioBuffer | null>(null);
  const playingRef = useRef<Playing | null>(null);
  const offsetRef = useRef(0);
  const [playing, setPlaying] = useState(false);
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    let cancelled = false;
    const context = new AudioContext();
    contextRef.current = context;
    void getRecordingMomentNote(artifactId, moment)
      .then((bytes) => context.decodeAudioData(bytes))
      .then((buffer) => {
        if (!cancelled) bufferRef.current = buffer;
      })
      .catch((error: unknown) => {
        console.error("Could not load the voice note", error);
      });
    return () => {
      cancelled = true;
      playingRef.current?.source.stop();
      playingRef.current = null;
      bufferRef.current = null;
      offsetRef.current = 0;
      contextRef.current = null;
      void context.close();
    };
  }, [artifactId, moment]);

  useEffect(() => {
    if (!playing) return;
    let frame = 0;
    const step = () => {
      const context = contextRef.current;
      const current = playingRef.current;
      const duration = bufferRef.current?.duration ?? 0;
      if (context && current && duration > 0) {
        const heard = Math.max(0, heardAt(context) - current.startedAt);
        setProgress(Math.min(1, heard / duration));
      }
      frame = requestAnimationFrame(step);
    };
    frame = requestAnimationFrame(step);
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [playing]);

  const toggle = async () => {
    const context = contextRef.current;
    const buffer = bufferRef.current;
    if (!context || !buffer) return;
    const current = playingRef.current;
    if (current) {
      offsetRef.current = Math.max(0, heardAt(context) - current.startedAt);
      playingRef.current = null;
      current.source.stop();
      setPlaying(false);
      return;
    }
    if (offsetRef.current >= buffer.duration) offsetRef.current = 0;
    void emit(MOMENT_NOTE_PLAYING_EVENT);
    await context.resume();
    const source = context.createBufferSource();
    source.buffer = buffer;
    source.connect(context.destination);
    const offset = offsetRef.current;
    const next = { source, startedAt: context.currentTime - offset };
    source.addEventListener("ended", () => {
      // A pause stops the source too; only the note running out is the end.
      if (playingRef.current !== next) return;
      playingRef.current = null;
      offsetRef.current = buffer.duration;
      setProgress(1);
      setPlaying(false);
    });
    playingRef.current = next;
    source.start(0, offset);
    setPlaying(true);
  };

  return { playing, progress, toggle };
}
