// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

/** Seconds since mount, or since `resetKey` last changed, ticking every
 * frame. The stages' animations are read from it, so each loops in step
 * with what it shows. */
export function useClock(resetKey = "") {
  const [time, setTime] = useState(0);
  useEffect(() => {
    let frame = 0;
    const start = performance.now();
    const tick = (now: number) => {
      // A frame's timestamp is when the frame began, which can be just
      // before the effect ran.
      setTime(Math.max(0, (now - start) / 1000));
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [resetKey]);
  return time;
}
