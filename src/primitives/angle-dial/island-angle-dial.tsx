// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useReducedMotion } from "motion/react";
import { type ReactNode, type WheelEvent, useRef, useState } from "react";

import { motionDurationCss } from "../../lib/motion";
import { cn, elementFocusVisible, focusStyles } from "../../lib/styling";
import { IslandFill } from "../island/island-fill";

import { signedAngle, wrapAngle } from "./angle-geometry";
import { type DialSteps, EIGHTH_TURN, useAngleDial } from "./use-angle-dial";

// The angle dial in the island's skin: one of the island's circles, holding
// a glyph drawn pointing at twelve o'clock and turned to the value, so the
// circle shows the aim it sets. A drag aims it at the pointer, a scroll or an
// arrow key turns it. Stepped, it rests on the eighths, marked round its
// edge, and a scroll turns an eighth; free, it turns to the degree as a
// window's dial does, and a scroll turns it a degree at a time.
//
// The turn eases to each new aim the short way round, so a step across
// twelve o'clock does not spin the glyph back through the whole turn.

/** How far a scroll travels, in px, to turn an eighth: enough that a
 * trackpad's flick turns once or twice rather than spinning. */
const WHEEL_STEP = 30;
/** How far a scroll travels, in px, to turn a free dial a degree. */
const WHEEL_DEGREE = 3;
/** A line's worth of scroll, in px, for a wheel that reports lines. */
const WHEEL_LINE = 16;

const MARKS = Array.from({ length: 8 }, (_, index) => index * EIGHTH_TURN);

export function IslandAngleDial({
  "aria-label": label,
  children,
  isDisabled,
  isStepped,
  onChange,
  steps,
  value,
}: {
  "aria-label": string;
  children: ReactNode;
  isDisabled: boolean;
  isStepped: boolean;
  onChange: (degrees: number) => void;
  steps: DialSteps;
  value: number;
}) {
  const { dialProps, isDragging, isHovered } = useAngleDial({
    isDisabled,
    label,
    onChange,
    steps,
    value,
  });
  const transitionDuration = useReducedMotion()
    ? "0s"
    : motionDurationCss("travel");
  const scrolledRef = useRef(0);

  // The turn shown, unwrapped: each new value is reached by the shorter way
  // from the last, so the rotation never runs back through a whole turn.
  const [shown, setShown] = useState({ turn: value, value });
  if (shown.value !== value)
    setShown({
      turn: shown.turn + signedAngle(value - shown.value),
      value,
    });

  const turnByWheel = (event: WheelEvent) => {
    scrolledRef.current +=
      event.deltaMode === 1 ? event.deltaY * WHEEL_LINE : event.deltaY;
    const travel = isStepped ? WHEEL_STEP : WHEEL_DEGREE;
    if (Math.abs(scrolledRef.current) < travel) return;
    const turns = Math.trunc(scrolledRef.current / travel);
    scrolledRef.current -= turns * travel;
    onChange(wrapAngle(value + turns * (isStepped ? EIGHTH_TURN : 1)));
  };

  return (
    <div
      {...dialProps}
      className={cn(
        "group relative isolate flex size-island-control shrink-0 cursor-default touch-none items-center justify-center rounded-full select-none",
        focusStyles,
        elementFocusVisible,
      )}
      onWheel={turnByWheel}
    >
      <IslandFill isChosen={isDragging} isHovered={isHovered} />
      {isStepped ? (
        MARKS.map((mark) => (
          <span
            aria-hidden
            className="absolute inset-0.5"
            key={mark}
            style={{ rotate: `${String(mark)}deg` }}
          >
            <span className="absolute start-1/2 top-0 size-0.5 -translate-x-1/2 rounded-full bg-fg-tertiary rtl:translate-x-1/2" />
          </span>
        ))
      ) : (
        // With no steps to mark, a faint ring says it turns, in the
        // island's stronger fill so it sits back from the dot.
        <span
          aria-hidden
          className="absolute inset-0.5 rounded-full border border-fill-strong"
        />
      )}
      <span
        aria-hidden
        className="flex transform-gpu transition-[rotate] ease-out"
        style={{ rotate: `${String(shown.turn)}deg`, transitionDuration }}
      >
        {children}
      </span>
    </div>
  );
}
