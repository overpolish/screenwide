// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { springCoil } from "./spring-coil";

/** As tall in its box as Lucide's scissors, so it holds its own beside
 * them. A turn that tall leans less than the strip's, or its loops close
 * up, and two and a half of them fill the width: three arches, as the
 * strip's spring shows. */
const COIL = springCoil({
  coils: 2.5,
  lead: 1,
  lean: 0.2,
  middle: 12,
  radius: 8,
  width: 24,
});

/** A lull that squeezes with the rest, as the strip draws it: the coil of
 * its springs, its far half fainter and finer than its near. Drawn on a
 * Lucide icon's grid with its stroke, and carrying its class, so it takes
 * the sizes set for them. */
export function SpringIcon({ className }: { className?: string }) {
  return (
    <svg
      aria-hidden
      className={className ? `lucide ${className}` : "lucide"}
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      viewBox="0 0 24 24"
    >
      {COIL.end < 23 ? <path d={`M${COIL.end.toFixed(2)} 12H23`} /> : null}
      {COIL.runs.map(({ d, isNear }) => (
        <path
          d={d}
          key={d}
          opacity={isNear ? 1 : 0.45}
          strokeWidth={isNear ? 2 : 1.6}
        />
      ))}
    </svg>
  );
}
