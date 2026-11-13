// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { type OpenverseFound } from "./openverse-api";

// One sound found online heard at a time, before it is added, streamed from
// where it is published: the files arrive slowly (a song's preview can take
// ten seconds whole), but its first moments in a fraction of one, so it
// plays from those rather than waiting for all of it. Playing another stops
// the one before.

export function useFoundPreview() {
  const audioRef = useRef<HTMLAudioElement | null>(null);
  const [playingId, setPlayingId] = useState<string | null>(null);
  // Read by `stopIf`, which runs once a download settles, after renders the
  // caller's closure has not seen.
  const playingIdRef = useRef<string | null>(null);
  playingIdRef.current = playingId;
  const stop = () => {
    audioRef.current?.pause();
    audioRef.current = null;
    setPlayingId(null);
  };
  useEffect(
    () => () => {
      audioRef.current?.pause();
    },
    [],
  );
  return {
    isPlaying: (found: OpenverseFound) => playingId === found.id,
    /** Stops `found` where it is the one playing. */
    stopIf: (found: OpenverseFound) => {
      if (playingIdRef.current === found.id) stop();
    },
    /** Plays `found`, or stops it where it is the one playing. */
    toggle: (found: OpenverseFound) => {
      const wasPlaying = playingId === found.id;
      stop();
      if (wasPlaying) return;
      const audio = new Audio(found.url);
      audioRef.current = audio;
      audio.addEventListener("ended", () => {
        if (audioRef.current === audio) stop();
      });
      setPlayingId(found.id);
      audio.play().catch((cause: unknown) => {
        console.error("Could not play the sound found online", cause);
        if (audioRef.current === audio) stop();
      });
    },
  };
}
