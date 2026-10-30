// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { type Playhead } from "../scrub-playhead";
import {
  type Span,
  type TimeMap,
  timelineScale,
  type View,
} from "../timeline-scale";

// The whole recording under the strip: each beat a tick, the playhead, and
// a window round the part the strip shows. Drag the window to move along,
// its edges to zoom; press outside it to go there. Its length is the edited
// recording's against the recording's, so a squeeze visibly shortens it,
// and its end is where the squeeze is taken hold of.

type Drag = {
  edge: "end" | "move" | "start";
  view: View;
  width: number;
  x: number;
};

const share = (part: number) => `${String(part * 100)}%`;

export type MinimapProps = {
  /** The beats, drawn as ticks; a moment's in its kind's colour,
   * `#rrggbb`, so the moments worth keeping stand out of the whole. */
  beats: (Span & { color?: string })[];
  /** How the recording plays now, which places the ticks and sets how much
   * of the recording's length the overview spans. */
  map: TimeMap;
  /** The most the lulls can be squeezed by, in seconds. */
  maxSqueeze: number;
  /** Told how many seconds the lulls are squeezed by: 0 plays them as
   * recorded. */
  onSqueezeChange: (squeeze: number) => void;
  /** Told the part of the recording to show, which may run outside it or
   * be narrower than the strip allows: the caller keeps it in bounds. */
  onViewChange: (view: View) => void;
  /** Where the playhead is, as its share of the edited recording. */
  playhead: Playhead;
  /** How long the recording was recorded, in seconds. */
  recorded: number;
  /** How many seconds the lulls are squeezed by now. */
  squeeze: number;
  view: View;
};

export function Minimap({
  beats,
  map,
  maxSqueeze,
  onSqueezeChange,
  onViewChange,
  playhead,
  recorded,
  squeeze,
  view,
}: MinimapProps) {
  const ref = useRef<HTMLDivElement>(null);
  const playheadRef = useRef<HTMLSpanElement>(null);
  const dragRef = useRef<Drag | null>(null);
  const gripRef = useRef<{ squeeze: number; width: number; x: number } | null>(
    null,
  );
  const [isDragging, setIsDragging] = useState(false);
  const scale = timelineScale(map);

  // Written straight to the line, so playing never renders the overview.
  useEffect(
    () =>
      playhead.subscribe((_, ratio) => {
        if (playheadRef.current)
          playheadRef.current.style.insetInlineStart = share(ratio);
      }),
    [playhead],
  );

  const drag = (edge: Drag["edge"]) => ({
    onPointerDown: (event: React.PointerEvent) => {
      event.stopPropagation();
      event.currentTarget.setPointerCapture(event.pointerId);
      const width = ref.current?.getBoundingClientRect().width ?? 0;
      dragRef.current = { edge, view, width, x: event.clientX };
      setIsDragging(true);
    },
    onPointerMove: (event: React.PointerEvent) => {
      const held = dragRef.current;
      if (!held || held.width === 0) return;
      const by = (event.clientX - held.x) / held.width;
      onViewChange({
        end: held.view.end + (held.edge === "start" ? 0 : by),
        start: held.view.start + (held.edge === "end" ? 0 : by),
      });
    },
    onPointerUp: () => {
      dragRef.current = null;
      setIsDragging(false);
    },
  });

  return (
    <div
      className="relative h-3 touch-none"
      dir="ltr"
      onPointerDown={(event) => {
        // Pressed outside the window: the window goes there.
        const box = ref.current?.getBoundingClientRect();
        if (!box || box.width === 0) return;
        const to = (event.clientX - box.left) / box.width;
        const span = view.end - view.start;
        onViewChange({ end: to + span / 2, start: to - span / 2 });
      }}
      ref={ref}
      style={{
        width: share(map.duration / Math.max(recorded, map.duration, 1e-6)),
      }}
    >
      <span className="absolute inset-x-0 top-1/2 h-px -translate-y-1/2 bg-fill-strong" />
      {beats.map((beat) => (
        <span
          aria-hidden
          className="absolute inset-y-0.5 min-w-px rounded-full bg-fg-tertiary"
          key={beat.start}
          style={{
            backgroundColor: beat.color,
            insetInlineStart: share(scale.toX(beat.start)),
            width: share(scale.toX(beat.end, true) - scale.toX(beat.start)),
          }}
        />
      ))}
      <span
        aria-hidden
        className="absolute -inset-y-0.5 w-px bg-primary"
        ref={playheadRef}
      />
      <span
        aria-label={t("editor-timeline-view")}
        className={cn(
          "absolute -inset-y-1 cursor-grab rounded-sm border border-fg-secondary bg-fill transition-colors hover:border-fg",
          isDragging && "cursor-grabbing border-fg",
        )}
        role="group"
        style={{
          insetInlineStart: share(view.start),
          width: share(view.end - view.start),
        }}
        {...drag("move")}
      >
        <span
          className="absolute inset-y-0 -start-1 w-2 cursor-ew-resize"
          {...drag("start")}
        />
        <span
          className="absolute inset-y-0 -end-1 w-2 cursor-ew-resize"
          {...drag("end")}
        />
      </span>
      <button
        aria-label={t("editor-timeline-squeeze")}
        className="absolute -end-4 -top-2 -bottom-2 flex w-4 cursor-ew-resize items-center justify-center"
        // Double-clicked, every lull plays as recorded again.
        onDoubleClick={() => {
          onSqueezeChange(0);
        }}
        onPointerDown={(event) => {
          event.stopPropagation();
          event.currentTarget.setPointerCapture(event.pointerId);
          // Measured against the room the whole recording would take.
          const width =
            ref.current?.parentElement?.getBoundingClientRect().width ?? 0;
          gripRef.current = { squeeze, width, x: event.clientX };
        }}
        onPointerMove={(event) => {
          const grip = gripRef.current;
          if (!grip || grip.width === 0) return;
          const seconds = ((grip.x - event.clientX) / grip.width) * recorded;
          onSqueezeChange(
            Math.min(Math.max(grip.squeeze + seconds, 0), maxSqueeze),
          );
        }}
        onPointerUp={() => {
          gripRef.current = null;
        }}
        type="button"
      >
        <span className="h-5 w-1.5 rounded-full bg-fg-secondary" />
      </button>
    </div>
  );
}
