// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../../lib/styling";
import { PanelWindow } from "../../shared/panel-window";
import { Still } from "../picture/still";
import { type Recording } from "../recording";

import { at, type StripScale } from "./strip-scale";

// The playhead, and the frame under the pointer: thin marks over the
// beats, so what is not being worked on stays out of the way instead of
// taking a lane. Scenes have their own rail (`scene-rail.tsx`).

/** Where you are. Given `onScrub`, its knob can be dragged; its hit area is
 * larger than the dot drawn. */
export function Playhead({
  onScrub,
  scale,
  seconds,
}: {
  scale: StripScale;
  seconds: number;
  onScrub?: (event: React.PointerEvent) => void;
}) {
  return (
    <span
      className="pointer-events-none absolute inset-y-0 z-10 w-0.5 -translate-x-1/2 bg-primary"
      style={at(scale, seconds)}
    >
      <span
        className={cn(
          "absolute -top-3 start-1/2 flex size-6 -translate-x-1/2 items-center justify-center",
          onScrub && "pointer-events-auto cursor-ew-resize touch-none",
        )}
        onPointerDown={(event) => {
          if (!onScrub) return;
          event.stopPropagation();
          event.currentTarget.setPointerCapture(event.pointerId);
          onScrub(event);
        }}
        onPointerMove={(event) => {
          if (onScrub && event.currentTarget.hasPointerCapture(event.pointerId))
            onScrub(event);
        }}
      >
        <span className="size-3 rounded-full bg-primary primary-raised" />
      </span>
    </span>
  );
}

/** The frame under the pointer, above the strip. A native panel in the
 * app, as nothing floats in the page. */
export function MomentPreview({
  recording,
  scale,
  seconds,
}: {
  recording: Recording;
  scale: StripScale;
  seconds: number;
}) {
  const near = recording.actions.find(({ at }) => Math.abs(at - seconds) < 1.5);
  return (
    <div
      className="pointer-events-none absolute bottom-full z-30 mb-3 -translate-x-1/2"
      style={at(scale, seconds)}
    >
      <PanelWindow className="p-1.5">
        <Still
          className="rounded-sm"
          element={near?.element}
          recording={recording}
          seconds={seconds}
          width={176}
        />
      </PanelWindow>
    </div>
  );
}
