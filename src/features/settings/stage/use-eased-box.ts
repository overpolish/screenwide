// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { animate, useReducedMotion } from "motion/react";
import { useEffect, useState } from "react";

import { motionDurations, motionEasings } from "../../../lib/motion";

type Box = { height: number; width: number; x: number; y: number };

const mix = (from: Box, to: Box, share: number): Box => ({
  height: from.height + (to.height - from.height) * share,
  width: from.width + (to.width - from.width) * share,
  x: from.x + (to.x - from.x) * share,
  y: from.y + (to.y - from.y) * share,
});

const same = (one: Box, other: Box) =>
  one.height === other.height &&
  one.width === other.width &&
  one.x === other.x &&
  one.y === other.y;

/** `box` eased to each new value from where it was drawn, so a picture
 * that takes it reshapes rather than jumping; at once under reduced
 * motion. */
export function useEasedBox({ height, width, x, y }: Box): Box {
  const reduceMotion = useReducedMotion() === true;
  const target = { height, width, x, y };
  // The way being eased along, from where the box was drawn when it last
  // changed to where it is going, and how far along it the box is drawn.
  const [path, setPath] = useState({ from: target, to: target });
  const [share, setShare] = useState(1);
  if (!same(path.to, target)) {
    // Set while rendering, as React allows for state that follows a prop.
    setPath({ from: mix(path.from, path.to, share), to: target });
    setShare(0);
  }

  useEffect(() => {
    if (reduceMotion || same(path.from, path.to)) return;
    // Each step is an ordinary update, so the browser draws between them.
    const controls = animate(0, 1, {
      duration: motionDurations.reshape,
      ease: motionEasings.out,
      onUpdate: setShare,
    });
    return () => {
      controls.stop();
    };
  }, [path, reduceMotion]);

  return reduceMotion ? target : mix(path.from, path.to, share);
}
