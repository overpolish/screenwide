// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * How far the content fades at each end, in px: up to `length` at an end with
 * more beyond it, growing with the first `length` px scrolled from that end
 * so the fade arrives with the scroll rather than all at once.
 */
export function getEdgeFades(
  position: number,
  maximum: number,
  { effect, length }: { effect: "fade" | "none"; length: number },
) {
  if (maximum <= 0 || effect === "none") return { end: 0, start: 0 };
  const offset = Math.max(0, Math.min(maximum, position));
  return {
    end: Math.min(length, maximum - offset),
    start: Math.min(length, offset),
  };
}
