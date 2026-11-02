// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { isWindows } from "../../../lib/platform";
import { type IslandNotch } from "../../../primitives/island/island-geometry";
import {
  menuBarHeight,
  paintMenuBar,
} from "../preview/scenes/sample-system-bars";

// macOS's menu bar along the top of a picture's screen, painted as the
// recording scenes paint it, around the notch on a Mac with one, so the
// island hangs from the screen it does on this Mac. Windows has no bar at
// the top.

export function StageMenuBar({
  notch,
  width,
}: {
  notch: IslandNotch | null;
  /** The screen's width, in points. */
  width: number;
}) {
  const ref = useRef<HTMLCanvasElement>(null);
  const height = menuBarHeight(notch);

  useEffect(() => {
    const canvas = ref.current;
    const context = canvas?.getContext("2d");
    if (!canvas || !context) return;
    const scale = window.devicePixelRatio;
    canvas.width = Math.round(width * scale);
    canvas.height = Math.round(height * scale);
    context.scale(scale, scale);
    paintMenuBar(context, width, notch);
  }, [height, notch, width]);

  if (isWindows()) return null;
  return (
    <canvas
      className="absolute start-0 top-0"
      ref={ref}
      style={{ height, width }}
    />
  );
}
