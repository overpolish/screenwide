// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

// A level that moves like a voice: bursts of syllables with breaths between,
// or nothing at all for a microphone that hears nothing. 0 to 1.

export function useSimulatedLevel(isSilent: boolean, seed = 0) {
  const [level, setLevel] = useState(0);
  useEffect(() => {
    let frame = 0;
    const tick = (now: number) => {
      const t = now / 1000 + seed * 3.1;
      const phrase = Math.max(Math.sin(t * 0.9) + 0.35, 0);
      const syllables = 0.5 + 0.5 * Math.sin(t * 11) * Math.sin(t * 3.7 + seed);
      setLevel(
        isSilent
          ? 0.02
          : Math.min(phrase * syllables * 1.1 + Math.random() * 0.05, 1),
      );
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [isSilent, seed]);
  return level;
}
