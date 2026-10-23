// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../../lib/styling";
import { boxStyle, elementById } from "../../ideas/ideas-data";
import { MockApp } from "../../ideas/mock-app";
import { H, W } from "../../ideas/point-geometry";

// The recorded screen at a small size: drawn at its full size and scaled, so
// its type and corners shrink with it. `element` outlines what a moment is
// about, in the orange, as a selection on the picture is.

export function Still({
  className,
  element,
  width,
}: {
  width: number;
  className?: string;
  element?: string;
}) {
  const target = elementById(element);
  return (
    <div
      className={cn("relative shrink-0 overflow-hidden", className)}
      style={{ height: (width * H) / W, width }}
    >
      <div
        className="absolute start-0 top-0 origin-top-left"
        style={{
          height: H,
          transform: `scale(${String(width / W)})`,
          width: W,
        }}
      >
        <MockApp className="size-full">
          {target ? (
            <span
              className="absolute rounded-[8px] outline-4 outline-offset-4 outline-emphasis outline-solid"
              style={boxStyle(target)}
            />
          ) : null}
        </MockApp>
      </div>
    </div>
  );
}
