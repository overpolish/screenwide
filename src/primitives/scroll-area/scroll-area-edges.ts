// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

export function getEdgeOpacities(
  position: number,
  maximum: number,
  { effect }: { effect: "shadow" | "none" },
) {
  if (maximum <= 0 || effect === "none") return { end: 0, start: 0 };
  const offset = Math.max(0, Math.min(maximum, position));
  return {
    end: (maximum - offset) / maximum,
    start: offset / maximum,
  };
}
