// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import { useReducedMotion } from "motion/react";
import { useEffect, useRef } from "react";

import { cn } from "../../../../lib/styling";

import type { CaptionPreviewStrip } from "../../../../bindings/CaptionPreviewStrip";

/** Which frame stands for the whole preview where motion is reduced: one a
 * little way in, with a caption up and a word being said. */
const STILL_SHARE = 0.3;

/**
 * A caption style's preview, its frames stepped through one by one at the
 * rate they were drawn at, over and over: the strip the compositor drew,
 * so the tile is true to the export. Held on one frame where motion is
 * reduced.
 */
export function CaptionPreviewStripView({
  strip,
}: {
  strip: CaptionPreviewStrip | undefined;
}) {
  const ref = useRef<HTMLSpanElement>(null);
  const reducedMotion = useReducedMotion() ?? false;
  const frames = strip?.frames ?? 1;
  const framesPerSecond = strip?.framesPerSecond ?? 1;
  useEffect(() => {
    const element = ref.current;
    if (!element || reducedMotion || frames < 2) return;
    // The strip is the frames one above another, so stepping its position
    // from the top to the bottom shows each frame in turn.
    const animation = element.animate(
      [{ backgroundPositionY: "0%" }, { backgroundPositionY: "100%" }],
      {
        duration: (frames / framesPerSecond) * 1_000,
        easing: `steps(${frames.toString()}, jump-none)`,
        iterations: Infinity,
      },
    );
    return () => {
      animation.cancel();
    };
  }, [frames, framesPerSecond, reducedMotion]);
  const still =
    Math.round((frames - 1) * STILL_SHARE) / Math.max(1, frames - 1);
  return (
    <span
      aria-hidden
      // Until its strip is drawn, the tile pulses where it will play.
      className={cn(
        "block aspect-video w-full rounded-control bg-fill-secondary bg-no-repeat",
        !strip && "animate-pulse motion-reduce:animate-none",
      )}
      ref={ref}
      style={
        strip
          ? {
              backgroundImage: `url("${convertFileSrc(strip.path)}")`,
              backgroundPositionY: `${(still * 100).toString()}%`,
              backgroundSize: `100% ${(frames * 100).toString()}%`,
            }
          : undefined
      }
    />
  );
}
