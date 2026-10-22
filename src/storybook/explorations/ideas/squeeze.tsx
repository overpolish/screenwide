// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Pin } from "lucide-react";
import { animate } from "motion/react";
import { useEffect, useRef, useState } from "react";

import { cn } from "../../../lib/styling";
import { Waveform } from "../shared/picture";

import { IdeaFrame } from "./idea-frame";
import { clock, RECORDING_S, SEGMENTS } from "./ideas-data";

// Time as a material: speech is rigid and pauses are springs. Squeezing the
// recording takes time out of the pauses, the slackest first, and never out
// of what was said. Past what can go, it resists and springs back.

const PX_PER_S = 19;
/** The shortest a pause gets: a breath, not a jump cut. */
const FLOOR_S = 0.3;

// Apple's rubber band: the further past the edge, the less it follows.
const rubber = (over: number, dimension: number) =>
  (over * dimension * 0.55) / (dimension + 0.55 * Math.abs(over));

function layout(requested: number, pinned: Set<number>) {
  const slack = SEGMENTS.map((segment, index) =>
    segment.kind === "pause" && !pinned.has(index)
      ? Math.max(segment.end - segment.start - FLOOR_S, 0)
      : 0,
  );
  const available = slack.reduce((sum, value) => sum + value, 0);
  const removed = Math.min(Math.max(requested, 0), available);
  const over = Math.max(requested - available, 0);
  // Past the limit, everything squashes a little, speech too, so the
  // strip says there is nothing more to give.
  const squash = 1 - rubber(over, RECORDING_S) / RECORDING_S;
  const durations = SEGMENTS.map((segment, index) => {
    const own = segment.end - segment.start;
    const share = available > 0 ? (slack[index] / available) * removed : 0;
    return (own - share) * squash;
  });
  return { available, durations, removed };
}

/** A pause drawn as bellows: the folds pack tighter as it is squeezed. */
function Bellows({ isPinned, ratio }: { isPinned: boolean; ratio: number }) {
  const folds = 7;
  const points = Array.from({ length: folds * 2 + 1 }, (_, index) => {
    const x = (index / (folds * 2)) * 100;
    const y = index % 2 === 0 ? 30 : 70;
    return `${String(x)},${String(y)}`;
  }).join(" ");
  return (
    <svg
      aria-hidden
      className={cn(
        "size-full",
        isPinned ? "text-fg-secondary" : "text-fg-tertiary",
      )}
      preserveAspectRatio="none"
      viewBox="0 0 100 100"
    >
      {isPinned ? (
        <line
          stroke="currentColor"
          strokeWidth={2}
          vectorEffect="non-scaling-stroke"
          x1={0}
          x2={100}
          y1={50}
          y2={50}
        />
      ) : (
        <polyline
          fill="none"
          points={points}
          stroke="currentColor"
          strokeLinejoin="round"
          strokeWidth={1.5}
          style={{ opacity: 0.45 + 0.55 * (1 - ratio) }}
          vectorEffect="non-scaling-stroke"
        />
      )}
    </svg>
  );
}

export function Squeeze() {
  const [requested, setRequested] = useState(0);
  const [pinned, setPinned] = useState<Set<number>>(() => new Set());
  const dragRef = useRef<{
    from: number;
    samples: { t: number; v: number }[];
    start: number;
  } | null>(null);
  const stripRef = useRef<HTMLDivElement>(null);
  const { available, durations, removed } = layout(requested, pinned);
  const total = durations.reduce((sum, value) => sum + value, 0);

  const settle = (velocity: number) => {
    const target = Math.min(Math.max(requested, 0), available);
    animate(requested, target, {
      bounce: 0.25,
      duration: 0.5,
      onUpdate: setRequested,
      type: "spring",
      velocity,
    });
  };

  // A trackpad pinch squeezes too: it arrives as a wheel with Control held.
  useEffect(() => {
    const element = stripRef.current;
    if (!element) return;
    const onWheel = (event: WheelEvent) => {
      if (!event.ctrlKey) return;
      event.preventDefault();
      setRequested((value) => Math.max(value + event.deltaY * 0.08, -2));
    };
    element.addEventListener("wheel", onWheel, { passive: false });
    return () => {
      element.removeEventListener("wheel", onWheel);
    };
  }, []);

  return (
    <IdeaFrame
      hint="Drag the end of the recording left, or pinch over it. Double-click a pause to keep it as a beat."
      pitch="Speech is rigid, pauses are springs. Squeeze the recording and dead air gives way first, the longest pauses most, and nothing that was said is ever cut."
      title="Squeeze"
    >
      <div
        className="flex w-full max-w-[940px] flex-col gap-separate"
        dir="ltr"
      >
        <div className="flex items-baseline justify-between">
          <span className="text-heading tabular-nums">
            {clock(RECORDING_S)}
            <span className="text-fg-tertiary"> → </span>
            {clock(total)}
          </span>
          <span className="text-caption text-fg-tertiary tabular-nums">
            {removed > 0.05
              ? `${removed.toFixed(1)} s of pauses taken out, speech untouched`
              : "Nothing taken out yet"}
          </span>
        </div>
        <div className="relative h-20" ref={stripRef}>
          {/* The recording's full length, so what has gone stays visible. */}
          <span
            className="absolute inset-y-0 rounded-control border border-dashed border-fg-tertiary/40"
            style={{ width: RECORDING_S * PX_PER_S }}
          />
          <div className="absolute inset-y-0 flex">
            {SEGMENTS.map((segment, index) => {
              const width = durations[index] * PX_PER_S;
              if (segment.kind === "speech") {
                return (
                  <div
                    className="flex h-full shrink-0 flex-col justify-between overflow-hidden rounded-part bg-fill-strong px-1.5 py-1"
                    key={segment.start}
                    style={{ width }}
                  >
                    <span className="truncate text-caption text-fg-secondary">
                      {segment.text}
                    </span>
                    <span className="flex h-8 text-fg-secondary">
                      <Waveform bars={Math.round(width / 4)} seed={index + 2} />
                    </span>
                  </div>
                );
              }
              const own = segment.end - segment.start;
              const isPinned = pinned.has(index);
              return (
                <button
                  aria-label={isPinned ? "Pinned pause" : "Pause"}
                  className="relative h-full shrink-0 cursor-default px-px"
                  key={segment.start}
                  onDoubleClick={() => {
                    setPinned((current) => {
                      const next = new Set(current);
                      if (next.has(index)) next.delete(index);
                      else next.add(index);
                      return next;
                    });
                  }}
                  style={{ width }}
                  type="button"
                >
                  <Bellows isPinned={isPinned} ratio={durations[index] / own} />
                  {isPinned ? (
                    <Pin className="absolute inset-x-0 top-1 mx-auto size-3 text-fg-secondary" />
                  ) : null}
                </button>
              );
            })}
            {/* The end of the recording: the handle that squeezes it. */}
            <span
              className="ms-1 h-full w-2 shrink-0 cursor-ew-resize touch-none rounded-full bg-fg-secondary transition-[width] hover:w-2.5 active:bg-fg"
              onPointerDown={(event) => {
                event.currentTarget.setPointerCapture(event.pointerId);
                dragRef.current = {
                  from: event.clientX,
                  samples: [],
                  start: Math.max(requested, 0),
                };
              }}
              onPointerMove={(event) => {
                const current = dragRef.current;
                if (!current) return;
                const value =
                  current.start + (current.from - event.clientX) / PX_PER_S;
                current.samples = [
                  ...current.samples.slice(-4),
                  { t: event.timeStamp, v: value },
                ];
                setRequested(value);
              }}
              onPointerUp={() => {
                const samples = dragRef.current?.samples ?? [];
                dragRef.current = null;
                const first = samples[0] as
                  (typeof samples)[number] | undefined;
                const last = samples[samples.length - 1] as
                  (typeof samples)[number] | undefined;
                const velocity =
                  first && last && last.t > first.t
                    ? ((last.v - first.v) / (last.t - first.t)) * 1000
                    : 0;
                settle(velocity);
              }}
            />
          </div>
        </div>
      </div>
    </IdeaFrame>
  );
}
