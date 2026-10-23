// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import type { IslandHost } from "./island-window";

type Shown<K> = {
  /** Out as far as the swap goes: false for the frame a new island is laid
   * out hidden before it comes out. */
  isOut: boolean;
  /** Tucked away with the window, so nothing on screen shows the change. */
  isTucked: boolean;
  kind: K;
};

/**
 * Which island the window shows, and the host to hand it. The window shows
 * one at a time: changing what it shows while out tucks the old one away
 * into the notch and brings the new one out of it, as a single island would
 * reshape. Tucked away, it changes at once.
 */
export function useIslandSwap<K extends string>(kind: K, host: IslandHost) {
  const [shown, setShown] = useState<Shown<K>>({
    isOut: true,
    isTucked: !host.isShown,
    kind,
  });

  // Adjusted while rendering rather than in an effect, so the new island is
  // laid out hidden in the same frame and comes out when the window does.
  let next = shown;
  if (next.isTucked && next.kind !== kind)
    next = { isOut: true, isTucked: true, kind };
  if (host.isShown && next.isTucked) next = { ...next, isTucked: false };
  if (next !== shown) setShown(next);

  useEffect(() => {
    if (next.isOut) return;
    // A frame after the new island is laid out hidden, so it is seen coming
    // out rather than appearing.
    const frame = requestAnimationFrame(() => {
      setShown((current) => ({ ...current, isOut: true }));
    });
    return () => {
      cancelAnimationFrame(frame);
    };
  }, [next.isOut]);

  return {
    host: {
      ...host,
      isShown: host.isShown && next.kind === kind && next.isOut,
      onHidden: () => {
        const isSwapping = next.kind !== kind;
        setShown({
          isOut: !isSwapping || !host.isShown,
          isTucked: !host.isShown,
          kind,
        });
        if (!host.isShown) host.onHidden();
      },
    },
    kind: next.kind,
  };
}
