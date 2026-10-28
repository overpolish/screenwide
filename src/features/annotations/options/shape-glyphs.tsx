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
