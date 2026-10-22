// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CounterMark } from "./look-counter";
import { ArrowMark, FocusMark, MarkDefs } from "./look-marks";
import { corner, type Mark, type Point, swatchOf } from "./look-pens-model";
import { lookById, type LookId, SIZE_SCALE, swatchColor } from "./looks";
import { H, px, W } from "./point-geometry";

// Every mark drawn in its look. A mark is keyed by its look too, so changing
// the look draws it on afresh in the new style rather than snapping to it.

export function PlacedMarks({
  draft,
  isSelecting,
  lookId,
  marks,
  onSelect,
}: {
  draft: { from: Point; to: Point } | null;
  isSelecting: boolean;
  lookId: LookId;
  marks: Mark[];
  onSelect: (id: number) => void;
}) {
  const counters = marks.filter(({ pen }) => pen === "counter");
  return (
    <svg
      aria-hidden
      className="pointer-events-none absolute inset-0"
      height={H}
      width={W}
    >
      <MarkDefs />
      {marks.map((mark) => {
        const look = lookById(mark.lookOverride ?? lookId);
        const common = {
          color: swatchColor(look, swatchOf(mark, lookId)),
          look,
          scale: SIZE_SCALE[mark.size],
        };
        return (
          <g
            className="cursor-pointer"
            key={`${String(mark.id)}-${look.id}`}
            onPointerDown={(event) => {
              if (!isSelecting) return;
              event.stopPropagation();
              onSelect(mark.id);
            }}
            style={{ pointerEvents: isSelecting ? "auto" : "none" }}
          >
            {mark.pen === "focus" ? (
              <FocusMark
                {...common}
                radius={mark.element.radius}
                rect={px(mark.element)}
              />
            ) : mark.pen === "counter" ? (
              <CounterMark
                {...common}
                {...corner(mark.element)}
                number={counters.indexOf(mark) + 1}
              />
            ) : mark.from && mark.to ? (
              <ArrowMark {...common} from={mark.from} to={mark.to} />
            ) : null}
          </g>
        );
      })}
      {draft ? (
        <ArrowMark
          color={lookById(lookId).ink}
          from={draft.from}
          look={lookById(lookId)}
          scale={1}
          to={draft.to}
        />
      ) : null}
    </svg>
  );
}
