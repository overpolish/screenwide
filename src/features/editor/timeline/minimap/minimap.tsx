// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { type Playhead } from "../scrub-playhead";
import { type TakenOut, takenTotal } from "../strip/taken-out";
import { SQUEEZED_HATCH } from "../strip/taken-out-band";
import {
  type Span,
  type TimeMap,
  timelineScale,
  type View,
} from "../timeline-scale";

// The whole recording under the strip: each beat a tick, the playhead, and
// a window round the part the strip shows. Drag the window to move along,
// its edges to zoom; press outside it to go there. Its length is the edited
// recording's against the recording's, so a cut or a squeeze visibly
// shortens it; what was taken out is drawn past its end in the strip's two
// segments, what the squeeze took hatched next to it and what the cuts
// deleted after. The squeeze is taken hold of at the end of what plays and
// follows the pointer: dragged in, the overview shortens under it and the
// hatch grows beyond it; dragged out, the time comes back until the hatch
// is gone.

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
  /** How many seconds the lulls are squeezed by now. */
  squeeze: number;
  /** What is taken out of the recording, drawn past the end. */
  taken: TakenOut;
  view: View;
};

export function Minimap({
  beats,
  map,
  maxSqueeze,
  onSqueezeChange,
  onViewChange,
  playhead,
  squeeze,
  taken,
  view,
}: MinimapProps) {
  const ref = useRef<HTMLDivElement>(null);
  const playheadRef = useRef<HTMLSpanElement>(null);
  const dragRef = useRef<Drag | null>(null);
  const gripRef = useRef<{
    /** How many seconds a pixel of the overview stands for. */
    perPx: number;
    squeeze: number;
    x: number;
  } | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const scale = timelineScale(map);
  /** The recording's length, which the overview's room spans. */
  const whole = Math.max(map.duration + takenTotal(taken), 1e-6);
  const played = map.duration / whole;

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
    <div className="relative h-3" dir="ltr">
      {/* In shares of the whole recording, beside what plays rather than
          in it, so they glide with it when the edit changes; drawn first,
          so the squeeze's handle stays over them. */}
      <span
        aria-hidden
        className={cn(
          "pointer-events-none absolute inset-y-0.5 rounded-full",
          SQUEEZED_HATCH,
        )}
        style={{
          insetInlineStart: share(played),
          width: share(taken.squeezed / whole),
        }}
      />
      <span
        aria-hidden
        // Solid, as small as it is drawn here, where the strip's is flat.
        className="pointer-events-none absolute inset-y-0.5 rounded-full bg-fill-strong"
        style={{
          insetInlineStart: share(played + taken.squeezed / whole),
          width: share(taken.deleted / whole),
        }}
      />
      <div
        className="relative h-full touch-none"
        onPointerDown={(event) => {
          // Pressed outside the window: the window goes there.
          const box = ref.current?.getBoundingClientRect();
          if (!box || box.width === 0) return;
          const to = (event.clientX - box.left) / box.width;
          const span = view.end - view.start;
          onViewChange({ end: to + span / 2, start: to - span / 2 });
        }}
        ref={ref}
        style={{ width: share(played) }}
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
          // At the end of what plays, where the hatch begins.
          className="absolute start-full -top-2 -bottom-2 flex w-3 cursor-ew-resize items-center"
          // Double-clicked, every lull plays as recorded again.
          onDoubleClick={() => {
            onSqueezeChange(0);
          }}
          onPointerDown={(event) => {
            event.stopPropagation();
            event.currentTarget.setPointerCapture(event.pointerId);
            // A second is as wide in the overview however much is squeezed,
            // the room it stands in being the whole recording's, so the end
            // of what plays, and the handle on it, follow the pointer.
            const width = ref.current?.getBoundingClientRect().width ?? 0;
            gripRef.current = {
              perPx: width > 0 ? map.duration / width : 0,
              squeeze,
              x: event.clientX,
            };
          }}
          onPointerMove={(event) => {
            const grip = gripRef.current;
            if (!grip || grip.perPx === 0) return;
            const seconds = (grip.x - event.clientX) * grip.perPx;
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
    </div>
  );
}
