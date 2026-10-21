// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { span, STRETCHES } from "./editor-data";

// Stills along the Picture row, so footage is recognised by what it shows.
// Each stretch either side of the cut gets its own strip, the two meeting
// where the cut closed up; a strip clips the stills that do not fit.

/** One frame of the screen recording, small. */
function ScreenStill({ index }: { index: number }) {
  return (
    <span
      className="relative aspect-video h-full shrink-0 overflow-hidden rounded-[2px]"
      style={{
        background: `linear-gradient(135deg, #f6d365, hsl(${String(20 + (index % 5) * 4)} 95% 72%))`,
      }}
    >
      <span className="absolute inset-[18%_12%_14%] rounded-[1px] bg-white/90">
        <span className="absolute inset-x-0 top-0 h-[16%] bg-black/10" />
        <span
          className="absolute top-[34%] start-[30%] h-[14%] rounded-[1px] bg-black/70"
          style={{ width: `${String(22 + ((index * 7) % 30))}%` }}
        />
      </span>
    </span>
  );
}

/** One frame of the camera, small. Its background is removed, so the
 * person stands on the checks that mean "nothing here". */
function CameraStill() {
  return (
    <span className="relative aspect-video h-full shrink-0 overflow-hidden rounded-[2px] bg-[repeating-conic-gradient(rgb(0_0_0/10%)_0_25%,rgb(255_255_255/60%)_0_50%)] bg-size-[6px_6px]">
      <span className="absolute start-1/2 top-[20%] aspect-square w-[20%] -translate-x-1/2 rounded-full bg-[#e5b796] rtl:translate-x-1/2" />
      <span className="absolute start-1/2 top-[58%] h-[60%] w-[50%] -translate-x-1/2 rounded-t-full bg-[#2563eb] rtl:translate-x-1/2" />
    </span>
  );
}

/** More than the widest stretch can show at the smallest still; the strip
 * clips the rest. */
const STILLS_PER_STRETCH = 60;

export function Filmstrip({ source }: { source: "camera" | "screen" }) {
  return STRETCHES.map(([start, end], stretch) => (
    <span
      className="absolute inset-y-0 flex gap-px overflow-hidden rounded-[3px] bg-fill-strong"
      key={start}
      style={span(start, end)}
    >
      {Array.from({ length: STILLS_PER_STRETCH }, (_, index) =>
        source === "screen" ? (
          <ScreenStill
            index={index + stretch * STILLS_PER_STRETCH}
            // A fixed strip of stills: never reordered.

            key={index}
          />
        ) : (
          <CameraStill key={index} />
        ),
      )}
    </span>
  ));
}
