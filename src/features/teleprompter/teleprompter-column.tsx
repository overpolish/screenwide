// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { type RefObject, useRef } from "react";
import { useFocusRing, useMove } from "react-aria";

import { t } from "../../i18n/i18n";
import { cn } from "../../lib/styling";

import { columnStyle } from "./column-style";
import { COLUMN_WIDTHS } from "./teleprompter-preferences";

/** Within this many points of the frame's width a line dragged out lets
 * the column fill the frame again. */
const FILLS_WITHIN = 12;
/** How far an arrow key moves the column's edges, in points. */
const KEY_STEP = 16;

type Guide = {
  frameRef: RefObject<HTMLDivElement | null>;
  onWidthChange: (width: number | null) => void;
  width: number | null;
};

/** One of the column's edges: a faint dashed line that moves both edges
 * together as it is dragged, or stepped with the arrow keys, out to widen
 * the column and in to narrow it, whichever side it is on and however the
 * window is mirrored. Out to the frame's edge, the column fills the frame. */
function ColumnEdge({
  className,
  frameRef,
  onWidthChange,
  width,
}: Guide & { className: string }) {
  const edgeRef = useRef<HTMLDivElement>(null);
  const reachRef = useRef(0);
  const { focusProps, isFocusVisible } = useFocusRing();
  const { moveProps } = useMove({
    onMove: ({ deltaX, pointerType }) => {
      const frame = frameRef.current?.getBoundingClientRect();
      const edge = edgeRef.current?.getBoundingClientRect();
      if (!frame || !edge) return;
      const outward =
        edge.left + edge.width / 2 < frame.left + frame.width / 2 ? -1 : 1;
      const step = pointerType === "keyboard" ? KEY_STEP : 1;
      // The width the hand has reached, which may run past either bound:
      // only what is applied is held within them, so coming back from past
      // a bound moves nothing until the hand is back over the line.
      reachRef.current += 2 * outward * deltaX * step;
      const applied = Math.min(
        frame.width,
        Math.max(COLUMN_WIDTHS.min, reachRef.current),
      );
      onWidthChange(
        applied >= frame.width - FILLS_WITHIN ? null : Math.round(applied),
      );
    },
    onMoveStart: () => {
      const frame = frameRef.current?.clientWidth ?? 0;
      reachRef.current = Math.min(width ?? frame, frame);
    },
  });
  return (
    <div
      {...moveProps}
      {...focusProps}
      aria-label={t("teleprompter-column-width")}
      aria-orientation="vertical"
      aria-valuemax={frameRef.current?.clientWidth}
      aria-valuemin={COLUMN_WIDTHS.min}
      aria-valuenow={width ?? frameRef.current?.clientWidth}
      className={cn(
        "pointer-events-auto absolute inset-y-0 flex w-3 cursor-ew-resize touch-none outline-none",
        className,
      )}
      ref={edgeRef}
      role="separator"
      tabIndex={0}
    >
      {/* The line on the column's own edge, its grip reaching in from it,
          so a column filling the window can still be taken at its edge. */}
      <span
        className={cn(
          "block h-full w-0 border-s border-dashed transition-colors",
          isFocusVisible ? "border-focus" : "border-fg-tertiary/50",
        )}
      />
    </div>
  );
}

/**
 * The column the text runs in, drawn over it: its two edges, which set how
 * wide it is, and the reading line's mark at its start. The edges stay
 * faint and out of the way, and give way to the text while it plays until
 * the pointer comes back to the window.
 */
export function TeleprompterColumn({
  isPlaying,
  readingTop,
  ...guide
}: Guide & {
  isPlaying: boolean;
  /** Where the reading line's mark stands, as a CSS length. */
  readingTop: string;
}) {
  return (
    <div
      className="pointer-events-none absolute inset-0 z-10 mx-auto"
      style={columnStyle(guide.width)}
    >
      <span
        aria-hidden
        className="absolute start-0 h-[0.15rem] w-inset-sm rounded-e-full bg-fg-secondary"
        style={{ top: readingTop }}
      />
      <div
        className={cn(
          "transition-opacity duration-200",
          isPlaying &&
            "opacity-0 group-hover:opacity-100 focus-within:opacity-100",
        )}
      >
        <ColumnEdge {...guide} className="start-0 justify-start" />
        <ColumnEdge {...guide} className="end-0 justify-end" />
      </div>
    </div>
  );
}
