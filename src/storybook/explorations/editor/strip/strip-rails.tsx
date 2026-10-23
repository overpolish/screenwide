// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../../lib/styling";
import { clock } from "../../ideas/ideas-data";
import { PanelWindow } from "../../shared/panel-window";
import { ACTIONS, type Layout, LAYOUTS, STEPS } from "../editor-data";
import { Still } from "../picture/still";

import { at, placed, type StripScale } from "./strip-scale";

// The thin rails and marks over the beats: where each step starts, the scene
// rail while Scene is open, the playhead, and the frame under the lens. What
// is not being worked on stays out of the way instead of taking a lane.

export function Chapters({ scale }: { scale: StripScale }) {
  return (
    <div className="pointer-events-none absolute inset-0" dir="ltr">
      {STEPS.map((step) => (
        <span
          className="absolute inset-y-0"
          key={step.start}
          style={at(scale, step.start)}
        >
          <span className="absolute top-0 ps-1.5 text-caption whitespace-nowrap text-fg-tertiary">
            {step.title}
          </span>
          {step.start > 0 ? (
            <span className="absolute inset-y-0 start-0 w-px bg-divider" />
          ) : null}
        </span>
      ))}
    </div>
  );
}

const LAYOUT_NAMES = Object.fromEntries(
  LAYOUTS.map(({ id, label }) => [id, label]),
);

export function SceneRail({
  current,
  scale,
  scenes,
}: {
  current: number;
  scale: StripScale;
  scenes: Layout[];
}) {
  return (
    <div className="relative h-4" dir="ltr">
      {STEPS.map((step, index) => (
        <span
          className={cn(
            "absolute inset-y-0 flex items-center overflow-hidden rounded-[5px] border-x border-recess px-1.5 text-caption whitespace-nowrap",
            index === current
              ? "bg-primary text-on-primary"
              : "bg-fill text-fg-secondary",
          )}
          key={step.start}
          style={placed(scale, step.start, step.end)}
        >
          {LAYOUT_NAMES[scenes[index]]}
        </span>
      ))}
    </div>
  );
}

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
      className="pointer-events-none absolute inset-y-0 z-10 w-0.5 -translate-x-1/2 bg-emphasis"
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
        <span className="size-2.5 rounded-full bg-emphasis" />
      </span>
    </span>
  );
}

/** The frame under the lens, above the strip. A native panel in the app, as
 * nothing floats in the page. */
export function LensPreview({
  scale,
  seconds,
}: {
  scale: StripScale;
  seconds: number;
}) {
  const near = ACTIONS.reduce((best, action) =>
    Math.abs(action.at - seconds) < Math.abs(best.at - seconds) ? action : best,
  );
  return (
    <div
      className="pointer-events-none absolute bottom-full z-30 mb-3 -translate-x-1/2"
      style={at(scale, seconds)}
    >
      <PanelWindow className="items-center gap-1 p-1.5">
        <Still
          className="rounded-sm"
          element={Math.abs(near.at - seconds) < 1.5 ? near.element : undefined}
          width={176}
        />
        <span className="text-caption text-fg tabular-nums">
          {clock(seconds)}
        </span>
      </PanelWindow>
    </div>
  );
}
