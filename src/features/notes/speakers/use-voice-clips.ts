// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useRef, useState } from "react";

import { readVoiceClip } from "../api";

// The few seconds of each unnamed speaker's voice kept with the notes, so
// whoever took them can hear who a speaker is. One plays at a time; playing
// another, or the same one again, stops the one playing.

/** The number of the speaker the notes label `name`, as their voice clip is
 * kept under, or null for a speaker who has been named. The label is the
 * English the notes are written in. */
export const voiceNumber = (name: string) => {
  const match = /^Speaker (\d+)$/.exec(name);
  return match ? Number(match[1]) : null;
};

export function useVoiceClips(project: string) {
  const [voices, setVoices] = useState<readonly number[]>([]);
  const [playing, setPlaying] = useState<number | null>(null);
  const audioRef = useRef<HTMLAudioElement | null>(null);
  // Counts each clip asked for, so one read after another was asked for,
  // or after a stop, never plays.
  const requestRef = useRef(0);

  const stop = useCallback(() => {
    requestRef.current += 1;
    setPlaying(null);
    const audio = audioRef.current;
    audioRef.current = null;
    if (!audio) return;
    audio.pause();
    URL.revokeObjectURL(audio.src);
  }, []);

  const hear = useCallback(
    (number: number) => {
      const again = playing === number;
      stop();
      if (again) return;
      const request = requestRef.current;
      setPlaying(number);
      readVoiceClip(project, number).then(
        (clip) => {
          if (request !== requestRef.current) return;
          const audio = new Audio(
            URL.createObjectURL(new Blob([clip], { type: "audio/wav" })),
          );
          audioRef.current = audio;
          audio.onended = stop;
          audio.play().catch(stop);
        },
        () => {
          if (request === requestRef.current) setPlaying(null);
        },
      );
    },
    [playing, project, stop],
  );

  useEffect(() => stop, [stop]);

  return { hear, playing, setVoices, voices };
}
