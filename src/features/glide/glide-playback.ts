// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { useEffect, useState } from "react";

// Gestures played out for Glide's stories, the way the native side reports
// them: states as the hand moves and the app's icon on its own beat. Each
// plays on a loop.

/** One report, `after` ms after the one before it. */
export type Beat<State> = {
  after: number;
  /** The app's icon has resolved, or a new session has none yet. */
  icon?: boolean;
  state?: Partial<State>;
};

/** Plays beats round and round: the state they have reached, and whether
 * the icon has resolved. */
export function usePlayback<State>(
  beats: Beat<State>[] | undefined,
  initial: State,
) {
  const [played, setPlayed] = useState({
    icon: false,
    state: initial,
  });

  useEffect(() => {
    if (!beats) return;
    let index = 0;
    let timer = 0;
    const play = () => {
      const beat = beats[index];
      timer = window.setTimeout(() => {
        setPlayed((current) => ({
          icon: beat.icon ?? current.icon,
          state: { ...current.state, ...beat.state },
        }));
        index = (index + 1) % beats.length;
        play();
      }, beat.after);
    };
    play();
    return () => {
      window.clearTimeout(timer);
    };
  }, [beats]);

  return played;
}
