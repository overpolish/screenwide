// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { guideGapLabelPoint, guideGaps } from "./guide-gaps";
import { PixelSize, Point } from "./pixel-analysis";
import { Guide } from "./ruler-types";
import { SelectedLine } from "./use-ruler-deletion";

/** Label affected when an artifact line or outline is right-clicked. */
export function labelKeyForLine({
  cursor,
  guides,
  selected,
  viewport,
}: {
  cursor: Point;
  guides: readonly Guide[];
  selected: SelectedLine;
  viewport: PixelSize;
}): string | undefined {
  if (selected.kind === "measurement") return `m${String(selected.id)}`;
  if (selected.kind === "probe") return `p${String(selected.id)}`;
  if (selected.kind === "radius") return `r${String(selected.id)}`;

  const gaps = guideGaps({ guides, viewport }).filter((gap) =>
    gap.guideIds.includes(selected.id),
  );
  let nearest: { distance: number; key: string } | undefined;
  for (const gap of gaps) {
    const point = guideGapLabelPoint(gap, viewport);
    const distance = Math.hypot(point.x - cursor.x, point.y - cursor.y);
    if (!nearest || distance < nearest.distance)
      nearest = { distance, key: gap.key };
  }
  return nearest?.key;
}
