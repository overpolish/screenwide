// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { AnnotationHead } from "../../../bindings/AnnotationHead";

// Which ends of an arrow carry a head, drawn as the arrow in the text's colour,
// as the tool's icon is. One weight whatever the arrow's own: the weight is
// chosen beside it, and a glyph this small could not show it truly anyway.
// The head keeps the renderer's proportions, four strokes long and as wide,
// so it reads as the same arrow.

/** The arrow runs corner to corner of a 24 unit box, tail at bottom start. */
const TAIL = { x: 4, y: 20 };
const TIP = { x: 20, y: 4 };
const STROKE = 1.75;
const HEAD = STROKE * 4;
/** The renderer rounds a head's corners by this share of the stroke. */
const ROUNDING = STROKE * 0.35;

const LENGTH = Math.hypot(TIP.x - TAIL.x, TIP.y - TAIL.y);
const ALONG = { x: (TIP.x - TAIL.x) / LENGTH, y: (TIP.y - TAIL.y) / LENGTH };
const ANGLE = (Math.atan2(ALONG.y, ALONG.x) * 180) / Math.PI;
/** A unit head pointing along +x from its tip at the origin. */
const HEAD_PATH = `M0 0 L${String(-HEAD)} ${String(-HEAD / 2)} L${String(-HEAD)} ${String(HEAD / 2)} Z`;

export function ArrowHeadGlyph({
  className,
  head,
}: {
  head: AnnotationHead;
  className?: string;
}) {
  const hasEnd = head !== "none";
  const hasStart = head === "both";
  // A headed end of the shaft stops under its head, so the round cap never
  // shows past the head's sides.
  const inset = (hasHead: boolean) => (hasHead ? HEAD * 0.8 : STROKE / 2);
  const drawHead = (point: typeof TIP, angle: number) => (
    <path
      d={HEAD_PATH}
      strokeLinejoin="round"
      strokeWidth={ROUNDING * 2}
      transform={`translate(${String(point.x)} ${String(point.y)}) rotate(${String(angle)})`}
    />
  );
  return (
    <svg
      aria-hidden
      className={className}
      fill="currentColor"
      stroke="currentColor"
      viewBox="0 0 24 24"
    >
      <line
        strokeLinecap="round"
        strokeWidth={STROKE}
        x1={TAIL.x + ALONG.x * inset(hasStart)}
        x2={TIP.x - ALONG.x * inset(hasEnd)}
        y1={TAIL.y + ALONG.y * inset(hasStart)}
        y2={TIP.y - ALONG.y * inset(hasEnd)}
      />
      {hasStart ? drawHead(TAIL, ANGLE + 180) : null}
      {hasEnd ? drawHead(TIP, ANGLE) : null}
    </svg>
  );
}
