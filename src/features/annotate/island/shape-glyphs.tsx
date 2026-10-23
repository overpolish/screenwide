// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type ReactNode } from "react";

// A shape's choices drawn as the shape, an outline in the text's colour at
// the icons' stroke, as the tool's icon is. Every glyph draws the same 14
// unit box, so the circles read as one shape changing rather than three
// shapes.

const BOX = { at: 5, side: 14 };

function Glyph({ children }: { children: ReactNode }) {
  return (
    <svg
      aria-hidden
      className="size-icon-lg"
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      viewBox="0 0 24 24"
    >
      {children}
    </svg>
  );
}

/** The box with its corners rounded by `radius` percent of its side, as the
 * renderer rounds a shape's by its shorter side: half is a circle. */
export function CornerGlyph({ radius }: { radius: number }) {
  return (
    <Glyph>
      <rect
        height={BOX.side}
        rx={(BOX.side * radius) / 100}
        width={BOX.side}
        x={BOX.at}
        y={BOX.at}
      />
    </Glyph>
  );
}

/** Clean, the box drawn in one closed lap. By hand, the renderer's stray
 * (`outline/geometry.rs`): the box sits a touch off square, its sides bow a
 * little, and from its start by the top-left corner the pen goes once round
 * and runs on past where it began, lifting as it goes, so the run past is
 * drawn finer and crosses back over the first pass. Deeper than the
 * renderer's stray, so it reads at this size. */
export function ShapeStrokeGlyph({ isHandDrawn }: { isHandDrawn: boolean }) {
  return (
    <Glyph>
      {isHandDrawn ? (
        <g transform="rotate(-3 12 12)">
          <path d="M7.5 5.6C11 5.1 15 4.9 18.2 4.9C19.1 4.9 19.4 5.5 19.4 6.4C19.6 10 19.6 14.5 19.2 17.9C19.1 18.8 18.5 19.2 17.6 19.2C13.5 19.5 9.5 19.6 6.3 19.3C5.3 19.2 4.8 18.6 4.8 17.6C4.5 13.5 4.6 9.5 4.9 6.3C5 4.8 6 4.2 7.5 4.2" />
          <path d="M7.5 4.2C9.2 4.2 10.8 4.6 12.4 5.3" strokeWidth={1.25} />
        </g>
      ) : (
        <rect height={BOX.side} rx={2} width={BOX.side} x={BOX.at} y={BOX.at} />
      )}
    </Glyph>
  );
}
