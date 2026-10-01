// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { formatDuration } from "../duration";

import { Playhead } from "./scrub-playhead";

/** The elapsed half of the time readout, written straight to the text node. */
export function ElapsedTime({ playhead }: { playhead: Playhead }) {
  const ref = useRef<HTMLSpanElement>(null);

  useEffect(
    () =>
      playhead.subscribe((seconds) => {
        const text = formatDuration(seconds * 1000);
        if (ref.current && ref.current.textContent !== text)
          ref.current.textContent = text;
      }),
    [playhead],
  );

  return <span ref={ref}>{formatDuration(0)}</span>;
}
