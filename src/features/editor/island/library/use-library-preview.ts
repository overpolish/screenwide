// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import {
  MOST_PREVIEW_MS,
  previewSound,
  type Sound,
} from "../../sound-track/sound-library";
import { useIslandSoundPlayback } from "../../timeline/moments/use-note-playback";

// One sound of the library heard at a time, from where it was asked to
// play, as far as a preview plays, in the island's own window as a clip is
// heard there. Playing another, or the same from elsewhere, stops the one
// before and starts the new one as soon as it has loaded.

/** A sound heard from a moment of it, in ms. */
type Heard = { fromMs: number; sound: Sound };

const lengthOf = ({ fromMs, sound }: Heard) =>
  Math.min(sound.durationMs - fromMs, MOST_PREVIEW_MS);

export function useLibraryPreview() {
  const [heard, setHeard] = useState<Heard | null>(null);
  /** Whether what was just chosen is to play as soon as its hook knows it. */
  const askedRef = useRef(false);
  const playback = useIslandSoundPlayback(
    heard ? `${heard.sound.name}:${String(heard.fromMs)}` : null,
    () =>
      heard
        ? previewSound(heard.sound.name, heard.fromMs, lengthOf(heard))
        : Promise.reject(new Error("No sound is being heard")),
  );
  const { toggle } = playback;
  useEffect(() => {
    if (!askedRef.current) return;
    askedRef.current = false;
    void toggle();
  }, [heard, toggle]);
  const isHeard = (sound: Sound) => heard?.sound.name === sound.name;
  return {
    /** Whether `sound` is playing now. */
    isPlaying: (sound: Sound) => playback.playing && isHeard(sound),
    /** Plays `sound` from `fromMs`, its start if not given. */
    playFrom: (sound: Sound, fromMs = 0) => {
      askedRef.current = true;
      setHeard({
        fromMs: Math.round(Math.min(Math.max(fromMs, 0), sound.durationMs)),
        sound,
      });
    },
    /** How far into `sound` it has played, from 0 to 1. */
    playedOf: (sound: Sound) =>
      heard && isHeard(sound)
        ? (heard.fromMs + playback.progress * lengthOf(heard)) /
          Math.max(sound.durationMs, 1)
        : 0,
    /** Plays `sound` from its start, or stops or carries on with it where it
     * is the one heard. */
    toggle: (sound: Sound) => {
      if (isHeard(sound)) {
        void toggle();
        return;
      }
      askedRef.current = true;
      setHeard({ fromMs: 0, sound });
    },
  };
}
