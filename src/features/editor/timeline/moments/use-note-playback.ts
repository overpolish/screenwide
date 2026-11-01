// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { emit, listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";

import { getRecordingMomentNote } from "./recording-moments";

/** Told to the editor when a note starts, so the timeline stops playing
 * under it. */
const MOMENT_NOTE_PLAYING_EVENT = "moments://note-playing";

/** Pauses the timeline whenever a voice note starts playing, so the two are
 * never heard over each other. */
export function usePauseForMomentNotes(pause: () => void) {
  const pauseRef = useRef(pause);
  pauseRef.current = pause;
  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen(MOMENT_NOTE_PLAYING_EVENT, () => {
      pauseRef.current();
    }).then((stop) => {
      if (disposed) stop();
      else unlisten = stop;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
}

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
 * With `moment` null there is no note to play. Playing and progress are
 * kept against the note they are for, so moving on to another note starts
 * it from its beginning, not playing.
 */
export function useNotePlayback(artifactId: number, moment: number | null) {
  const key =
    moment === null ? null : `${String(artifactId)}:${String(moment)}`;
  const contextRef = useRef<AudioContext | null>(null);
  const bufferRef = useRef<AudioBuffer | null>(null);
  const playingRef = useRef<Playing | null>(null);
  const offsetRef = useRef(0);
  const [playingKey, setPlayingKey] = useState<string | null>(null);
  const [heard, setHeard] = useState<{ key: string; progress: number } | null>(
    null,
  );
  const playing = key !== null && playingKey === key;
  const progress = heard?.key === key ? heard.progress : 0;

  useEffect(() => {
    if (moment === null) return;
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
        const since = Math.max(0, heardAt(context) - current.startedAt);
        setHeard({ key, progress: Math.min(1, since / duration) });
      }
      frame = requestAnimationFrame(step);
    };
    frame = requestAnimationFrame(step);
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [key, playing]);

  const toggle = async () => {
    const context = contextRef.current;
    const buffer = bufferRef.current;
    if (!context || !buffer || key === null) return;
    const current = playingRef.current;
    if (current) {
      offsetRef.current = Math.max(0, heardAt(context) - current.startedAt);
      playingRef.current = null;
      current.source.stop();
      setPlayingKey(null);
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
      setHeard({ key, progress: 1 });
      setPlayingKey(null);
    });
    playingRef.current = next;
    source.start(0, offset);
    setPlayingKey(key);
  };

  return { playing, progress, toggle };
}
