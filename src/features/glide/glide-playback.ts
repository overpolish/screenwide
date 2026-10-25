// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

// Gestures played out for Glide's stories, the way the native side reports
// them: states as the hand moves, the app's icon on its own beat, and rests
// arming the next gesture. Each plays on a loop.

/** One report, `after` ms after the one before it. */
export type Beat<State> = {
  after: number;
  /** The app's icon has resolved, or a new session has none yet. */
  icon?: boolean;
  /** The hand has rested: the ready breath. */
  rest?: true;
  state?: Partial<State>;
};

/** Plays beats round and round: the state they have reached, whether the
 * icon has resolved, and how many rests they have made. */
export function usePlayback<State>(
  beats: Beat<State>[] | undefined,
  initial: State,
) {
  const [played, setPlayed] = useState({
    icon: false,
    pulse: 0,
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
          pulse: current.pulse + (beat.rest ? 1 : 0),
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
