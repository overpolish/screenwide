// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Axis } from "./gradient-field";
import { PixelSize } from "./pixel-analysis";
import { Guide } from "./ruler-types";

/** Below this the chip has nowhere to sit and only adds clutter. */
const MINIMUM_GAP = 4;
/** Above this the pair is not really a rhythm worth labelling. */
const MAXIMUM_SHARE = 0.25;
const LABEL_MARGIN = 24;

export type GuideGap = {
  /** Cross-axis world coordinate the chip parks at. */
  anchor: number;
  axis: Axis;
  centre: number;
  guideIds: readonly [number, number];
  key: string;
  value: number;
};

const axisGaps = ({
  axis,
  guides,
  limit,
}: {
  axis: Axis;
  guides: readonly Guide[];
  limit: number;
}) => {
  const sorted = guides
    .filter((guide) => guide.axis === axis)
    .sort((a, b) => a.position - b.position);
  const gaps: GuideGap[] = [];
  for (let index = 1; index < sorted.length; index += 1) {
    const previous = sorted[index - 1];
    const guide = sorted[index];
    const span = guide.position - previous.position;
    if (span < MINIMUM_GAP || span > limit * MAXIMUM_SHARE) continue;
    gaps.push({
      anchor: (previous.anchor + guide.anchor) / 2,
      axis,
      centre: (previous.position + guide.position) / 2,
      guideIds: [previous.id, guide.id],
      key: `g${axis}:${String(previous.id)}-${String(guide.id)}`,
      // Logical (CSS) px - the app-wide display unit, matching box and probe
      // labels. Never multiply display values by the device scale.
      value: Math.round(span),
    });
  }
  return gaps;
};

/** Adjacent-pair gaps for each axis; the two axes never compare against each other. */
export const guideGaps = ({
  guides,
  viewport,
}: {
  guides: readonly Guide[];
  viewport: PixelSize;
}): readonly GuideGap[] => [
  ...axisGaps({ axis: "x", guides, limit: viewport.width }),
  ...axisGaps({ axis: "y", guides, limit: viewport.height }),
];

const clampLabel = (value: number, limit: number) =>
  Math.min(
    Math.max(value, LABEL_MARGIN),
    Math.max(LABEL_MARGIN, limit - LABEL_MARGIN),
  );

/** World-space centre of a gap's label chip, including its viewport clamp. */
export const guideGapLabelPoint = (gap: GuideGap, viewport: PixelSize) => {
  const across = clampLabel(
    gap.anchor,
    gap.axis === "x" ? viewport.height : viewport.width,
  );
  return gap.axis === "x"
    ? { x: gap.centre, y: across }
    : { x: across, y: gap.centre };
};
