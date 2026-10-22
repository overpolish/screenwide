// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { animate } from "motion/react";
import { type ReactNode, useEffect, useState } from "react";

// The picture ripples where a mark lands, as water does under a drop: a ring
// of magnified picture travels out from the point and settles. The copy of
// the picture inside the ring is what bends; on top there is only the wave's
// own light, bright on its leading edge and shadowed on its trailing one.

const REACH = 170;
const BAND = 22;

const ringAt = (centre: { x: number; y: number }, stops: string) =>
  `radial-gradient(circle at ${String(centre.x)}px ${String(centre.y)}px, ${stops})`;

export function ScreenRipple({
  centre,
  children,
}: {
  centre: { x: number; y: number };
  children: ReactNode;
}) {
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    const controls = animate(0, 1, {
      duration: 0.95,
      ease: [0.1, 0.55, 0.3, 1],
      onUpdate: setProgress,
    });
    return () => {
      controls.stop();
    };
  }, []);

  if (progress >= 1) return null;
  const radius = progress * REACH;
  const strength = (1 - progress) ** 1.4;
  const px = (value: number) => `${String(Math.max(value, 0))}px`;
  const band = ringAt(
    centre,
    `transparent ${px(radius - BAND)}, #000 ${px(radius)}, transparent ${px(radius + BAND)}`,
  );
  const light = ringAt(
    centre,
    `transparent ${px(radius - BAND)}, rgba(0,0,0,${String(0.16 * strength)}) ${px(radius - BAND * 0.45)}, transparent ${px(radius - 2)}, rgba(255,255,255,${String(0.5 * strength)}) ${px(radius + 3)}, transparent ${px(radius + BAND * 0.6)}`,
  );
  return (
    <div className="pointer-events-none absolute inset-0">
      <div
        className="absolute inset-0"
        style={{ WebkitMaskImage: band, maskImage: band }}
      >
        <div
          className="size-full"
          style={{
            transform: `scale(${String(1 + 0.1 * strength)})`,
            transformOrigin: `${String(centre.x)}px ${String(centre.y)}px`,
          }}
        >
          {children}
        </div>
      </div>
      <div className="absolute inset-0" style={{ background: light }} />
    </div>
  );
}
