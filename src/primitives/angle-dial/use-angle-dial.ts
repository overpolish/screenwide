// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef, useState } from "react";
import { useFocusRing, useHover } from "react-aria";

import { angleAtPoint, snapAngle, wrapAngle } from "./angle-geometry";

/** How far a dial moves: under a drag, and for each arrow key, with Shift
 * held and without, in degrees. */
export type DialSteps = {
  drag: { plain: number; shifted: number };
  key: { plain: number; shifted: number };
};

/** Eighths of a turn: the quarters and the diagonals between them. */
export const EIGHTH_TURN = 45;

// What turns a dial, whatever it looks like: a slider of degrees clockwise
// from twelve o'clock. A press on the dial aims it at the pointer; a press on
// its notch picks it up where it is, so it does not jump out from under the
// finger. Either way the drag goes on outside the control, where the arm is
// long and a degree is a wide movement. The value wraps: the arrows carry it
// past the turn in both directions rather than stopping a step short of
// where they started. Page Up and Down step an eighth of a turn.
export function useAngleDial({
  isDisabled,
  label,
  onChange,
  steps,
  value,
}: {
  isDisabled: boolean;
  label: string;
  onChange: (degrees: number) => void;
  steps: DialSteps;
  value: number;
}) {
  const dialRef = useRef<HTMLDivElement>(null);
  const notchRef = useRef<HTMLSpanElement>(null);
  /** How far the press landed from the notch, while a grab on it is held. */
  const grabOffsetRef = useRef<number | null>(null);
  /** The last degree sent, so a drag across one of them sends it once. */
  const sentRef = useRef<number | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const { focusProps, isFocusVisible } = useFocusRing();
  const { hoverProps, isHovered } = useHover({ isDisabled });
  /** The finest the dial is read to, which is what it reports. */
  const finest = Math.min(
    steps.drag.plain,
    steps.drag.shifted,
    steps.key.plain,
    steps.key.shifted,
  );

  const send = (degrees: number) => {
    if (degrees === sentRef.current) return;
    sentRef.current = degrees;
    onChange(degrees);
  };

  const aimAtPointer = (event: React.PointerEvent<HTMLDivElement>) => {
    const dial = dialRef.current;
    if (!dial) return;
    const bearing = angleAtPoint(dial.getBoundingClientRect(), event);
    if (bearing === null) return;
    const grabOffset = grabOffsetRef.current;
    send(
      snapAngle(
        grabOffset === null ? bearing : bearing - grabOffset,
        event.shiftKey ? steps.drag.shifted : steps.drag.plain,
      ),
    );
  };

  const beginDrag = (event: React.PointerEvent<HTMLDivElement>) => {
    const dial = dialRef.current;
    if (isDisabled || event.button !== 0 || !dial) return;
    const heldNotch = notchRef.current?.contains(event.target as Node) ?? false;
    const bearing = angleAtPoint(dial.getBoundingClientRect(), event);
    grabOffsetRef.current =
      heldNotch && bearing !== null ? wrapAngle(bearing - value) : null;
    sentRef.current = snapAngle(value, finest);
    setIsDragging(true);
    dial.setPointerCapture(event.pointerId);
    dial.focus({ preventScroll: true });
    aimAtPointer(event);
  };

  const endDrag = (event: React.PointerEvent<HTMLDivElement>) => {
    if (!isDragging) return;
    dialRef.current?.releasePointerCapture(event.pointerId);
    grabOffsetRef.current = null;
    sentRef.current = null;
    setIsDragging(false);
  };

  const turn = (event: React.KeyboardEvent<HTMLDivElement>, by: number) => {
    event.preventDefault();
    onChange(snapAngle(value + by, finest));
  };

  const handleKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    if (isDisabled) return;
    const stride = event.shiftKey ? steps.key.shifted : steps.key.plain;
    switch (event.key) {
      case "ArrowDown":
      case "ArrowLeft": {
        turn(event, -stride);
        break;
      }
      case "ArrowRight":
      case "ArrowUp": {
        turn(event, stride);
        break;
      }
      case "End": {
        event.preventDefault();
        onChange(360 - finest);
        break;
      }
      case "Home": {
        event.preventDefault();
        onChange(0);
        break;
      }
      case "PageDown": {
        turn(event, -EIGHTH_TURN);
        break;
      }
      case "PageUp": {
        turn(event, EIGHTH_TURN);
        break;
      }
      default: {
        break;
      }
    }
  };

  const degrees = snapAngle(value, finest);

  return {
    dialProps: {
      ...focusProps,
      ...hoverProps,
      "aria-disabled": isDisabled || undefined,
      "aria-label": label,
      "aria-valuemax": 360 - finest,
      "aria-valuemin": 0,
      "aria-valuenow": degrees,
      "aria-valuetext": `${String(degrees)}°`,
      "data-disabled": isDisabled || undefined,
      "data-dragging": isDragging || undefined,
      "data-focus-visible": isFocusVisible || undefined,
      "data-hovered": isHovered || undefined,
      // A clock face: clockwise from twelve in every language.
      dir: "ltr",
      onKeyDown: handleKeyDown,
      onPointerCancel: endDrag,
      onPointerDown: beginDrag,
      onPointerMove: (event: React.PointerEvent<HTMLDivElement>) => {
        if (isDragging) aimAtPointer(event);
      },
      onPointerUp: endDrag,
      ref: dialRef,
      role: "slider",
      tabIndex: isDisabled ? undefined : 0,
    },
    isDragging,
    isHovered,
    notchRef,
  };
}
