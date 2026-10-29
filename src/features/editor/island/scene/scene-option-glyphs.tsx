// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

import type { SceneCorner } from "../../../../bindings/SceneCorner";

// The layout's choices drawn as what they give: the video as an outline,
// the camera in it as a filled box where it sits and as large as it is.
// Each drawing is the result of picking it, so a side's pair draws the
// camera at the size in use and a size's pair at the side in use. They are
// drawn on a frame of their own, not on the video's shape, since a choice
// of side or size reads the same on any shape and the chosen tile above
// already shows the video as it is. Left, right, top and bottom are the
// video's own, so the drawings never mirror.

/** A frame's outline, in the glyph's 24 units, and the room inside it the
 * camera's box is drawn in, kept off the outline so the two read apart. */
type Frame = {
  inner: { height: number; width: number; x: number; y: number };
  outline: { height: number; width: number; x: number; y: number };
};

const WIDE: Frame = {
  inner: { height: 9, width: 13, x: 5.5, y: 7.5 },
  outline: { height: 14, width: 18, x: 3, y: 5 },
};

const TALL: Frame = {
  inner: { height: 13, width: 9, x: 7.5, y: 5.5 },
  outline: { height: 18, width: 14, x: 5, y: 3 },
};

/** A picture in picture's camera, small or large, in its frame's units. */
const INSET_SIZES = {
  large: { height: 5, width: 7 },
  small: { height: 3.5, width: 4.5 },
};

function GlyphBox({ children, frame }: { children: ReactNode; frame: Frame }) {
  return (
    <svg
      aria-hidden
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      viewBox="0 0 24 24"
    >
      <rect {...frame.outline} rx={2} />
      {children}
    </svg>
  );
}

function CameraBox(box: {
  height: number;
  width: number;
  x: number;
  y: number;
}) {
  return <rect {...box} fill="currentColor" rx={1} stroke="none" />;
}

/** A pair's camera: beside the screen or, `isStacked`, above or below it;
 * `isFirst` on the left or at the top; a third of the video or two. */
export function PairGlyph({
  isFirst,
  isLarge,
  isStacked,
}: {
  isFirst: boolean;
  isLarge: boolean;
  isStacked: boolean;
}) {
  const frame = isStacked ? TALL : WIDE;
  const { inner } = frame;
  const share = isLarge ? 2 / 3 : 1 / 3;
  const box = isStacked
    ? {
        height: inner.height * share,
        width: inner.width,
        x: inner.x,
        y: isFirst ? inner.y : inner.y + inner.height * (1 - share),
      }
    : {
        height: inner.height,
        width: inner.width * share,
        x: isFirst ? inner.x : inner.x + inner.width * (1 - share),
        y: inner.y,
      };
  return (
    <GlyphBox frame={frame}>
      <CameraBox {...box} />
    </GlyphBox>
  );
}

/** A picture in picture's camera, small or large, in its corner. */
export function CornerCameraGlyph({
  corner,
  size,
}: {
  corner: SceneCorner;
  size: "large" | "small";
}) {
  const { inner } = WIDE;
  const { height, width } = INSET_SIZES[size];
  return (
    <GlyphBox frame={WIDE}>
      <CameraBox
        height={height}
        width={width}
        x={corner.endsWith("left") ? inner.x : inner.x + inner.width - width}
        y={corner.startsWith("top") ? inner.y : inner.y + inner.height - height}
      />
    </GlyphBox>
  );
}
