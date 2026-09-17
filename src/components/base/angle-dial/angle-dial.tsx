// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef, useState } from "react";
import { useFocusRing, useHover } from "react-aria";

import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";

import { angleAtPoint, snapAngle, wrapAngle } from "./angle-geometry";

/** A degree at a time, as fine as the dial can be read. */
const STEP = 1;
/** Held Shift takes the arrows ten degrees at a time. */
const COARSE_STEP = 10;
/** Eighths of a turn: the quarters and the diagonals between them. */
const SNAP_STEP = 45;

// The knob. A bezel's disc rather than a slider's raised thumb, so it sits in
// a row of fields and buttons as one of them, at the same height. The notch is
// the value: a dimple in the outer band that the disc turns under, drawn in
// the secondary label colour and brought up to the full one while the hand is
// on it. Drawn as a rotation of the whole span rather than a moved dot, so the
// turn stays on the compositor.
const dialStyles = cn(
  "group relative size-control-height shrink-0 cursor-default touch-none rounded-full bg-control-fill select-none control-stroke",
  "data-[hovered]:bg-control-fill-hover",
  "data-[disabled]:bg-control-fill-disabled",
  focusStyles,
  elementFocusVisible,
);

const notchStyles = cn(
  "absolute top-0.5 left-1/2 size-1.5 -translate-x-1/2 rounded-full bg-content-fg-secondary transition-colors windows:top-1.5",
  "group-data-[hovered]:bg-content-fg group-data-[dragging]:bg-content-fg",
  "group-data-[disabled]:bg-control-fg-disabled",
);

export type AngleDialProps = {
  "aria-label": string;
  onChange: (degrees: number) => void;
  /** Where the notch points, in degrees clockwise from twelve o'clock. */
  value: number;
  className?: string;
  isDisabled?: boolean;
};

/**
 * A circular dial for an angle: the value is a direction, so the control is
 * one too, and the notch stands where the thing being aimed stands.
 *
 * A press on the disc aims the dial at the pointer; a press on the notch picks
 * it up where it is, so it does not jump out from under the finger. Either way
 * the drag goes on outside the control, where the arm is long and a degree is
 * a wide movement, and holding Shift through it snaps to the eight aims that
 * matter. The value wraps: the arrows carry it past the turn in both
 * directions rather than stopping a degree short of where they started.
 */
export function AngleDial({
  "aria-label": label,
  className,
  isDisabled = false,
  onChange,
  value,
}: AngleDialProps) {
  const dialRef = useRef<HTMLDivElement>(null);
  const notchRef = useRef<HTMLSpanElement>(null);
  /** How far the press landed from the notch, while a grab on it is held. */
  const grabOffsetRef = useRef<number | null>(null);
  /** The last degree sent, so a drag across one of them sends it once. */
  const sentRef = useRef<number | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const { focusProps, isFocusVisible } = useFocusRing();
  const { hoverProps, isHovered } = useHover({ isDisabled });

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
        event.shiftKey ? SNAP_STEP : STEP,
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
    sentRef.current = snapAngle(value, STEP);
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
    onChange(snapAngle(value + by, STEP));
  };

  const handleKeyDown = (event: React.KeyboardEvent<HTMLDivElement>) => {
    if (isDisabled) return;
    const stride = event.shiftKey ? COARSE_STEP : STEP;
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
        onChange(360 - STEP);
        break;
      }
      case "Home": {
        event.preventDefault();
        onChange(0);
        break;
      }
      case "PageDown": {
        turn(event, -SNAP_STEP);
        break;
      }
      case "PageUp": {
        turn(event, SNAP_STEP);
        break;
      }
      default: {
        break;
      }
    }
  };

  const degrees = snapAngle(value, STEP);

  return (
    <div
      {...focusProps}
      {...hoverProps}
      aria-disabled={isDisabled || undefined}
      aria-label={label}
      aria-valuemax={360 - STEP}
      aria-valuemin={0}
      aria-valuenow={degrees}
      aria-valuetext={`${String(degrees)}°`}
      className={cn(dialStyles, className)}
      data-disabled={isDisabled || undefined}
      data-dragging={isDragging || undefined}
      data-focus-visible={isFocusVisible || undefined}
      data-hovered={isHovered || undefined}
      onKeyDown={handleKeyDown}
      onPointerCancel={endDrag}
      onPointerDown={beginDrag}
      onPointerMove={(event) => {
        if (isDragging) aimAtPointer(event);
      }}
      onPointerUp={endDrag}
      ref={dialRef}
      role="slider"
      tabIndex={isDisabled ? undefined : 0}
    >
      <span
        className="absolute inset-0 transform-gpu"
        style={{ rotate: `${String(wrapAngle(value))}deg` }}
      >
        <span className={notchStyles} ref={notchRef} />
      </span>
    </div>
  );
}
